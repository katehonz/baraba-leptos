use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_delete};
use crate::context::{use_company_context, use_user_context};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub legal_name: Option<String>,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub user_count: Option<i64>,
    // VAT submission fields
    pub manager_name: Option<String>,
    pub manager_egn: Option<String>,
    pub authorized_person_name: Option<String>,
    pub authorized_person_egn: Option<String>,
    pub representative_type: Option<String>,  // MANAGER or AUTHORIZED_PERSON
    pub vat_branch_number: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub pages: i64,
}

#[component]
pub fn Companies() -> impl IntoView {
    let company_ctx = use_company_context();
    let user_ctx = use_user_context();

    let companies = RwSignal::new(Vec::<Company>::new());
    let loading = RwSignal::new(true);
    let saving = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let picked_id = RwSignal::new(Option::<i64>::None);

    // Search and filter state
    let search_query = RwSignal::new(String::new());
    let city_filter = RwSignal::new(String::new());
    let pagination = RwSignal::new(PaginationMeta::default());
    let current_page = RwSignal::new(1i64);

    // Modal state
    let show_modal = RwSignal::new(false);
    let editing_id = RwSignal::new(Option::<i64>::None);

    // Form fields
    let form_name = RwSignal::new(String::new());
    let form_legal_name = RwSignal::new(String::new());
    let form_eik = RwSignal::new(String::new());
    let form_vat = RwSignal::new(String::new());
    let form_address = RwSignal::new(String::new());
    let form_city = RwSignal::new(String::new());
    let form_postal = RwSignal::new(String::new());
    let form_email = RwSignal::new(String::new());
    let form_phone = RwSignal::new(String::new());
    // VAT submission fields
    let form_manager_name = RwSignal::new(String::new());
    let form_manager_egn = RwSignal::new(String::new());
    let form_authorized_name = RwSignal::new(String::new());
    let form_authorized_egn = RwSignal::new(String::new());
    let form_representative_type = RwSignal::new("MANAGER".to_string());
    let form_vat_branch = RwSignal::new(String::new());

    // Load companies with filters. Super admins use the searchable list;
    // everyone else sees only firms they have access to.
    let load_companies = move || {
        if user_ctx.user.get().is_none() {
            return;
        }
        loading.set(true);
        let is_admin = user_ctx.is_super_admin();
        spawn_local(async move {
            if is_admin {
                let mut url = format!("/api/companies?page={}", current_page.get());

                let q = search_query.get();
                if !q.is_empty() {
                    url.push_str(&format!("&q={}", urlencoding::encode(&q)));
                }

                let city = city_filter.get();
                if !city.is_empty() {
                    url.push_str(&format!("&city={}", urlencoding::encode(&city)));
                }

                match api_get(&url).await {
                    Ok(data) => {
                        if data.get("success").and_then(|v| v.as_bool()) == Some(false) {
                            let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Грешка при зареждане");
                            error_msg.set(msg.to_string());
                        } else {
                            if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                                let items: Vec<Company> = arr.iter().filter_map(parse_company_json).collect();
                                companies.set(items);
                            }
                            if let Some(meta) = data.get("meta") {
                                if let Ok(m) = serde_json::from_value::<PaginationMeta>(meta.clone()) {
                                    pagination.set(m);
                                }
                            }
                        }
                    }
                    Err(e) => error_msg.set(e),
                }
            } else {
                match api_get("/api/my_companies").await {
                    Ok(data) => {
                        if data.get("success").and_then(|v| v.as_bool()) == Some(false) {
                            let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Грешка при зареждане");
                            error_msg.set(msg.to_string());
                        } else if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                            let q = search_query.get().to_lowercase();
                            let city = city_filter.get().to_lowercase();
                            let mut items: Vec<Company> = arr.iter().filter_map(parse_company_json).collect();
                            if !q.is_empty() {
                                items.retain(|c| {
                                    c.name.to_lowercase().contains(&q)
                                        || c.eik.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                        || c.vat_number.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                });
                            }
                            if !city.is_empty() {
                                items.retain(|c| {
                                    c.city.as_deref().unwrap_or("").to_lowercase().contains(&city)
                                });
                            }
                            let total = items.len() as i64;
                            pagination.set(PaginationMeta {
                                total,
                                page: 1,
                                per_page: total.max(1),
                                pages: 1,
                            });
                            companies.set(items);
                        }
                    }
                    Err(e) => error_msg.set(e),
                }
            }
            loading.set(false);
        });
    };

    // Search handler
    let do_search = move || {
        current_page.set(1);
        load_companies();
    };

    Effect::new(move |_| {
        if user_ctx.user.get().is_none() {
            return;
        }
        load_companies();
    });

    Effect::new(move |_| {
        if picked_id.get_untracked().is_none() {
            if let Some(id) = company_ctx.selected_company_id.get() {
                picked_id.set(Some(id));
            }
        }
    });

    let activate_company = move |id: i64| {
        if company_ctx.selected_company_id.get_untracked() == Some(id) {
            success_msg.set("Тази фирма вече е активна".to_string());
            return;
        }
        company_ctx.select_company(id);
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/dashboard");
        }
    };

    // Clear form
    let clear_form = move || {
        form_name.set(String::new());
        form_legal_name.set(String::new());
        form_eik.set(String::new());
        form_vat.set(String::new());
        form_address.set(String::new());
        form_city.set(String::new());
        form_postal.set(String::new());
        form_email.set(String::new());
        form_phone.set(String::new());
        form_manager_name.set(String::new());
        form_manager_egn.set(String::new());
        form_authorized_name.set(String::new());
        form_authorized_egn.set(String::new());
        form_representative_type.set("MANAGER".to_string());
        form_vat_branch.set(String::new());
        editing_id.set(None);
    };

    // Open modal for new company
    let open_new = move |_| {
        clear_form();
        show_modal.set(true);
    };

    // Open modal for edit
    let open_edit = move |company: Company| {
        editing_id.set(Some(company.id));
        form_name.set(company.name);
        form_legal_name.set(company.legal_name.unwrap_or_default());
        form_eik.set(company.eik.unwrap_or_default());
        form_vat.set(company.vat_number.unwrap_or_default());
        form_address.set(company.address.unwrap_or_default());
        form_city.set(company.city.unwrap_or_default());
        form_postal.set(company.postal_code.unwrap_or_default());
        form_email.set(company.email.unwrap_or_default());
        form_phone.set(company.phone.unwrap_or_default());
        form_manager_name.set(company.manager_name.unwrap_or_default());
        form_manager_egn.set(company.manager_egn.unwrap_or_default());
        form_authorized_name.set(company.authorized_person_name.unwrap_or_default());
        form_authorized_egn.set(company.authorized_person_egn.unwrap_or_default());
        form_representative_type.set(company.representative_type.unwrap_or_else(|| "MANAGER".to_string()));
        form_vat_branch.set(company.vat_branch_number.map(|n| n.to_string()).unwrap_or_default());
        show_modal.set(true);
    };

    // Save company
    let save_company = move |_| {
        let name = form_name.get();
        if name.is_empty() {
            error_msg.set("Името е задължително".to_string());
            return;
        }

        saving.set(true);
        let id = editing_id.get();

        spawn_local(async move {
            let vat_branch: Option<i32> = form_vat_branch.get().parse().ok();
            let body = serde_json::json!({
                "company": {
                    "name": form_name.get(),
                    "legal_name": form_legal_name.get(),
                    "eik": form_eik.get(),
                    "vat_number": form_vat.get(),
                    "address": form_address.get(),
                    "city": form_city.get(),
                    "post_code": form_postal.get(),
                    "email": form_email.get(),
                    "phone": form_phone.get(),
                    "manager_name": form_manager_name.get(),
                    "manager_egn": form_manager_egn.get(),
                    "authorized_person_name": form_authorized_name.get(),
                    "authorized_person_egn": form_authorized_egn.get(),
                    "representative_type": form_representative_type.get(),
                    "vat_branch_number": vat_branch,
                }
            });

            let url = if let Some(company_id) = id {
                format!("/api/companies/{}", company_id)
            } else {
                "/api/companies".to_string()
            };

            match api_post(&url, &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        show_modal.set(false);
                        clear_form();
                        success_msg.set(if id.is_some() { "Фирмата е обновена" } else { "Фирмата е създадена" }.to_string());
                        loading.set(true);
                        load_companies();
                    } else {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Грешка");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Delete company
    let delete_company = move |id: i64, name: String| {
        if !web_sys::window()
            .unwrap()
            .confirm_with_message(&format!("Сигурни ли сте, че искате да изтриете \"{}\"?", name))
            .unwrap_or(false)
        {
            return;
        }

        spawn_local(async move {
            match api_delete(&format!("/api/companies/{}", id)).await {
                Ok(_) => {
                    companies.update(|list| list.retain(|c| c.id != id));
                    success_msg.set("Фирмата е изтрита".to_string());
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Auto-hide messages
    Effect::new(move |_| {
        if !success_msg.get().is_empty() {
            set_timeout(move || success_msg.set(String::new()), std::time::Duration::from_secs(3));
        }
    });

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <div>
                    <h1 style="margin: 0;">"Фирми"</h1>
                    <p style="margin: 6px 0 0 0; color: #7f8c8d; font-size: 14px;">
                        "Потърсете фирма, изберете я и натиснете „Направи активна“."
                    </p>
                </div>
                {move || {
                    if user_ctx.is_super_admin() {
                        view! {
                            <button
                                style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                on:click=open_new
                            >
                                "+ Нова фирма"
                            </button>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>

            // Search and filters
            <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                <div style="display: flex; gap: 15px; flex-wrap: wrap; align-items: flex-end;">
                    // Search input
                    <div style="flex: 1; min-width: 250px;">
                        <label style="display: block; margin-bottom: 5px; font-size: 13px; color: #7f8c8d;">"Търсене"</label>
                        <input
                            type="text"
                            placeholder="Име, ЕИК, ДДС номер..."
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                            prop:value=move || search_query.get()
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                            on:keypress=move |ev| {
                                if ev.key() == "Enter" {
                                    do_search();
                                }
                            }
                        />
                    </div>

                    // City filter
                    <div style="min-width: 150px;">
                        <label style="display: block; margin-bottom: 5px; font-size: 13px; color: #7f8c8d;">"Град"</label>
                        <input
                            type="text"
                            placeholder="София..."
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                            prop:value=move || city_filter.get()
                            on:input=move |ev| city_filter.set(event_target_value(&ev))
                        />
                    </div>

                    // Search button
                    <button
                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=move |_| do_search()
                    >
                        "Търси"
                    </button>

                    // Clear filters button
                    <button
                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=move |_| {
                            search_query.set(String::new());
                            city_filter.set(String::new());
                            current_page.set(1);
                            load_companies();
                        }
                    >
                        "Изчисти"
                    </button>
                </div>

                // Results count
                {move || {
                    let meta = pagination.get();
                    if meta.total > 0 {
                        view! {
                            <p style="margin: 15px 0 0 0; color: #7f8c8d; font-size: 13px;">
                                {format!("Намерени: {} фирми", meta.total)}
                            </p>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>

            // Messages
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {err}
                            <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;" on:click=move |_| error_msg.set(String::new())>"×"</button>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {msg}
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Selected company action bar
            {move || {
                let id = picked_id.get();
                let Some(id) = id else {
                    return view! { <span></span> }.into_any();
                };
                let list = companies.get();
                let Some(company) = list.iter().find(|c| c.id == id).cloned() else {
                    return view! { <span></span> }.into_any();
                };
                let is_active = company_ctx.selected_company_id.get() == Some(company.id);
                let eik = company.eik.clone().unwrap_or_default();
                let name = company.name.clone();
                let company_id = company.id;

                view! {
                    <div style="background: #ecfdf5; border: 2px solid #10b981; border-radius: 10px; padding: 18px 22px; margin-top: 20px; display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
                        <div>
                            <p style="margin: 0 0 4px 0; font-size: 12px; font-weight: 600; color: #047857; text-transform: uppercase; letter-spacing: 0.04em;">
                                "Избрана фирма"
                            </p>
                            <p style="margin: 0; font-size: 20px; font-weight: 700; color: #064e3b;">{name}</p>
                            {if !eik.is_empty() {
                                view! { <p style="margin: 4px 0 0 0; color: #047857; font-size: 14px;">{format!("ЕИК: {}", eik)}</p> }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </div>
                        {if is_active {
                            view! {
                                <div style="background: #10b981; color: white; padding: 16px 28px; border-radius: 8px; font-size: 18px; font-weight: 700;">
                                    "Активна фирма"
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    style="background: #16a34a; color: white; border: none; padding: 18px 36px; border-radius: 10px; cursor: pointer; font-size: 20px; font-weight: 800; letter-spacing: 0.02em; box-shadow: 0 8px 20px rgba(22,163,74,0.35);"
                                    on:click=move |_| activate_company(company_id)
                                >
                                    "Направи активна"
                                </button>
                            }.into_any()
                        }}
                    </div>
                }.into_any()
            }}

            // Loading
            {move || {
                if loading.get() {
                    view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d;">"Зареждане..."</p>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Companies grid
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_any();
                }

                let list = companies.get();
                if list.is_empty() {
                    view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 60px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 18px; margin-bottom: 20px;">"Няма намерени фирми"</p>
                            {if user_ctx.is_super_admin() {
                                view! {
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; font-size: 16px;"
                                        on:click=open_new
                                    >
                                        "Създай първата фирма"
                                    </button>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </div>
                    }.into_any()
                } else {
                    let selected = picked_id.get();
                    let active_id = company_ctx.selected_company_id.get();
                    let is_admin = user_ctx.is_super_admin();
                    let meta = pagination.get();

                    view! {
                        <div>
                        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 20px; margin-top: 20px;">
                            {list.into_iter().map(|company| {
                                let company_for_edit = company.clone();
                                let company_id = company.id;
                                let company_name = company.name.clone();
                                let is_picked = selected == Some(company.id);
                                let is_active = active_id == Some(company.id);
                                let border = if is_picked {
                                    "2px solid #16a34a"
                                } else if is_active {
                                    "2px solid #93c5fd"
                                } else {
                                    "2px solid transparent"
                                };
                                let bg = if is_picked { "#f0fdf4" } else { "white" };

                                view! {
                                    <div
                                        style=format!("background: {}; border-radius: 8px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); border: {}; cursor: pointer;", bg, border)
                                        on:click=move |_| picked_id.set(Some(company_id))
                                    >
                                        <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 15px;">
                                            <div>
                                                <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 5px; flex-wrap: wrap;">
                                                    <h3 style="margin: 0; color: #2c3e50;">{company.name.clone()}</h3>
                                                    {if is_active {
                                                        view! {
                                                            <span style="background: #16a34a; color: white; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 700; text-transform: uppercase;">
                                                                "Активна"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                    <span style="background: #e8f4fd; color: #3498db; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-family: monospace;">
                                                        "ID: " {company.id}
                                                    </span>
                                                </div>
                                                {company.legal_name.as_ref().map(|ln| {
                                                    if !ln.is_empty() {
                                                        view! { <p style="margin: 0; color: #7f8c8d; font-size: 13px;">{ln.clone()}</p> }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }
                                                })}
                                            </div>
                                            {if is_admin {
                                                view! {
                                                    <div style="display: flex; gap: 5px;">
                                                        <button
                                                            style="background: #3498db; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                            title="Редактирай"
                                                            on:click=move |ev| {
                                                                ev.stop_propagation();
                                                                open_edit(company_for_edit.clone());
                                                            }
                                                        >
                                                            "✏️"
                                                        </button>
                                                        <button
                                                            style="background: #e74c3c; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                            title="Изтрий"
                                                            on:click=move |ev| {
                                                                ev.stop_propagation();
                                                                delete_company(company_id, company_name.clone());
                                                            }
                                                        >
                                                            "🗑️"
                                                        </button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                        </div>

                                        <div style="font-size: 14px; color: #34495e;">
                                            {company.eik.as_ref().map(|v| {
                                                if !v.is_empty() {
                                                    view! { <p style="margin: 5px 0;"><strong>"ЕИК: "</strong>{v.clone()}</p> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }
                                            })}
                                            {company.vat_number.as_ref().map(|v| {
                                                if !v.is_empty() {
                                                    view! { <p style="margin: 5px 0;"><strong>"ДДС: "</strong>{v.clone()}</p> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }
                                            })}
                                            {company.address.as_ref().map(|v| {
                                                if !v.is_empty() {
                                                    view! { <p style="margin: 5px 0;"><strong>"Адрес: "</strong>{v.clone()}</p> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }
                                            })}
                                            {company.email.as_ref().map(|v| {
                                                if !v.is_empty() {
                                                    view! { <p style="margin: 5px 0;"><strong>"Email: "</strong>{v.clone()}</p> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }
                                            })}
                                        </div>

                                        <div style="margin-top: 16px;">
                                            {if is_active {
                                                view! {
                                                    <div style="width: 100%; text-align: center; background: #dcfce7; color: #166534; padding: 14px 16px; border-radius: 8px; font-size: 16px; font-weight: 700; box-sizing: border-box;">
                                                        "Това е активната фирма"
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <button
                                                        style="width: 100%; background: #16a34a; color: white; border: none; padding: 14px 16px; border-radius: 8px; cursor: pointer; font-size: 16px; font-weight: 800;"
                                                        on:click=move |ev| {
                                                            ev.stop_propagation();
                                                            picked_id.set(Some(company_id));
                                                            activate_company(company_id);
                                                        }
                                                    >
                                                        "Направи активна"
                                                    </button>
                                                }.into_any()
                                            }}
                                        </div>

                                        {if is_admin {
                                            view! {
                                                <div style="margin-top: 15px; padding-top: 15px; border-top: 1px solid #ecf0f1;">
                                                    <a
                                                        href=format!("/settings?company={}", company.id)
                                                        style="color: #3498db; text-decoration: none; font-size: 13px;"
                                                        on:click=move |ev| ev.stop_propagation()
                                                    >
                                                        "⚙️ Настройки на фирмата"
                                                    </a>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                }
                            }).collect_view()}
                        </div>

                        {if meta.pages > 1 {
                            let current = current_page.get();
                            view! {
                                <div style="display: flex; justify-content: center; align-items: center; gap: 10px; padding: 24px 0 8px;">
                                    <button
                                        style="background: #3498db; color: white; border: none; padding: 8px 15px; border-radius: 4px; cursor: pointer;"
                                        disabled=move || current_page.get() <= 1
                                        on:click=move |_| {
                                            if current_page.get() > 1 {
                                                current_page.update(|p| *p -= 1);
                                                load_companies();
                                            }
                                        }
                                    >
                                        "« Предишна"
                                    </button>
                                    <span style="color: #7f8c8d;">
                                        {format!("Страница {} от {}", current, meta.pages)}
                                    </span>
                                    <button
                                        style="background: #3498db; color: white; border: none; padding: 8px 15px; border-radius: 4px; cursor: pointer;"
                                        disabled=move || current_page.get() >= pagination.get().pages
                                        on:click=move |_| {
                                            if current_page.get() < pagination.get().pages {
                                                current_page.update(|p| *p += 1);
                                                load_companies();
                                            }
                                        }
                                    >
                                        "Следваща »"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                        </div>
                    }.into_any()
                }
            }}

            // Modal
            {move || {
                if show_modal.get() {
                    let is_edit = editing_id.get().is_some();

                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 600px; max-height: 90vh; overflow-y: auto;">
                                <h2 style="margin: 0 0 20px 0;">
                                    {if is_edit { "Редактиране на фирма" } else { "Нова фирма" }}
                                </h2>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Име *"</label>
                                        <input
                                            type="text"
                                            placeholder="Кратко име на фирмата"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_name.get()
                                            on:input=move |ev| form_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Юридическо име"</label>
                                        <input
                                            type="text"
                                            placeholder="Пълно юридическо наименование"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_legal_name.get()
                                            on:input=move |ev| form_legal_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"ЕИК"</label>
                                        <input
                                            type="text"
                                            placeholder="123456789"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_eik.get()
                                            on:input=move |ev| form_eik.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"ДДС номер"</label>
                                        <input
                                            type="text"
                                            placeholder="BG123456789"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_vat.get()
                                            on:input=move |ev| form_vat.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Адрес"</label>
                                        <input
                                            type="text"
                                            placeholder="Улица, номер"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_address.get()
                                            on:input=move |ev| form_address.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Град"</label>
                                        <input
                                            type="text"
                                            placeholder="София"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_city.get()
                                            on:input=move |ev| form_city.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Пощенски код"</label>
                                        <input
                                            type="text"
                                            placeholder="1000"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_postal.get()
                                            on:input=move |ev| form_postal.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Email"</label>
                                        <input
                                            type="email"
                                            placeholder="office@company.bg"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_email.get()
                                            on:input=move |ev| form_email.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Телефон"</label>
                                        <input
                                            type="text"
                                            placeholder="+359 2 123 4567"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_phone.get()
                                            on:input=move |ev| form_phone.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>

                                // VAT Submission Section
                                <div style="margin-top: 25px; padding-top: 20px; border-top: 2px solid #ecf0f1;">
                                    <h3 style="margin: 0 0 15px 0; color: #2c3e50; font-size: 16px;">"📋 Данни за подаване на ДДС декларация"</h3>

                                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                        // Manager section
                                        <div style="grid-column: span 2; background: #f8f9fa; padding: 15px; border-radius: 6px;">
                                            <h4 style="margin: 0 0 12px 0; color: #34495e; font-size: 14px;">"Управител"</h4>
                                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-size: 13px;">"Име"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="Иван Петров Иванов"
                                                        style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || form_manager_name.get()
                                                        on:input=move |ev| form_manager_name.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-size: 13px;">"ЕГН"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="1234567890"
                                                        style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || form_manager_egn.get()
                                                        on:input=move |ev| form_manager_egn.set(event_target_value(&ev))
                                                    />
                                                </div>
                                            </div>
                                        </div>

                                        // Authorized person section
                                        <div style="grid-column: span 2; background: #f8f9fa; padding: 15px; border-radius: 6px;">
                                            <h4 style="margin: 0 0 12px 0; color: #34495e; font-size: 14px;">"Пълномощник"</h4>
                                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-size: 13px;">"Име"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="Мария Георгиева Петрова"
                                                        style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || form_authorized_name.get()
                                                        on:input=move |ev| form_authorized_name.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-size: 13px;">"ЕГН"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="0987654321"
                                                        style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || form_authorized_egn.get()
                                                        on:input=move |ev| form_authorized_egn.set(event_target_value(&ev))
                                                    />
                                                </div>
                                            </div>
                                        </div>

                                        // Representative type selector
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Лице, подаващо данните"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; background: white;"
                                                prop:value=move || form_representative_type.get()
                                                on:change=move |ev| form_representative_type.set(event_target_value(&ev))
                                            >
                                                <option value="MANAGER">"Управител"</option>
                                                <option value="AUTHORIZED_PERSON">"Пълномощник"</option>
                                            </select>
                                        </div>

                                        // Branch number
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Номер на поделение"</label>
                                            <input
                                                type="text"
                                                placeholder="0 (за централа)"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                prop:value=move || form_vat_branch.get()
                                                on:input=move |ev| form_vat_branch.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                </div>

                                <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 25px;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_modal.set(false)
                                    >
                                        "Отказ"
                                    </button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=save_company
                                        disabled=move || saving.get()
                                    >
                                        {move || if saving.get() { "Запазване..." } else if is_edit { "Запази" } else { "Създай" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
        </Layout>
    }
}

fn parse_company_json(v: &serde_json::Value) -> Option<Company> {
    if let Ok(c) = serde_json::from_value::<Company>(v.clone()) {
        return Some(c);
    }
    Some(Company {
        id: v.get("id")?.as_i64()?,
        name: v.get("name")?.as_str()?.to_string(),
        legal_name: v.get("legal_name").and_then(|x| x.as_str()).map(|s| s.to_string()),
        eik: v.get("eik").and_then(|x| x.as_str()).map(|s| s.to_string()),
        vat_number: v.get("vat_number").and_then(|x| x.as_str()).map(|s| s.to_string()),
        address: v.get("address").and_then(|x| x.as_str()).map(|s| s.to_string()),
        city: v.get("city").and_then(|x| x.as_str()).map(|s| s.to_string()),
        postal_code: v.get("postal_code").and_then(|x| x.as_str()).map(|s| s.to_string()),
        email: v.get("email").and_then(|x| x.as_str()).map(|s| s.to_string()),
        phone: v.get("phone").and_then(|x| x.as_str()).map(|s| s.to_string()),
        user_count: v.get("user_count").and_then(|x| x.as_i64()),
        ..Default::default()
    })
}
