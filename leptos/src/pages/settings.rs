use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_put, api_post, api_delete, company_api_url, get_selected_company_id};
use crate::context::use_user_context;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BeneficialOwner {
    pub id: Option<i64>,
    pub first_name_bg: String,
    pub last_name_bg: String,
    pub egn: Option<String>,
    pub first_name_latin: Option<String>,
    pub last_name_latin: Option<String>,
    pub country: String,
    pub ownership_percentage: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UltimateParent {
    pub id: Option<i64>,
    pub name_bg: String,
    pub name_latin: Option<String>,
    pub uic: Option<String>,
    pub country: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompanyLocation {
    pub id: Option<i64>,
    pub name: String,
    pub location_type: String,
    pub street_name: Option<String>,
    pub building_number: Option<String>,
    pub city: Option<String>,
    pub post_code: Option<String>,
    pub region: Option<String>,
    pub country: String,
    pub is_main: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompanyUser {
    pub id: i64,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_super_admin: bool,
    pub role: String,
    pub role_id: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[component]
pub fn Settings() -> impl IntoView {
    let active_tab = create_rw_signal(0); // 0=Company, 1=Accounts, 2=SAFt, 3=Features, 4=Integrations
    let loading = create_rw_signal(true);
    let saving = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // Company settings
    let company_name = create_rw_signal(String::new());
    let company_legal_name = create_rw_signal(String::new());
    let company_eik = create_rw_signal(String::new());
    let company_vat = create_rw_signal(String::new());
    let company_address = create_rw_signal(String::new());
    let company_city = create_rw_signal(String::new());
    let company_postal = create_rw_signal(String::new());
    let company_phone = create_rw_signal(String::new());
    let company_email = create_rw_signal(String::new());

    // VAT submission fields
    let manager_name = create_rw_signal(String::new());
    let manager_egn = create_rw_signal(String::new());
    let authorized_name = create_rw_signal(String::new());
    let authorized_egn = create_rw_signal(String::new());
    let representative_type = create_rw_signal("MANAGER".to_string());
    let vat_branch = create_rw_signal(String::new());

    // Default accounts
    let accounts = create_rw_signal(Vec::<Account>::new());
    let cash_account = create_rw_signal(0i64);
    let bank_account = create_rw_signal(0i64);
    let customers_account = create_rw_signal(0i64);
    let suppliers_account = create_rw_signal(0i64);
    let vat_payable_account = create_rw_signal(0i64);
    let vat_receivable_account = create_rw_signal(0i64);
    let revenues_account = create_rw_signal(0i64);
    let expenses_account = create_rw_signal(0i64);

    // SAFt fields
    let street_name = create_rw_signal(String::new());
    let building_number = create_rw_signal(String::new());
    let region = create_rw_signal(String::new());
    let tax_accounting_basis = create_rw_signal("A".to_string());
    let inventory_valuation_method = create_rw_signal("FIFO".to_string());
    let is_part_of_group = create_rw_signal(String::new());
    // Multiple beneficial owners
    let beneficial_owners = create_rw_signal(Vec::<BeneficialOwner>::new());
    // Multiple ultimate parents
    let ultimate_parents = create_rw_signal(Vec::<UltimateParent>::new());
    // Multiple company locations
    let company_locations = create_rw_signal(Vec::<CompanyLocation>::new());

    // Features
    let vies_validation = create_rw_signal(false);
    let ai_scanning = create_rw_signal(false);

    // Integrations (company-specific)
    let mistral_enabled = create_rw_signal(false);
    let mistral_api_key = create_rw_signal(String::new());
    let s3_enabled = create_rw_signal(false);
    let s3_bucket = create_rw_signal(String::new());

    // Colleagues (company users)
    let colleagues = create_rw_signal(Vec::<CompanyUser>::new());
    let available_roles = create_rw_signal(Vec::<Role>::new());
    let new_user_email = create_rw_signal(String::new());
    let new_user_role = create_rw_signal(String::new());
    let user_ctx = use_user_context();

    // Load settings
    create_effect(move |_| {
        spawn_local(async move {
            // Load company
            match api_get(&company_api_url()).await {
                Ok(data) => {
                    if let Some(company) = data.get("data") {
                        company_name.set(company.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        company_legal_name.set(company.get("legal_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        company_eik.set(company.get("eik").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        company_vat.set(company.get("vat_number").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        company_address.set(company.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        company_city.set(company.get("city").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        company_postal.set(company.get("postal_code").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        company_phone.set(company.get("phone").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        company_email.set(company.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string());

                        // VAT submission fields
                        manager_name.set(company.get("manager_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        manager_egn.set(company.get("manager_egn").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        authorized_name.set(company.get("authorized_person_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        authorized_egn.set(company.get("authorized_person_egn").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        representative_type.set(company.get("representative_type").and_then(|v| v.as_str()).unwrap_or("MANAGER").to_string());
                        vat_branch.set(company.get("vat_branch_number").and_then(|v| v.as_i64()).map(|n| n.to_string()).unwrap_or_default());

                        // Default accounts
                        if let Some(id) = company.get("cash_account_id").and_then(|v| v.as_i64()) {
                            cash_account.set(id);
                        }
                        if let Some(id) = company.get("bank_account_id").and_then(|v| v.as_i64()) {
                            bank_account.set(id);
                        }
                        if let Some(id) = company.get("customers_account_id").and_then(|v| v.as_i64()) {
                            customers_account.set(id);
                        }
                        if let Some(id) = company.get("suppliers_account_id").and_then(|v| v.as_i64()) {
                            suppliers_account.set(id);
                        }
                        if let Some(id) = company.get("vat_payable_account_id").and_then(|v| v.as_i64()) {
                            vat_payable_account.set(id);
                        }
                        if let Some(id) = company.get("vat_receivable_account_id").and_then(|v| v.as_i64()) {
                            vat_receivable_account.set(id);
                        }
                        if let Some(id) = company.get("revenues_account_id").and_then(|v| v.as_i64()) {
                            revenues_account.set(id);
                        }
                        if let Some(id) = company.get("expenses_account_id").and_then(|v| v.as_i64()) {
                            expenses_account.set(id);
                        }

                        // SAFt fields
                        street_name.set(company.get("street_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        building_number.set(company.get("building_number").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        region.set(company.get("region").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        tax_accounting_basis.set(company.get("tax_accounting_basis").and_then(|v| v.as_str()).unwrap_or("A").to_string());
                        inventory_valuation_method.set(company.get("inventory_valuation_method").and_then(|v| v.as_str()).unwrap_or("FIFO").to_string());
                        is_part_of_group.set(company.get("is_part_of_group").and_then(|v| v.as_str()).unwrap_or("").to_string());

                        // Features & Integrations from settings
                        if let Some(settings) = company.get("settings") {
                            if let Some(vies) = settings.get("vies") {
                                vies_validation.set(vies.get("validation_enabled").and_then(|v| v.as_bool()).unwrap_or(false));
                            }
                            if let Some(ai) = settings.get("ai_scanning") {
                                ai_scanning.set(ai.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false));
                            }
                            if let Some(mistral) = settings.get("mistral") {
                                mistral_enabled.set(mistral.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false));
                                mistral_api_key.set(mistral.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string());
                            }
                            if let Some(s3) = settings.get("s3") {
                                s3_enabled.set(s3.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false));
                                s3_bucket.set(s3.get("bucket").and_then(|v| v.as_str()).unwrap_or("").to_string());
                            }
                        }
                    }
                }
                Err(e) => error_msg.set(format!("Грешка при зареждане: {}", e)),
            }

            // Load ALL accounts for dropdowns
            match api_get(&format!("{}/accounts?per_page=1000", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Account> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        accounts.set(items);
                    }
                }
                _ => {}
            }

            // Load beneficial owners
            match api_get(&format!("{}/beneficial_owners", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<BeneficialOwner> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        beneficial_owners.set(items);
                    }
                }
                _ => {}
            }

            // Load ultimate parents
            match api_get(&format!("{}/ultimate_parents", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<UltimateParent> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        ultimate_parents.set(items);
                    }
                }
                _ => {}
            }

            // Load company locations
            match api_get(&format!("{}/locations", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<CompanyLocation> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        company_locations.set(items);
                    }
                }
                _ => {}
            }

            // Load company users (colleagues)
            if let Some(company_id) = get_selected_company_id() {
                match api_get(&format!("/api/companies/{}/users", company_id)).await {
                    Ok(data) => {
                        if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                            let items: Vec<CompanyUser> = arr.iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect();
                            colleagues.set(items);
                        }
                    }
                    _ => {}
                }
            }

            // Load available roles
            match api_get("/api/roles").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Role> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        available_roles.set(items);
                    }
                }
                _ => {}
            }

            loading.set(false);
        });
    });

    // Save company settings
    let save_company = move |_| {
        saving.set(true);
        spawn_local(async move {
            let vat_branch_num: Option<i32> = vat_branch.get().parse().ok();
            let body = serde_json::json!({
                "company": {
                    "name": company_name.get(),
                    "legal_name": company_legal_name.get(),
                    "eik": company_eik.get(),
                    "vat_number": company_vat.get(),
                    "address": company_address.get(),
                    "city": company_city.get(),
                    "postal_code": company_postal.get(),
                    "phone": company_phone.get(),
                    "email": company_email.get(),
                    "manager_name": manager_name.get(),
                    "manager_egn": manager_egn.get(),
                    "authorized_person_name": authorized_name.get(),
                    "authorized_person_egn": authorized_egn.get(),
                    "representative_type": representative_type.get(),
                    "vat_branch_number": vat_branch_num,
                }
            });

            match api_put(&company_api_url(), &body).await {
                Ok(_) => success_msg.set("Настройките са запазени".to_string()),
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Save default accounts
    let save_accounts = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "company": {
                    "cash_account_id": if cash_account.get() == 0 { serde_json::Value::Null } else { serde_json::json!(cash_account.get()) },
                    "bank_account_id": if bank_account.get() == 0 { serde_json::Value::Null } else { serde_json::json!(bank_account.get()) },
                    "customers_account_id": if customers_account.get() == 0 { serde_json::Value::Null } else { serde_json::json!(customers_account.get()) },
                    "suppliers_account_id": if suppliers_account.get() == 0 { serde_json::Value::Null } else { serde_json::json!(suppliers_account.get()) },
                    "vat_payable_account_id": if vat_payable_account.get() == 0 { serde_json::Value::Null } else { serde_json::json!(vat_payable_account.get()) },
                    "vat_receivable_account_id": if vat_receivable_account.get() == 0 { serde_json::Value::Null } else { serde_json::json!(vat_receivable_account.get()) },
                    "revenues_account_id": if revenues_account.get() == 0 { serde_json::Value::Null } else { serde_json::json!(revenues_account.get()) },
                    "expenses_account_id": if expenses_account.get() == 0 { serde_json::Value::Null } else { serde_json::json!(expenses_account.get()) },
                }
            });

            match api_put(&company_api_url(), &body).await {
                Ok(_) => success_msg.set("Сметките са запазени".to_string()),
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Save features
    let save_features = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "settings": {
                    "vies": {
                        "validation_enabled": vies_validation.get()
                    },
                    "ai_scanning": {
                        "enabled": ai_scanning.get()
                    }
                }
            });

            match api_put(&format!("{}/settings", company_api_url()), &body).await {
                Ok(_) => success_msg.set("Функциите са запазени".to_string()),
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Save SAFt settings (company-level only, owners are saved separately)
    let save_saft = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "company": {
                    "street_name": street_name.get(),
                    "building_number": building_number.get(),
                    "region": region.get(),
                    "tax_accounting_basis": tax_accounting_basis.get(),
                    "inventory_valuation_method": inventory_valuation_method.get(),
                    "is_part_of_group": if is_part_of_group.get().is_empty() { serde_json::Value::Null } else { serde_json::json!(is_part_of_group.get()) },
                }
            });

            match api_put(&company_api_url(), &body).await {
                Ok(_) => success_msg.set("SAFt настройките са запазени".to_string()),
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Add beneficial owner
    let add_beneficial_owner = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "beneficial_owner": {
                    "first_name_bg": "Име",
                    "last_name_bg": "Фамилия",
                    "country": "BG"
                }
            });
            match api_post(&format!("{}/beneficial_owners", company_api_url()), &body).await {
                Ok(data) => {
                    if let Some(owner) = data.get("data") {
                        if let Ok(new_owner) = serde_json::from_value::<BeneficialOwner>(owner.clone()) {
                            beneficial_owners.update(|list| list.push(new_owner));
                        }
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Add ultimate parent
    let add_ultimate_parent = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "ultimate_parent": {
                    "name_bg": "Ново предприятие",
                    "country": "BG"
                }
            });
            match api_post(&format!("{}/ultimate_parents", company_api_url()), &body).await {
                Ok(data) => {
                    if let Some(parent) = data.get("data") {
                        if let Ok(new_parent) = serde_json::from_value::<UltimateParent>(parent.clone()) {
                            ultimate_parents.update(|list| list.push(new_parent));
                        }
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Add company location
    let add_company_location = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "company_location": {
                    "name": "Нов обект",
                    "location_type": "OFFICE",
                    "country": "BG"
                }
            });
            match api_post(&format!("{}/locations", company_api_url()), &body).await {
                Ok(data) => {
                    if let Some(loc) = data.get("data") {
                        if let Ok(new_loc) = serde_json::from_value::<CompanyLocation>(loc.clone()) {
                            company_locations.update(|list| list.push(new_loc));
                        }
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Save integrations
    let save_integrations = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "settings": {
                    "mistral": {
                        "enabled": mistral_enabled.get(),
                        "api_key": mistral_api_key.get(),
                    },
                    "s3": {
                        "enabled": s3_enabled.get(),
                        "bucket": s3_bucket.get(),
                    }
                }
            });

            match api_put(&format!("{}/settings", company_api_url()), &body).await {
                Ok(_) => success_msg.set("Интеграциите са запазени".to_string()),
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Auto-hide messages
    create_effect(move |_| {
        if !success_msg.get().is_empty() {
            set_timeout(move || success_msg.set(String::new()), std::time::Duration::from_secs(3));
        }
    });

    // Searchable account selector component
    let account_select = move |label: &'static str, signal: RwSignal<i64>| {
        let search_text = create_rw_signal(String::new());
        let show_dropdown = create_rw_signal(false);

        // Initialize search text from currently selected account
        create_effect(move |_| {
            let id = signal.get();
            if id != 0 {
                if let Some(acc) = accounts.get().iter().find(|a| a.id == id) {
                    let display = format!("{} - {}", acc.code, acc.name);
                    if search_text.get() != display {
                        search_text.set(display);
                    }
                }
            }
        });

        view! {
            <div style="margin-bottom: 15px; position: relative;">
                <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">{label}</label>
                <div style="position: relative;">
                    <input
                        type="text"
                        placeholder="Търсене по код или име..."
                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                        prop:value=move || search_text.get()
                        on:input=move |ev| {
                            search_text.set(event_target_value(&ev));
                            signal.set(0);
                            show_dropdown.set(true);
                        }
                        on:focus=move |_| show_dropdown.set(true)
                        on:blur=move |_| {
                            // Delay to allow mousedown on dropdown item to fire first
                            set_timeout(move || show_dropdown.set(false), std::time::Duration::from_millis(200));
                        }
                    />
                    {move || if signal.get() != 0 {
                        view! {
                            <button
                                style="position: absolute; right: 8px; top: 50%; transform: translateY(-50%); background: none; border: none; cursor: pointer; color: #999; font-size: 16px; padding: 4px;"
                                on:click=move |_| {
                                    signal.set(0);
                                    search_text.set(String::new());
                                }
                            >"×"</button>
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    }}
                </div>
                {move || if show_dropdown.get() && signal.get() == 0 {
                    let search = search_text.get().to_lowercase();
                    let filtered: Vec<_> = if search.is_empty() {
                        // Show all accounts when no search
                        accounts.get().into_iter().collect()
                    } else {
                        // Filter by search term
                        accounts.get().into_iter()
                            .filter(|acc| {
                                acc.code.to_lowercase().contains(&search) ||
                                acc.name.to_lowercase().contains(&search)
                            })
                            .collect()
                    };

                    if filtered.is_empty() {
                        view! { <div style="position: absolute; z-index: 100; background: white; border: 1px solid #ddd; border-radius: 4px; width: 100%; box-shadow: 0 4px 12px rgba(0,0,0,0.15); margin-top: 2px;">
                            <div style="padding: 15px; text-align: center; color: #7f8c8d;">"Няма намерени сметки"</div>
                        </div> }.into_view()
                    } else {
                        view! {
                            <div style="position: absolute; z-index: 100; background: white; border: 1px solid #ddd; border-radius: 4px; width: 100%; max-height: 250px; overflow-y: auto; box-shadow: 0 4px 12px rgba(0,0,0,0.15); margin-top: 2px;">
                                {filtered.into_iter().map(|acc| {
                                    let id = acc.id;
                                    let display = format!("{} - {}", acc.code, acc.name);
                                    let display_click = display.clone();
                                    view! {
                                        <div
                                            class="account-option"
                                            style="padding: 10px 12px; cursor: pointer; border-bottom: 1px solid #f0f0f0; font-size: 14px;"
                                            on:mousedown=move |_| {
                                                signal.set(id);
                                                search_text.set(display_click.clone());
                                                show_dropdown.set(false);
                                            }
                                        >
                                            {display}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_view()
                    }
                } else {
                    view! { <div></div> }.into_view()
                }}
            </div>
        }
    };

    // Memos for item counts (prevents full DOM re-render on field edit)
    let loc_count = create_memo(move |_| company_locations.with(|l| l.len()));
    let owner_count = create_memo(move |_| beneficial_owners.with(|l| l.len()));
    let parent_count = create_memo(move |_| ultimate_parents.with(|l| l.len()));

    view! {
        <Layout>
            <style>".account-option:hover { background: #f0f4f8 !important; }"</style>
            <h1>"Настройки на фирмата"</h1>

            // Tabs - show Колеги only for admin/super_admin
            <div style="display: flex; gap: 0; margin-top: 20px; border-bottom: 2px solid #3498db;">
                {move || {
                    let company_id = get_selected_company_id().unwrap_or(0);
                    let is_admin = user_ctx.is_admin_for_company(company_id);

                    let mut tabs = vec![
                        (0, "Фирма"),
                        (1, "Сметки"),
                        (2, "SAFt"),
                        (3, "Функции"),
                        (4, "Интеграции"),
                    ];

                    if is_admin {
                        tabs.push((5, "Колеги"));
                    }

                    tabs.into_iter().map(|(idx, label)| {
                        view! {
                            <button
                                style=move || format!(
                                    "padding: 12px 24px; border: none; cursor: pointer; font-size: 14px; font-weight: 500; transition: all 0.2s; {}",
                                    if active_tab.get() == idx {
                                        "background: #3498db; color: white; border-radius: 4px 4px 0 0;"
                                    } else {
                                        "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                                    }
                                )
                                on:click=move |_| active_tab.set(idx)
                            >
                                {label}
                            </button>
                        }
                    }).collect_view()
                }}
            </div>

            // Messages
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {err}
                            <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;" on:click=move |_| error_msg.set(String::new())>"x"</button>
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

            // Tab content
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                match active_tab.get() {
                    0 => {
                        // Company settings
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 700px;">
                                <h3 style="margin: 0 0 20px 0; color: #2c3e50;">"Данни за фирмата"</h3>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Име *"</label>
                                        <input
                                            type="text"
                                            placeholder="Кратко име на фирмата"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_name.get()
                                            on:input=move |ev| company_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Юридическо име"</label>
                                        <input
                                            type="text"
                                            placeholder="Пълно юридическо наименование"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_legal_name.get()
                                            on:input=move |ev| company_legal_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"ЕИК"</label>
                                        <input
                                            type="text"
                                            placeholder="123456789"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_eik.get()
                                            on:input=move |ev| company_eik.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"ДДС номер"</label>
                                        <input
                                            type="text"
                                            placeholder="BG000000000"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_vat.get()
                                            on:input=move |ev| company_vat.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Адрес"</label>
                                        <input
                                            type="text"
                                            placeholder="Улица, номер"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_address.get()
                                            on:input=move |ev| company_address.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Град"</label>
                                        <input
                                            type="text"
                                            placeholder="София"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_city.get()
                                            on:input=move |ev| company_city.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Пощенски код"</label>
                                        <input
                                            type="text"
                                            placeholder="1000"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_postal.get()
                                            on:input=move |ev| company_postal.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Email"</label>
                                        <input
                                            type="email"
                                            placeholder="office@company.bg"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_email.get()
                                            on:input=move |ev| company_email.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Телефон"</label>
                                        <input
                                            type="text"
                                            placeholder="+359 2 123 4567"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || company_phone.get()
                                            on:input=move |ev| company_phone.set(event_target_value(&ev))
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
                                                        prop:value=move || manager_name.get()
                                                        on:input=move |ev| manager_name.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-size: 13px;">"ЕГН"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="1234567890"
                                                        style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || manager_egn.get()
                                                        on:input=move |ev| manager_egn.set(event_target_value(&ev))
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
                                                        prop:value=move || authorized_name.get()
                                                        on:input=move |ev| authorized_name.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-size: 13px;">"ЕГН"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="0987654321"
                                                        style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || authorized_egn.get()
                                                        on:input=move |ev| authorized_egn.set(event_target_value(&ev))
                                                    />
                                                </div>
                                            </div>
                                        </div>

                                        // Representative type selector
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Лице, подаващо данните"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; background: white;"
                                                prop:value=move || representative_type.get()
                                                on:change=move |ev| representative_type.set(event_target_value(&ev))
                                            >
                                                <option value="MANAGER" selected=move || representative_type.get() == "MANAGER">"Управител"</option>
                                                <option value="AUTHORIZED_PERSON" selected=move || representative_type.get() == "AUTHORIZED_PERSON">"Пълномощник"</option>
                                            </select>
                                        </div>

                                        // Branch number
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Номер на поделение"</label>
                                            <input
                                                type="text"
                                                placeholder="0 (за централа)"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                prop:value=move || vat_branch.get()
                                                on:input=move |ev| vat_branch.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                </div>

                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; margin-top: 25px;"
                                    on:click=save_company
                                    disabled=move || saving.get()
                                >
                                    {move || if saving.get() { "Запазване..." } else { "Запази настройките" }}
                                </button>
                            </div>
                        }.into_view()
                    }
                    1 => {
                        // Default accounts
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 700px;">
                                <h3 style="margin: 0 0 5px 0; color: #2c3e50;">"Сметки по подразбиране"</h3>
                                <p style="color: #7f8c8d; margin: 0 0 25px 0; font-size: 14px;">
                                    "Тези сметки ще се използват автоматично при създаване на документи"
                                </p>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                                    <div>{account_select("Каса", cash_account)}</div>
                                    <div>{account_select("Банка", bank_account)}</div>
                                    <div>{account_select("Вземания от клиенти", customers_account)}</div>
                                    <div>{account_select("Задължения към доставчици", suppliers_account)}</div>
                                    <div>{account_select("ДДС за плащане", vat_payable_account)}</div>
                                    <div>{account_select("ДДС за възстановяване", vat_receivable_account)}</div>
                                    <div>{account_select("Приходи", revenues_account)}</div>
                                    <div>{account_select("Разходи", expenses_account)}</div>
                                </div>

                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; margin-top: 25px;"
                                    on:click=save_accounts
                                    disabled=move || saving.get()
                                >
                                    {move || if saving.get() { "Запазване..." } else { "Запази сметките" }}
                                </button>
                            </div>
                        }.into_view()
                    }
                    2 => {
                        // SAFt settings
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 800px;">
                                <h3 style="margin: 0 0 5px 0; color: #2c3e50;">"SAFt настройки"</h3>
                                <p style="color: #7f8c8d; margin: 0 0 25px 0; font-size: 14px;">
                                    "Данни за SAF-T BG експорт съгласно българската схема"
                                </p>

                                // Company locations section (multiple)
                                <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                        <div>
                                            <h4 style="margin: 0 0 5px 0; color: #34495e;">"Търговски обекти и локации"</h4>
                                            <p style="color: #7f8c8d; margin: 0; font-size: 12px;">
                                                "Седалище, магазини, офиси, складове, клонове"
                                            </p>
                                        </div>
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 8px 15px; border-radius: 4px; cursor: pointer; font-size: 13px;"
                                            on:click=add_company_location
                                        >
                                            "+ Добави локация"
                                        </button>
                                    </div>

                                    {move || {
                                        let count = loc_count.get();
                                        if count == 0 {
                                            view! {
                                                <p style="color: #7f8c8d; font-size: 13px; text-align: center; padding: 20px;">
                                                    "Няма добавени локации. Добавете седалище и търговски обекти."
                                                </p>
                                            }.into_view()
                                        } else {
                                            (0..count).map(|idx| {
                                                let init_name = company_locations.with(|l| l.get(idx).map(|v| v.name.clone()).unwrap_or_default());
                                                let init_type = company_locations.with(|l| l.get(idx).map(|v| v.location_type.clone()).unwrap_or("OFFICE".to_string()));
                                                let init_street = company_locations.with(|l| l.get(idx).and_then(|v| v.street_name.clone()).unwrap_or_default());
                                                let init_building = company_locations.with(|l| l.get(idx).and_then(|v| v.building_number.clone()).unwrap_or_default());
                                                let init_city = company_locations.with(|l| l.get(idx).and_then(|v| v.city.clone()).unwrap_or_default());
                                                let init_post_code = company_locations.with(|l| l.get(idx).and_then(|v| v.post_code.clone()).unwrap_or_default());
                                                let init_region = company_locations.with(|l| l.get(idx).and_then(|v| v.region.clone()).unwrap_or_default());
                                                let is_main = company_locations.with(|l| l.get(idx).map(|v| v.is_main).unwrap_or(false));
                                                view! {
                                                    <div style="background: white; padding: 15px; border-radius: 6px; margin-bottom: 10px; border: 1px solid #ddd;">
                                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                                                            <div style="display: flex; align-items: center; gap: 10px;">
                                                                <span style="font-weight: 600; color: #34495e; font-size: 14px;">
                                                                    {format!("Локация #{}", idx + 1)}
                                                                </span>
                                                                {if is_main {
                                                                    view! {
                                                                        <span style="background: #27ae60; color: white; padding: 2px 8px; border-radius: 4px; font-size: 11px;">
                                                                            "Седалище"
                                                                        </span>
                                                                    }.into_view()
                                                                } else {
                                                                    view! { <span></span> }.into_view()
                                                                }}
                                                            </div>
                                                            <button
                                                                style="background: #e74c3c; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                                on:click=move |_| {
                                                                    if let Some(id) = company_locations.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                        spawn_local(async move {
                                                                            let _ = api_delete(&format!("{}/locations/{}", company_api_url(), id)).await;
                                                                            company_locations.update(|list| list.retain(|l| l.id != Some(id)));
                                                                        });
                                                                    }
                                                                }
                                                            >
                                                                "Изтрий"
                                                            </button>
                                                        </div>
                                                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px;">
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Наименование"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_name
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = company_locations.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = val.clone();
                                                                            company_locations.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.name = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/locations/{}", company_api_url(), id), &serde_json::json!({"company_location": {"name": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Тип"</label>
                                                                <select
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px; background: white;"
                                                                    prop:value=init_type
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = company_locations.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = val.clone();
                                                                            company_locations.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.location_type = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/locations/{}", company_api_url(), id), &serde_json::json!({"company_location": {"location_type": val}})).await; });
                                                                        }
                                                                    }
                                                                >
                                                                    <option value="OFFICE">"Офис"</option>
                                                                    <option value="STORE">"Магазин"</option>
                                                                    <option value="WAREHOUSE">"Склад"</option>
                                                                    <option value="BRANCH">"Клон"</option>
                                                                </select>
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Улица"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_street
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = company_locations.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = Some(val.clone());
                                                                            company_locations.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.street_name = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/locations/{}", company_api_url(), id), &serde_json::json!({"company_location": {"street_name": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; gap: 10px; margin-top: 10px;">
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"№"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_building
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = company_locations.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = Some(val.clone());
                                                                            company_locations.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.building_number = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/locations/{}", company_api_url(), id), &serde_json::json!({"company_location": {"building_number": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Град"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_city
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = company_locations.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = Some(val.clone());
                                                                            company_locations.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.city = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/locations/{}", company_api_url(), id), &serde_json::json!({"company_location": {"city": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"П.К."</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_post_code
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = company_locations.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = Some(val.clone());
                                                                            company_locations.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.post_code = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/locations/{}", company_api_url(), id), &serde_json::json!({"company_location": {"post_code": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Област"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_region
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = company_locations.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = Some(val.clone());
                                                                            company_locations.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.region = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/locations/{}", company_api_url(), id), &serde_json::json!({"company_location": {"region": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }
                                    }}
                                </div>

                                // Accounting settings section
                                <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                                    <h4 style="margin: 0 0 15px 0; color: #34495e;">"Счетоводни настройки"</h4>
                                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-size: 13px;">"Счетоводна база"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; background: white;"
                                                on:change=move |ev| tax_accounting_basis.set(event_target_value(&ev))
                                            >
                                                <option value="A" selected=move || tax_accounting_basis.get() == "A">"A - Начисляване"</option>
                                                <option value="P" selected=move || tax_accounting_basis.get() == "P">"P - Касова основа"</option>
                                                <option value="BANK" selected=move || tax_accounting_basis.get() == "BANK">"BANK - Банкови институции"</option>
                                                <option value="INSURANCE" selected=move || tax_accounting_basis.get() == "INSURANCE">"INSURANCE - Застрахователи"</option>
                                            </select>
                                        </div>
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-size: 13px;">"Метод за оценка на запасите"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; background: white;"
                                                on:change=move |ev| inventory_valuation_method.set(event_target_value(&ev))
                                            >
                                                <option value="FIFO" selected=move || inventory_valuation_method.get() == "FIFO">"FIFO - Първа входяща, първа изходяща"</option>
                                                <option value="LIFO" selected=move || inventory_valuation_method.get() == "LIFO">"LIFO - Последна входяща, първа изходяща"</option>
                                                <option value="WEIGHTED_AVG" selected=move || inventory_valuation_method.get() == "WEIGHTED_AVG">"Средно претеглена цена"</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>

                                // Ownership structure section
                                <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                                    <h4 style="margin: 0 0 15px 0; color: #34495e;">"Структура на собствеността"</h4>
                                    <div style="margin-bottom: 15px;">
                                        <label style="display: block; margin-bottom: 5px; font-size: 13px;">"Принадлежност към група"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; background: white;"
                                            on:change=move |ev| is_part_of_group.set(event_target_value(&ev))
                                        >
                                            <option value="" selected=move || is_part_of_group.get().is_empty()>"-- Не е посочено --"</option>
                                            <option value="1" selected=move || is_part_of_group.get() == "1">"1 - Централа на група"</option>
                                            <option value="2" selected=move || is_part_of_group.get() == "2">"2 - Част от група"</option>
                                            <option value="3" selected=move || is_part_of_group.get() == "3">"3 - Не е част от група"</option>
                                            <option value="4" selected=move || is_part_of_group.get() == "4">"4 - Едноличен търговец"</option>
                                            <option value="5" selected=move || is_part_of_group.get() == "5">"5 - Друго"</option>
                                        </select>
                                    </div>
                                </div>

                                // Beneficial owners section (multiple)
                                <div style="background: #e8f4fd; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                        <h4 style="margin: 0; color: #34495e;">"Действителни собственици"</h4>
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 8px 15px; border-radius: 4px; cursor: pointer; font-size: 13px;"
                                            on:click=add_beneficial_owner
                                        >
                                            "+ Добави собственик"
                                        </button>
                                    </div>

                                    {move || {
                                        let count = owner_count.get();
                                        if count == 0 {
                                            view! {
                                                <p style="color: #7f8c8d; font-size: 13px; text-align: center; padding: 20px;">
                                                    "Няма добавени собственици. Кликнете бутона за да добавите."
                                                </p>
                                            }.into_view()
                                        } else {
                                            (0..count).map(|idx| {
                                                let init_first = beneficial_owners.with(|l| l.get(idx).map(|v| v.first_name_bg.clone()).unwrap_or_default());
                                                let init_last = beneficial_owners.with(|l| l.get(idx).map(|v| v.last_name_bg.clone()).unwrap_or_default());
                                                let init_egn = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.egn.clone()).unwrap_or_default());
                                                let init_first_lat = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.first_name_latin.clone()).unwrap_or_default());
                                                let init_last_lat = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.last_name_latin.clone()).unwrap_or_default());
                                                let init_country = beneficial_owners.with(|l| l.get(idx).map(|v| v.country.clone()).unwrap_or("BG".to_string()));
                                                let init_pct = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.ownership_percentage).map(|p| format!("{}", p)).unwrap_or_default());
                                                view! {
                                                    <div style="background: white; padding: 15px; border-radius: 6px; margin-bottom: 10px; border: 1px solid #ddd;">
                                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                                                            <span style="font-weight: 600; color: #34495e; font-size: 14px;">
                                                                {format!("Собственик #{}", idx + 1)}
                                                            </span>
                                                            <button
                                                                style="background: #e74c3c; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                                on:click=move |_| {
                                                                    if let Some(id) = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                        spawn_local(async move {
                                                                            let _ = api_delete(&format!("{}/beneficial_owners/{}", company_api_url(), id)).await;
                                                                            beneficial_owners.update(|list| list.retain(|o| o.id != Some(id)));
                                                                        });
                                                                    }
                                                                }
                                                            >
                                                                "Изтрий"
                                                            </button>
                                                        </div>
                                                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; gap: 10px;">
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Име (БГ)"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_first
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = val.clone();
                                                                            beneficial_owners.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.first_name_bg = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/beneficial_owners/{}", company_api_url(), id), &serde_json::json!({"beneficial_owner": {"first_name_bg": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Фамилия (БГ)"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_last
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = val.clone();
                                                                            beneficial_owners.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.last_name_bg = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/beneficial_owners/{}", company_api_url(), id), &serde_json::json!({"beneficial_owner": {"last_name_bg": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"ЕГН"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_egn
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = if val.is_empty() { None } else { Some(val.clone()) };
                                                                            beneficial_owners.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.egn = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/beneficial_owners/{}", company_api_url(), id), &serde_json::json!({"beneficial_owner": {"egn": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Държава"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_country
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = val.clone();
                                                                            beneficial_owners.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.country = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/beneficial_owners/{}", company_api_url(), id), &serde_json::json!({"beneficial_owner": {"country": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; margin-top: 10px;">
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"First name (Latin)"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_first_lat
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = if val.is_empty() { None } else { Some(val.clone()) };
                                                                            beneficial_owners.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.first_name_latin = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/beneficial_owners/{}", company_api_url(), id), &serde_json::json!({"beneficial_owner": {"first_name_latin": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Last name (Latin)"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_last_lat
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = if val.is_empty() { None } else { Some(val.clone()) };
                                                                            beneficial_owners.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.last_name_latin = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/beneficial_owners/{}", company_api_url(), id), &serde_json::json!({"beneficial_owner": {"last_name_latin": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"% собственост"</label>
                                                                <input type="text" inputmode="decimal"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_pct
                                                                    on:change=move |ev| {
                                                                        let raw = event_target_value(&ev);
                                                                        let val: Option<f64> = raw.replace(',', ".").parse().ok();
                                                                        if let Some(id) = beneficial_owners.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            beneficial_owners.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.ownership_percentage = val; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/beneficial_owners/{}", company_api_url(), id), &serde_json::json!({"beneficial_owner": {"ownership_percentage": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }
                                    }}
                                </div>

                                // Ultimate parents section (multiple)
                                <div style="background: #fff3e0; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                        <div>
                                            <h4 style="margin: 0 0 5px 0; color: #34495e;">"Крайни предприятия майки"</h4>
                                            <p style="color: #7f8c8d; margin: 0; font-size: 12px;">
                                                "Попълва се само ако фирмата е част от група"
                                            </p>
                                        </div>
                                        <button
                                            style="background: #e67e22; color: white; border: none; padding: 8px 15px; border-radius: 4px; cursor: pointer; font-size: 13px;"
                                            on:click=add_ultimate_parent
                                        >
                                            "+ Добави предприятие"
                                        </button>
                                    </div>

                                    {move || {
                                        let count = parent_count.get();
                                        if count == 0 {
                                            view! {
                                                <p style="color: #7f8c8d; font-size: 13px; text-align: center; padding: 20px;">
                                                    "Няма добавени предприятия майки."
                                                </p>
                                            }.into_view()
                                        } else {
                                            (0..count).map(|idx| {
                                                let init_name = ultimate_parents.with(|l| l.get(idx).map(|v| v.name_bg.clone()).unwrap_or_default());
                                                let init_name_lat = ultimate_parents.with(|l| l.get(idx).and_then(|v| v.name_latin.clone()).unwrap_or_default());
                                                let init_uic = ultimate_parents.with(|l| l.get(idx).and_then(|v| v.uic.clone()).unwrap_or_default());
                                                let init_country = ultimate_parents.with(|l| l.get(idx).map(|v| v.country.clone()).unwrap_or("BG".to_string()));
                                                view! {
                                                    <div style="background: white; padding: 15px; border-radius: 6px; margin-bottom: 10px; border: 1px solid #ddd;">
                                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                                                            <span style="font-weight: 600; color: #34495e; font-size: 14px;">
                                                                {format!("Предприятие #{}", idx + 1)}
                                                            </span>
                                                            <button
                                                                style="background: #e74c3c; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                                on:click=move |_| {
                                                                    if let Some(id) = ultimate_parents.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                        spawn_local(async move {
                                                                            let _ = api_delete(&format!("{}/ultimate_parents/{}", company_api_url(), id)).await;
                                                                            ultimate_parents.update(|list| list.retain(|p| p.id != Some(id)));
                                                                        });
                                                                    }
                                                                }
                                                            >
                                                                "Изтрий"
                                                            </button>
                                                        </div>
                                                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; gap: 10px;">
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Наименование (БГ)"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_name
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = ultimate_parents.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = val.clone();
                                                                            ultimate_parents.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.name_bg = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/ultimate_parents/{}", company_api_url(), id), &serde_json::json!({"ultimate_parent": {"name_bg": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Name (Latin)"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_name_lat
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = ultimate_parents.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = if val.is_empty() { None } else { Some(val.clone()) };
                                                                            ultimate_parents.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.name_latin = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/ultimate_parents/{}", company_api_url(), id), &serde_json::json!({"ultimate_parent": {"name_latin": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"ЕИК"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_uic
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = ultimate_parents.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = if val.is_empty() { None } else { Some(val.clone()) };
                                                                            ultimate_parents.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.uic = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/ultimate_parents/{}", company_api_url(), id), &serde_json::json!({"ultimate_parent": {"uic": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                            <div>
                                                                <label style="display: block; margin-bottom: 3px; font-size: 12px; color: #7f8c8d;">"Държава"</label>
                                                                <input type="text"
                                                                    style="width: 100%; padding: 7px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-size: 13px;"
                                                                    attr:value=init_country
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        if let Some(id) = ultimate_parents.with(|l| l.get(idx).and_then(|v| v.id)) {
                                                                            let v = val.clone();
                                                                            ultimate_parents.update(|list| { if let Some(item) = list.iter_mut().find(|x| x.id == Some(id)) { item.country = v; } });
                                                                            spawn_local(async move { let _ = api_put(&format!("{}/ultimate_parents/{}", company_api_url(), id), &serde_json::json!({"ultimate_parent": {"country": val}})).await; });
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }
                                    }}
                                </div>

                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer;"
                                    on:click=save_saft
                                    disabled=move || saving.get()
                                >
                                    {move || if saving.get() { "Запазване..." } else { "Запази SAFt настройките" }}
                                </button>
                            </div>
                        }.into_view()
                    }
                    3 => {
                        // Features
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 700px;">
                                <h3 style="margin: 0 0 25px 0; color: #2c3e50;">"Допълнителни функции"</h3>

                                <div style="border: 1px solid #ecf0f1; border-radius: 8px; overflow: hidden;">
                                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 20px; border-bottom: 1px solid #ecf0f1;">
                                        <div>
                                            <h4 style="margin: 0 0 5px 0; color: #2c3e50;">"VIES валидация"</h4>
                                            <p style="margin: 0; color: #7f8c8d; font-size: 13px;">"Автоматична проверка на ДДС номера в европейската система VIES"</p>
                                        </div>
                                        <label style="position: relative; display: inline-block; width: 50px; height: 26px;">
                                            <input
                                                type="checkbox"
                                                style="opacity: 0; width: 0; height: 0;"
                                                checked=move || vies_validation.get()
                                                on:change=move |ev| vies_validation.set(event_target_checked(&ev))
                                            />
                                            <span style=move || format!(
                                                "position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; border-radius: 26px; transition: .3s; {}",
                                                if vies_validation.get() { "background: #27ae60;" } else { "background: #ccc;" }
                                            )>
                                                <span style=move || format!(
                                                    "position: absolute; height: 20px; width: 20px; left: 3px; bottom: 3px; background: white; border-radius: 50%; transition: .3s; {}",
                                                    if vies_validation.get() { "transform: translateX(24px);" } else { "" }
                                                )></span>
                                            </span>
                                        </label>
                                    </div>

                                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 20px;">
                                        <div>
                                            <h4 style="margin: 0 0 5px 0; color: #2c3e50;">"AI сканиране на документи"</h4>
                                            <p style="margin: 0; color: #7f8c8d; font-size: 13px;">"Автоматично разпознаване на фактури чрез Mistral AI"</p>
                                        </div>
                                        <label style="position: relative; display: inline-block; width: 50px; height: 26px;">
                                            <input
                                                type="checkbox"
                                                style="opacity: 0; width: 0; height: 0;"
                                                checked=move || ai_scanning.get()
                                                on:change=move |ev| ai_scanning.set(event_target_checked(&ev))
                                            />
                                            <span style=move || format!(
                                                "position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; border-radius: 26px; transition: .3s; {}",
                                                if ai_scanning.get() { "background: #27ae60;" } else { "background: #ccc;" }
                                            )>
                                                <span style=move || format!(
                                                    "position: absolute; height: 20px; width: 20px; left: 3px; bottom: 3px; background: white; border-radius: 50%; transition: .3s; {}",
                                                    if ai_scanning.get() { "transform: translateX(24px);" } else { "" }
                                                )></span>
                                            </span>
                                        </label>
                                    </div>
                                </div>

                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; margin-top: 25px;"
                                    on:click=save_features
                                    disabled=move || saving.get()
                                >
                                    {move || if saving.get() { "Запазване..." } else { "Запази настройките" }}
                                </button>
                            </div>
                        }.into_view()
                    }
                    4 => {
                        // Integrations (company-specific)
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 700px;">
                                <h3 style="margin: 0 0 5px 0; color: #2c3e50;">"Интеграции"</h3>
                                <p style="color: #7f8c8d; margin: 0 0 25px 0; font-size: 14px;">
                                    "Конфигурирайте външни услуги за тази фирма"
                                </p>

                                <div style="border: 1px solid #ecf0f1; border-radius: 8px; overflow: hidden;">
                                    // Mistral AI
                                    <div style="padding: 20px; border-bottom: 1px solid #ecf0f1;">
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                            <div>
                                                <h4 style="margin: 0 0 5px 0; color: #2c3e50;">"Mistral AI"</h4>
                                                <p style="margin: 0; color: #7f8c8d; font-size: 13px;">"AI разпознаване на фактури (Pixtral)"</p>
                                            </div>
                                            <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                                <span style="font-size: 14px; color: #7f8c8d;">"Активен"</span>
                                                <input
                                                    type="checkbox"
                                                    style="width: 18px; height: 18px; cursor: pointer;"
                                                    checked=move || mistral_enabled.get()
                                                    on:change=move |ev| mistral_enabled.set(event_target_checked(&ev))
                                                />
                                            </label>
                                        </div>
                                        {move || if mistral_enabled.get() {
                                            view! {
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; font-size: 14px;">"API Key"</label>
                                                    <input
                                                        type="password"
                                                        placeholder="sk-..."
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || mistral_api_key.get()
                                                        on:input=move |ev| mistral_api_key.set(event_target_value(&ev))
                                                    />
                                                    <p style="margin: 8px 0 0 0; color: #7f8c8d; font-size: 12px;">
                                                        "Вземете API ключ от "<a href="https://console.mistral.ai/" target="_blank" style="color: #3498db;">"console.mistral.ai"</a>
                                                    </p>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }}
                                    </div>

                                    // S3 Storage
                                    <div style="padding: 20px;">
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                            <div>
                                                <h4 style="margin: 0 0 5px 0; color: #2c3e50;">"S3 Storage"</h4>
                                                <p style="margin: 0; color: #7f8c8d; font-size: 13px;">"Съхранение на документи в облака"</p>
                                            </div>
                                            <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                                <span style="font-size: 14px; color: #7f8c8d;">"Активен"</span>
                                                <input
                                                    type="checkbox"
                                                    style="width: 18px; height: 18px; cursor: pointer;"
                                                    checked=move || s3_enabled.get()
                                                    on:change=move |ev| s3_enabled.set(event_target_checked(&ev))
                                                />
                                            </label>
                                        </div>
                                        {move || if s3_enabled.get() {
                                            view! {
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; font-size: 14px;">"Bucket"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="my-bucket"
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || s3_bucket.get()
                                                        on:input=move |ev| s3_bucket.set(event_target_value(&ev))
                                                    />
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }}
                                    </div>
                                </div>

                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; margin-top: 25px;"
                                    on:click=save_integrations
                                    disabled=move || saving.get()
                                >
                                    {move || if saving.get() { "Запазване..." } else { "Запази интеграциите" }}
                                </button>
                            </div>
                        }.into_view()
                    }
                    5 => {
                        // Colleagues - Company Users
                        let company_id = get_selected_company_id().unwrap_or(0);
                        let is_super_admin = user_ctx.is_super_admin();

                        // Determine which roles the current user can assign
                        let assignable_roles: Vec<Role> = available_roles.get().into_iter().filter(|r| {
                            if is_super_admin {
                                true // super_admin can assign any role
                            } else {
                                // admin can only assign accountant or observer
                                r.name == "accountant" || r.name == "observer"
                            }
                        }).collect();

                        let add_colleague = move |_| {
                            let email = new_user_email.get();
                            let role = new_user_role.get();
                            if email.is_empty() || role.is_empty() {
                                error_msg.set("Моля въведете email и изберете роля".to_string());
                                return;
                            }

                            saving.set(true);
                            spawn_local(async move {
                                let body = serde_json::json!({
                                    "email": email,
                                    "role": role
                                });
                                match api_post(&format!("/api/companies/{}/users", company_id), &body).await {
                                    Ok(_) => {
                                        success_msg.set("Потребителят е добавен".to_string());
                                        new_user_email.set(String::new());
                                        new_user_role.set(String::new());
                                        // Reload colleagues
                                        if let Ok(data) = api_get(&format!("/api/companies/{}/users", company_id)).await {
                                            if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                                                let items: Vec<CompanyUser> = arr.iter()
                                                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                                    .collect();
                                                colleagues.set(items);
                                            }
                                        }
                                    }
                                    Err(e) => error_msg.set(format!("Грешка: {}", e)),
                                }
                                saving.set(false);
                            });
                        };

                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px;">
                                <h3 style="margin: 0 0 5px 0; color: #2c3e50;">"Колеги във фирмата"</h3>
                                <p style="color: #7f8c8d; margin: 0 0 25px 0; font-size: 14px;">
                                    "Управлявайте потребителите, които имат достъп до тази фирма"
                                </p>

                                // Add new colleague form
                                <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 25px;">
                                    <h4 style="margin: 0 0 15px 0; color: #34495e;">"Добави нов колега"</h4>
                                    <div style="display: flex; gap: 15px; align-items: end;">
                                        <div style="flex: 2;">
                                            <label style="display: block; margin-bottom: 5px; font-size: 13px;">"Email на потребителя"</label>
                                            <input
                                                type="email"
                                                placeholder="colleague@example.com"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                prop:value=move || new_user_email.get()
                                                on:input=move |ev| new_user_email.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div style="flex: 1;">
                                            <label style="display: block; margin-bottom: 5px; font-size: 13px;">"Роля"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; background: white;"
                                                prop:value=move || new_user_role.get()
                                                on:change=move |ev| new_user_role.set(event_target_value(&ev))
                                            >
                                                <option value="">"-- Избери роля --"</option>
                                                {assignable_roles.clone().into_iter().map(|role| {
                                                    let role_label = match role.name.as_str() {
                                                        "super_admin" => "Супер Админ".to_string(),
                                                        "admin" => "Администратор".to_string(),
                                                        "accountant" => "Счетоводител".to_string(),
                                                        "observer" => "Наблюдател".to_string(),
                                                        _ => role.name.clone()
                                                    };
                                                    view! {
                                                        <option value=role.name.clone()>{role_label}</option>
                                                    }
                                                }).collect_view()}
                                            </select>
                                        </div>
                                        <button
                                            style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; white-space: nowrap;"
                                            on:click=add_colleague
                                            disabled=move || saving.get()
                                        >
                                            "+ Добави"
                                        </button>
                                    </div>
                                    <p style="margin: 10px 0 0 0; color: #7f8c8d; font-size: 12px;">
                                        "Потребителят трябва първо да е регистриран в системата"
                                    </p>
                                </div>

                                // Colleagues list
                                <div style="border: 1px solid #ecf0f1; border-radius: 8px; overflow: hidden;">
                                    <table style="width: 100%; border-collapse: collapse;">
                                        <thead style="background: #f8f9fa;">
                                            <tr>
                                                <th style="padding: 12px; text-align: left; color: #7f8c8d; font-weight: 500;">"Потребител"</th>
                                                <th style="padding: 12px; text-align: left; color: #7f8c8d; font-weight: 500;">"Email"</th>
                                                <th style="padding: 12px; text-align: center; color: #7f8c8d; font-weight: 500;">"Роля"</th>
                                                <th style="padding: 12px; text-align: center; color: #7f8c8d; font-weight: 500;">"Статус"</th>
                                                <th style="padding: 12px; text-align: right; color: #7f8c8d; font-weight: 500;">"Действия"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {move || {
                                                let users = colleagues.get();
                                                if users.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="5" style="padding: 30px; text-align: center; color: #7f8c8d;">
                                                                "Няма добавени колеги"
                                                            </td>
                                                        </tr>
                                                    }.into_view()
                                                } else {
                                                    users.into_iter().map(|user| {
                                                        let user_id = user.id;
                                                        let full_name = user.full_name.clone().unwrap_or_else(|| {
                                                            format!("{} {}",
                                                                user.first_name.clone().unwrap_or_default(),
                                                                user.last_name.clone().unwrap_or_default()
                                                            ).trim().to_string()
                                                        });
                                                        let role_label = match user.role.as_str() {
                                                            "super_admin" => "Супер Админ".to_string(),
                                                            "admin" => "Администратор".to_string(),
                                                            "accountant" => "Счетоводител".to_string(),
                                                            "observer" => "Наблюдател".to_string(),
                                                            _ => user.role.clone()
                                                        };
                                                        let role_color = match user.role.as_str() {
                                                            "super_admin" => "#e74c3c",
                                                            "admin" => "#f39c12",
                                                            "accountant" => "#3498db",
                                                            "observer" => "#95a5a6",
                                                            _ => "#7f8c8d"
                                                        };

                                                        // Can remove if: super_admin, or admin removing accountant/observer
                                                        let can_remove = is_super_admin ||
                                                            (user.role == "accountant" || user.role == "observer");

                                                        view! {
                                                            <tr style="border-bottom: 1px solid #ecf0f1;">
                                                                <td style="padding: 12px;">
                                                                    <div style="font-weight: 500;">
                                                                        {if full_name.is_empty() { "-".to_string() } else { full_name }}
                                                                    </div>
                                                                </td>
                                                                <td style="padding: 12px; color: #7f8c8d;">
                                                                    {user.email}
                                                                </td>
                                                                <td style="padding: 12px; text-align: center;">
                                                                    <span style=format!(
                                                                        "display: inline-block; padding: 4px 12px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                                        role_color
                                                                    )>
                                                                        {role_label}
                                                                    </span>
                                                                </td>
                                                                <td style="padding: 12px; text-align: center;">
                                                                    <span style=format!(
                                                                        "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                                        if user.is_active { "#27ae60" } else { "#95a5a6" }
                                                                    )>
                                                                        {if user.is_active { "Активен" } else { "Неактивен" }}
                                                                    </span>
                                                                </td>
                                                                <td style="padding: 12px; text-align: right;">
                                                                    {if can_remove {
                                                                        view! {
                                                                            <button
                                                                                style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                                                on:click=move |_| {
                                                                                    spawn_local(async move {
                                                                                        match api_delete(&format!("/api/companies/{}/users/{}", company_id, user_id)).await {
                                                                                            Ok(_) => {
                                                                                                colleagues.update(|list| list.retain(|u| u.id != user_id));
                                                                                            }
                                                                                            Err(e) => {
                                                                                                web_sys::console::error_1(&format!("Error: {}", e).into());
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                "Премахни"
                                                                            </button>
                                                                        }.into_view()
                                                                    } else {
                                                                        view! {
                                                                            <span style="color: #bdc3c7; font-size: 12px;">"-"</span>
                                                                        }.into_view()
                                                                    }}
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }
                                            }}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        }.into_view()
                    }
                    _ => view! { <span></span> }.into_view()
                }
            }}
        </Layout>
    }
}
