use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{FileReader, ProgressEvent, DragEvent};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_delete};
use crate::context::use_company_context;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalLine {
    pub account_id: Option<i64>,
    pub debit: Option<f64>,
    pub credit: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BankTransaction {
    pub id: i64,
    pub bank_account_id: i64,
    pub bank_account_name: Option<String>,
    pub date: String,
    pub amount: f64,
    pub currency: String,
    pub amount_base: Option<f64>,
    pub description: Option<String>,
    pub contra_account: Option<String>,
    pub contra_name: Option<String>,
    pub reference: Option<String>,
    pub journal_entry_id: Option<i64>,
    pub is_booked: bool,
    #[serde(default)]
    pub is_allocated: bool,
    pub journal_lines: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreviewTransaction {
    pub date: String,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
    pub contra_account: Option<String>,
    pub contra_name: Option<String>,
    pub reference: Option<String>,
    pub is_duplicate: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreviewResult {
    pub bank_format: Option<String>,
    pub account_iban: Option<String>,
    pub account_currency: Option<String>,
    pub period_from: Option<String>,
    pub period_to: Option<String>,
    pub opening_balance: Option<f64>,
    pub closing_balance: Option<f64>,
    pub total_count: i32,
    pub duplicate_count: i32,
    pub new_count: i32,
    pub bank_account_id: Option<i64>,
    pub bank_account_name: Option<String>,
    pub transactions: Vec<PreviewTransaction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: i64,
    pub name: String,
    pub iban: Option<String>,
    pub currency: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GLAccount {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[component]
pub fn BankTransactions() -> impl IntoView {
    let ctx = use_company_context();

    // Wizard state
    let current_step = create_rw_signal(0); // 0 = list view, 1 = upload, 2 = preview, 3 = done
    let file_name = create_rw_signal(String::new());
    let file_content = create_rw_signal(Vec::<u8>::new());
    let file_input_ref = create_node_ref::<html::Input>();
    let loading = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());
    let is_dragging = create_rw_signal(false);

    // Preview data
    let preview_result = create_rw_signal(PreviewResult::default());
    let selected_bank_account_id = create_rw_signal(String::new());

    // List view state
    let transactions = create_rw_signal(Vec::<BankTransaction>::new());
    let bank_accounts = create_rw_signal(Vec::<BankAccount>::new());
    let gl_accounts = create_rw_signal(Vec::<GLAccount>::new());
    let filter_account = create_rw_signal(String::new());
    let filter_status = create_rw_signal(String::new());
    let filter_date_from = create_rw_signal(String::new());
    let filter_date_to = create_rw_signal(String::new());

    // Booking modal (for transactions without journal entry)
    let show_booking_modal = create_rw_signal(false);
    let booking_transaction = create_rw_signal(Option::<BankTransaction>::None);
    let booking_contra_account_id = create_rw_signal(String::new());

    // Reallocate modal (for transactions with buffer account)
    let show_reallocate_modal = create_rw_signal(false);
    let reallocate_transaction = create_rw_signal(Option::<BankTransaction>::None);
    let reallocate_account_id = create_rw_signal(String::new());

    // Import result
    let import_result = create_rw_signal(Option::<serde_json::Value>::None);

    // Load transactions
    let load_transactions = move || {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        loading.set(true);

        let account_filter = filter_account.get();
        let status_filter = filter_status.get();
        let date_from = filter_date_from.get();
        let date_to = filter_date_to.get();

        spawn_local(async move {
            let mut url = format!("/api/companies/{}/bank_transactions?limit=500", company_id);

            if !account_filter.is_empty() {
                url.push_str(&format!("&bank_account_id={}", account_filter));
            }
            if !status_filter.is_empty() && status_filter != "unallocated" && status_filter != "allocated" {
                url.push_str(&format!("&status={}", status_filter));
            }
            if status_filter == "unallocated" || status_filter == "allocated" {
                // These are booked transactions, filter by booked first
                url.push_str("&status=booked");
            }
            if !date_from.is_empty() {
                url.push_str(&format!("&date_from={}", date_from));
            }
            if !date_to.is_empty() {
                url.push_str(&format!("&date_to={}", date_to));
            }

            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let mut list: Vec<BankTransaction> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();

                        // Client-side filter for allocated/unallocated
                        if status_filter == "unallocated" {
                            list.retain(|tx| tx.is_booked && !tx.is_allocated);
                        } else if status_filter == "allocated" {
                            list.retain(|tx| tx.is_booked && tx.is_allocated);
                        }

                        transactions.set(list);
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Load bank accounts
    let load_bank_accounts = move || {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);

        spawn_local(async move {
            let url = format!("/api/companies/{}/bank_accounts", company_id);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let list: Vec<BankAccount> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        bank_accounts.set(list);
                    }
                }
                Err(_) => {}
            }
        });
    };

    // Load GL accounts for booking/reallocating
    let load_gl_accounts = move || {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);

        spawn_local(async move {
            let url = format!("/api/companies/{}/accounts?per_page=1000", company_id);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let list: Vec<GLAccount> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        gl_accounts.set(list);
                    }
                }
                Err(_) => {}
            }
        });
    };

    // Initial load
    create_effect(move |_| {
        ctx.selected_company_id.get();
        load_transactions();
        load_bank_accounts();
        load_gl_accounts();
    });

    // Process file
    let process_file = move |file: web_sys::File| {
        file_name.set(file.name());
        let reader = FileReader::new().unwrap();
        let reader_clone = reader.clone();

        let onload = Closure::wrap(Box::new(move |_: ProgressEvent| {
            if let Ok(result) = reader_clone.result() {
                if let Some(array_buffer) = result.dyn_ref::<js_sys::ArrayBuffer>() {
                    let uint8_array = js_sys::Uint8Array::new(array_buffer);
                    let bytes: Vec<u8> = uint8_array.to_vec();
                    file_content.set(bytes);
                }
            }
        }) as Box<dyn FnMut(_)>);

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
        let _ = reader.read_as_array_buffer(&file);
    };

    // File input change
    let on_file_change = move |ev: ev::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        if let Some(files) = target.files() {
            if let Some(file) = files.get(0) {
                process_file(file);
            }
        }
        target.set_value("");
    };

    // File select click
    let on_file_select = move |_| {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    // Drag handlers
    let on_drag_over = move |ev: DragEvent| {
        ev.prevent_default();
        is_dragging.set(true);
    };

    let on_drag_leave = move |ev: DragEvent| {
        ev.prevent_default();
        is_dragging.set(false);
    };

    let on_drop = move |ev: DragEvent| {
        ev.prevent_default();
        is_dragging.set(false);

        if let Some(data_transfer) = ev.data_transfer() {
            if let Some(files) = data_transfer.files() {
                if let Some(file) = files.get(0) {
                    process_file(file);
                }
            }
        }
    };

    // Preview file
    let do_preview = move |_| {
        let content = file_content.get();
        if content.is_empty() {
            error_msg.set("Моля изберете файл".to_string());
            return;
        }

        loading.set(true);
        error_msg.set(String::new());

        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        let base64_content = base64_encode(&content);

        spawn_local(async move {
            let payload = serde_json::json!({
                "file_content": base64_content,
            });

            let url = format!("/api/companies/{}/bank_transactions/preview", company_id);
            match api_post(&url, &payload).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Some(preview_data) = data.get("data") {
                            if let Ok(preview) = serde_json::from_value::<PreviewResult>(preview_data.clone()) {
                                // Auto-select detected bank account
                                if let Some(acc_id) = preview.bank_account_id {
                                    selected_bank_account_id.set(acc_id.to_string());
                                }
                                preview_result.set(preview);
                                current_step.set(2);
                            }
                        }
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при парсване");
                        error_msg.set(err.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Import transactions
    let do_import = move |_| {
        let content = file_content.get();
        let bank_acc_id = selected_bank_account_id.get();

        if bank_acc_id.is_empty() {
            error_msg.set("Моля изберете банкова сметка".to_string());
            return;
        }

        loading.set(true);
        error_msg.set(String::new());

        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        let base64_content = base64_encode(&content);

        spawn_local(async move {
            let payload = serde_json::json!({
                "file_content": base64_content,
                "bank_account_id": bank_acc_id.parse::<i64>().ok(),
            });

            let url = format!("/api/companies/{}/bank_transactions/import", company_id);
            match api_post(&url, &payload).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        import_result.set(data.get("data").cloned());
                        current_step.set(3);
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при импорт");
                        error_msg.set(err.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Book transaction (for transactions without journal entry)
    let book_transaction = move |_| {
        let tx = booking_transaction.get();
        let contra_id = booking_contra_account_id.get();

        if contra_id.is_empty() {
            error_msg.set("Моля изберете кореспондираща сметка".to_string());
            return;
        }

        if let Some(tx) = tx {
            let company_id = ctx.selected_company_id.get().unwrap_or(1);
            loading.set(true);
            error_msg.set(String::new());

            spawn_local(async move {
                let payload = serde_json::json!({
                    "contra_account_id": contra_id.parse::<i64>().ok(),
                });

                let url = format!("/api/companies/{}/bank_transactions/{}/book", company_id, tx.id);
                match api_post(&url, &payload).await {
                    Ok(data) => {
                        if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            let je_id = data.get("data")
                                .and_then(|d| d.get("journal_entry_id"))
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0);
                            success_msg.set(format!("Създаден е счетоводен запис #{}", je_id));
                            show_booking_modal.set(false);
                            load_transactions();
                        } else {
                            let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка");
                            error_msg.set(err.to_string());
                        }
                    }
                    Err(e) => {
                        error_msg.set(format!("Грешка: {}", e));
                    }
                }
                loading.set(false);
            });
        }
    };

    // Reallocate transaction (change buffer account to real account)
    let do_reallocate = move |_| {
        let tx = reallocate_transaction.get();
        let new_account_id = reallocate_account_id.get();

        if new_account_id.is_empty() {
            error_msg.set("Моля изберете сметка".to_string());
            return;
        }

        if let Some(tx) = tx {
            let company_id = ctx.selected_company_id.get().unwrap_or(1);
            loading.set(true);
            error_msg.set(String::new());

            spawn_local(async move {
                let payload = serde_json::json!({
                    "account_id": new_account_id.parse::<i64>().ok(),
                });

                let url = format!("/api/companies/{}/bank_transactions/{}/reallocate", company_id, tx.id);
                match api_post(&url, &payload).await {
                    Ok(data) => {
                        if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            let msg = data.get("data")
                                .and_then(|d| d.get("message"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Успешно преразпределяне");
                            success_msg.set(msg.to_string());
                            show_reallocate_modal.set(false);
                            load_transactions();
                        } else {
                            let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка");
                            error_msg.set(err.to_string());
                        }
                    }
                    Err(e) => {
                        error_msg.set(format!("Грешка: {}", e));
                    }
                }
                loading.set(false);
            });
        }
    };

    // Delete transaction
    let delete_transaction = move |id: i64| {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);

        spawn_local(async move {
            let url = format!("/api/companies/{}/bank_transactions/{}", company_id, id);
            match api_delete(&url).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Транзакцията е изтрита".to_string());
                        load_transactions();
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка");
                        error_msg.set(err.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
        });
    };

    // Reset wizard
    let reset_wizard = move |_| {
        current_step.set(0);
        file_name.set(String::new());
        file_content.set(Vec::new());
        preview_result.set(PreviewResult::default());
        import_result.set(None);
        error_msg.set(String::new());
        success_msg.set(String::new());
        load_transactions();
    };

    // Go to processing view after import
    let go_to_processing = move |_| {
        current_step.set(0);
        file_name.set(String::new());
        file_content.set(Vec::new());
        preview_result.set(PreviewResult::default());
        import_result.set(None);
        error_msg.set(String::new());
        success_msg.set(String::new());
        filter_status.set("unallocated".to_string());
        load_transactions();
    };

    // Start import
    let start_import = move |_| {
        current_step.set(1);
        file_name.set(String::new());
        file_content.set(Vec::new());
        error_msg.set(String::new());
    };

    // Open booking modal
    let open_booking_modal = move |tx: BankTransaction| {
        booking_transaction.set(Some(tx));
        booking_contra_account_id.set(String::new());
        show_booking_modal.set(true);
    };

    // Open reallocate modal
    let open_reallocate_modal = move |tx: BankTransaction| {
        reallocate_transaction.set(Some(tx));
        reallocate_account_id.set(String::new());
        show_reallocate_modal.set(true);
    };

    // Helper: find account name by id
    let find_account_name = move |account_id: i64| -> String {
        gl_accounts.with(|list| {
            list.iter()
                .find(|a| a.id == account_id)
                .map(|a| format!("{} - {}", a.code, a.name))
                .unwrap_or_else(|| format!("#{}", account_id))
        })
    };

    view! {
        <Layout>
            // Hidden file input
            <input
                type="file"
                accept=".xml,.csv,.txt,.mt940"
                style="display: none;"
                node_ref=file_input_ref
                on:change=on_file_change
            />

            // Error message
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                            {err}
                            <button
                                style="float: right; background: transparent; border: none; color: white; cursor: pointer; font-size: 1.2em;"
                                on:click=move |_| error_msg.set(String::new())
                            >"✕"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Success message
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                            {msg}
                            <button
                                style="float: right; background: transparent; border: none; color: white; cursor: pointer; font-size: 1.2em;"
                                on:click=move |_| success_msg.set(String::new())
                            >"✕"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            {move || match current_step.get() {
                // List view
                0 => {
                    let tx_list = transactions.get();
                    let acc_list = bank_accounts.get();
                    let current_filter = filter_status.get();

                    view! {
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <h1 style="margin: 0;">"Банкови транзакции"</h1>
                                <div style="display: flex; gap: 10px;">
                                    <a
                                        href="/bank-accounts"
                                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; text-decoration: none;"
                                    >
                                        "Сметки"
                                    </a>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=start_import
                                    >
                                        "Импорт от файл"
                                    </button>
                                </div>
                            </div>

                            // Filters
                            <div style="background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; display: flex; gap: 15px; flex-wrap: wrap; align-items: flex-end;">
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-size: 0.9em; color: #7f8c8d;">"Банкова сметка"</label>
                                    <select
                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px; min-width: 200px;"
                                        on:change=move |ev| {
                                            filter_account.set(event_target_value(&ev));
                                            load_transactions();
                                        }
                                    >
                                        <option value="">"Всички"</option>
                                        {acc_list.iter().map(|acc| {
                                            let selected = filter_account.get() == acc.id.to_string();
                                            view! {
                                                <option value={acc.id.to_string()} selected=selected>
                                                    {&acc.name}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </div>
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-size: 0.9em; color: #7f8c8d;">"Статус"</label>
                                    <select
                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                        on:change=move |ev| {
                                            filter_status.set(event_target_value(&ev));
                                            load_transactions();
                                        }
                                    >
                                        <option value="" selected={current_filter.is_empty()}>"Всички"</option>
                                        <option value="unbooked" selected={current_filter == "unbooked"}>"Неосчетоводени"</option>
                                        <option value="unallocated" selected={current_filter == "unallocated"}>"За обработка"</option>
                                        <option value="allocated" selected={current_filter == "allocated"}>"Обработени"</option>
                                        <option value="booked" selected={current_filter == "booked"}>"Всички осчетоводени"</option>
                                    </select>
                                </div>
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-size: 0.9em; color: #7f8c8d;">"От дата"</label>
                                    <input
                                        type="date"
                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                        prop:value=move || filter_date_from.get()
                                        on:change=move |ev| {
                                            filter_date_from.set(event_target_value(&ev));
                                            load_transactions();
                                        }
                                    />
                                </div>
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-size: 0.9em; color: #7f8c8d;">"До дата"</label>
                                    <input
                                        type="date"
                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                        prop:value=move || filter_date_to.get()
                                        on:change=move |ev| {
                                            filter_date_to.set(event_target_value(&ev));
                                            load_transactions();
                                        }
                                    />
                                </div>
                            </div>

                            // Transactions table
                            <div style="background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                {if loading.get() {
                                    view! {
                                        <div style="padding: 40px; text-align: center; color: #7f8c8d;">
                                            "Зареждане..."
                                        </div>
                                    }.into_view()
                                } else if tx_list.is_empty() {
                                    view! {
                                        <div style="padding: 40px; text-align: center; color: #7f8c8d;">
                                            "Няма транзакции"
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <table style="width: 100%; border-collapse: collapse;">
                                            <thead>
                                                <tr style="background: #f8f9fa;">
                                                    <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Дата"</th>
                                                    <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Сметка"</th>
                                                    <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Описание"</th>
                                                    <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Контрагент"</th>
                                                    <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Сума"</th>
                                                    <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6;">"Статус"</th>
                                                    <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6;">"Действия"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {tx_list.iter().map(|tx| {
                                                    let tx_clone = tx.clone();
                                                    let tx_clone2 = tx.clone();
                                                    let tx_id = tx.id;
                                                    let is_booked = tx.is_booked;
                                                    let is_allocated = tx.is_allocated;
                                                    let amount_style = if tx.amount >= 0.0 {
                                                        "color: #27ae60; font-weight: bold;"
                                                    } else {
                                                        "color: #e74c3c; font-weight: bold;"
                                                    };
                                                    let date_display = if tx.date.len() >= 10 { tx.date[..10].to_string() } else { tx.date.clone() };

                                                    // Row background for unallocated
                                                    let row_style = if is_booked && !is_allocated {
                                                        "border-bottom: 1px solid #dee2e6; background: #fff8e1;"
                                                    } else {
                                                        "border-bottom: 1px solid #dee2e6;"
                                                    };

                                                    view! {
                                                        <tr style=row_style>
                                                            <td style="padding: 12px;">{date_display}</td>
                                                            <td style="padding: 12px;">{tx.bank_account_name.clone().unwrap_or("-".to_string())}</td>
                                                            <td style="padding: 12px; max-width: 250px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                                                                {tx.description.clone().unwrap_or("-".to_string())}
                                                            </td>
                                                            <td style="padding: 12px;">{tx.contra_name.clone().unwrap_or("-".to_string())}</td>
                                                            <td style=format!("padding: 12px; text-align: right; {}", amount_style)>
                                                                {format!("{:+.2} {}", tx.amount, tx.currency)}
                                                            </td>
                                                            <td style="padding: 12px; text-align: center;">
                                                                {if !is_booked {
                                                                    view! {
                                                                        <span style="background: #95a5a6; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                            "Неосчетоводено"
                                                                        </span>
                                                                    }.into_view()
                                                                } else if !is_allocated {
                                                                    view! {
                                                                        <span style="background: #f39c12; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                            "За обработка"
                                                                        </span>
                                                                    }.into_view()
                                                                } else {
                                                                    view! {
                                                                        <span style="background: #27ae60; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                            "Обработено"
                                                                        </span>
                                                                    }.into_view()
                                                                }}
                                                            </td>
                                                            <td style="padding: 12px; text-align: center;">
                                                                {if !is_booked {
                                                                    // Not booked - show book and delete buttons
                                                                    view! {
                                                                        <div style="display: flex; gap: 5px; justify-content: center;">
                                                                            <button
                                                                                style="background: #27ae60; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                                                on:click=move |_| open_booking_modal(tx_clone.clone())
                                                                            >
                                                                                "Осчетоводи"
                                                                            </button>
                                                                            <button
                                                                                style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                                                on:click=move |_| {
                                                                                    if web_sys::window().unwrap().confirm_with_message("Сигурни ли сте?").unwrap_or(false) {
                                                                                        delete_transaction(tx_id);
                                                                                    }
                                                                                }
                                                                            >
                                                                                "Изтрий"
                                                                            </button>
                                                                        </div>
                                                                    }.into_view()
                                                                } else if !is_allocated {
                                                                    // Booked but unallocated - show reallocate button
                                                                    view! {
                                                                        <div style="display: flex; gap: 5px; justify-content: center;">
                                                                            <button
                                                                                style="background: #e67e22; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                                                on:click=move |_| open_reallocate_modal(tx_clone2.clone())
                                                                            >
                                                                                "Обработи"
                                                                            </button>
                                                                            <a
                                                                                href=format!("/journal-entries?id={}", tx.journal_entry_id.unwrap_or(0))
                                                                                style="color: #3498db; text-decoration: none; padding: 6px 8px;"
                                                                            >
                                                                                {format!("JE #{}", tx.journal_entry_id.unwrap_or(0))}
                                                                            </a>
                                                                        </div>
                                                                    }.into_view()
                                                                } else {
                                                                    // Fully allocated - just show journal entry link
                                                                    view! {
                                                                        <a
                                                                            href=format!("/journal-entries?id={}", tx.journal_entry_id.unwrap_or(0))
                                                                            style="color: #3498db; text-decoration: none;"
                                                                        >
                                                                            {format!("JE #{}", tx.journal_entry_id.unwrap_or(0))}
                                                                        </a>
                                                                    }.into_view()
                                                                }}
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    }.into_view()
                                }}
                            </div>
                        </div>
                    }.into_view()
                },

                // Step 1: Upload
                1 => {
                    view! {
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <h1 style="margin: 0;">"Импорт на банково извлечение"</h1>
                                <button
                                    style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=reset_wizard
                                >
                                    "Отказ"
                                </button>
                            </div>

                            // Step indicator
                            <div style="display: flex; gap: 20px; margin: 30px 0;">
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #3498db; color: white;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"1"</div>
                                    <div>"Качване"</div>
                                </div>
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #ecf0f1;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"2"</div>
                                    <div>"Преглед"</div>
                                </div>
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #ecf0f1;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"3"</div>
                                    <div>"Готово"</div>
                                </div>
                            </div>

                            <div style="background: white; border-radius: 8px; padding: 40px; text-align: center;">
                                // Supported formats info
                                <div style="margin-bottom: 30px; padding: 15px; background: #e8f4fd; border-radius: 8px; text-align: left;">
                                    <h4 style="margin: 0 0 10px 0; color: #2980b9;">"Поддържани формати:"</h4>
                                    <ul style="margin: 0; padding-left: 20px; color: #34495e;">
                                        <li>"OBB (XML)"</li>
                                        <li>"ISO 20022 camt.053 (Paysera, Revolut, Wise)"</li>
                                        <li>"Postbank (XML)"</li>
                                        <li>"MT940/SWIFT (Unicredit)"</li>
                                        <li>"CSV (CCBank)"</li>
                                    </ul>
                                </div>

                                // Drop zone
                                <div
                                    style=move || format!(
                                        "border: 3px dashed {}; border-radius: 8px; padding: 60px; margin-bottom: 20px; transition: all 0.2s; {}",
                                        if is_dragging.get() { "#3498db" } else { "#bdc3c7" },
                                        if is_dragging.get() { "background: #ebf5fb;" } else { "" }
                                    )
                                    on:dragover=on_drag_over
                                    on:dragleave=on_drag_leave
                                    on:drop=on_drop
                                >
                                    <div style="font-size: 3em; margin-bottom: 10px;">"+"</div>
                                    <h3 style="margin: 10px 0;">"Плъзнете файл тук"</h3>
                                    <p style="color: #7f8c8d; margin: 10px 0;">"или"</p>
                                    <button
                                        style="background: #3498db; color: white; border: none; padding: 12px 24px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                                        on:click=on_file_select
                                    >
                                        "Изберете файл"
                                    </button>
                                    {move || {
                                        let name = file_name.get();
                                        if !name.is_empty() {
                                            view! {
                                                <p style="color: #27ae60; margin-top: 15px; font-weight: bold; font-size: 1.1em;">
                                                    "Избран файл: "{name}
                                                </p>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <p style="color: #7f8c8d; margin-top: 15px;">
                                                    "Поддържани формати: XML, CSV, MT940"
                                                </p>
                                            }.into_view()
                                        }
                                    }}
                                </div>

                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 15px 40px; border-radius: 4px; cursor: pointer; font-size: 1.1em;"
                                    on:click=do_preview
                                    disabled=move || loading.get() || file_content.get().is_empty()
                                >
                                    {move || if loading.get() { "Обработка..." } else { "Преглед" }}
                                </button>
                            </div>
                        </div>
                    }.into_view()
                },

                // Step 2: Preview
                2 => {
                    let preview = preview_result.get();
                    let acc_list = bank_accounts.get();

                    view! {
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <h1 style="margin: 0;">"Преглед на транзакции"</h1>
                                <button
                                    style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=move |_| current_step.set(1)
                                >
                                    "Назад"
                                </button>
                            </div>

                            // Step indicator
                            <div style="display: flex; gap: 20px; margin: 30px 0;">
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #3498db; color: white;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"1"</div>
                                    <div>"Качване"</div>
                                </div>
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #3498db; color: white;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"2"</div>
                                    <div>"Преглед"</div>
                                </div>
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #ecf0f1;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"3"</div>
                                    <div>"Готово"</div>
                                </div>
                            </div>

                            // File info
                            <div style="background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                                <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px;">
                                    <div>
                                        <div style="font-size: 0.9em; color: #7f8c8d;">"Формат"</div>
                                        <div style="font-weight: bold;">{preview.bank_format.clone().unwrap_or("-".to_string())}</div>
                                    </div>
                                    <div>
                                        <div style="font-size: 0.9em; color: #7f8c8d;">"IBAN"</div>
                                        <div style="font-weight: bold; font-family: monospace;">{preview.account_iban.clone().unwrap_or("-".to_string())}</div>
                                    </div>
                                    <div>
                                        <div style="font-size: 0.9em; color: #7f8c8d;">"Период"</div>
                                        <div style="font-weight: bold;">
                                            {format!("{} - {}",
                                                preview.period_from.clone().unwrap_or("-".to_string()),
                                                preview.period_to.clone().unwrap_or("-".to_string())
                                            )}
                                        </div>
                                    </div>
                                    <div>
                                        <div style="font-size: 0.9em; color: #7f8c8d;">"Валута"</div>
                                        <div style="font-weight: bold;">{preview.account_currency.clone().unwrap_or("-".to_string())}</div>
                                    </div>
                                </div>

                                // Stats
                                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-top: 20px; padding-top: 20px; border-top: 1px solid #dee2e6;">
                                    <div style="text-align: center; padding: 15px; background: #e8f5e9; border-radius: 8px;">
                                        <div style="font-size: 2em; font-weight: bold; color: #27ae60;">{preview.new_count}</div>
                                        <div style="color: #7f8c8d;">"Нови транзакции"</div>
                                    </div>
                                    <div style="text-align: center; padding: 15px; background: #fff3e0; border-radius: 8px;">
                                        <div style="font-size: 2em; font-weight: bold; color: #f39c12;">{preview.duplicate_count}</div>
                                        <div style="color: #7f8c8d;">"Дубликати"</div>
                                    </div>
                                    <div style="text-align: center; padding: 15px; background: #e3f2fd; border-radius: 8px;">
                                        <div style="font-size: 2em; font-weight: bold; color: #3498db;">{preview.total_count}</div>
                                        <div style="color: #7f8c8d;">"Общо във файла"</div>
                                    </div>
                                </div>
                            </div>

                            // Bank account selector
                            <div style="background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                                <label style="display: block; margin-bottom: 10px; font-weight: bold;">"Изберете банкова сметка за импорт:"</label>
                                <select
                                    style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em;"
                                    on:change=move |ev| selected_bank_account_id.set(event_target_value(&ev))
                                >
                                    <option value="">"-- Изберете сметка --"</option>
                                    {acc_list.iter().map(|acc| {
                                        let selected = selected_bank_account_id.get() == acc.id.to_string();
                                        let display = if let Some(iban) = &acc.iban {
                                            format!("{} ({})", acc.name, iban)
                                        } else {
                                            acc.name.clone()
                                        };
                                        view! {
                                            <option value={acc.id.to_string()} selected=selected>
                                                {display}
                                            </option>
                                        }
                                    }).collect_view()}
                                </select>
                                {move || {
                                    let preview = preview_result.get();
                                    if let Some(name) = preview.bank_account_name {
                                        view! {
                                            <p style="margin-top: 10px; color: #27ae60;">
                                                "Автоматично разпозната сметка: "{name}
                                            </p>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }
                                }}
                            </div>

                            // Transactions preview table
                            <div style="background: white; border-radius: 8px; overflow: hidden; margin-bottom: 20px;">
                                <h3 style="padding: 20px; margin: 0; border-bottom: 1px solid #dee2e6;">"Преглед на транзакции (първите 50)"</h3>
                                <div style="max-height: 400px; overflow-y: auto;">
                                    <table style="width: 100%; border-collapse: collapse;">
                                        <thead>
                                            <tr style="background: #f8f9fa;">
                                                <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Дата"</th>
                                                <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Описание"</th>
                                                <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Контрагент"</th>
                                                <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Сума"</th>
                                                <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6;">"Статус"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {preview.transactions.iter().map(|tx| {
                                                let amount_style = if tx.amount >= 0.0 {
                                                    "color: #27ae60; font-weight: bold;"
                                                } else {
                                                    "color: #e74c3c; font-weight: bold;"
                                                };
                                                let row_style = if tx.is_duplicate {
                                                    "background: #ffeaa7; border-bottom: 1px solid #dee2e6;"
                                                } else {
                                                    "border-bottom: 1px solid #dee2e6;"
                                                };
                                                let date_display = if tx.date.len() >= 10 { tx.date[..10].to_string() } else { tx.date.clone() };

                                                view! {
                                                    <tr style=row_style>
                                                        <td style="padding: 12px;">{date_display}</td>
                                                        <td style="padding: 12px; max-width: 250px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                                                            {tx.description.clone().unwrap_or("-".to_string())}
                                                        </td>
                                                        <td style="padding: 12px;">{tx.contra_name.clone().unwrap_or("-".to_string())}</td>
                                                        <td style=format!("padding: 12px; text-align: right; {}", amount_style)>
                                                            {format!("{:+.2} {}", tx.amount, tx.currency)}
                                                        </td>
                                                        <td style="padding: 12px; text-align: center;">
                                                            {if tx.is_duplicate {
                                                                view! {
                                                                    <span style="background: #f39c12; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                        "Дубликат"
                                                                    </span>
                                                                }.into_view()
                                                            } else {
                                                                view! {
                                                                    <span style="background: #27ae60; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                        "Нова"
                                                                    </span>
                                                                }.into_view()
                                                            }}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            // Import button
                            <div style="text-align: center;">
                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 15px 40px; border-radius: 4px; cursor: pointer; font-size: 1.1em;"
                                    on:click=do_import
                                    disabled=move || loading.get() || selected_bank_account_id.get().is_empty()
                                >
                                    {move || if loading.get() { "Импортиране...".to_string() } else { format!("Импортирай {} нови транзакции", preview_result.get().new_count) }}
                                </button>
                            </div>
                        </div>
                    }.into_view()
                },

                // Step 3: Done
                3 => {
                    let result = import_result.get();
                    let imported = result.as_ref()
                        .and_then(|r| r.get("imported_count"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let journal_count = result.as_ref()
                        .and_then(|r| r.get("journal_count"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let auto_journal = result.as_ref()
                        .and_then(|r| r.get("auto_journal"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    view! {
                        <div>
                            // Step indicator
                            <div style="display: flex; gap: 20px; margin: 30px 0;">
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #3498db; color: white;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"1"</div>
                                    <div>"Качване"</div>
                                </div>
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #3498db; color: white;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"2"</div>
                                    <div>"Преглед"</div>
                                </div>
                                <div style="flex: 1; text-align: center; padding: 15px; border-radius: 8px; background: #3498db; color: white;">
                                    <div style="font-size: 1.5em; font-weight: bold;">"3"</div>
                                    <div>"Готово"</div>
                                </div>
                            </div>

                            <div style="background: white; border-radius: 8px; padding: 60px; text-align: center;">
                                <div style="font-size: 4em; color: #27ae60; margin-bottom: 20px;">"OK"</div>
                                <h2 style="color: #27ae60; margin-bottom: 10px;">"Импортът е успешен!"</h2>
                                <p style="color: #7f8c8d; font-size: 1.2em; margin-bottom: 10px;">
                                    {format!("Импортирани са {} нови транзакции", imported)}
                                </p>
                                {if auto_journal {
                                    view! {
                                        <p style="color: #3498db; font-size: 1.1em; margin-bottom: 30px;">
                                            {format!("Автоматично създадени {} счетоводни записа с буферна сметка", journal_count)}
                                        </p>
                                    }.into_view()
                                } else {
                                    view! {
                                        <p style="color: #7f8c8d; font-size: 0.95em; margin-bottom: 30px;">
                                            "Настройте буферна сметка в Банкови сметки за автоматично осчетоводяване при импорт."
                                        </p>
                                    }.into_view()
                                }}
                                <div style="display: flex; gap: 15px; justify-content: center;">
                                    {if auto_journal && journal_count > 0 {
                                        view! {
                                            <button
                                                style="background: #e67e22; color: white; border: none; padding: 15px 30px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                                                on:click=go_to_processing
                                            >
                                                "Обработи транзакциите"
                                            </button>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    <button
                                        style="background: #3498db; color: white; border: none; padding: 15px 30px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                                        on:click=reset_wizard
                                    >
                                        "Виж транзакциите"
                                    </button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 15px 30px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                                        on:click=move |_| current_step.set(1)
                                    >
                                        "Импортирай друг файл"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                },

                _ => view! { <div></div> }.into_view()
            }}

            // Booking modal (for transactions without journal entry)
            {move || {
                if show_booking_modal.get() {
                    let tx = booking_transaction.get();
                    let gl_list = gl_accounts.get();

                    if let Some(tx) = tx {
                        let amount_display = format!("{:+.2} {}", tx.amount, tx.currency);
                        let is_deposit = tx.amount >= 0.0;

                        view! {
                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                                <div style="background: white; border-radius: 8px; padding: 30px; width: 500px; max-height: 90vh; overflow-y: auto;">
                                    <h2 style="margin: 0 0 20px 0;">"Осчетоводяване на транзакция"</h2>

                                    <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                        <div style="margin-bottom: 10px;">
                                            <strong>"Дата: "</strong>{tx.date.clone()}
                                        </div>
                                        <div style="margin-bottom: 10px;">
                                            <strong>"Описание: "</strong>{tx.description.clone().unwrap_or("-".to_string())}
                                        </div>
                                        <div style="margin-bottom: 10px;">
                                            <strong>"Контрагент: "</strong>{tx.contra_name.clone().unwrap_or("-".to_string())}
                                        </div>
                                        <div style=format!("font-size: 1.3em; font-weight: bold; color: {};", if is_deposit { "#27ae60" } else { "#e74c3c" })>
                                            {amount_display}
                                        </div>
                                    </div>

                                    <div style="margin-bottom: 20px;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">
                                            {if is_deposit { "Кредитна сметка (откъде идват парите):" } else { "Дебитна сметка (за какво са парите):" }}
                                        </label>
                                        <select
                                            style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em;"
                                            on:change=move |ev| booking_contra_account_id.set(event_target_value(&ev))
                                        >
                                            <option value="">"-- Изберете сметка --"</option>
                                            {gl_list.iter().map(|gl| {
                                                let selected = booking_contra_account_id.get() == gl.id.to_string();
                                                view! {
                                                    <option value={gl.id.to_string()} selected=selected>
                                                        {format!("{} - {}", gl.code, gl.name)}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>

                                    <div style="background: #e8f4fd; padding: 15px; border-radius: 8px; margin-bottom: 20px; font-size: 0.9em;">
                                        <strong>"Ще се създаде запис:"</strong>
                                        <div style="margin-top: 10px;">
                                            {if is_deposit {
                                                view! {
                                                    <div>
                                                        <div>"Дт Банка (5xx) - "{format!("{:.2}", tx.amount.abs())}</div>
                                                        <div>"Кт Кореспондираща сметка - "{format!("{:.2}", tx.amount.abs())}</div>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div>
                                                        <div>"Дт Кореспондираща сметка - "{format!("{:.2}", tx.amount.abs())}</div>
                                                        <div>"Кт Банка (5xx) - "{format!("{:.2}", tx.amount.abs())}</div>
                                                    </div>
                                                }.into_view()
                                            }}
                                        </div>
                                    </div>

                                    <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                        <button
                                            style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| show_booking_modal.set(false)
                                        >
                                            "Отказ"
                                        </button>
                                        <button
                                            style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=book_transaction
                                            disabled=move || loading.get() || booking_contra_account_id.get().is_empty()
                                        >
                                            {move || if loading.get() { "Осчетоводяване..." } else { "Осчетоводи" }}
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

            // Reallocate modal (change buffer account to real account)
            {move || {
                if show_reallocate_modal.get() {
                    let tx = reallocate_transaction.get();
                    let gl_list = gl_accounts.get();

                    if let Some(tx) = tx {
                        let amount_display = format!("{:+.2} {}", tx.amount, tx.currency);
                        let is_deposit = tx.amount >= 0.0;

                        // Parse journal lines to show the current entry
                        let lines: Vec<JournalLine> = tx.journal_lines.as_ref()
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();

                        view! {
                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                                <div style="background: white; border-radius: 8px; padding: 30px; width: 600px; max-height: 90vh; overflow-y: auto;">
                                    <h2 style="margin: 0 0 20px 0;">"Обработка на транзакция"</h2>

                                    // Transaction info
                                    <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                                            <div>
                                                <strong>"Дата: "</strong>{if tx.date.len() >= 10 { tx.date[..10].to_string() } else { tx.date.clone() }}
                                            </div>
                                            <div style=format!("text-align: right; font-size: 1.2em; font-weight: bold; color: {};", if is_deposit { "#27ae60" } else { "#e74c3c" })>
                                                {amount_display}
                                            </div>
                                        </div>
                                        <div style="margin-top: 8px;">
                                            <strong>"Описание: "</strong>{tx.description.clone().unwrap_or("-".to_string())}
                                        </div>
                                        {if tx.contra_name.is_some() {
                                            view! {
                                                <div style="margin-top: 5px;">
                                                    <strong>"Контрагент: "</strong>{tx.contra_name.clone().unwrap_or_default()}
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }}
                                    </div>

                                    // Current journal entry lines
                                    <div style="margin-bottom: 20px;">
                                        <h4 style="margin: 0 0 10px 0;">"Текущ счетоводен запис:"</h4>
                                        {lines.iter().enumerate().map(|(i, line)| {
                                            let acct_id = line.account_id.unwrap_or(0);
                                            let acct_name = find_account_name(acct_id);
                                            let debit = line.debit.unwrap_or(0.0);
                                            let credit = line.credit.unwrap_or(0.0);
                                            let is_bank_line = i == 0 && is_deposit || i == 1 && !is_deposit;

                                            let bg_color = if is_bank_line {
                                                "#e3f2fd" // Blue for bank line
                                            } else {
                                                "#fff8e1" // Amber for buffer line
                                            };

                                            view! {
                                                <div style=format!("padding: 12px; border-radius: 6px; margin-bottom: 8px; background: {};", bg_color)>
                                                    <div style="display: flex; justify-content: space-between; align-items: center;">
                                                        <div>
                                                            <strong>{&acct_name}</strong>
                                                            {if is_bank_line {
                                                                view! { <span style="font-size: 0.8em; color: #2196F3; margin-left: 8px;">"(банка)"</span> }.into_view()
                                                            } else {
                                                                view! { <span style="font-size: 0.8em; color: #FF9800; margin-left: 8px;">"(буфер)"</span> }.into_view()
                                                            }}
                                                        </div>
                                                        <div style="font-family: monospace;">
                                                            {if debit > 0.0 {
                                                                format!("Дт {:.2}", debit)
                                                            } else {
                                                                format!("Кт {:.2}", credit)
                                                            }}
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>

                                    // New account selector
                                    <div style="margin-bottom: 20px;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">
                                            "Заменете буферната сметка с:"
                                        </label>
                                        <select
                                            style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em;"
                                            on:change=move |ev| reallocate_account_id.set(event_target_value(&ev))
                                        >
                                            <option value="">"-- Изберете сметка --"</option>
                                            {gl_list.iter().map(|gl| {
                                                let selected = reallocate_account_id.get() == gl.id.to_string();
                                                view! {
                                                    <option value={gl.id.to_string()} selected=selected>
                                                        {format!("{} - {}", gl.code, gl.name)}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>

                                    <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                        <button
                                            style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| show_reallocate_modal.set(false)
                                        >
                                            "Отказ"
                                        </button>
                                        <button
                                            style="background: #e67e22; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=do_reallocate
                                            disabled=move || loading.get() || reallocate_account_id.get().is_empty()
                                        >
                                            {move || if loading.get() { "Преразпределяне..." } else { "Преразпредели" }}
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

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}
