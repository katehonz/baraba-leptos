use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use crate::api::{api_get, company_api_url};
use crate::components::Layout;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChronologicalRow {
    row_number: i32,
    entry_id: i64,
    entry_date: String,
    document_number: String,
    description: String,
    counterpart_name: String,
    debit_account: String,
    credit_account: String,
    debit_amount: f64,
    credit_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountLedgerRow {
    date: String,
    document_number: String,
    description: String,
    counterpart_name: String,
    correspondent_account: String,
    debit: f64,
    credit: f64,
    balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrialBalanceRow {
    account_code: String,
    account_name: String,
    counterpart_id: Option<i64>,
    counterpart_name: Option<String>,
    opening_debit: f64,
    opening_credit: f64,
    turnover_debit: f64,
    turnover_credit: f64,
    closing_debit: f64,
    closing_credit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeneralLedgerTransaction {
    entry_date: String,
    document_number: String,
    description: String,
    counterpart_name: String,
    counterpart_codes: String,
    debit: f64,
    credit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeneralLedgerAccount {
    account_code: String,
    account_name: String,
    opening_debit: f64,
    opening_credit: f64,
    transactions: Vec<GeneralLedgerTransaction>,
    turnover_debit: f64,
    turnover_credit: f64,
    closing_debit: f64,
    closing_credit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CounterpartOption {
    id: i64,
    name: String,
    eik: Option<String>,
}

#[component]
pub fn ReportsPage() -> impl IntoView {
    // State
    let active_report = create_rw_signal(String::from("trial_balance"));
    let date_from = create_rw_signal(String::new());
    let date_to = create_rw_signal(String::new());
    let account_filter = create_rw_signal(String::new());
    let group_by_level = create_rw_signal(String::from("0"));
    let loading = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());

    // Trial balance data
    let tb_rows = create_rw_signal(Vec::<TrialBalanceRow>::new());
    let tb_totals = create_rw_signal(Option::<serde_json::Value>::None);
    let tb_company = create_rw_signal(Option::<serde_json::Value>::None);

    // Chronological data
    let chr_rows = create_rw_signal(Vec::<ChronologicalRow>::new());
    let chr_totals = create_rw_signal(Option::<serde_json::Value>::None);

    // General ledger data
    let gl_accounts = create_rw_signal(Vec::<GeneralLedgerAccount>::new());
    let gl_totals = create_rw_signal(Option::<serde_json::Value>::None);

    // Trial balance by counterpart data
    let tbc_rows = create_rw_signal(Vec::<TrialBalanceRow>::new());
    let tbc_totals = create_rw_signal(Option::<serde_json::Value>::None);
    let tbc_counterpart_name = create_rw_signal(String::new());

    // Account ledger data
    let al_rows = create_rw_signal(Vec::<AccountLedgerRow>::new());
    let al_totals = create_rw_signal(Option::<serde_json::Value>::None);
    let al_opening = create_rw_signal(Option::<serde_json::Value>::None);
    let al_account_info = create_rw_signal(Option::<serde_json::Value>::None);

    // Counterpart selector state
    let counterparts = create_rw_signal(Vec::<CounterpartOption>::new());
    let selected_counterpart_id = create_rw_signal(String::new());
    let counterpart_search = create_rw_signal(String::new());
    let show_counterpart_dropdown = create_rw_signal(false);
    let selected_counterpart_label = create_rw_signal(String::new());

    // Fetch counterparts list
    create_effect(move |_| {
        spawn_local(async move {
            let url = format!("{}/counterparts", company_api_url());
            if let Ok(response) = api_get(&url).await {
                if let Some(data) = response.get("data").and_then(|d| d.as_array()) {
                    let parsed: Vec<CounterpartOption> = data.iter()
                        .filter_map(|c| {
                            let id = c.get("id")?.as_i64()?;
                            let name = c.get("name")?.as_str()?.to_string();
                            let eik = c.get("eik").and_then(|v| v.as_str()).map(|s| s.to_string());
                            Some(CounterpartOption { id, name, eik })
                        })
                        .collect();
                    counterparts.set(parsed);
                }
            }
        });
    });

    // Load trial balance
    let load_trial_balance = move || {
        loading.set(true);
        error_msg.set(String::new());

        let df = date_from.get();
        let dt = date_to.get();
        let acc = account_filter.get();
        let level = group_by_level.get();

        spawn_local(async move {
            let url = format!(
                "{}/reports/trial_balance?date_from={}&date_to={}&account_filter={}&group_by_level={}",
                company_api_url(), df, dt, acc, level
            );

            match api_get(&url).await {
                Ok(response) => {
                    if let Some(data) = response.get("data") {
                        // Parse rows
                        if let Some(rows) = data.get("rows").and_then(|r| r.as_array()) {
                            let parsed: Vec<TrialBalanceRow> = rows.iter()
                                .filter_map(|r| serde_json::from_value(r.clone()).ok())
                                .collect();
                            tb_rows.set(parsed);
                        }
                        // Store totals and company
                        tb_totals.set(data.get("totals").cloned());
                        tb_company.set(data.get("company").cloned());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Load chronological
    let load_chronological = move || {
        loading.set(true);
        error_msg.set(String::new());

        let df = date_from.get();
        let dt = date_to.get();
        let acc = account_filter.get();
        let cp_id = selected_counterpart_id.get();

        spawn_local(async move {
            let url = format!(
                "{}/reports/chronological?date_from={}&date_to={}&account_code={}&counterpart_id={}",
                company_api_url(), df, dt, acc, cp_id
            );

            match api_get(&url).await {
                Ok(response) => {
                    if let Some(data) = response.get("data") {
                        if let Some(rows) = data.get("rows").and_then(|r| r.as_array()) {
                            let parsed: Vec<ChronologicalRow> = rows.iter()
                                .filter_map(|r| serde_json::from_value(r.clone()).ok())
                                .collect();
                            chr_rows.set(parsed);
                        }
                        chr_totals.set(data.get("totals").cloned());
                        tb_company.set(data.get("company").cloned());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Load general ledger
    let load_general_ledger = move || {
        loading.set(true);
        error_msg.set(String::new());

        let df = date_from.get();
        let dt = date_to.get();
        let acc = account_filter.get();

        spawn_local(async move {
            let url = format!(
                "{}/reports/general_ledger?date_from={}&date_to={}&account_filter={}",
                company_api_url(), df, dt, acc
            );

            match api_get(&url).await {
                Ok(response) => {
                    if let Some(data) = response.get("data") {
                        if let Some(accounts) = data.get("accounts").and_then(|r| r.as_array()) {
                            let parsed: Vec<GeneralLedgerAccount> = accounts.iter()
                                .filter_map(|a| serde_json::from_value(a.clone()).ok())
                                .collect();
                            gl_accounts.set(parsed);
                        }
                        gl_totals.set(data.get("totals").cloned());
                        tb_company.set(data.get("company").cloned());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Load trial balance by counterpart
    let load_trial_balance_by_counterpart = move || {
        loading.set(true);
        error_msg.set(String::new());

        let df = date_from.get();
        let dt = date_to.get();
        let acc = account_filter.get();
        let cp_id = selected_counterpart_id.get();

        spawn_local(async move {
            let url = format!(
                "{}/reports/trial_balance_by_counterpart?date_from={}&date_to={}&counterpart_id={}&account_filter={}",
                company_api_url(), df, dt, cp_id, acc
            );

            match api_get(&url).await {
                Ok(response) => {
                    if let Some(data) = response.get("data") {
                        if let Some(rows) = data.get("rows").and_then(|r| r.as_array()) {
                            let parsed: Vec<TrialBalanceRow> = rows.iter()
                                .filter_map(|r| serde_json::from_value(r.clone()).ok())
                                .collect();
                            tbc_rows.set(parsed);
                        }
                        tbc_totals.set(data.get("totals").cloned());
                        tb_company.set(data.get("company").cloned());
                        if let Some(cp) = data.get("counterpart") {
                            let name = cp.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            tbc_counterpart_name.set(name);
                        } else {
                            tbc_counterpart_name.set("Всички контрагенти".to_string());
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Load account ledger
    let load_account_ledger = move || {
        let acc_val = account_filter.get();
        if acc_val.is_empty() {
            error_msg.set("Моля, въведете сметка".to_string());
            return;
        }

        loading.set(true);
        error_msg.set(String::new());

        let df = date_from.get();
        let dt = date_to.get();
        let cp_id = selected_counterpart_id.get();

        spawn_local(async move {
            // First we need to find the account ID for the code
            let accounts_url = format!("{}/accounts", company_api_url());
            let mut target_account_id = None;
            
            if let Ok(response) = api_get(&accounts_url).await {
                if let Some(data) = response.get("data").and_then(|d| d.as_array()) {
                    for acc in data {
                        if acc.get("code").and_then(|c| c.as_str()) == Some(&acc_val) {
                            target_account_id = acc.get("id").and_then(|i| i.as_i64());
                            break;
                        }
                    }
                }
            }

            if let Some(acc_id) = target_account_id {
                let url = format!(
                    "{}/reports/account_ledger?date_from={}&date_to={}&account_id={}&counterpart_id={}",
                    company_api_url(), df, dt, acc_id, cp_id
                );

                match api_get(&url).await {
                    Ok(response) => {
                        if let Some(data) = response.get("data") {
                            if let Some(rows) = data.get("rows").and_then(|r| r.as_array()) {
                                let parsed: Vec<AccountLedgerRow> = rows.iter()
                                    .filter_map(|r| serde_json::from_value(r.clone()).ok())
                                    .collect();
                                al_rows.set(parsed);
                            }
                            al_totals.set(data.get("totals").cloned());
                            al_opening.set(data.get("opening_balance").cloned());
                            al_account_info.set(data.get("account").cloned());
                            tb_company.set(data.get("company").cloned());
                        }
                    }
                    Err(e) => {
                        error_msg.set(format!("Грешка: {}", e));
                    }
                }
            } else {
                error_msg.set(format!("Сметка {} не е намерена", acc_val));
            }
            loading.set(false);
        });
    };

    // Generate report
    let generate_report = move |_| {
        let report = active_report.get();
        if report == "trial_balance" {
            load_trial_balance();
        } else if report == "general_ledger" {
            load_general_ledger();
        } else if report == "trial_balance_counterpart" {
            load_trial_balance_by_counterpart();
        } else if report == "account_ledger" {
            load_account_ledger();
        } else {
            load_chronological();
        }
    };


    // Export to Excel (using SheetJS)
    let export_excel = move |_| {
        let report_type = active_report.get();
        let _ = js_sys::eval(&format!(r#"
            (function() {{
                if (typeof XLSX === 'undefined') {{
                    alert('SheetJS не е зареден. Моля, опреснете страницата.');
                    return;
                }}

                var container = document.getElementById('report-content');
                if (!container) {{
                    alert('Моля, първо генерирайте отчета.');
                    return;
                }}

                var tables = container.querySelectorAll('table');
                if (tables.length === 0) {{
                    alert('Моля, първо генерирайте отчета.');
                    return;
                }}

                if (tables.length === 1) {{
                    var wb = XLSX.utils.table_to_book(tables[0], {{sheet: "Отчет"}});
                    XLSX.writeFile(wb, '{}_report.xlsx');
                }} else {{
                    // Multiple tables (General Ledger) - combine into one sheet
                    var wb = XLSX.utils.book_new();
                    var ws = XLSX.utils.table_to_sheet(tables[0]);
                    for (var i = 1; i < tables.length; i++) {{
                        var range = XLSX.utils.decode_range(ws['!ref'] || 'A1');
                        var startRow = range.e.r + 2;
                        XLSX.utils.sheet_add_dom(ws, tables[i], {{origin: startRow}});
                    }}
                    XLSX.utils.book_append_sheet(wb, ws, "Главна книга");
                    XLSX.writeFile(wb, '{}_report.xlsx');
                }}
            }})();
        "#, report_type, report_type));
    };

    // Print / Export PDF - opens in new tab
    let print_report = move |_| {
        let _ = js_sys::eval(r#"
            (function() {
                var reportContent = document.getElementById('report-content');
                if (!reportContent) {
                    alert('Моля, първо генерирайте отчета.');
                    return;
                }

                var printWindow = window.open('', '_blank');
                printWindow.document.write(`
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <meta charset="UTF-8">
                        <title>Отчет</title>
                        <style>
                            body {
                                font-family: Arial, sans-serif;
                                padding: 20px;
                                margin: 0;
                            }
                            table {
                                width: 100%;
                                border-collapse: collapse;
                                font-size: 11px;
                            }
                            th, td {
                                border: 1px solid #333;
                                padding: 6px;
                            }
                            th {
                                background: #34495e;
                                color: white;
                            }
                            tfoot td {
                                background: #ecf0f1;
                                font-weight: bold;
                            }
                            h2, h3 {
                                text-align: center;
                                margin: 10px 0;
                            }
                            p {
                                text-align: center;
                                color: #666;
                            }
                            .gl-account-header {
                                background: #2c3e50;
                                color: white;
                                padding: 8px 12px;
                                margin-top: 20px;
                                font-weight: bold;
                            }
                            @media print {
                                @page {
                                    size: landscape;
                                    margin: 1cm;
                                }
                                .gl-account-header {
                                    break-before: auto;
                                }
                            }
                        </style>
                    </head>
                    <body>
                        ${reportContent.innerHTML}
                        <script>
                            window.onload = function() {
                                window.print();
                            };
                        <\/script>
                    </body>
                    </html>
                `);
                printWindow.document.close();
            })();
        "#);
    };

    view! {
        <Layout>
        <div style="padding: 20px; max-width: 1400px; margin: 0 auto;">
            <h2 style="color: #2c3e50; margin-bottom: 20px;">"Отчети"</h2>

            // Report selector and filters
            <div style="background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                <div style="display: flex; gap: 20px; flex-wrap: wrap; align-items: flex-end;">
                    // Report type
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">"Тип отчет"</label>
                        <select
                            style="padding: 10px; border: 1px solid #ddd; border-radius: 4px; min-width: 200px;"
                            on:change=move |ev| {
                                active_report.set(event_target_value(&ev));
                                tb_rows.set(vec![]);
                                chr_rows.set(vec![]);
                                gl_accounts.set(vec![]);
                                tbc_rows.set(vec![]);
                                tbc_totals.set(None);
                            }
                        >
                            <option value="trial_balance" selected>"Оборотна ведомост"</option>
                            <option value="trial_balance_counterpart">"Оборотна ведомост по контрагенти"</option>
                            <option value="account_ledger">"Картон на сметка"</option>
                            <option value="general_ledger">"Главна книга"</option>
                            <option value="chronological">"Хронологична справка"</option>
                        </select>
                    </div>

                    // Synthetic aggregation level
                    {move || {
                        if active_report.get() == "trial_balance" {
                            view! {
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">"Ниво (синтетично)"</label>
                                    <select
                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                        on:change=move |ev| group_by_level.set(event_target_value(&ev))
                                    >
                                        <option value="0" selected>"Аналитично (всички)"</option>
                                        <option value="1">"1-ви знак"</option>
                                        <option value="2">"2-ри знак"</option>
                                        <option value="3">"3-ти знак"</option>
                                        <option value="4">"4-ти знак"</option>
                                    </select>
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }
                    }}

                    // Date from
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">"От дата"</label>
                        <input
                            type="date"
                            style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                            prop:value=move || date_from.get()
                            on:input=move |ev| date_from.set(event_target_value(&ev))
                        />
                    </div>

                    // Date to
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">"До дата"</label>
                        <input
                            type="date"
                            style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                            prop:value=move || date_to.get()
                            on:input=move |ev| date_to.set(event_target_value(&ev))
                        />
                    </div>

                    // Counterpart selector (shown for certain report types)
                    {move || {
                        let report = active_report.get();
                        if report == "trial_balance_counterpart" || report == "chronological" || report == "account_ledger" {
                            view! {
                                <div style="position: relative;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                                        {if report == "trial_balance_counterpart" { "Контрагент (опц.)" } else { "Контрагент" }}
                                    </label>
                                    <input
                                        type="text"
                                        placeholder="Търсене на контрагент..."
                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px; width: 250px;"
                                        prop:value=move || {
                                            if show_counterpart_dropdown.get() {
                                                counterpart_search.get()
                                            } else {
                                                selected_counterpart_label.get()
                                            }
                                        }
                                        on:input=move |ev| {
                                            counterpart_search.set(event_target_value(&ev));
                                            show_counterpart_dropdown.set(true);
                                        }
                                        on:focus=move |_| {
                                            show_counterpart_dropdown.set(true);
                                            counterpart_search.set(String::new());
                                        }
                                    />
                                    <button 
                                        style="position: absolute; right: 5px; top: 32px; background: none; border: none; cursor: pointer; color: #999;"
                                        on:click=move |_| {
                                            selected_counterpart_id.set(String::new());
                                            selected_counterpart_label.set(String::new());
                                        }
                                    >
                                        "✕"
                                    </button>
                                    {move || {
                                        if show_counterpart_dropdown.get() {
                                            let search = counterpart_search.get().to_lowercase();
                                            let filtered: Vec<CounterpartOption> = counterparts.get().into_iter()
                                                .filter(|c| {
                                                    if search.is_empty() { return true; }
                                                    c.name.to_lowercase().contains(&search) ||
                                                    c.eik.as_deref().unwrap_or("").contains(&search)
                                                })
                                                .collect();

                                            if filtered.is_empty() {
                                                view! {
                                                    <div style="position: absolute; top: 100%; left: 0; right: 0; background: white; border: 1px solid #ddd; border-radius: 0 0 4px 4px; max-height: 250px; overflow-y: auto; z-index: 1000; box-shadow: 0 4px 8px rgba(0,0,0,0.15);">
                                                        <div style="padding: 10px; color: #999; text-align: center;">"Няма резултати"</div>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div style="position: absolute; top: 100%; left: 0; right: 0; background: white; border: 1px solid #ddd; border-radius: 0 0 4px 4px; max-height: 250px; overflow-y: auto; z-index: 1000; box-shadow: 0 4px 8px rgba(0,0,0,0.15);">
                                                        {filtered.into_iter().map(|c| {
                                                            let id = c.id;
                                                            let name = c.name.clone();
                                                            let eik = c.eik.clone().unwrap_or_default();
                                                            let label = if eik.is_empty() {
                                                                name.clone()
                                                            } else {
                                                                format!("{} ({})", name, eik)
                                                            };
                                                            let label_clone = label.clone();
                                                            view! {
                                                                <div
                                                                    style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid #f0f0f0; font-size: 13px;"
                                                                    on:mousedown=move |_| {
                                                                        selected_counterpart_id.set(id.to_string());
                                                                        selected_counterpart_label.set(label_clone.clone());
                                                                        show_counterpart_dropdown.set(false);
                                                                        counterpart_search.set(String::new());
                                                                    }
                                                                >
                                                                    <div style="font-weight: 500;">{name}</div>
                                                                    {if !eik.is_empty() {
                                                                        view! { <div style="font-size: 11px; color: #999;">"ЕИК: "{eik}</div> }.into_view()
                                                                    } else {
                                                                        view! { <div></div> }.into_view()
                                                                    }}
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_view()
                                            }
                                        } else {
                                            view! { <div></div> }.into_view()
                                        }
                                    }}
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }
                    }}

                    // Account filter (shown for all report types)
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                            {move || {
                                let report = active_report.get();
                                if report == "chronological" {
                                    "Сметка (опц.)"
                                } else if report == "account_ledger" {
                                    "Сметка"
                                } else {
                                    "Филтър сметки"
                                }
                            }}
                        </label>
                        <input
                            type="text"
                            placeholder=move || {
                                let report = active_report.get();
                                if report == "chronological" || report == "account_ledger" {
                                    "напр. 401"
                                } else {
                                    "напр. 6* или 601*"
                                }
                            }
                            style="padding: 10px; border: 1px solid #ddd; border-radius: 4px; width: 120px;"
                            prop:value=move || account_filter.get()
                            on:input=move |ev| account_filter.set(event_target_value(&ev))
                        />
                    </div>

                    // Generate button
                    <button
                        style="padding: 10px 25px; background: #3498db; color: white; border: none; border-radius: 4px; cursor: pointer; font-weight: 500;"
                        on:click=generate_report
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Зареждане..." } else { "Генерирай" }}
                    </button>

                    // Export buttons
                    <button
                        style="padding: 10px 20px; background: #27ae60; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        on:click=export_excel
                        title="Експорт в Excel"
                    >
                        "Excel"
                    </button>

                    <button
                        style="padding: 10px 20px; background: #e74c3c; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        on:click=print_report
                        title="Печат / PDF"
                    >
                        "Печат/PDF"
                    </button>
                </div>
            </div>

            // Error message
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                            {err}
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}

            // Report content
            <div id="report-content" style="background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                {move || {
                    let report = active_report.get();
                    if report == "trial_balance" {
                        view! {
                            <TrialBalanceReport
                                rows=tb_rows
                                totals=tb_totals
                                company=tb_company
                                date_from=date_from
                                date_to=date_to
                            />
                        }.into_view()
                    } else if report == "trial_balance_counterpart" {
                        view! {
                            <TrialBalanceByCounterpartReport
                                rows=tbc_rows
                                totals=tbc_totals
                                company=tb_company
                                date_from=date_from
                                date_to=date_to
                                counterpart_name=tbc_counterpart_name
                            />
                        }.into_view()
                    } else if report == "account_ledger" {
                        view! {
                            <AccountLedgerReport
                                rows=al_rows
                                totals=al_totals
                                opening=al_opening
                                account_info=al_account_info
                                company=tb_company
                                date_from=date_from
                                date_to=date_to
                            />
                        }.into_view()
                    } else if report == "general_ledger" {
                        view! {
                            <GeneralLedgerReport
                                accounts=gl_accounts
                                totals=gl_totals
                                company=tb_company
                                date_from=date_from
                                date_to=date_to
                                account_filter=account_filter
                            />
                        }.into_view()
                    } else {
                        view! {
                            <ChronologicalReport
                                rows=chr_rows
                                totals=chr_totals
                                company=tb_company
                                date_from=date_from
                                date_to=date_to
                                account_filter=account_filter
                            />
                        }.into_view()
                    }
                }}
            </div>
        </div>
        </Layout>
    }
}

#[component]
fn TrialBalanceReport(
    rows: RwSignal<Vec<TrialBalanceRow>>,
    totals: RwSignal<Option<serde_json::Value>>,
    company: RwSignal<Option<serde_json::Value>>,
    date_from: RwSignal<String>,
    date_to: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div>
            // Header
            <div style="text-align: center; margin-bottom: 20px;">
                {move || {
                    if let Some(c) = company.get() {
                        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let eik = c.get("eik").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        view! {
                            <div>
                                <h3 style="margin: 0;">{name}</h3>
                                <p style="color: #7f8c8d; margin: 5px 0;">"ЕИК: "{eik}</p>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}
                <h2 style="margin: 15px 0;">"ОБОРОТНА ВЕДОМОСТ"</h2>
                <p style="color: #7f8c8d;">
                    "Период: "{move || date_from.get()}" - "{move || date_to.get()}
                </p>
            </div>

            // Table
            <table id="report-table" style="width: 100%; border-collapse: collapse; font-size: 13px;">
                <thead>
                    <tr style="background: #34495e; color: white;">
                        <th rowspan="2" style="padding: 10px; border: 1px solid #2c3e50; width: 80px;">"Сметка"</th>
                        <th rowspan="2" style="padding: 10px; border: 1px solid #2c3e50;">"Наименование"</th>
                        <th colspan="2" style="padding: 8px; border: 1px solid #2c3e50; background: #2c3e50;">"Начално салдо"</th>
                        <th colspan="2" style="padding: 8px; border: 1px solid #2c3e50; background: #2c3e50;">"Обороти"</th>
                        <th colspan="2" style="padding: 8px; border: 1px solid #2c3e50; background: #2c3e50;">"Крайно салдо"</th>
                    </tr>
                    <tr style="background: #34495e; color: white;">
                        <th style="padding: 8px; border: 1px solid #2c3e50; width: 100px;">"Дебит"</th>
                        <th style="padding: 8px; border: 1px solid #2c3e50; width: 100px;">"Кредит"</th>
                        <th style="padding: 8px; border: 1px solid #2c3e50; width: 100px;">"Дебит"</th>
                        <th style="padding: 8px; border: 1px solid #2c3e50; width: 100px;">"Кредит"</th>
                        <th style="padding: 8px; border: 1px solid #2c3e50; width: 100px;">"Дебит"</th>
                        <th style="padding: 8px; border: 1px solid #2c3e50; width: 100px;">"Кредит"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        rows.get().into_iter().map(|row| {
                            view! {
                                <tr style="border-bottom: 1px solid #eee;">
                                    <td style="padding: 8px; border: 1px solid #ddd; text-align: center; font-family: monospace;">{row.account_code}</td>
                                    <td style="padding: 8px; border: 1px solid #ddd;">{row.account_name}</td>
                                    <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.opening_debit > 0.0 { format!("{:.2}", row.opening_debit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.opening_credit > 0.0 { format!("{:.2}", row.opening_credit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.turnover_debit > 0.0 { format!("{:.2}", row.turnover_debit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.turnover_credit > 0.0 { format!("{:.2}", row.turnover_credit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.closing_debit > 0.0 { format!("{:.2}", row.closing_debit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.closing_credit > 0.0 { format!("{:.2}", row.closing_credit) } else { String::new() }}
                                    </td>
                                </tr>
                            }
                        }).collect_view()
                    }}
                </tbody>
                <tfoot>
                    {move || {
                        if let Some(t) = totals.get() {
                            let od = t.get("opening_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let oc = t.get("opening_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let td = t.get("turnover_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let tc = t.get("turnover_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let cd = t.get("closing_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let cc = t.get("closing_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);

                            view! {
                                <tr style="background: #ecf0f1; font-weight: bold;">
                                    <td colspan="2" style="padding: 10px; border: 1px solid #ddd; text-align: right;">"ОБЩО:"</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", od)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", oc)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", td)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", tc)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", cd)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", cc)}</td>
                                </tr>
                            }.into_view()
                        } else {
                            view! { <tr></tr> }.into_view()
                        }
                    }}
                </tfoot>
            </table>

            {move || {
                if rows.get().is_empty() {
                    view! {
                        <div style="text-align: center; padding: 40px; color: #7f8c8d;">
                            "Натиснете \"Генерирай\" за да заредите отчета"
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn TrialBalanceByCounterpartReport(
    rows: RwSignal<Vec<TrialBalanceRow>>,
    totals: RwSignal<Option<serde_json::Value>>,
    company: RwSignal<Option<serde_json::Value>>,
    date_from: RwSignal<String>,
    date_to: RwSignal<String>,
    counterpart_name: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div>
            // Header
            <div style="text-align: center; margin-bottom: 20px;">
                {move || {
                    if let Some(c) = company.get() {
                        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let eik = c.get("eik").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        view! {
                            <div>
                                <h3 style="margin: 0;">{name}</h3>
                                <p style="color: #7f8c8d; margin: 5px 0;">"ЕИК: "{eik}</p>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}
                <h2 style="margin: 15px 0;">"ОБОРОТНА ВЕДОМОСТ ПО КОНТРАГЕНТИ"</h2>
                <p style="color: #7f8c8d;">
                    "Контрагент: "{move || counterpart_name.get()}
                </p>
                <p style="color: #7f8c8d;">
                    "Период: "{move || date_from.get()}" - "{move || date_to.get()}
                </p>
            </div>

            // Table
            <table id="report-table" style="width: 100%; border-collapse: collapse; font-size: 13px;">
                <thead>
                    <tr style="background: #8e44ad; color: white;">
                        <th rowspan="2" style="padding: 10px; border: 1px solid #7d3c98; width: 80px;">"Сметка"</th>
                        <th rowspan="2" style="padding: 10px; border: 1px solid #7d3c98;">"Наименование"</th>
                        <th rowspan="2" style="padding: 10px; border: 1px solid #7d3c98;">"Контрагент"</th>
                        <th colspan="2" style="padding: 8px; border: 1px solid #7d3c98; background: #7d3c98;">"Начално салдо"</th>
                        <th colspan="2" style="padding: 8px; border: 1px solid #7d3c98; background: #7d3c98;">"Обороти"</th>
                        <th colspan="2" style="padding: 8px; border: 1px solid #7d3c98; background: #7d3c98;">"Крайно салдо"</th>
                    </tr>
                    <tr style="background: #8e44ad; color: white;">
                        <th style="padding: 8px; border: 1px solid #7d3c98; width: 100px;">"Дебит"</th>
                        <th style="padding: 8px; border: 1px solid #7d3c98; width: 100px;">"Кредит"</th>
                        <th style="padding: 8px; border: 1px solid #7d3c98; width: 100px;">"Дебит"</th>
                        <th style="padding: 8px; border: 1px solid #7d3c98; width: 100px;">"Кредит"</th>
                        <th style="padding: 8px; border: 1px solid #7d3c98; width: 100px;">"Дебит"</th>
                        <th style="padding: 8px; border: 1px solid #7d3c98; width: 100px;">"Кредит"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let all_rows = rows.get();
                        let mut views = Vec::<View>::new();
                        let mut cur_code = String::new();
                        let mut cur_name = String::new();
                        let mut sub = [0.0_f64; 6]; // od, oc, td, tc, cd, cc

                        let flush_subtotal = |views: &mut Vec<View>, code: &str, sub: &[f64; 6]| {
                            let label = format!("Общо с/ка {}", code);
                            let vals: Vec<String> = sub.iter().map(|v| if *v != 0.0 { format!("{:.2}", v) } else { String::new() }).collect();
                            views.push(view! {
                                <tr style="background: #f5eef8; font-weight: 600; font-size: 12px;">
                                    <td colspan="3" style="padding: 6px 8px; border: 1px solid #ddd; text-align: right;">{label}</td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{vals[0].clone()}</td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{vals[1].clone()}</td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{vals[2].clone()}</td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{vals[3].clone()}</td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{vals[4].clone()}</td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{vals[5].clone()}</td>
                                </tr>
                            }.into_view());
                        };

                        for row in all_rows.iter() {
                            if row.account_code != cur_code {
                                // Subtotal for previous group
                                if !cur_code.is_empty() {
                                    flush_subtotal(&mut views, &cur_code, &sub);
                                }
                                // Account header
                                let hdr_code = row.account_code.clone();
                                let hdr_name = row.account_name.clone();
                                views.push(view! {
                                    <tr style="background: #e8daef;">
                                        <td style="padding: 8px; border: 1px solid #ddd; text-align: center; font-family: monospace; font-weight: bold;">{hdr_code}</td>
                                        <td colspan="8" style="padding: 8px; border: 1px solid #ddd; font-weight: bold;">{hdr_name}</td>
                                    </tr>
                                }.into_view());
                                cur_code = row.account_code.clone();
                                cur_name = row.account_name.clone();
                                sub = [0.0; 6];
                            }
                            sub[0] += row.opening_debit;
                            sub[1] += row.opening_credit;
                            sub[2] += row.turnover_debit;
                            sub[3] += row.turnover_credit;
                            sub[4] += row.closing_debit;
                            sub[5] += row.closing_credit;

                            let cp = row.counterpart_name.clone().unwrap_or_default();
                            views.push(view! {
                                <tr style="border-bottom: 1px solid #eee;">
                                    <td style="padding: 6px 8px; border: 1px solid #ddd;"></td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd;"></td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; font-size: 12px;">{cp}</td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.opening_debit > 0.0 { format!("{:.2}", row.opening_debit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.opening_credit > 0.0 { format!("{:.2}", row.opening_credit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.turnover_debit > 0.0 { format!("{:.2}", row.turnover_debit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.turnover_credit > 0.0 { format!("{:.2}", row.turnover_credit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.closing_debit > 0.0 { format!("{:.2}", row.closing_debit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.closing_credit > 0.0 { format!("{:.2}", row.closing_credit) } else { String::new() }}
                                    </td>
                                </tr>
                            }.into_view());
                        }
                        // Last group subtotal
                        if !cur_code.is_empty() {
                            flush_subtotal(&mut views, &cur_code, &sub);
                        }
                        views.collect_view()
                    }}
                </tbody>
                <tfoot>
                    {move || {
                        if let Some(t) = totals.get() {
                            let od = t.get("opening_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let oc = t.get("opening_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let td = t.get("turnover_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let tc = t.get("turnover_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let cd = t.get("closing_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let cc = t.get("closing_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);

                            view! {
                                <tr style="background: #f5eef8; font-weight: bold;">
                                    <td colspan="3" style="padding: 10px; border: 1px solid #ddd; text-align: right;">"ОБЩО:"</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", od)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", oc)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", td)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", tc)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", cd)}</td>
                                    <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", cc)}</td>
                                </tr>
                            }.into_view()
                        } else {
                            view! { <tr></tr> }.into_view()
                        }
                    }}
                </tfoot>
            </table>

            {move || {
                if rows.get().is_empty() {
                    view! {
                        <div style="text-align: center; padding: 40px; color: #7f8c8d;">
                            "Изберете контрагент и натиснете \"Генерирай\""
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn ChronologicalReport(
    rows: RwSignal<Vec<ChronologicalRow>>,
    totals: RwSignal<Option<serde_json::Value>>,
    company: RwSignal<Option<serde_json::Value>>,
    date_from: RwSignal<String>,
    date_to: RwSignal<String>,
    account_filter: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div>
            // Header
            <div style="text-align: center; margin-bottom: 20px;">
                {move || {
                    if let Some(c) = company.get() {
                        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let eik = c.get("eik").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        view! {
                            <div>
                                <h3 style="margin: 0;">{name}</h3>
                                <p style="color: #7f8c8d; margin: 5px 0;">"ЕИК: "{eik}</p>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}
                <h2 style="margin: 15px 0;">"ХРОНОЛОГИЧНА СПРАВКА"</h2>
                <p style="color: #7f8c8d;">
                    "Период: "{move || date_from.get()}" - "{move || date_to.get()}
                    {move || {
                        let acc = account_filter.get();
                        if !acc.is_empty() {
                            format!(" | Сметка: {}", acc)
                        } else {
                            String::new()
                        }
                    }}
                </p>
            </div>

            // Table
            <table id="report-table" style="width: 100%; border-collapse: collapse; font-size: 12px;">
                <thead>
                    <tr style="background: #336699; color: white;">
                        <th style="padding: 10px; border: 1px solid #2c5282; width: 40px;">"№"</th>
                        <th style="padding: 10px; border: 1px solid #2c5282; width: 80px;">"Дата"</th>
                        <th style="padding: 10px; border: 1px solid #2c5282; width: 100px;">"Документ"</th>
                        <th style="padding: 10px; border: 1px solid #2c5282;">"Описание"</th>
                        <th style="padding: 10px; border: 1px solid #2c5282; width: 120px;">"Контрагент"</th>
                        <th style="padding: 10px; border: 1px solid #2c5282; width: 80px;">"Дт с/ка"</th>
                        <th style="padding: 10px; border: 1px solid #2c5282; width: 80px;">"Кт с/ка"</th>
                        <th style="padding: 10px; border: 1px solid #2c5282; width: 100px;">"Дебит"</th>
                        <th style="padding: 10px; border: 1px solid #2c5282; width: 100px;">"Кредит"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        rows.get().into_iter().map(|row| {
                            view! {
                                <tr style="border-bottom: 1px solid #eee;">
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: center;">{row.row_number}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: center; font-family: monospace;">{row.entry_date}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: center;">{row.document_number}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd;">{row.description}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; font-size: 11px;">{row.counterpart_name}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: center; font-family: monospace;">{row.debit_account}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: center; font-family: monospace;">{row.credit_account}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.debit_amount > 0.0 { format!("{:.2}", row.debit_amount) } else { String::new() }}
                                    </td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.credit_amount > 0.0 { format!("{:.2}", row.credit_amount) } else { String::new() }}
                                    </td>
                                </tr>
                            }
                        }).collect_view()
                    }}
                </tbody>
                <tfoot>
                    {move || {
                        if let Some(t) = totals.get() {
                            let td = t.get("debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let tc = t.get("credit").and_then(|v| v.as_f64()).unwrap_or(0.0);

                            view! {
                                <tr style="background: #336699; color: white; font-weight: bold;">
                                    <td colspan="7" style="padding: 10px; border: 1px solid #2c5282; text-align: right;">"ОБЩО:"</td>
                                    <td style="padding: 10px; border: 1px solid #2c5282; text-align: right; font-family: monospace;">{format!("{:.2}", td)}</td>
                                    <td style="padding: 10px; border: 1px solid #2c5282; text-align: right; font-family: monospace;">{format!("{:.2}", tc)}</td>
                                </tr>
                            }.into_view()
                        } else {
                            view! { <tr></tr> }.into_view()
                        }
                    }}
                </tfoot>
            </table>

            {move || {
                if rows.get().is_empty() {
                    view! {
                        <div style="text-align: center; padding: 40px; color: #7f8c8d;">
                            "Натиснете \"Генерирай\" за да заредите отчета"
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn GeneralLedgerReport(
    accounts: RwSignal<Vec<GeneralLedgerAccount>>,
    totals: RwSignal<Option<serde_json::Value>>,
    company: RwSignal<Option<serde_json::Value>>,
    date_from: RwSignal<String>,
    date_to: RwSignal<String>,
    account_filter: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div>
            // Header
            <div style="text-align: center; margin-bottom: 20px;">
                {move || {
                    if let Some(c) = company.get() {
                        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let eik = c.get("eik").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        view! {
                            <div>
                                <h3 style="margin: 0;">{name}</h3>
                                <p style="color: #7f8c8d; margin: 5px 0;">"ЕИК: "{eik}</p>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}
                <h2 style="margin: 15px 0;">"ГЛАВНА КНИГА"</h2>
                <p style="color: #7f8c8d;">
                    "Период: "{move || date_from.get()}" - "{move || date_to.get()}
                    {move || {
                        let acc = account_filter.get();
                        if !acc.is_empty() {
                            format!(" | Филтър: {}", acc)
                        } else {
                            String::new()
                        }
                    }}
                </p>
            </div>

            // Account sections
            {move || {
                let accs = accounts.get();
                if accs.is_empty() {
                    view! {
                        <div style="text-align: center; padding: 40px; color: #7f8c8d;">
                            "Натиснете \"Генерирай\" за да заредите отчета"
                        </div>
                    }.into_view()
                } else {
                    accs.into_iter().map(|account| {
                        let tx_count = account.transactions.len();
                        view! {
                            <div style="margin-bottom: 25px;">
                                // Account header
                                <div class="gl-account-header" style="background: #2c3e50; color: white; padding: 10px 15px; border-radius: 4px 4px 0 0; font-weight: bold; font-size: 14px;">
                                    {format!("Сметка {} - {}", account.account_code, account.account_name)}
                                </div>

                                <table style="width: 100%; border-collapse: collapse; font-size: 12px; margin-bottom: 0;">
                                    <thead>
                                        <tr style="background: #ecf0f1;">
                                            <th style="padding: 8px; border: 1px solid #bdc3c7; width: 80px; text-align: left;">"Дата"</th>
                                            <th style="padding: 8px; border: 1px solid #bdc3c7; width: 100px; text-align: left;">"Документ"</th>
                                            <th style="padding: 8px; border: 1px solid #bdc3c7; text-align: left;">"Описание"</th>
                                            <th style="padding: 8px; border: 1px solid #bdc3c7; width: 100px; text-align: left;">"Кор. сметки"</th>
                                            <th style="padding: 8px; border: 1px solid #bdc3c7; width: 110px; text-align: right;">"Дебит"</th>
                                            <th style="padding: 8px; border: 1px solid #bdc3c7; width: 110px; text-align: right;">"Кредит"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        // Opening balance row
                                        <tr style="background: #fdf2e9; font-style: italic;">
                                            <td colspan="4" style="padding: 8px; border: 1px solid #ddd; font-weight: 500;">"Начално салдо"</td>
                                            <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace; font-weight: 500;">
                                                {if account.opening_debit > 0.0 { format!("{:.2}", account.opening_debit) } else { String::new() }}
                                            </td>
                                            <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace; font-weight: 500;">
                                                {if account.opening_credit > 0.0 { format!("{:.2}", account.opening_credit) } else { String::new() }}
                                            </td>
                                        </tr>

                                        // Transaction rows
                                        {account.transactions.into_iter().map(|tx| {
                                            view! {
                                                <tr style="border-bottom: 1px solid #eee;">
                                                    <td style="padding: 6px 8px; border: 1px solid #ddd; font-family: monospace; white-space: nowrap;">{tx.entry_date}</td>
                                                    <td style="padding: 6px 8px; border: 1px solid #ddd; font-size: 11px;">{tx.document_number}</td>
                                                    <td style="padding: 6px 8px; border: 1px solid #ddd; font-size: 11px;">
                                                        {tx.description}
                                                        {if !tx.counterpart_name.is_empty() {
                                                            format!(" ({})", tx.counterpart_name)
                                                        } else {
                                                            String::new()
                                                        }}
                                                    </td>
                                                    <td style="padding: 6px 8px; border: 1px solid #ddd; font-family: monospace; text-align: center; font-size: 11px;">{tx.counterpart_codes}</td>
                                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                                        {if tx.debit > 0.0 { format!("{:.2}", tx.debit) } else { String::new() }}
                                                    </td>
                                                    <td style="padding: 6px 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                                        {if tx.credit > 0.0 { format!("{:.2}", tx.credit) } else { String::new() }}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                    <tfoot>
                                        // Turnover row
                                        <tr style="background: #ecf0f1; font-weight: bold;">
                                            <td colspan="4" style="padding: 8px; border: 1px solid #bdc3c7; text-align: right;">
                                                {format!("Обороти ({} операции):", tx_count)}
                                            </td>
                                            <td style="padding: 8px; border: 1px solid #bdc3c7; text-align: right; font-family: monospace;">
                                                {format!("{:.2}", account.turnover_debit)}
                                            </td>
                                            <td style="padding: 8px; border: 1px solid #bdc3c7; text-align: right; font-family: monospace;">
                                                {format!("{:.2}", account.turnover_credit)}
                                            </td>
                                        </tr>
                                        // Closing balance row
                                        <tr style="background: #d5dbdb; font-weight: bold;">
                                            <td colspan="4" style="padding: 8px; border: 1px solid #bdc3c7; text-align: right;">"Крайно салдо:"</td>
                                            <td style="padding: 8px; border: 1px solid #bdc3c7; text-align: right; font-family: monospace;">
                                                {if account.closing_debit > 0.0 { format!("{:.2}", account.closing_debit) } else { String::new() }}
                                            </td>
                                            <td style="padding: 8px; border: 1px solid #bdc3c7; text-align: right; font-family: monospace;">
                                                {if account.closing_credit > 0.0 { format!("{:.2}", account.closing_credit) } else { String::new() }}
                                            </td>
                                        </tr>
                                    </tfoot>
                                </table>
                            </div>
                        }
                    }).collect_view().into_view()
                }
            }}

            // Grand totals
            {move || {
                if let Some(t) = totals.get() {
                    let od = t.get("opening_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let oc = t.get("opening_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let td_val = t.get("turnover_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let tc = t.get("turnover_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let cd = t.get("closing_debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let cc = t.get("closing_credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let acc_count = accounts.get().len();

                    if acc_count > 0 {
                        view! {
                            <div style="margin-top: 20px;">
                                <table style="width: 100%; border-collapse: collapse; font-size: 13px;">
                                    <thead>
                                        <tr style="background: #2c3e50; color: white;">
                                            <th style="padding: 10px; border: 1px solid #1a252f;">"ОБОБЩЕНИЕ"</th>
                                            <th style="padding: 10px; border: 1px solid #1a252f; width: 120px;">"Дебит"</th>
                                            <th style="padding: 10px; border: 1px solid #1a252f; width: 120px;">"Кредит"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr>
                                            <td style="padding: 8px; border: 1px solid #ddd; font-weight: 500;">"Начално салдо"</td>
                                            <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", od)}</td>
                                            <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", oc)}</td>
                                        </tr>
                                        <tr>
                                            <td style="padding: 8px; border: 1px solid #ddd; font-weight: 500;">"Обороти"</td>
                                            <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", td_val)}</td>
                                            <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", tc)}</td>
                                        </tr>
                                        <tr style="background: #ecf0f1; font-weight: bold;">
                                            <td style="padding: 10px; border: 1px solid #ddd;">
                                                {format!("Крайно салдо ({} сметки)", acc_count)}
                                            </td>
                                            <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", cd)}</td>
                                            <td style="padding: 10px; border: 1px solid #ddd; text-align: right; font-family: monospace;">{format!("{:.2}", cc)}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn AccountLedgerReport(
    rows: RwSignal<Vec<AccountLedgerRow>>,
    totals: RwSignal<Option<serde_json::Value>>,
    opening: RwSignal<Option<serde_json::Value>>,
    account_info: RwSignal<Option<serde_json::Value>>,
    company: RwSignal<Option<serde_json::Value>>,
    date_from: RwSignal<String>,
    date_to: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div>
            // Header
            <div style="text-align: center; margin-bottom: 20px;">
                {move || {
                    if let Some(c) = company.get() {
                        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let eik = c.get("eik").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        view! {
                            <div>
                                <h3 style="margin: 0;">{name}</h3>
                                <p style="color: #7f8c8d; margin: 5px 0;">"ЕИК: "{eik}</p>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}
                <h2 style="margin: 15px 0;">"КАРТОН НА СМЕТКА"</h2>
                {move || {
                    if let Some(a) = account_info.get() {
                        let code = a.get("code").and_then(|v| v.as_str()).unwrap_or("");
                        let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        view! {
                            <h3 style="margin: 5px 0; color: #2c3e50;">{format!("Сметка {} - {}", code, name)}</h3>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}
                <p style="color: #7f8c8d;">
                    "Период: "{move || date_from.get()}" - "{move || date_to.get()}
                </p>
            </div>

            // Table
            <table id="report-table" style="width: 100%; border-collapse: collapse; font-size: 12px;">
                <thead>
                    <tr style="background: #27ae60; color: white;">
                        <th style="padding: 10px; border: 1px solid #1e8449; width: 80px;">"Дата"</th>
                        <th style="padding: 10px; border: 1px solid #1e8449; width: 100px;">"Документ"</th>
                        <th style="padding: 10px; border: 1px solid #1e8449;">"Описание"</th>
                        <th style="padding: 10px; border: 1px solid #1e8449; width: 120px;">"Контрагент"</th>
                        <th style="padding: 10px; border: 1px solid #1e8449; width: 100px;">"Кор. сметки"</th>
                        <th style="padding: 10px; border: 1px solid #1e8449; width: 100px;">"Дебит"</th>
                        <th style="padding: 10px; border: 1px solid #1e8449; width: 100px;">"Кредит"</th>
                        <th style="padding: 10px; border: 1px solid #1e8449; width: 100px;">"Салдо"</th>
                    </tr>
                </thead>
                <tbody>
                    // Opening balance row
                    <tr style="background: #e9f7ef; font-style: italic; font-weight: bold;">
                        <td colspan="5" style="padding: 8px; border: 1px solid #ddd;">"Начално салдо"</td>
                        <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                            {move || {
                                if let Some(o) = opening.get() {
                                    let d = o.get("debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    if d > 0.0 { format!("{:.2}", d) } else { String::new() }
                                } else { String::new() }
                            }}
                        </td>
                        <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                            {move || {
                                if let Some(o) = opening.get() {
                                    let c = o.get("credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    if c > 0.0 { format!("{:.2}", c) } else { String::new() }
                                } else { String::new() }
                            }}
                        </td>
                        <td style="padding: 8px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                            {move || {
                                if let Some(o) = opening.get() {
                                    let t = o.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    format!("{:.2}", t)
                                } else { "0.00".to_string() }
                            }}
                        </td>
                    </tr>

                    // Transactions
                    {move || {
                        rows.get().into_iter().map(|row| {
                            view! {
                                <tr style="border-bottom: 1px solid #eee;">
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: center; font-family: monospace;">{row.date}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: center;">{row.document_number}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd;">{row.description}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; font-size: 11px;">{row.counterpart_name}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: center; font-family: monospace;">{row.correspondent_account}</td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.debit > 0.0 { format!("{:.2}", row.debit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: right; font-family: monospace;">
                                        {if row.credit > 0.0 { format!("{:.2}", row.credit) } else { String::new() }}
                                    </td>
                                    <td style="padding: 6px; border: 1px solid #ddd; text-align: right; font-family: monospace; font-weight: 500;">
                                        {format!("{:.2}", row.balance)}
                                    </td>
                                </tr>
                            }
                        }).collect_view()
                    }}
                </tbody>
                <tfoot>
                    {move || {
                        if let Some(t) = totals.get() {
                            let td = t.get("debit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let tc = t.get("credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let fb = t.get("final_balance").and_then(|v| v.as_f64()).unwrap_or(0.0);

                            view! {
                                <tr style="background: #27ae60; color: white; font-weight: bold;">
                                    <td colspan="5" style="padding: 10px; border: 1px solid #1e8449; text-align: right;">"ОБОРОТИ И КРАЙНО САЛДО:"</td>
                                    <td style="padding: 10px; border: 1px solid #1e8449; text-align: right; font-family: monospace;">{format!("{:.2}", td)}</td>
                                    <td style="padding: 10px; border: 1px solid #1e8449; text-align: right; font-family: monospace;">{format!("{:.2}", tc)}</td>
                                    <td style="padding: 10px; border: 1px solid #1e8449; text-align: right; font-family: monospace;">{format!("{:.2}", fb)}</td>
                                </tr>
                            }.into_view()
                        } else {
                            view! { <tr></tr> }.into_view()
                        }
                    }}
                </tfoot>
            </table>

            {move || {
                if rows.get().is_empty() {
                    view! {
                        <div style="text-align: center; padding: 40px; color: #7f8c8d;">
                            "Въведете сметка и натиснете \"Генерирай\" за да заредите картона"
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}
