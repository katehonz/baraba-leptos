use leptos::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete, company_api_url};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub product_type: Option<String>,
    pub price: f64,
    pub measure_unit: Option<String>,
    pub commodity_code: Option<String>,
    pub tax_type: Option<String>,
    pub tax_code: Option<String>,
    pub is_active: bool,
    pub inventory_account_id: Option<i64>,
    pub revenue_account_id: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaftProductType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
}

// SAF-T Product Types (Bulgarian)
fn get_product_types() -> Vec<(&'static str, &'static str)> {
    vec![
        ("10", "Материали"),
        ("20", "Продукция"),
        ("30", "Стоки"),
        ("40", "Незавършено производство"),
        ("50", "Инвестиция в материален запас"),
        ("S", "Услуги"),
    ]
}

// Units of Measure (UN/CEFACT codes with Bulgarian names)
fn get_units_of_measure() -> Vec<(&'static str, &'static str)> {
    vec![
        ("C62", "брой"),
        ("KGM", "кг"),
        ("GRM", "грам"),
        ("TNE", "тон"),
        ("LTR", "литър"),
        ("MTR", "метър"),
        ("MTK", "кв.м"),
        ("MTQ", "куб.м"),
        ("HUR", "час"),
        ("DAY", "ден"),
        ("MON", "месец"),
        ("ANN", "година"),
        ("SET", "комплект"),
        ("PR", "чифт"),
        ("PCE", "парче"),
    ]
}

fn get_product_type_name(code: &str) -> &'static str {
    match code {
        "10" => "Материали",
        "20" => "Продукция",
        "30" => "Стоки",
        "40" => "Незавършено производство",
        "50" => "Инвестиция",
        "S" => "Услуги",
        _ => "Друго",
    }
}

fn get_product_type_color(code: &str) -> &'static str {
    match code {
        "10" => "#3498db",  // Materials - blue
        "20" => "#27ae60",  // Production - green
        "30" => "#9b59b6",  // Goods - purple
        "40" => "#f39c12",  // WIP - orange
        "50" => "#e74c3c",  // Investment - red
        "S" => "#1abc9c",   // Services - teal
        _ => "#7f8c8d",
    }
}

fn get_unit_name(code: &str) -> &'static str {
    match code {
        "C62" => "брой",
        "KGM" => "кг",
        "GRM" => "грам",
        "TNE" => "тон",
        "LTR" => "литър",
        "MTR" => "метър",
        "MTK" => "кв.м",
        "MTQ" => "куб.м",
        "HUR" => "час",
        "DAY" => "ден",
        "MON" => "месец",
        "ANN" => "година",
        "SET" => "комплект",
        "PR" => "чифт",
        "PCE" => "парче",
        _ => "бр.",
    }
}

#[component]
pub fn Products() -> impl IntoView {
    let products = create_rw_signal(Vec::<Product>::new());
    let accounts = create_rw_signal(Vec::<Account>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());
    let search_query = create_rw_signal(String::new());
    let filter_type = create_rw_signal(String::new());

    // Modal state
    let show_modal = create_rw_signal(false);
    let editing_product = create_rw_signal(Option::<Product>::None);
    let saving = create_rw_signal(false);

    // Form fields
    let form_code = create_rw_signal(String::new());
    let form_name = create_rw_signal(String::new());
    let form_description = create_rw_signal(String::new());
    let form_product_type = create_rw_signal(String::from("30")); // Default: Goods
    let form_price = create_rw_signal(String::from("0.00"));
    let form_measure_unit = create_rw_signal(String::from("C62")); // Default: pieces
    let form_tax_code = create_rw_signal(String::from("20")); // Default: 20% VAT
    let form_inventory_account_id = create_rw_signal(String::new());
    let form_revenue_account_id = create_rw_signal(String::new());

    // Delete modal
    let show_delete_modal = create_rw_signal(false);
    let delete_product = create_rw_signal(Option::<Product>::None);
    let deleting = create_rw_signal(false);

    // Load products
    let reload = move || {
        loading.set(true);
        spawn_local(async move {
            match api_get(&format!("{}/products", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let prods: Vec<Product> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        products.set(prods);
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Load accounts for dropdowns
    let load_accounts = move || {
        spawn_local(async move {
            match api_get(&format!("{}/accounts?per_page=1000", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let accs: Vec<Account> = arr.iter()
                            .filter_map(|v| {
                                let id = v.get("id")?.as_i64()?;
                                let code = v.get("code")?.as_str()?.to_string();
                                let name = v.get("name")?.as_str()?.to_string();
                                Some(Account { id, code, name })
                            })
                            .collect();
                        accounts.set(accs);
                    }
                }
                Err(_) => {}
            }
        });
    };

    // Initial load
    create_effect(move |_| {
        reload();
        load_accounts();
    });

    // Reset form
    let reset_form = move || {
        form_code.set(String::new());
        form_name.set(String::new());
        form_description.set(String::new());
        form_product_type.set("30".to_string());
        form_price.set("0.00".to_string());
        form_measure_unit.set("C62".to_string());
        form_tax_code.set("20".to_string());
        form_inventory_account_id.set(String::new());
        form_revenue_account_id.set(String::new());
        editing_product.set(None);
    };

    // Open modal for new product
    let open_new = move |_| {
        reset_form();
        show_modal.set(true);
    };

    // Open modal for edit
    let open_edit = move |p: Product| {
        form_code.set(p.code.clone());
        form_name.set(p.name.clone());
        form_description.set(p.description.clone().unwrap_or_default());
        form_product_type.set(p.product_type.clone().unwrap_or_else(|| "30".to_string()));
        form_price.set(format!("{:.2}", p.price));
        form_measure_unit.set(p.measure_unit.clone().unwrap_or_else(|| "C62".to_string()));
        form_tax_code.set(p.tax_code.clone().unwrap_or_else(|| "20".to_string()));
        form_inventory_account_id.set(p.inventory_account_id.map(|id| id.to_string()).unwrap_or_default());
        form_revenue_account_id.set(p.revenue_account_id.map(|id| id.to_string()).unwrap_or_default());
        editing_product.set(Some(p));
        show_modal.set(true);
    };

    // Save product
    let save_product = move |_| {
        let code = form_code.get();
        let name = form_name.get();

        if code.is_empty() || name.is_empty() {
            error_msg.set("Кодът и името са задължителни".to_string());
            return;
        }

        saving.set(true);
        error_msg.set(String::new());

        let price: f64 = form_price.get().parse().unwrap_or(0.0);
        let inv_acc: Option<i64> = form_inventory_account_id.get().parse().ok();
        let rev_acc: Option<i64> = form_revenue_account_id.get().parse().ok();

        let payload = serde_json::json!({
            "product": {
                "code": code,
                "name": name,
                "description": form_description.get(),
                "product_type": form_product_type.get(),
                "price": price,
                "measure_unit": form_measure_unit.get(),
                "tax_code": form_tax_code.get(),
                "inventory_account_id": inv_acc,
                "revenue_account_id": rev_acc,
                "is_active": true
            }
        });

        let is_edit = editing_product.get().is_some();
        let product_id = editing_product.get().map(|p| p.id);

        spawn_local(async move {
            let result = if is_edit {
                api_put(&format!("{}/products/{}", company_api_url(), product_id.unwrap()), &payload).await
            } else {
                api_post(&format!("{}/products", company_api_url()), &payload).await
            };

            match result {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set(if is_edit {
                            "Продуктът е редактиран успешно".to_string()
                        } else {
                            "Продуктът е създаден успешно".to_string()
                        });
                        show_modal.set(false);
                        reset_form();
                        reload();
                    } else {
                        let err = response.get("errors")
                            .and_then(|e| e.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|e| e.get("messages").and_then(|m| m.as_array()))
                                .flatten()
                                .filter_map(|m| m.as_str())
                                .collect::<Vec<_>>()
                                .join(", "))
                            .unwrap_or_else(|| "Неизвестна грешка".to_string());
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            saving.set(false);
        });
    };

    // Delete product
    let do_delete = move |_| {
        if let Some(p) = delete_product.get() {
            deleting.set(true);
            let p_id = p.id;
            spawn_local(async move {
                match api_delete(&format!("{}/products/{}", company_api_url(), p_id)).await {
                    Ok(response) => {
                        if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            success_msg.set("Продуктът е изтрит успешно".to_string());
                            reload();
                        } else {
                            error_msg.set("Грешка при изтриване".to_string());
                        }
                    }
                    Err(e) => {
                        error_msg.set(format!("Грешка: {}", e));
                    }
                }
                show_delete_modal.set(false);
                delete_product.set(None);
                deleting.set(false);
            });
        }
    };

    // Filtered products
    let filtered = move || {
        let query = search_query.get().to_lowercase();
        let type_filter = filter_type.get();

        products.get()
            .into_iter()
            .filter(|p| {
                // Type filter
                if !type_filter.is_empty() {
                    if p.product_type.as_ref().map(|t| t.as_str()) != Some(type_filter.as_str()) {
                        return false;
                    }
                }
                // Search filter
                if query.is_empty() {
                    return true;
                }
                p.code.to_lowercase().contains(&query) ||
                p.name.to_lowercase().contains(&query) ||
                p.description.as_ref().map(|d| d.to_lowercase().contains(&query)).unwrap_or(false)
            })
            .collect::<Vec<_>>()
    };

    // Product type counts
    let type_counts = move || {
        let prods = products.get();
        let mut counts = std::collections::HashMap::new();
        for p in &prods {
            if let Some(t) = &p.product_type {
                *counts.entry(t.clone()).or_insert(0) += 1;
            }
        }
        counts
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Номенклатура"</h1>
                <button
                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                    on:click=open_new
                >
                    "+ Нов артикул"
                </button>
            </div>

            // Success message
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {msg}
                            <button
                                style="float: right; background: transparent; border: none; color: white; cursor: pointer; font-size: 18px;"
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
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {err}
                            <button
                                style="float: right; background: transparent; border: none; color: white; cursor: pointer; font-size: 18px;"
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
                    // Search input
                    <div style="flex: 1; min-width: 250px;">
                        <input
                            type="text"
                            placeholder="Търсене по код, име или описание..."
                            style="width: 100%; padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                            prop:value=move || search_query.get()
                        />
                    </div>

                    // Type filter buttons
                    <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                        <button
                            style=move || format!(
                                "padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; {}",
                                if filter_type.get().is_empty() {
                                    "background: #34495e; color: white; border: none;"
                                } else {
                                    "background: white; color: #34495e; border: 1px solid #ddd;"
                                }
                            )
                            on:click=move |_| filter_type.set(String::new())
                        >
                            "Всички ("{move || products.get().len()}")"
                        </button>
                        {move || {
                            let counts = type_counts();
                            get_product_types().into_iter().map(|(code, label)| {
                                let count = counts.get(code).copied().unwrap_or(0);
                                let code_str = code.to_string();
                                let code_str2 = code.to_string();
                                let color = get_product_type_color(code);
                                view! {
                                    <button
                                        style=move || format!(
                                            "padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; {}",
                                            if filter_type.get() == code_str {
                                                format!("background: {}; color: white; border: none;", color)
                                            } else {
                                                "background: white; color: #34495e; border: 1px solid #ddd;".to_string()
                                            }
                                        )
                                        on:click=move |_| filter_type.set(code_str2.clone())
                                    >
                                        {label}" ("{count}")"
                                    </button>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>
            </div>

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

            // Table
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                let list = filtered();
                if list.is_empty() {
                    return view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 1.1em;">
                                {if products.get().is_empty() {
                                    "Няма добавени артикули"
                                } else {
                                    "Няма намерени резултати"
                                }}
                            </p>
                        </div>
                    }.into_view();
                }

                view! {
                    <div style="background: white; border-radius: 8px; margin-top: 20px; overflow: hidden;">
                        <table style="width: 100%; border-collapse: collapse;">
                            <thead style="background: #34495e; color: white;">
                                <tr>
                                    <th style="padding: 12px; text-align: left; width: 100px;">"Код"</th>
                                    <th style="padding: 12px; text-align: left;">"Наименование"</th>
                                    <th style="padding: 12px; text-align: left; width: 140px;">"Тип"</th>
                                    <th style="padding: 12px; text-align: right; width: 100px;">"Цена"</th>
                                    <th style="padding: 12px; text-align: center; width: 80px;">"Мярка"</th>
                                    <th style="padding: 12px; text-align: center; width: 80px;">"ДДС"</th>
                                    <th style="padding: 12px; text-align: center; width: 120px;">"Действия"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {list.into_iter().map(|p| {
                                    let p_for_edit = p.clone();
                                    let p_for_delete = p.clone();
                                    let type_code = p.product_type.clone().unwrap_or_default();
                                    let type_name = get_product_type_name(&type_code);
                                    let type_color = get_product_type_color(&type_code);
                                    let unit = p.measure_unit.clone().unwrap_or_default();
                                    let unit_name = get_unit_name(&unit);
                                    let tax = p.tax_code.clone().unwrap_or_else(|| "20".to_string());

                                    view! {
                                        <tr style="border-bottom: 1px solid #ecf0f1;">
                                            <td style="padding: 12px; font-family: monospace; font-weight: bold;">
                                                {p.code.clone()}
                                            </td>
                                            <td style="padding: 12px;">
                                                <div style="font-weight: 500;">{p.name.clone()}</div>
                                                {p.description.as_ref().map(|d| view! {
                                                    <div style="color: #7f8c8d; font-size: 12px; margin-top: 2px;">{d.clone()}</div>
                                                })}
                                            </td>
                                            <td style="padding: 12px;">
                                                <span style=format!(
                                                    "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                    type_color
                                                )>
                                                    {type_name}
                                                </span>
                                            </td>
                                            <td style="padding: 12px; text-align: right; font-family: monospace;">
                                                {format!("{:.2}", p.price)}
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                {unit_name}
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                {tax}"%"
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                <button
                                                    style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-right: 5px;"
                                                    title="Редактиране"
                                                    on:click=move |_| open_edit(p_for_edit.clone())
                                                >
                                                    "✏"
                                                </button>
                                                <button
                                                    style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                    title="Изтриване"
                                                    on:click=move |_| {
                                                        delete_product.set(Some(p_for_delete.clone()));
                                                        show_delete_modal.set(true);
                                                    }
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
                }.into_view()
            }}

            // Add/Edit Modal
            {move || {
                if show_modal.get() {
                    let is_edit = editing_product.get().is_some();
                    let accs = accounts.get();
                    let inventory_accounts: Vec<_> = accs.iter()
                        .filter(|a| a.code.starts_with("30") || a.code.starts_with("31"))
                        .collect();
                    let revenue_accounts: Vec<_> = accs.iter()
                        .filter(|a| a.code.starts_with("70") || a.code.starts_with("71"))
                        .collect();

                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: flex-start; justify-content: center; z-index: 1000; padding: 20px; overflow-y: auto;">
                            <div style="background: white; border-radius: 8px; padding: 30px; max-width: 800px; width: 100%; margin: 20px 0;">
                                <h3 style="margin-top: 0;">
                                    {if is_edit { "Редактиране на артикул" } else { "Нов артикул" }}
                                </h3>

                                // Form fields
                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Код *"</label>
                                        <input
                                            type="text"
                                            placeholder="напр. PRD001"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_code.set(event_target_value(&ev))
                                            prop:value=move || form_code.get()
                                        />
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Тип"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| form_product_type.set(event_target_value(&ev))
                                        >
                                            {get_product_types().into_iter().map(|(code, label)| {
                                                let code_str = code.to_string();
                                                view! {
                                                    <option
                                                        value=code
                                                        selected=move || form_product_type.get() == code_str
                                                    >
                                                        {label}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>

                                    <div style="grid-column: span 2;">
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Наименование *"</label>
                                        <input
                                            type="text"
                                            placeholder="Име на продукта/услугата"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_name.set(event_target_value(&ev))
                                            prop:value=move || form_name.get()
                                        />
                                    </div>

                                    <div style="grid-column: span 2;">
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Описание"</label>
                                        <textarea
                                            placeholder="Допълнително описание..."
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; min-height: 60px; resize: vertical;"
                                            on:input=move |ev| form_description.set(event_target_value(&ev))
                                            prop:value=move || form_description.get()
                                        />
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Цена"</label>
                                        <input
                                            type="text"
                                            inputmode="decimal"
                                            placeholder="0.00"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_price.set(event_target_value(&ev))
                                            prop:value=move || form_price.get()
                                        />
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Мерна единица"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| form_measure_unit.set(event_target_value(&ev))
                                        >
                                            {get_units_of_measure().into_iter().map(|(code, label)| {
                                                let code_str = code.to_string();
                                                view! {
                                                    <option
                                                        value=code
                                                        selected=move || form_measure_unit.get() == code_str
                                                    >
                                                        {label}" ("{code}")"
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"ДДС %"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| form_tax_code.set(event_target_value(&ev))
                                        >
                                            <option value="20" selected=move || form_tax_code.get() == "20">"20% (стандартна)"</option>
                                            <option value="9" selected=move || form_tax_code.get() == "9">"9% (намалена)"</option>
                                            <option value="0" selected=move || form_tax_code.get() == "0">"0% (освободена)"</option>
                                        </select>
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Сметка материални запаси"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| form_inventory_account_id.set(event_target_value(&ev))
                                        >
                                            <option value="">"-- Изберете --"</option>
                                            {inventory_accounts.iter().map(|a| {
                                                let id_str = a.id.to_string();
                                                let id_str2 = a.id.to_string();
                                                view! {
                                                    <option
                                                        value=id_str.clone()
                                                        selected=move || form_inventory_account_id.get() == id_str2
                                                    >
                                                        {format!("{} - {}", a.code, a.name)}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Сметка приходи"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| form_revenue_account_id.set(event_target_value(&ev))
                                        >
                                            <option value="">"-- Изберете --"</option>
                                            {revenue_accounts.iter().map(|a| {
                                                let id_str = a.id.to_string();
                                                let id_str2 = a.id.to_string();
                                                view! {
                                                    <option
                                                        value=id_str.clone()
                                                        selected=move || form_revenue_account_id.get() == id_str2
                                                    >
                                                        {format!("{} - {}", a.code, a.name)}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                </div>

                                // Action buttons
                                <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 25px;">
                                    <button
                                        style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| {
                                            show_modal.set(false);
                                            reset_form();
                                        }
                                    >
                                        "Отказ"
                                    </button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        disabled=move || saving.get()
                                        on:click=save_product
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
                if show_delete_modal.get() {
                    if let Some(p) = delete_product.get() {
                        view! {
                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                                <div style="background: white; border-radius: 8px; padding: 30px; max-width: 500px; width: 90%;">
                                    <h3 style="margin-top: 0; color: #e74c3c;">"Потвърждение за изтриване"</h3>

                                    <p>"Сигурни ли сте, че искате да изтриете артикула:"</p>
                                    <p style="font-weight: 500; font-size: 1.1em;">{p.code.clone()}" - "{p.name.clone()}</p>

                                    <p style="color: #e74c3c; font-size: 13px;">"Това действие не може да бъде отменено!"</p>

                                    <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px;">
                                        <button
                                            style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| {
                                                show_delete_modal.set(false);
                                                delete_product.set(None);
                                            }
                                        >
                                            "Отказ"
                                        </button>
                                        <button
                                            style="background: #e74c3c; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            disabled=move || deleting.get()
                                            on:click=do_delete
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
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
        </Layout>
    }
}
