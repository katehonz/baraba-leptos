use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_delete};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    let companies = create_rw_signal(Vec::<Company>::new());
    let loading = create_rw_signal(true);
    let saving = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // Search and filter state
    let search_query = create_rw_signal(String::new());
    let city_filter = create_rw_signal(String::new());
    let pagination = create_rw_signal(PaginationMeta::default());
    let current_page = create_rw_signal(1i64);

    // Modal state
    let show_modal = create_rw_signal(false);
    let editing_id = create_rw_signal(Option::<i64>::None);

    // Form fields
    let form_name = create_rw_signal(String::new());
    let form_legal_name = create_rw_signal(String::new());
    let form_eik = create_rw_signal(String::new());
    let form_vat = create_rw_signal(String::new());
    let form_address = create_rw_signal(String::new());
    let form_city = create_rw_signal(String::new());
    let form_postal = create_rw_signal(String::new());
    let form_email = create_rw_signal(String::new());
    let form_phone = create_rw_signal(String::new());
    // VAT submission fields
    let form_manager_name = create_rw_signal(String::new());
    let form_manager_egn = create_rw_signal(String::new());
    let form_authorized_name = create_rw_signal(String::new());
    let form_authorized_egn = create_rw_signal(String::new());
    let form_representative_type = create_rw_signal("MANAGER".to_string());
    let form_vat_branch = create_rw_signal(String::new());

    // Load companies with filters
    let load_companies = move || {
        loading.set(true);
        spawn_local(async move {
            let mut url = format!("/api/companies?page={}", current_page.get());

            let q = search_query.get();
            if !q.is_empty() {
                url.push_str(&format!("&q={}", q));
            }

            let city = city_filter.get();
            if !city.is_empty() {
                url.push_str(&format!("&city={}", city));
            }

            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Company> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        companies.set(items);
                    }
                    if let Some(meta) = data.get("meta") {
                        if let Ok(m) = serde_json::from_value::<PaginationMeta>(meta.clone()) {
                            pagination.set(m);
                        }
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            loading.set(false);
        });
    };

    // Search handler
    let do_search = move || {
        current_page.set(1);
        load_companies();
    };

    create_effect(move |_| {
        load_companies();
    });

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
    create_effect(move |_| {
        if !success_msg.get().is_empty() {
            set_timeout(move || success_msg.set(String::new()), std::time::Duration::from_secs(3));
        }
    });

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Фирми"</h1>
                <button
                    style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                    on:click=open_new
                >
                    "+ Нова фирма"
                </button>
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
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
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
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {msg}
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Loading
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

            // Companies grid
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                let list = companies.get();
                if list.is_empty() {
                    view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 60px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 18px; margin-bottom: 20px;">"Няма създадени фирми"</p>
                            <button
                                style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; font-size: 16px;"
                                on:click=open_new
                            >
                                "Създай първата фирма"
                            </button>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 20px; margin-top: 20px;">
                            {list.into_iter().map(|company| {
                                let company_for_edit = company.clone();
                                let company_id = company.id;
                                let company_name = company.name.clone();

                                view! {
                                    <div style="background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                        <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 15px;">
                                            <div>
                                                <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 5px;">
                                                    <h3 style="margin: 0; color: #2c3e50;">{&company.name}</h3>
                                                    <span style="background: #e8f4fd; color: #3498db; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-family: monospace;">
                                                        "ID: " {company.id}
                                                    </span>
                                                </div>
                                                {company.legal_name.as_ref().map(|ln| {
                                                    if !ln.is_empty() {
                                                        view! { <p style="margin: 0; color: #7f8c8d; font-size: 13px;">{ln}</p> }.into_view()
                                                    } else {
                                                        view! { <span></span> }.into_view()
                                                    }
                                                })}
                                            </div>
                                            <div style="display: flex; gap: 5px;">
                                                <button
                                                    style="background: #3498db; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                    title="Редактирай"
                                                    on:click=move |_| open_edit(company_for_edit.clone())
                                                >
                                                    "✏️"
                                                </button>
                                                <button
                                                    style="background: #e74c3c; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                    title="Изтрий"
                                                    on:click=move |_| delete_company(company_id, company_name.clone())
                                                >
                                                    "🗑️"
                                                </button>
                                            </div>
                                        </div>

                                        <div style="font-size: 14px; color: #34495e;">
                                            {company.eik.as_ref().map(|v| {
                                                if !v.is_empty() {
                                                    view! { <p style="margin: 5px 0;"><strong>"ЕИК: "</strong>{v}</p> }.into_view()
                                                } else {
                                                    view! { <span></span> }.into_view()
                                                }
                                            })}
                                            {company.vat_number.as_ref().map(|v| {
                                                if !v.is_empty() {
                                                    view! { <p style="margin: 5px 0;"><strong>"ДДС: "</strong>{v}</p> }.into_view()
                                                } else {
                                                    view! { <span></span> }.into_view()
                                                }
                                            })}
                                            {company.address.as_ref().map(|v| {
                                                if !v.is_empty() {
                                                    view! { <p style="margin: 5px 0;"><strong>"Адрес: "</strong>{v}</p> }.into_view()
                                                } else {
                                                    view! { <span></span> }.into_view()
                                                }
                                            })}
                                            {company.email.as_ref().map(|v| {
                                                if !v.is_empty() {
                                                    view! { <p style="margin: 5px 0;"><strong>"Email: "</strong>{v}</p> }.into_view()
                                                } else {
                                                    view! { <span></span> }.into_view()
                                                }
                                            })}
                                        </div>

                                        <div style="margin-top: 15px; padding-top: 15px; border-top: 1px solid #ecf0f1;">
                                            <a
                                                href=format!("/settings?company={}", company.id)
                                                style="color: #3498db; text-decoration: none; font-size: 13px;"
                                            >
                                                "⚙️ Настройки на фирмата"
                                            </a>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
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
                                            placeholder="BG000000000"
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
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
        </Layout>
    }
}
