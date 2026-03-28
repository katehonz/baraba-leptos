use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Currency {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub name_bg: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: i32,
    pub is_active: bool,
    pub is_base_currency: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub id: i64,
    pub from_currency_id: i64,
    pub to_currency_id: i64,
    pub rate: f64,
    pub reverse_rate: Option<f64>,
    pub valid_date: String,
    pub rate_source: String,
    pub is_active: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IsoCurrency {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub name_bg: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: i32,
}

fn get_source_name(source: &str) -> &'static str {
    match source {
        "ecb" => "ECB (ЕЦБ)",
        "fixed" => "Фиксиран курс",
        "manual" => "Ръчно въведен",
        _ => "Друг",
    }
}

fn get_source_color(source: &str) -> &'static str {
    match source {
        "ecb" => "#3498db",
        "fixed" => "#27ae60",
        "manual" => "#f39c12",
        _ => "#7f8c8d",
    }
}

#[component]
pub fn Currencies() -> impl IntoView {
    // State
    let currencies = create_rw_signal(Vec::<Currency>::new());
    let exchange_rates = create_rw_signal(Vec::<ExchangeRate>::new());
    let iso_currencies = create_rw_signal(Vec::<IsoCurrency>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());
    let active_tab = create_rw_signal("currencies".to_string()); // currencies | rates

    // ECB Import dialog state
    let show_ecb_dialog = create_rw_signal(false);
    let import_type = create_rw_signal("latest".to_string()); // latest | date | month
    let import_date = create_rw_signal(String::new());
    let import_year = create_rw_signal(2026i32);
    let import_month = create_rw_signal(1i32);
    let importing = create_rw_signal(false);

    // New currency dialog state
    let show_new_currency_dialog = create_rw_signal(false);
    let selected_iso_currency = create_rw_signal(String::new());

    // Initialize import date with today
    create_effect(move |_| {
        let today = js_sys::Date::new_0();
        let year = today.get_full_year();
        let month = today.get_month() + 1;
        let day = today.get_date();
        import_date.set(format!("{:04}-{:02}-{:02}", year, month, day));
        import_year.set(year as i32);
        import_month.set(month as i32);
    });

    // Fetch data on mount
    create_effect(move |_| {
        spawn_local(async move {
            loading.set(true);
            error_msg.set(String::new());

            // Fetch currencies
            match api_get("/api/currencies").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let currs: Vec<Currency> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        currencies.set(currs);
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане на валути: {}", e));
                }
            }

            // Fetch exchange rates
            match api_get("/api/exchange_rates").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let rates: Vec<ExchangeRate> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        exchange_rates.set(rates);
                    }
                }
                Err(e) => {
                    if error_msg.get().is_empty() {
                        error_msg.set(format!("Грешка при зареждане на обменни курсове: {}", e));
                    }
                }
            }

            // Fetch ISO currencies for reference
            match api_get("/api/nomenclatures/iso/currencies").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let iso_currs: Vec<IsoCurrency> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        iso_currencies.set(iso_currs);
                    }
                }
                Err(_) => {
                    // ISO currencies are optional, don't show error
                }
            }

            loading.set(false);
        });
    });

    // ECB Import handler
    let handle_ecb_import = move |_| {
        importing.set(true);
        error_msg.set(String::new());
        success_msg.set(String::new());

        let import_type_val = import_type.get();
        let import_date_val = import_date.get();
        let year = import_year.get();
        let month = import_month.get();

        spawn_local(async move {
            let result = match import_type_val.as_str() {
                "latest" => {
                    api_post("/api/exchange_rates/fetch_latest", &serde_json::json!({})).await
                }
                "date" => {
                    api_post(&format!("/api/exchange_rates/fetch_date?date={}", import_date_val), &serde_json::json!({})).await
                }
                "month" => {
                    api_post(&format!("/api/exchange_rates/fetch_month?year={}&month={}", year, month), &serde_json::json!({})).await
                }
                _ => Err("Невалиден тип импорт".to_string())
            };

            match result {
                Ok(data) => {
                    let count = data.get("imported_count")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    success_msg.set(format!("Успешно импортирани {} курса от ЕЦБ", count));
                    show_ecb_dialog.set(false);

                    // Refresh exchange rates
                    if let Ok(data) = api_get("/api/exchange_rates").await {
                        if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                            let rates: Vec<ExchangeRate> = arr.iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect();
                            exchange_rates.set(rates);
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при импорт от ЕЦБ: {}", e));
                }
            }
            importing.set(false);
        });
    };

    // Add currency handler
    let handle_add_currency = move |_| {
        let code = selected_iso_currency.get();
        if code.is_empty() {
            error_msg.set("Моля изберете валута".to_string());
            return;
        }

        // Find the ISO currency
        let iso_curr = iso_currencies.get()
            .into_iter()
            .find(|c| c.code == code);

        if let Some(iso) = iso_curr {
            spawn_local(async move {
                let payload = serde_json::json!({
                    "currency": {
                        "code": iso.code,
                        "name": iso.name,
                        "name_bg": iso.name_bg,
                        "symbol": iso.symbol,
                        "decimal_places": iso.decimal_places,
                        "is_active": true,
                        "is_base_currency": iso.code == "EUR"
                    }
                });

                match api_post("/api/currencies", &payload).await {
                    Ok(_) => {
                        success_msg.set(format!("Валута {} добавена успешно", code));
                        show_new_currency_dialog.set(false);
                        selected_iso_currency.set(String::new());

                        // Refresh currencies
                        if let Ok(data) = api_get("/api/currencies").await {
                            if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                                let currs: Vec<Currency> = arr.iter()
                                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                    .collect();
                                currencies.set(currs);
                            }
                        }
                    }
                    Err(e) => {
                        error_msg.set(format!("Грешка при добавяне на валута: {}", e));
                    }
                }
            });
        }
    };

    // Get currency code by ID
    let get_currency_code = move |id: i64| {
        currencies.get()
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.code.clone())
            .unwrap_or_else(|| "???".to_string())
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Валути и обменни курсове"</h1>
                <div style="display: flex; gap: 10px;">
                    <button
                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=move |_| show_ecb_dialog.set(true)
                    >
                        "Импорт от ЕЦБ"
                    </button>
                    <button
                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=move |_| show_new_currency_dialog.set(true)
                    >
                        "+ Нова валута"
                    </button>
                </div>
            </div>

            // Info box about Eurozone
            <div style="background: #e8f6f3; border-left: 4px solid #27ae60; padding: 15px; margin-top: 20px; border-radius: 4px;">
                <p style="margin: 0; color: #1e8449;">
                    <strong>"България е член на Еврозоната."</strong>
                    " EUR е основната валута. Обменните курсове се използват за справочни цели и отчитане в други валути."
                </p>
            </div>

            // Success message
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-top: 15px; display: flex; justify-content: space-between; align-items: center;">
                            <span>{msg}</span>
                            <button
                                style="background: transparent; border: none; color: white; cursor: pointer; font-size: 18px;"
                                on:click=move |_| success_msg.set(String::new())
                            >
                                "×"
                            </button>
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
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 15px; display: flex; justify-content: space-between; align-items: center;">
                            <span>{err}</span>
                            <button
                                style="background: transparent; border: none; color: white; cursor: pointer; font-size: 18px;"
                                on:click=move |_| error_msg.set(String::new())
                            >
                                "×"
                            </button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Tabs
            <div style="display: flex; gap: 0; margin-top: 20px; border-bottom: 2px solid #ddd;">
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; {}",
                        if active_tab.get() == "currencies" {
                            "background: #3498db; color: white; border-radius: 8px 8px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 8px 8px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set("currencies".to_string())
                >
                    "Валути"
                </button>
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; {}",
                        if active_tab.get() == "rates" {
                            "background: #3498db; color: white; border-radius: 8px 8px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 8px 8px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set("rates".to_string())
                >
                    "Обменни курсове"
                </button>
            </div>

            // Loading state
            {move || {
                if loading.get() {
                    view! {
                        <div style="background: white; border-radius: 0 0 8px 8px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d;">"Зареждане..."</p>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Currencies tab
            {move || {
                if loading.get() || active_tab.get() != "currencies" {
                    return view! { <span></span> }.into_view();
                }

                let currs = currencies.get();

                if currs.is_empty() {
                    return view! {
                        <div style="background: white; border-radius: 0 0 8px 8px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 1.1em;">"Няма добавени валути"</p>
                            <p style="color: #95a5a6; margin-top: 10px;">"Добавете EUR като базова валута и други валути за справка."</p>
                        </div>
                    }.into_view();
                }

                view! {
                    <div style="background: white; border-radius: 0 0 8px 8px; overflow: hidden;">
                        <table style="width: 100%; border-collapse: collapse;">
                            <thead style="background: #34495e; color: white;">
                                <tr>
                                    <th style="padding: 12px; text-align: left; width: 80px;">"Код"</th>
                                    <th style="padding: 12px; text-align: left;">"Наименование"</th>
                                    <th style="padding: 12px; text-align: left;">"На български"</th>
                                    <th style="padding: 12px; text-align: center; width: 80px;">"Символ"</th>
                                    <th style="padding: 12px; text-align: center; width: 100px;">"Знаци"</th>
                                    <th style="padding: 12px; text-align: center; width: 100px;">"Статус"</th>
                                    <th style="padding: 12px; text-align: center; width: 100px;">"Действия"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {currs.into_iter().map(|curr| {
                                    view! {
                                        <tr style="border-bottom: 1px solid #ecf0f1;">
                                            <td style="padding: 12px; font-family: monospace; font-weight: bold;">
                                                {curr.code.clone()}
                                                {if curr.is_base_currency {
                                                    view! {
                                                        <span style="margin-left: 8px; background: #27ae60; color: white; padding: 2px 6px; border-radius: 4px; font-size: 10px;">
                                                            "BASE"
                                                        </span>
                                                    }.into_view()
                                                } else {
                                                    view! { <span></span> }.into_view()
                                                }}
                                            </td>
                                            <td style="padding: 12px;">{curr.name}</td>
                                            <td style="padding: 12px; color: #7f8c8d;">{curr.name_bg.unwrap_or_default()}</td>
                                            <td style="padding: 12px; text-align: center; font-size: 18px;">{curr.symbol.unwrap_or_default()}</td>
                                            <td style="padding: 12px; text-align: center;">{curr.decimal_places}</td>
                                            <td style="padding: 12px; text-align: center;">
                                                {if curr.is_active {
                                                    view! {
                                                        <span style="background: #27ae60; color: white; padding: 4px 10px; border-radius: 12px; font-size: 12px;">
                                                            "Активна"
                                                        </span>
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <span style="background: #95a5a6; color: white; padding: 4px 10px; border-radius: 12px; font-size: 12px;">
                                                            "Неактивна"
                                                        </span>
                                                    }.into_view()
                                                }}
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                <button
                                                    style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                    title="Редактиране"
                                                >
                                                    "✏"
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

            // Exchange Rates tab
            {move || {
                if loading.get() || active_tab.get() != "rates" {
                    return view! { <span></span> }.into_view();
                }

                let rates = exchange_rates.get();

                if rates.is_empty() {
                    return view! {
                        <div style="background: white; border-radius: 0 0 8px 8px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 1.1em;">"Няма добавени обменни курсове"</p>
                            <p style="color: #95a5a6; margin-top: 10px;">"Импортирайте курсове от ЕЦБ или добавете ръчно."</p>
                        </div>
                    }.into_view();
                }

                view! {
                    <div style="background: white; border-radius: 0 0 8px 8px; overflow: hidden;">
                        <table style="width: 100%; border-collapse: collapse;">
                            <thead style="background: #34495e; color: white;">
                                <tr>
                                    <th style="padding: 12px; text-align: left; width: 100px;">"От"</th>
                                    <th style="padding: 12px; text-align: left; width: 100px;">"Към"</th>
                                    <th style="padding: 12px; text-align: right; width: 150px;">"Курс"</th>
                                    <th style="padding: 12px; text-align: right; width: 150px;">"Обратен"</th>
                                    <th style="padding: 12px; text-align: center; width: 120px;">"Дата"</th>
                                    <th style="padding: 12px; text-align: center; width: 120px;">"Източник"</th>
                                    <th style="padding: 12px; text-align: left;">"Бележки"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {rates.into_iter().map(|rate| {
                                    let from_code = get_currency_code(rate.from_currency_id);
                                    let to_code = get_currency_code(rate.to_currency_id);
                                    let source_name = get_source_name(&rate.rate_source);
                                    let source_color = get_source_color(&rate.rate_source);

                                    view! {
                                        <tr style="border-bottom: 1px solid #ecf0f1;">
                                            <td style="padding: 12px; font-family: monospace; font-weight: bold;">{from_code}</td>
                                            <td style="padding: 12px; font-family: monospace; font-weight: bold;">{to_code}</td>
                                            <td style="padding: 12px; text-align: right; font-family: monospace;">
                                                {format!("{:.6}", rate.rate)}
                                            </td>
                                            <td style="padding: 12px; text-align: right; font-family: monospace; color: #7f8c8d;">
                                                {rate.reverse_rate.map(|r| format!("{:.6}", r)).unwrap_or_else(|| "-".to_string())}
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                {rate.valid_date.split('T').next().unwrap_or(&rate.valid_date).to_string()}
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                <span style=format!(
                                                    "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                    source_color
                                                )>
                                                    {source_name}
                                                </span>
                                            </td>
                                            <td style="padding: 12px; color: #7f8c8d; font-size: 13px;">
                                                {rate.notes.unwrap_or_default()}
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_view()
            }}

            // ECB Import Dialog
            {move || {
                if !show_ecb_dialog.get() {
                    return view! { <span></span> }.into_view();
                }

                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                        <div style="background: white; border-radius: 8px; padding: 30px; width: 500px; max-width: 90%;">
                            <h2 style="margin-top: 0; margin-bottom: 20px;">"Импорт на курсове от ЕЦБ"</h2>

                            <p style="color: #7f8c8d; margin-bottom: 20px;">
                                "Избери период за импорт на обменни курсове от Европейската централна банка."
                            </p>

                            // Import type selection
                            <div style="margin-bottom: 20px;">
                                <label style="display: block; margin-bottom: 10px; font-weight: bold;">"Тип импорт:"</label>
                                <div style="display: flex; flex-direction: column; gap: 10px;">
                                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                        <input
                                            type="radio"
                                            name="import_type"
                                            value="latest"
                                            checked=move || import_type.get() == "latest"
                                            on:change=move |_| import_type.set("latest".to_string())
                                        />
                                        "Последни курсове"
                                    </label>
                                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                        <input
                                            type="radio"
                                            name="import_type"
                                            value="date"
                                            checked=move || import_type.get() == "date"
                                            on:change=move |_| import_type.set("date".to_string())
                                        />
                                        "По дата"
                                    </label>
                                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                        <input
                                            type="radio"
                                            name="import_type"
                                            value="month"
                                            checked=move || import_type.get() == "month"
                                            on:change=move |_| import_type.set("month".to_string())
                                        />
                                        "По месец"
                                    </label>
                                </div>
                            </div>

                            // Date input (shown when import_type is "date")
                            {move || {
                                if import_type.get() == "date" {
                                    view! {
                                        <div style="margin-bottom: 20px;">
                                            <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Дата:"</label>
                                            <input
                                                type="date"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                prop:value=move || import_date.get()
                                                on:input=move |ev| import_date.set(event_target_value(&ev))
                                            />
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}

                            // Month/Year inputs (shown when import_type is "month")
                            {move || {
                                if import_type.get() == "month" {
                                    view! {
                                        <div style="margin-bottom: 20px; display: flex; gap: 15px;">
                                            <div style="flex: 1;">
                                                <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Година:"</label>
                                                <select
                                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                    on:change=move |ev| {
                                                        if let Ok(y) = event_target_value(&ev).parse::<i32>() {
                                                            import_year.set(y);
                                                        }
                                                    }
                                                >
                                                    <option value="2026" selected=move || import_year.get() == 2026>"2026"</option>
                                                    <option value="2025" selected=move || import_year.get() == 2025>"2025"</option>
                                                    <option value="2024" selected=move || import_year.get() == 2024>"2024"</option>
                                                </select>
                                            </div>
                                            <div style="flex: 1;">
                                                <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Месец:"</label>
                                                <select
                                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                    on:change=move |ev| {
                                                        if let Ok(m) = event_target_value(&ev).parse::<i32>() {
                                                            import_month.set(m);
                                                        }
                                                    }
                                                >
                                                    <option value="1" selected=move || import_month.get() == 1>"Януари"</option>
                                                    <option value="2" selected=move || import_month.get() == 2>"Февруари"</option>
                                                    <option value="3" selected=move || import_month.get() == 3>"Март"</option>
                                                    <option value="4" selected=move || import_month.get() == 4>"Април"</option>
                                                    <option value="5" selected=move || import_month.get() == 5>"Май"</option>
                                                    <option value="6" selected=move || import_month.get() == 6>"Юни"</option>
                                                    <option value="7" selected=move || import_month.get() == 7>"Юли"</option>
                                                    <option value="8" selected=move || import_month.get() == 8>"Август"</option>
                                                    <option value="9" selected=move || import_month.get() == 9>"Септември"</option>
                                                    <option value="10" selected=move || import_month.get() == 10>"Октомври"</option>
                                                    <option value="11" selected=move || import_month.get() == 11>"Ноември"</option>
                                                    <option value="12" selected=move || import_month.get() == 12>"Декември"</option>
                                                </select>
                                            </div>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}

                            // Dialog buttons
                            <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px;">
                                <button
                                    style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=move |_| show_ecb_dialog.set(false)
                                    disabled=move || importing.get()
                                >
                                    "Отказ"
                                </button>
                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=handle_ecb_import
                                    disabled=move || importing.get()
                                >
                                    {move || if importing.get() { "Импортиране..." } else { "Импорт" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_view()
            }}

            // New Currency Dialog
            {move || {
                if !show_new_currency_dialog.get() {
                    return view! { <span></span> }.into_view();
                }

                let iso_currs = iso_currencies.get();
                let existing_codes: Vec<String> = currencies.get().iter().map(|c| c.code.clone()).collect();
                let available_currs: Vec<IsoCurrency> = iso_currs.into_iter()
                    .filter(|c| !existing_codes.contains(&c.code))
                    .collect();

                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                        <div style="background: white; border-radius: 8px; padding: 30px; width: 500px; max-width: 90%;">
                            <h2 style="margin-top: 0; margin-bottom: 20px;">"Добавяне на валута"</h2>

                            <div style="margin-bottom: 20px;">
                                <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Изберете валута:"</label>
                                <select
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                    on:change=move |ev| selected_iso_currency.set(event_target_value(&ev))
                                >
                                    <option value="">"-- Изберете --"</option>
                                    {available_currs.into_iter().map(|curr| {
                                        let code = curr.code.clone();
                                        let display = format!("{} - {} ({})", curr.code, curr.name, curr.name_bg.unwrap_or_default());
                                        view! {
                                            <option value={code}>{display}</option>
                                        }
                                    }).collect_view()}
                                </select>
                            </div>

                            // Dialog buttons
                            <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px;">
                                <button
                                    style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=move |_| {
                                        show_new_currency_dialog.set(false);
                                        selected_iso_currency.set(String::new());
                                    }
                                >
                                    "Отказ"
                                </button>
                                <button
                                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=handle_add_currency
                                >
                                    "Добави"
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_view()
            }}
        </Layout>
    }
}
