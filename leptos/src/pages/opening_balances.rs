use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete, company_api_url};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CounterpartInfo {
    pub id: i64,
    pub name: String,
    pub vat: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductInfo {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WarehouseInfo {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpeningBalance {
    pub id: i64,
    pub date: String,
    pub debit: f64,
    pub credit: f64,
    pub quantity: Option<f64>,
    pub description: Option<String>,
    pub balance_type: String,
    pub account: AccountInfo,
    pub counterpart: Option<CounterpartInfo>,
    pub product: Option<ProductInfo>,
    pub warehouse: Option<WarehouseInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub account_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Counterpart {
    pub id: i64,
    pub name: String,
    pub vat_number: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub measure_unit: Option<String>,
}

fn get_balance_type_label(balance_type: &str) -> &'static str {
    match balance_type {
        "account" => "Сметка",
        "counterpart" => "Контрагент",
        "product" => "Продукт",
        _ => "Друг",
    }
}

fn get_balance_type_color(balance_type: &str) -> &'static str {
    match balance_type {
        "account" => "#3498db",
        "counterpart" => "#9b59b6",
        "product" => "#27ae60",
        _ => "#7f8c8d",
    }
}

fn get_today_date() -> String {
    // Use JavaScript Date to get current date
    let date = js_sys::Date::new_0();
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}

#[component]
pub fn OpeningBalances() -> impl IntoView {
    let balances = create_rw_signal(Vec::<OpeningBalance>::new());
    let accounts = create_rw_signal(Vec::<Account>::new());
    let counterparts = create_rw_signal(Vec::<Counterpart>::new());
    let products = create_rw_signal(Vec::<Product>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // Filter state
    let filter_type = create_rw_signal(String::new());
    let filter_account = create_rw_signal(String::new());

    // Form state
    let show_form = create_rw_signal(false);
    let editing_id = create_rw_signal(Option::<i64>::None);
    let form_date = create_rw_signal(get_today_date());
    let form_debit = create_rw_signal(0.0_f64);
    let form_credit = create_rw_signal(0.0_f64);
    let form_quantity = create_rw_signal(Option::<f64>::None);
    let form_description = create_rw_signal(String::new());
    let form_balance_type = create_rw_signal("account".to_string());
    let form_account_id = create_rw_signal(Option::<i64>::None);
    let form_counterpart_id = create_rw_signal(Option::<i64>::None);
    let form_product_id = create_rw_signal(Option::<i64>::None);

    // Load all data
    let load_data = move || {
        spawn_local(async move {
            loading.set(true);
            error_msg.set(String::new());

            // Load balances
            match api_get("/api/companies/1/opening_balances").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<OpeningBalance> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        balances.set(items);
                    }
                }
                Err(e) => error_msg.set(format!("Грешка при зареждане на салда: {}", e)),
            }

            // Load accounts
            match api_get(&format!("{}/accounts?per_page=1000", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Account> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        accounts.set(items);
                    }
                }
                Err(_) => {}
            }

            // Load counterparts
            match api_get("/api/companies/1/counterparts").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Counterpart> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        counterparts.set(items);
                    }
                }
                Err(_) => {}
            }

            // Load products
            match api_get("/api/companies/1/products").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Product> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        products.set(items);
                    }
                }
                Err(_) => {}
            }

            loading.set(false);
        });
    };

    // Load data on mount
    create_effect(move |_| {
        load_data();
    });

    // Filtered balances
    let filtered_balances = move || {
        let type_filter = filter_type.get();
        let account_filter = filter_account.get();

        balances.get()
            .into_iter()
            .filter(|b| {
                if !type_filter.is_empty() && b.balance_type != type_filter {
                    return false;
                }
                if !account_filter.is_empty() {
                    let acc_id: i64 = account_filter.parse().unwrap_or(0);
                    if b.account.id != acc_id {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<_>>()
    };

    // Calculate totals
    let totals = move || {
        let bals = filtered_balances();
        let total_debit: f64 = bals.iter().map(|b| b.debit).sum();
        let total_credit: f64 = bals.iter().map(|b| b.credit).sum();
        (total_debit, total_credit, total_debit - total_credit)
    };

    // Reset form
    let reset_form = move || {
        editing_id.set(None);
        form_date.set(get_today_date());
        form_debit.set(0.0);
        form_credit.set(0.0);
        form_quantity.set(None);
        form_description.set(String::new());
        form_balance_type.set("account".to_string());
        form_account_id.set(None);
        form_counterpart_id.set(None);
        form_product_id.set(None);
    };

    // Open form for new balance
    let open_new_form = move |_| {
        reset_form();
        show_form.set(true);
    };

    // Open form for editing
    let open_edit_form = move |balance: OpeningBalance| {
        editing_id.set(Some(balance.id));
        form_date.set(balance.date.clone());
        form_debit.set(balance.debit);
        form_credit.set(balance.credit);
        form_quantity.set(balance.quantity);
        form_description.set(balance.description.unwrap_or_default());
        form_balance_type.set(balance.balance_type.clone());
        form_account_id.set(Some(balance.account.id));
        form_counterpart_id.set(balance.counterpart.map(|c| c.id));
        form_product_id.set(balance.product.map(|p| p.id));
        show_form.set(true);
    };

    // Save balance
    let save_balance = move |_| {
        let account_id = match form_account_id.get() {
            Some(id) => id,
            None => {
                error_msg.set("Изберете сметка".to_string());
                return;
            }
        };

        let mut body = serde_json::json!({
            "date": form_date.get(),
            "debit": form_debit.get(),
            "credit": form_credit.get(),
            "balance_type": form_balance_type.get(),
            "account_id": account_id,
        });

        if let Some(desc) = Some(form_description.get()).filter(|s| !s.is_empty()) {
            body["description"] = serde_json::json!(desc);
        }

        if let Some(qty) = form_quantity.get() {
            body["quantity"] = serde_json::json!(qty);
        }

        if let Some(cid) = form_counterpart_id.get() {
            body["counterpart_id"] = serde_json::json!(cid);
        }

        if let Some(pid) = form_product_id.get() {
            body["product_id"] = serde_json::json!(pid);
        }

        spawn_local(async move {
            let result = if let Some(id) = editing_id.get() {
                api_put(&format!("/api/companies/1/opening_balances/{}", id), &body).await
            } else {
                api_post("/api/companies/1/opening_balances", &body).await
            };

            match result {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set(if editing_id.get().is_some() {
                            "Салдото е обновено".to_string()
                        } else {
                            "Салдото е добавено".to_string()
                        });
                        show_form.set(false);
                        load_data();
                    } else {
                        error_msg.set("Грешка при запис".to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
        });
    };

    // Delete balance
    let delete_balance = move |id: i64| {
        spawn_local(async move {
            match api_delete(&format!("/api/companies/1/opening_balances/{}", id)).await {
                Ok(_) => {
                    success_msg.set("Салдото е изтрито".to_string());
                    load_data();
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при изтриване: {}", e));
                }
            }
        });
    };

    // Get accounts for Group 4 (counterpart accounts)
    let counterpart_accounts = move || {
        accounts.get()
            .into_iter()
            .filter(|a| a.code.starts_with("4"))
            .collect::<Vec<_>>()
    };

    // Get accounts for Group 3 (inventory accounts)
    let inventory_accounts = move || {
        accounts.get()
            .into_iter()
            .filter(|a| a.code.starts_with("3"))
            .collect::<Vec<_>>()
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Начални салда"</h1>
                <button
                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                    on:click=open_new_form
                >
                    "+ Ново салдо"
                </button>
            </div>

            // Success message
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-top: 15px;">
                            {msg}
                            <button
                                style="float: right; background: transparent; border: none; color: white; cursor: pointer; font-size: 16px;"
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
                let msg = error_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 15px;">
                            {msg}
                            <button
                                style="float: right; background: transparent; border: none; color: white; cursor: pointer; font-size: 16px;"
                                on:click=move |_| error_msg.set(String::new())
                            >"×"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Filters
            <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                <div style="display: flex; gap: 15px; flex-wrap: wrap; align-items: center;">
                    // Type filter buttons
                    <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                        <button
                            style=move || format!(
                                "padding: 8px 16px; border-radius: 4px; cursor: pointer; {}",
                                if filter_type.get().is_empty() {
                                    "background: #34495e; color: white; border: none;"
                                } else {
                                    "background: white; color: #34495e; border: 1px solid #ddd;"
                                }
                            )
                            on:click=move |_| filter_type.set(String::new())
                        >
                            "Всички"
                        </button>
                        <button
                            style=move || format!(
                                "padding: 8px 16px; border-radius: 4px; cursor: pointer; {}",
                                if filter_type.get() == "account" {
                                    "background: #3498db; color: white; border: none;"
                                } else {
                                    "background: white; color: #34495e; border: 1px solid #ddd;"
                                }
                            )
                            on:click=move |_| filter_type.set("account".to_string())
                        >
                            "Сметки"
                        </button>
                        <button
                            style=move || format!(
                                "padding: 8px 16px; border-radius: 4px; cursor: pointer; {}",
                                if filter_type.get() == "counterpart" {
                                    "background: #9b59b6; color: white; border: none;"
                                } else {
                                    "background: white; color: #34495e; border: 1px solid #ddd;"
                                }
                            )
                            on:click=move |_| filter_type.set("counterpart".to_string())
                        >
                            "Контрагенти"
                        </button>
                        <button
                            style=move || format!(
                                "padding: 8px 16px; border-radius: 4px; cursor: pointer; {}",
                                if filter_type.get() == "product" {
                                    "background: #27ae60; color: white; border: none;"
                                } else {
                                    "background: white; color: #34495e; border: 1px solid #ddd;"
                                }
                            )
                            on:click=move |_| filter_type.set("product".to_string())
                        >
                            "Продукти"
                        </button>
                    </div>

                    // Account filter
                    <select
                        style="padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; min-width: 200px;"
                        on:change=move |ev| filter_account.set(event_target_value(&ev))
                    >
                        <option value="">"Всички сметки"</option>
                        {move || accounts.get().into_iter().map(|acc| {
                            view! {
                                <option value={acc.id.to_string()}>
                                    {format!("{} - {}", acc.code, acc.name)}
                                </option>
                            }
                        }).collect_view()}
                    </select>
                </div>

                // Totals
                <div style="margin-top: 15px; padding-top: 15px; border-top: 1px solid #ecf0f1; display: flex; gap: 30px;">
                    {move || {
                        let (total_debit, total_credit, net) = totals();
                        view! {
                            <>
                                <div>
                                    <span style="color: #7f8c8d;">"Общо Дебит: "</span>
                                    <strong style="color: #27ae60;">{format!("{:.2}", total_debit)}</strong>
                                </div>
                                <div>
                                    <span style="color: #7f8c8d;">"Общо Кредит: "</span>
                                    <strong style="color: #e74c3c;">{format!("{:.2}", total_credit)}</strong>
                                </div>
                                <div>
                                    <span style="color: #7f8c8d;">"Салдо: "</span>
                                    <strong style=format!("color: {}", if net >= 0.0 { "#27ae60" } else { "#e74c3c" })>
                                        {format!("{:.2}", net)}
                                    </strong>
                                </div>
                            </>
                        }
                    }}
                </div>
            </div>

            // Form modal
            {move || {
                if show_form.get() {
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 500px; max-width: 90vw; max-height: 90vh; overflow-y: auto;">
                                <h2 style="margin-top: 0;">
                                    {if editing_id.get().is_some() { "Редактиране на салдо" } else { "Ново начално салдо" }}
                                </h2>

                                <div style="display: flex; flex-direction: column; gap: 15px;">
                                    // Balance type
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Тип салдо"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| form_balance_type.set(event_target_value(&ev))
                                            prop:value=move || form_balance_type.get()
                                        >
                                            <option value="account">"Сметка"</option>
                                            <option value="counterpart">"Контрагент (разчет)"</option>
                                            <option value="product">"Продукт/Материал"</option>
                                        </select>
                                    </div>

                                    // Date
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Дата"</label>
                                        <input
                                            type="date"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:input=move |ev| form_date.set(event_target_value(&ev))
                                            prop:value=move || form_date.get()
                                        />
                                    </div>

                                    // Account
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Сметка *"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                form_account_id.set(val.parse().ok());
                                            }
                                        >
                                            <option value="">"-- Изберете сметка --"</option>
                                            {move || {
                                                let bt = form_balance_type.get();
                                                let accs = if bt == "counterpart" {
                                                    counterpart_accounts()
                                                } else if bt == "product" {
                                                    inventory_accounts()
                                                } else {
                                                    accounts.get()
                                                };
                                                accs.into_iter().map(|acc| {
                                                    let selected = form_account_id.get() == Some(acc.id);
                                                    view! {
                                                        <option value={acc.id.to_string()} selected=selected>
                                                            {format!("{} - {}", acc.code, acc.name)}
                                                        </option>
                                                    }
                                                }).collect_view()
                                            }}
                                        </select>
                                    </div>

                                    // Counterpart (if type is counterpart)
                                    {move || {
                                        if form_balance_type.get() == "counterpart" {
                                            view! {
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Контрагент"</label>
                                                    <select
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                        on:change=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            form_counterpart_id.set(val.parse().ok());
                                                        }
                                                    >
                                                        <option value="">"-- Изберете контрагент --"</option>
                                                        {move || counterparts.get().into_iter().map(|c| {
                                                            let selected = form_counterpart_id.get() == Some(c.id);
                                                            view! {
                                                                <option value={c.id.to_string()} selected=selected>
                                                                    {c.name}
                                                                </option>
                                                            }
                                                        }).collect_view()}
                                                    </select>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }
                                    }}

                                    // Product (if type is product)
                                    {move || {
                                        if form_balance_type.get() == "product" {
                                            view! {
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Продукт/Материал"</label>
                                                    <select
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                        on:change=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            form_product_id.set(val.parse().ok());
                                                        }
                                                    >
                                                        <option value="">"-- Изберете продукт --"</option>
                                                        {move || products.get().into_iter().map(|p| {
                                                            let selected = form_product_id.get() == Some(p.id);
                                                            view! {
                                                                <option value={p.id.to_string()} selected=selected>
                                                                    {format!("{} - {}", p.code, p.name)}
                                                                </option>
                                                            }
                                                        }).collect_view()}
                                                    </select>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }
                                    }}

                                    // Debit & Credit
                                    <div style="display: flex; gap: 15px;">
                                        <div style="flex: 1;">
                                            <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Дебит"</label>
                                            <input
                                                type="text"
                                                inputmode="decimal"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    form_debit.set(val.parse().unwrap_or(0.0));
                                                }
                                                prop:value=move || form_debit.get().to_string()
                                            />
                                        </div>
                                        <div style="flex: 1;">
                                            <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Кредит"</label>
                                            <input
                                                type="text"
                                                inputmode="decimal"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    form_credit.set(val.parse().unwrap_or(0.0));
                                                }
                                                prop:value=move || form_credit.get().to_string()
                                            />
                                        </div>
                                    </div>

                                    // Quantity (for products)
                                    {move || {
                                        if form_balance_type.get() == "product" {
                                            view! {
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Количество"</label>
                                                    <input
                                                        type="text"
                                                        inputmode="decimal"
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        on:input=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            form_quantity.set(val.parse().ok());
                                                        }
                                                        prop:value=move || form_quantity.get().map(|q| q.to_string()).unwrap_or_default()
                                                    />
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }
                                    }}

                                    // Description
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Описание"</label>
                                        <input
                                            type="text"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            placeholder="Описание на салдото..."
                                            on:input=move |ev| form_description.set(event_target_value(&ev))
                                            prop:value=move || form_description.get()
                                        />
                                    </div>

                                    // Buttons
                                    <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 15px;">
                                        <button
                                            style="padding: 10px 20px; border: 1px solid #ddd; background: white; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| show_form.set(false)
                                        >
                                            "Отказ"
                                        </button>
                                        <button
                                            style="padding: 10px 20px; background: #3498db; color: white; border: none; border-radius: 4px; cursor: pointer;"
                                            on:click=save_balance
                                        >
                                            {if editing_id.get().is_some() { "Запази" } else { "Добави" }}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

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

            // Balances table
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                let filtered = filtered_balances();

                if filtered.is_empty() {
                    return view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 1.1em;">
                                "Няма начални салда"
                            </p>
                        </div>
                    }.into_view();
                }

                view! {
                    <div style="background: white; border-radius: 8px; margin-top: 20px; overflow: hidden;">
                        <table style="width: 100%; border-collapse: collapse;">
                            <thead style="background: #34495e; color: white;">
                                <tr>
                                    <th style="padding: 12px; text-align: left;">"Дата"</th>
                                    <th style="padding: 12px; text-align: left;">"Тип"</th>
                                    <th style="padding: 12px; text-align: left;">"Сметка"</th>
                                    <th style="padding: 12px; text-align: left;">"Аналитичност"</th>
                                    <th style="padding: 12px; text-align: right;">"Дебит"</th>
                                    <th style="padding: 12px; text-align: right;">"Кредит"</th>
                                    <th style="padding: 12px; text-align: right;">"Кол."</th>
                                    <th style="padding: 12px; text-align: center;">"Действия"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {filtered.into_iter().map(|balance| {
                                    let balance_clone = balance.clone();
                                    let balance_clone2 = balance.clone();
                                    let type_color = get_balance_type_color(&balance.balance_type);
                                    let type_label = get_balance_type_label(&balance.balance_type);

                                    let analyticity = match balance.balance_type.as_str() {
                                        "counterpart" => balance.counterpart.as_ref()
                                            .map(|c| c.name.clone())
                                            .unwrap_or_else(|| "-".to_string()),
                                        "product" => balance.product.as_ref()
                                            .map(|p| format!("{} - {}", p.code, p.name))
                                            .unwrap_or_else(|| "-".to_string()),
                                        _ => "-".to_string(),
                                    };

                                    view! {
                                        <tr style="border-bottom: 1px solid #ecf0f1;">
                                            <td style="padding: 12px;">{balance.date.clone()}</td>
                                            <td style="padding: 12px;">
                                                <span style=format!(
                                                    "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                    type_color
                                                )>
                                                    {type_label}
                                                </span>
                                            </td>
                                            <td style="padding: 12px;">
                                                <span style="font-family: monospace; font-weight: bold;">{balance.account.code.clone()}</span>
                                                " "{balance.account.name.clone()}
                                            </td>
                                            <td style="padding: 12px; color: #7f8c8d;">{analyticity}</td>
                                            <td style="padding: 12px; text-align: right; color: #27ae60; font-weight: bold;">
                                                {if balance.debit > 0.0 { format!("{:.2}", balance.debit) } else { "-".to_string() }}
                                            </td>
                                            <td style="padding: 12px; text-align: right; color: #e74c3c; font-weight: bold;">
                                                {if balance.credit > 0.0 { format!("{:.2}", balance.credit) } else { "-".to_string() }}
                                            </td>
                                            <td style="padding: 12px; text-align: right;">
                                                {balance.quantity.map(|q| format!("{:.3}", q)).unwrap_or_else(|| "-".to_string())}
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                <button
                                                    style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-right: 5px; font-size: 12px;"
                                                    title="Редактиране"
                                                    on:click=move |_| open_edit_form(balance_clone.clone())
                                                >
                                                    "✏"
                                                </button>
                                                <button
                                                    style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                    title="Изтриване"
                                                    on:click=move |_| delete_balance(balance_clone2.id)
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
        </Layout>
    }
}
