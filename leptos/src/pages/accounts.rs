use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::{Layout, Pagination, PaginationInfo};
use crate::api::{api_get, api_post, api_put, api_delete};
use crate::context::use_company_context;
use crate::stores::SaftAccountStore;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaftAccountInfo {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub is_active: bool,
    pub saft_account_id: Option<i64>,
    pub saft_account: Option<SaftAccountInfo>,
    #[serde(default)]
    pub tracks_articles: bool,
}

fn get_account_type_name(account_type: &str) -> &'static str {
    match account_type {
        "active" | "asset" => "Активна",
        "passive" | "liability" => "Пасивна",
        "bifunctional" | "active_passive" => "Активно-пасивна",
        "equity" => "Капитал",
        "revenue" => "Приход",
        "expense" => "Разход",
        _ => "Друг",
    }
}

fn get_account_type_color(account_type: &str) -> &'static str {
    match account_type {
        "active" | "asset" => "#3498db",
        "passive" | "liability" => "#e74c3c",
        "bifunctional" | "active_passive" => "#9b59b6",
        "equity" => "#9b59b6",
        "revenue" => "#27ae60",
        "expense" => "#f39c12",
        _ => "#7f8c8d",
    }
}

fn get_account_group_name(code: &str) -> &'static str {
    if code.is_empty() {
        return "Други";
    }
    match code.chars().next().unwrap_or('0') {
        '1' => "Раздел 1 - Капитал и заеми",
        '2' => "Раздел 2 - Дълготрайни активи",
        '3' => "Раздел 3 - Материални запаси",
        '4' => "Раздел 4 - Разчети",
        '5' => "Раздел 5 - Финансови средства",
        '6' => "Раздел 6 - Разходи",
        '7' => "Раздел 7 - Приходи",
        '9' => "Раздел 9 - Условни активи/пасиви",
        _ => "Други",
    }
}

#[component]
pub fn Accounts() -> impl IntoView {
    let accounts = create_rw_signal(Vec::<Account>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());
    let search_query = create_rw_signal(String::new());
    let selected_type = create_rw_signal(String::new());
    let group_by_section = create_rw_signal(false); // Disable grouping with pagination

    // Pagination state
    let current_page = create_rw_signal(1i32);
    let per_page = create_rw_signal(50i32);
    let pagination_info = create_rw_signal(PaginationInfo::default());

    // Modal state
    let show_modal = create_rw_signal(false);
    let editing_account = create_rw_signal(Option::<Account>::None);
    let form_code = create_rw_signal(String::new());
    let form_name = create_rw_signal(String::new());
    let form_type = create_rw_signal("active".to_string());
    let form_is_active = create_rw_signal(true);
    let saving = create_rw_signal(false);

    // SAF-T account mapping
    let form_saft_account_id = create_rw_signal(Option::<i64>::None);
    let form_saft_search = create_rw_signal(String::new());
    let form_saft_dropdown_open = create_rw_signal(false);

    // Tracks articles
    let form_tracks_articles = create_rw_signal(false);

    // SAF-T account store
    let saft_store = SaftAccountStore::new();

    // Delete confirmation
    let delete_account_id = create_rw_signal(Option::<i64>::None);
    let deleting = create_rw_signal(false);

    // Initialize accounts state
    let initializing = create_rw_signal(false);

    let ctx = use_company_context();

    // Load accounts function
    let load_accounts = move || {
        if let Some(id) = ctx.selected_company_id.get() {
            loading.set(true);
            error_msg.set(String::new());
            let page = current_page.get();
            let pp = per_page.get();
            let q = search_query.get();
            let type_filter = selected_type.get();

            spawn_local(async move {
                let mut url = format!("/api/companies/{}/accounts?page={}&per_page={}", id, page, pp);
                if !q.is_empty() {
                    url.push_str(&format!("&q={}", urlencoding::encode(&q)));
                }
                if !type_filter.is_empty() {
                    url.push_str(&format!("&type={}", type_filter));
                }

                match api_get(&url).await {
                    Ok(data) => {
                        if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                            let accs: Vec<Account> = arr.iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect();
                            accounts.set(accs);
                        }
                        // Parse pagination
                        if let Some(pag) = data.get("pagination") {
                            pagination_info.set(PaginationInfo {
                                page: pag.get("page").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                                per_page: pag.get("per_page").and_then(|v| v.as_i64()).unwrap_or(50) as i32,
                                total: pag.get("total").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                total_pages: pag.get("total_pages").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                            });
                        }
                    }
                    Err(e) => {
                        error_msg.set(format!("Грешка при зареждане: {}", e));
                    }
                }
                loading.set(false);
            });
        }
    };

    // Fetch accounts when company, page, or filters change
    create_effect(move |_| {
        let _ = ctx.selected_company_id.get();
        let _ = current_page.get();
        let _ = per_page.get();
        load_accounts();
    });

    // Reset to page 1 when search or type filter changes
    let on_search_change = move |ev: web_sys::Event| {
        let val = event_target_value(&ev);
        search_query.set(val);
        current_page.set(1);
        load_accounts();
    };

    let on_type_change = move |type_val: String| {
        selected_type.set(type_val);
        current_page.set(1);
        load_accounts();
    };

    // Pagination callbacks
    let on_page_change = Callback::new(move |page: i32| {
        current_page.set(page);
    });

    let on_per_page_change = Callback::new(move |pp: i32| {
        per_page.set(pp);
        current_page.set(1);
    });

    // Lazy-load SAF-T accounts when modal opens
    {
        let saft_store_clone = saft_store.clone();
        create_effect(move |_| {
            if show_modal.get() {
                let store = saft_store_clone.clone();
                spawn_local(async move {
                    store.fetch_accounts().await;
                });
            }
        });
    }

    // Open modal for new account
    let open_new_modal = move |_| {
        editing_account.set(None);
        form_code.set(String::new());
        form_name.set(String::new());
        form_type.set("active".to_string());
        form_is_active.set(true);
        form_saft_account_id.set(None);
        form_saft_search.set(String::new());
        form_saft_dropdown_open.set(false);
        form_tracks_articles.set(false);
        show_modal.set(true);
    };

    // Open modal for editing
    let open_edit_modal = move |acc: Account| {
        form_code.set(acc.code.clone());
        form_name.set(acc.name.clone());
        form_type.set(acc.account_type.clone());
        form_is_active.set(acc.is_active);
        form_tracks_articles.set(acc.tracks_articles);
        form_saft_account_id.set(acc.saft_account_id);
        if let Some(ref saft) = acc.saft_account {
            form_saft_search.set(format!("{} - {}", saft.code, saft.name));
        } else {
            form_saft_search.set(String::new());
        }
        form_saft_dropdown_open.set(false);
        editing_account.set(Some(acc));
        show_modal.set(true);
    };

    // Save account (create or update)
    let save_account = move |_| {
        let company_id = match ctx.selected_company_id.get() {
            Some(id) => id,
            None => return,
        };

        let code = form_code.get();
        let name = form_name.get();
        let acc_type = form_type.get();
        let is_active = form_is_active.get();
        let saft_id = form_saft_account_id.get();
        let tracks_articles = form_tracks_articles.get();

        if code.is_empty() || name.is_empty() {
            error_msg.set("Кодът и името са задължителни".to_string());
            return;
        }

        saving.set(true);
        error_msg.set(String::new());

        let editing = editing_account.get();

        spawn_local(async move {
            let body = serde_json::json!({
                "code": code,
                "name": name,
                "account_type": acc_type,
                "is_active": is_active,
                "saft_account_id": saft_id,
                "tracks_articles": tracks_articles
            });

            let result = if let Some(acc) = editing {
                api_put(&format!("/api/companies/{}/accounts/{}", company_id, acc.id), &body).await
            } else {
                api_post(&format!("/api/companies/{}/accounts", company_id), &body).await
            };

            match result {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set(if editing_account.get().is_some() {
                            "Сметката е обновена".to_string()
                        } else {
                            "Сметката е създадена".to_string()
                        });
                        show_modal.set(false);
                        load_accounts();
                    } else {
                        let err = data.get("error")
                            .or(data.get("errors").and_then(|e| e.get(0)).and_then(|e| e.get("messages")).and_then(|m| m.get(0)))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Неизвестна грешка");
                        error_msg.set(err.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            saving.set(false);
        });
    };

    // Delete account
    let confirm_delete = move |_| {
        let company_id = match ctx.selected_company_id.get() {
            Some(id) => id,
            None => return,
        };

        let account_id = match delete_account_id.get() {
            Some(id) => id,
            None => return,
        };

        deleting.set(true);
        error_msg.set(String::new());

        spawn_local(async move {
            match api_delete(&format!("/api/companies/{}/accounts/{}", company_id, account_id)).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Сметката е изтрита".to_string());
                        delete_account_id.set(None);
                        load_accounts();
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при изтриване");
                        error_msg.set(err.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            deleting.set(false);
        });
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Сметкоплан"</h1>
                <button
                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                    on:click=open_new_modal
                >
                    "+ Нова сметка"
                </button>
            </div>

            // Success message
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-top: 20px; display: flex; justify-content: space-between; align-items: center;">
                            <span>{msg}</span>
                            <button
                                style="background: transparent; border: none; color: white; cursor: pointer; font-size: 18px;"
                                on:click=move |_| success_msg.set(String::new())
                            >"×"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Error message
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 20px; display: flex; justify-content: space-between; align-items: center;">
                            <span>{err}</span>
                            <button
                                style="background: transparent; border: none; color: white; cursor: pointer; font-size: 18px;"
                                on:click=move |_| error_msg.set(String::new())
                            >"×"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Search and filters
            <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                <div style="display: flex; gap: 15px; flex-wrap: wrap; align-items: center;">
                    <div style="flex: 1; min-width: 250px;">
                        <input
                            type="text"
                            placeholder="Търсене по код или наименование... (Enter за търсене)"
                            style="width: 100%; padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                            on:keypress=move |ev: web_sys::KeyboardEvent| {
                                if ev.key() == "Enter" {
                                    on_search_change(ev.into());
                                }
                            }
                            on:change=on_search_change
                            prop:value=move || search_query.get()
                        />
                    </div>

                    <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                        <button
                            style=move || format!(
                                "padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; {}",
                                if selected_type.get().is_empty() {
                                    "background: #34495e; color: white; border: none;"
                                } else {
                                    "background: white; color: #34495e; border: 1px solid #ddd;"
                                }
                            )
                            on:click=move |_| on_type_change(String::new())
                        >
                            "Всички"
                        </button>
                        {
                            let types = vec![
                                ("active", "Активни"),
                                ("passive", "Пасивни"),
                                ("bifunctional", "Активно-пасивни"),
                                ("revenue", "Приходи"),
                                ("expense", "Разходи"),
                            ];
                            types.into_iter().map(|(t, label)| {
                                let t_clone = t.to_string();
                                let t_clone2 = t.to_string();
                                let t_clone3 = t.to_string();
                                view! {
                                    <button
                                        style=move || format!(
                                            "padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; {}",
                                            if selected_type.get() == t_clone {
                                                format!("background: {}; color: white; border: none;", get_account_type_color(&t_clone))
                                            } else {
                                                "background: white; color: #34495e; border: 1px solid #ddd;".to_string()
                                            }
                                        )
                                        on:click=move |_| on_type_change(t_clone2.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            }).collect_view()
                        }
                    </div>
                </div>
            </div>

            // Loading state
            {move || {
                if loading.get() {
                    view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d;">"Зареждане..."</p>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Accounts list
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                let accs = accounts.get();

                if accs.is_empty() && search_query.get().is_empty() && selected_type.get().is_empty() {
                    return view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 60px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 1.1em; margin-bottom: 20px;">
                                "Няма създадени сметки"
                            </p>
                            <button
                                style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; font-size: 16px;"
                                disabled=move || initializing.get()
                                on:click=move |_| {
                                    if let Some(company_id) = ctx.selected_company_id.get() {
                                        initializing.set(true);
                                        error_msg.set(String::new());
                                        spawn_local(async move {
                                            let url = format!("/api/companies/{}/initialize_accounts", company_id);
                                            match api_post(&url, &serde_json::json!({})).await {
                                                Ok(data) => {
                                                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                                                        let created = data.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
                                                        success_msg.set(format!("Заредени {} сметки от стандартния сметкоплан", created));
                                                        load_accounts();
                                                    } else {
                                                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Грешка");
                                                        error_msg.set(msg.to_string());
                                                    }
                                                }
                                                Err(e) => error_msg.set(format!("Грешка: {}", e)),
                                            }
                                            initializing.set(false);
                                        });
                                    }
                                }
                            >
                                {move || if initializing.get() { "Зареждане..." } else { "Зареди стандартен сметкоплан (SAF-T)" }}
                            </button>
                        </div>
                    }.into_view();
                } else if accs.is_empty() {
                    return view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 1.1em;">
                                "Няма намерени сметки"
                            </p>
                        </div>
                    }.into_view();
                }

                view! {
                    <div>
                        <div style="background: white; border-radius: 8px; margin-top: 15px; overflow: hidden;">
                            <table style="width: 100%; border-collapse: collapse;">
                                <thead style="background: #34495e; color: white;">
                                    <tr>
                                        <th style="padding: 12px; text-align: left; width: 100px;">"Код"</th>
                                        <th style="padding: 12px; text-align: left;">"Наименование"</th>
                                        <th style="padding: 12px; text-align: left; width: 120px;">"Тип"</th>
                                        <th style="padding: 12px; text-align: left; width: 200px;">"SAF-T сметка"</th>
                                        <th style="padding: 12px; text-align: center; width: 60px;">"Арт."</th>
                                        <th style="padding: 12px; text-align: center; width: 100px;">"Действия"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {accs.into_iter().map(|acc| {
                                                let acc_for_edit = acc.clone();
                                                let acc_id = acc.id;
                                                let type_color = get_account_type_color(&acc.account_type);
                                                let type_name = get_account_type_name(&acc.account_type);
                                                let saft_display = acc.saft_account.as_ref()
                                                    .map(|s| format!("{} - {}", s.code, s.name))
                                                    .unwrap_or_else(|| "-".to_string());
                                                let has_articles = acc.tracks_articles;

                                                view! {
                                                    <tr style="border-bottom: 1px solid #ecf0f1;">
                                                        <td style="padding: 12px; font-family: monospace; font-weight: bold;">
                                                            {acc.code}
                                                        </td>
                                                        <td style="padding: 12px;">
                                                            {acc.name}
                                                        </td>
                                                        <td style="padding: 12px;">
                                                            <span style=format!(
                                                                "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                                type_color
                                                            )>
                                                                {type_name}
                                                            </span>
                                                        </td>
                                                        <td style="padding: 12px; color: #7f8c8d; font-size: 13px;">
                                                            {saft_display}
                                                        </td>
                                                        <td style="padding: 12px; text-align: center;">
                                                            {if has_articles {
                                                                view! {
                                                                    <span style="display: inline-block; padding: 2px 8px; border-radius: 12px; font-size: 11px; color: white; background: #27ae60;">
                                                                        "Да"
                                                                    </span>
                                                                }.into_view()
                                                            } else {
                                                                view! { <span style="color: #bdc3c7;">"-"</span> }.into_view()
                                                            }}
                                                        </td>
                                                        <td style="padding: 12px; text-align: center;">
                                                            <button
                                                                style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-right: 5px; font-size: 12px;"
                                                                title="Редактиране"
                                                                on:click=move |_| open_edit_modal(acc_for_edit.clone())
                                                            >
                                                                "✏"
                                                            </button>
                                                            <button
                                                                style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                                title="Изтриване"
                                                                on:click=move |_| delete_account_id.set(Some(acc_id))
                                                            >
                                                                "🗑"
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>

                        // Pagination
                        <Pagination
                            pagination=pagination_info.into()
                            on_page_change=on_page_change
                            on_per_page_change=on_per_page_change
                        />
                    </div>
                }.into_view()
            }}

            // Create/Edit Modal
            {move || {
                if show_modal.get() {
                    let is_edit = editing_account.get().is_some();
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 500px; max-width: 90%;">
                                <h2 style="margin: 0 0 20px 0;">
                                    {if is_edit { "Редактиране на сметка" } else { "Нова сметка" }}
                                </h2>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Код *"</label>
                                    <input
                                        type="text"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        placeholder="напр. 401"
                                        prop:value=move || form_code.get()
                                        on:input=move |ev| form_code.set(event_target_value(&ev))
                                    />
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Наименование *"</label>
                                    <input
                                        type="text"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        placeholder="напр. Задължения към доставчици"
                                        prop:value=move || form_name.get()
                                        on:input=move |ev| form_name.set(event_target_value(&ev))
                                    />
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Тип сметка"</label>
                                    <select
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; background: white;"
                                        on:change=move |ev| form_type.set(event_target_value(&ev))
                                    >
                                        <option value="active" selected=move || form_type.get() == "active">"Активна"</option>
                                        <option value="passive" selected=move || form_type.get() == "passive">"Пасивна"</option>
                                        <option value="bifunctional" selected=move || form_type.get() == "bifunctional">"Активно-пасивна"</option>
                                        <option value="revenue" selected=move || form_type.get() == "revenue">"Приходна"</option>
                                        <option value="expense" selected=move || form_type.get() == "expense">"Разходна"</option>
                                    </select>
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: flex; align-items: center; gap: 8px; cursor: pointer;">
                                        <input
                                            type="checkbox"
                                            checked=move || form_is_active.get()
                                            on:change=move |ev| form_is_active.set(event_target_checked(&ev))
                                        />
                                        "Активна сметка"
                                    </label>
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: flex; align-items: center; gap: 8px; cursor: pointer;">
                                        <input
                                            type="checkbox"
                                            checked=move || form_tracks_articles.get()
                                            on:change=move |ev| form_tracks_articles.set(event_target_checked(&ev))
                                        />
                                        "Отчитане на артикули/продукти"
                                    </label>
                                </div>

                                // SAF-T account mapping
                                <div style="margin-bottom: 20px; position: relative;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"SAF-T стандартна сметка"</label>
                                    <div style="position: relative;">
                                        <input
                                            type="text"
                                            placeholder="Търсене по код или наименование..."
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; padding-right: 30px;"
                                            prop:value=move || form_saft_search.get()
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                form_saft_search.set(val);
                                                form_saft_account_id.set(None);
                                                form_saft_dropdown_open.set(true);
                                            }
                                            on:focus=move |_| {
                                                if form_saft_account_id.get().is_none() {
                                                    form_saft_dropdown_open.set(true);
                                                }
                                            }
                                        />
                                        {move || if form_saft_account_id.get().is_some() {
                                            view! {
                                                <button
                                                    type="button"
                                                    style="position: absolute; right: 8px; top: 50%; transform: translateY(-50%); background: none; border: none; cursor: pointer; color: #999; font-size: 16px; padding: 4px;"
                                                    on:click=move |_| {
                                                        form_saft_account_id.set(None);
                                                        form_saft_search.set(String::new());
                                                    }
                                                >"x"</button>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }}
                                    </div>
                                    // Dropdown results - inner closure reads search signal
                                    {move || {
                                        if !form_saft_dropdown_open.get() || form_saft_account_id.get().is_some() {
                                            return view! { <div></div> }.into_view();
                                        }
                                        let search = form_saft_search.get().to_lowercase();
                                        let all_saft = saft_store.accounts.get();
                                        let filtered: Vec<_> = if search.is_empty() {
                                            all_saft.into_iter().take(50).collect()
                                        } else {
                                            all_saft.into_iter()
                                                .filter(|acc| {
                                                    acc.code.to_lowercase().contains(&search) ||
                                                    acc.name.to_lowercase().contains(&search)
                                                })
                                                .take(50)
                                                .collect()
                                        };

                                        if filtered.is_empty() {
                                            view! {
                                                <div style="position: absolute; z-index: 1100; background: white; border: 1px solid #ddd; border-radius: 4px; width: 100%; box-shadow: 0 4px 12px rgba(0,0,0,0.15); margin-top: 2px;">
                                                    <div style="padding: 15px; text-align: center; color: #7f8c8d;">
                                                        "Няма намерени SAF-T сметки"
                                                    </div>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <div style="position: absolute; z-index: 1100; background: white; border: 1px solid #ddd; border-radius: 4px; width: 100%; max-height: 250px; overflow-y: auto; box-shadow: 0 4px 12px rgba(0,0,0,0.15); margin-top: 2px;">
                                                    {filtered.into_iter().map(|sa| {
                                                        let id = sa.id as i64;
                                                        let display = format!("{} - {}", sa.code, sa.name);
                                                        let display_click = display.clone();
                                                        view! {
                                                            <div
                                                                style="padding: 10px 12px; cursor: pointer; border-bottom: 1px solid #f0f0f0; font-size: 14px;"
                                                                on:mousedown=move |_| {
                                                                    form_saft_account_id.set(Some(id));
                                                                    form_saft_search.set(display_click.clone());
                                                                    form_saft_dropdown_open.set(false);
                                                                }
                                                            >
                                                                {display}
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_view()
                                        }
                                    }}
                                </div>

                                <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_modal.set(false)
                                    >
                                        "Отказ"
                                    </button>
                                    <button
                                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=save_account
                                        disabled=move || saving.get()
                                    >
                                        {move || if saving.get() { "Запазване..." } else { "Запази" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Delete confirmation modal
            {move || {
                if delete_account_id.get().is_some() {
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 400px; max-width: 90%;">
                                <h2 style="margin: 0 0 15px 0; color: #e74c3c;">"Изтриване на сметка"</h2>
                                <p style="margin-bottom: 20px; color: #7f8c8d;">
                                    "Сигурни ли сте, че искате да изтриете тази сметка? Това действие не може да бъде отменено."
                                </p>

                                <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| delete_account_id.set(None)
                                    >
                                        "Отказ"
                                    </button>
                                    <button
                                        style="background: #e74c3c; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=confirm_delete
                                        disabled=move || deleting.get()
                                    >
                                        {move || if deleting.get() { "Изтриване..." } else { "Изтрий" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
        </Layout>
    }
}
