use leptos::prelude::*;
use leptos::ev;
use leptos::html;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{FileReader, ProgressEvent, DragEvent};
use crate::components::{Layout, Pagination, PaginationInfo};
use crate::api::{api_get, api_post, api_put, api_delete};
use crate::context::use_company_context;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalLine {
    pub account_id: Option<i64>,
    #[serde(default)]
    pub account_code: Option<String>,
    pub debit: Option<f64>,
    pub credit: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CounterpartOption {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub eik: Option<String>,
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
    #[serde(default)]
    pub journal_entry_status: String,
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

#[derive(Debug, Clone, Default)]
struct EditJeLine {
    account_id: i64,
    account_code: String,
    debit: f64,
    credit: f64,
}

#[component]
pub fn BankTransactions() -> impl IntoView {
    let ctx = use_company_context();

    // Wizard state
    let current_step = RwSignal::new(0); // 0 = list view, 1 = upload, 2 = preview, 3 = done
    let file_name = RwSignal::new(String::new());
    let file_content = RwSignal::new(Vec::<u8>::new());
    let file_input_ref = NodeRef::<html::Input>::new();
    let loading = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let is_dragging = RwSignal::new(false);

    // Preview data
    let preview_result = RwSignal::new(PreviewResult::default());
    let selected_bank_account_id = RwSignal::new(String::new());

    // List view state
    let transactions = RwSignal::new(Vec::<BankTransaction>::new());
    let bank_accounts = RwSignal::new(Vec::<BankAccount>::new());
    let gl_accounts = RwSignal::new(Vec::<GLAccount>::new());
    let filter_account = RwSignal::new(String::new());
    let filter_status = RwSignal::new(String::new());
    let filter_date_from = RwSignal::new(String::new());
    let filter_date_to = RwSignal::new(String::new());
    let current_page = RwSignal::new(1i32);
    let per_page = RwSignal::new(25i32);
    let pagination_info = RwSignal::new(PaginationInfo::default());
    let load_seq = RwSignal::new(0u64);

    // Booking modal (for transactions without journal entry)
    let show_booking_modal = RwSignal::new(false);
    let booking_transaction = RwSignal::new(Option::<BankTransaction>::None);
    let booking_contra_account_id = RwSignal::new(String::new());

    // Preview / allocate modal (stays on this page)
    let show_reallocate_modal = RwSignal::new(false);
    let reallocate_transaction = RwSignal::new(Option::<BankTransaction>::None);
    let reallocate_account_id = RwSignal::new(String::new());
    let reallocate_account_search = RwSignal::new(String::new());
    let reallocate_counterpart_id = RwSignal::new(String::new());
    let reallocate_counterpart_search = RwSignal::new(String::new());
    let counterparts = RwSignal::new(Vec::<CounterpartOption>::new());
    let je_edit_mode = RwSignal::new(false);
    let je_edit_loading = RwSignal::new(false);
    let je_edit_saving = RwSignal::new(false);
    let je_edit_error = RwSignal::new(String::new());
    let je_edit_status = RwSignal::new(String::new());
    let je_edit_description = RwSignal::new(String::new());
    let je_edit_counterpart_id = RwSignal::new(String::new());
    let je_edit_counterpart_search = RwSignal::new(String::new());
    let je_edit_account_search = RwSignal::new(String::new());
    let je_edit_lines = RwSignal::new(Vec::<EditJeLine>::new());
    let selected_ids = RwSignal::new(std::collections::HashSet::<i64>::new());

    // Import result
    let import_result = RwSignal::new(Option::<serde_json::Value>::None);

    // Load transactions
    let load_transactions = move || {
        let company_id = ctx.selected_company_id.get_untracked().unwrap_or(1);
        loading.set(true);

        let account_filter = filter_account.get_untracked();
        let status_filter = filter_status.get_untracked();
        let date_from = filter_date_from.get_untracked();
        let date_to = filter_date_to.get_untracked();
        let page = current_page.get_untracked();
        let pp = per_page.get_untracked();
        let seq = load_seq.get_untracked() + 1;
        load_seq.set(seq);

        spawn_local(async move {
            let mut url = format!(
                "/api/companies/{}/bank_transactions?page={}&per_page={}",
                company_id, page, pp
            );

            if !account_filter.is_empty() {
                url.push_str(&format!("&bank_account_id={}", account_filter));
            }
            if !status_filter.is_empty() {
                url.push_str(&format!("&status={}", status_filter));
            }
            if !date_from.is_empty() {
                url.push_str(&format!("&date_from={}", date_from));
            }
            if !date_to.is_empty() {
                url.push_str(&format!("&date_to={}", date_to));
            }

            match api_get(&url).await {
                Ok(data) => {
                    if load_seq.get_untracked() != seq {
                        return;
                    }
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let list: Vec<BankTransaction> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        transactions.set(list);
                        selected_ids.set(std::collections::HashSet::new());
                    }
                    if let Some(pag) = data.get("pagination") {
                        pagination_info.set(PaginationInfo {
                            page: pag.get("page").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                            per_page: pag.get("per_page").and_then(|v| v.as_i64()).unwrap_or(25) as i32,
                            total: pag.get("total").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                            total_pages: pag.get("total_pages").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                        });
                    }
                }
                Err(e) => {
                    if load_seq.get_untracked() != seq {
                        return;
                    }
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            if load_seq.get_untracked() == seq {
                loading.set(false);
            }
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

    let load_counterparts = move || {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        spawn_local(async move {
            let url = format!("/api/companies/{}/counterparts", company_id);
            if let Ok(data) = api_get(&url).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    let list: Vec<CounterpartOption> = arr.iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect();
                    counterparts.set(list);
                }
            }
        });
    };

    // Initial load
    Effect::new(move |_| {
        ctx.selected_company_id.get();
        current_page.set(1);
        load_transactions();
        load_bank_accounts();
        load_gl_accounts();
        load_counterparts();
    });

    let apply_filters = move || {
        current_page.set(1);
        load_transactions();
    };

    let on_page_change = Callback::new(move |page: i32| {
        current_page.set(page);
        load_transactions();
    });

    let on_per_page_change = Callback::new(move |pp: i32| {
        per_page.set(pp);
        current_page.set(1);
        load_transactions();
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
            let counterpart_id = reallocate_counterpart_id.get();
            loading.set(true);
            error_msg.set(String::new());

            spawn_local(async move {
                let mut payload = serde_json::json!({
                    "account_id": new_account_id.parse::<i64>().ok(),
                });
                if let Ok(cp) = counterpart_id.parse::<i64>() {
                    payload["counterpart_id"] = serde_json::json!(cp);
                }

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

    // Delete transaction (booked, orphaned, or unbooked)
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

    let delete_selected = move |_| {
        let ids: Vec<i64> = selected_ids.get().into_iter().collect();
        if ids.is_empty() {
            error_msg.set("Няма избрани транзакции".to_string());
            return;
        }
        let n = ids.len();
        let ok = web_sys::window()
            .unwrap()
            .confirm_with_message(&format!("Изтриване на {} транзакции? Свързаните чернови записи също се махат.", n))
            .unwrap_or(false);
        if !ok {
            return;
        }
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        loading.set(true);
        spawn_local(async move {
            let url = format!("/api/companies/{}/bank_transactions/bulk_delete", company_id);
            match api_post(&url, &serde_json::json!({ "ids": ids })).await {
                Ok(data) => {
                    let deleted = data.pointer("/data/deleted").and_then(|v| v.as_i64()).unwrap_or(0);
                    let errs = data.pointer("/data/errors").and_then(|v| v.as_array())
                        .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("; "))
                        .unwrap_or_default();
                    if deleted > 0 {
                        success_msg.set(format!("Изтрити {} транзакции", deleted));
                    }
                    if !errs.is_empty() {
                        error_msg.set(errs);
                    }
                    load_transactions();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            loading.set(false);
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

    let auto_allocate = move |_| {
        let ok = web_sys::window()
            .unwrap()
            .confirm_with_message("Ще се обработят неосчетоводените и чакащите транзакции по ключовите думи (кирилица и латиница). Не се ползва Mistral. Продължавате?")
            .unwrap_or(false);
        if !ok {
            return;
        }
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        if company_id == 0 {
            error_msg.set("Няма избрана фирма".to_string());
            return;
        }
        loading.set(true);
        error_msg.set(String::new());
        spawn_local(async move {
            let url = format!("/api/companies/{}/bank_transactions/auto_allocate", company_id);
            match api_post(&url, &serde_json::json!({})).await {
                Ok(data) => {
                    let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Готово");
                    success_msg.set(msg.to_string());
                    load_transactions();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            loading.set(false);
        });
    };

    // Open booking modal
    let open_booking_modal = move |tx: BankTransaction| {
        booking_transaction.set(Some(tx));
        booking_contra_account_id.set(String::new());
        show_booking_modal.set(true);
    };

    // Open preview / allocate modal on this page (no navigation to journal-entries)
    let open_reallocate_modal = move |tx: BankTransaction| {
        je_edit_mode.set(false);
        je_edit_loading.set(false);
        je_edit_saving.set(false);
        je_edit_error.set(String::new());
        je_edit_lines.set(Vec::new());
        je_edit_account_search.set(String::new());
        je_edit_counterpart_id.set(String::new());
        je_edit_counterpart_search.set(String::new());
        reallocate_account_id.set(String::new());
        reallocate_account_search.set(String::new());
        reallocate_counterpart_id.set(String::new());
        let hint = tx.contra_name.clone().unwrap_or_default();
        reallocate_counterpart_search.set(hint.clone());
        if !hint.is_empty() {
            let needle = hint.to_lowercase();
            if let Some(cp) = counterparts.get().iter().find(|c| {
                c.name.to_lowercase() == needle
                    || c.name.to_lowercase().contains(&needle)
                    || needle.contains(&c.name.to_lowercase())
            }) {
                reallocate_counterpart_id.set(cp.id.to_string());
            }
        }
        je_edit_status.set(tx.journal_entry_status.clone());
        reallocate_transaction.set(Some(tx));
        show_reallocate_modal.set(true);
    };

    let start_je_edit = move |_| {
        let Some(tx) = reallocate_transaction.get() else { return; };
        let Some(je_id) = tx.journal_entry_id else {
            je_edit_error.set("Няма счетоводен запис".to_string());
            return;
        };
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        if company_id == 0 {
            je_edit_error.set("Няма избрана фирма".to_string());
            return;
        }
        je_edit_loading.set(true);
        je_edit_error.set(String::new());
        spawn_local(async move {
            let url = format!("/api/companies/{}/journal_entries/{}", company_id, je_id);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(d) = data.get("data") {
                        je_edit_description.set(
                            d.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string()
                        );
                        je_edit_status.set(
                            d.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string()
                        );
                        if let Some(id) = d.get("counterpart_id").and_then(|v| v.as_i64()) {
                            je_edit_counterpart_id.set(id.to_string());
                            if let Some(cp) = counterparts.get().iter().find(|c| c.id == id) {
                                je_edit_counterpart_search.set(cp.name.clone());
                            }
                        } else {
                            je_edit_counterpart_id.set(String::new());
                            let hint = reallocate_transaction.get()
                                .and_then(|t| t.contra_name.clone())
                                .unwrap_or_default();
                            je_edit_counterpart_search.set(hint);
                        }
                        let mut parsed: Vec<EditJeLine> = Vec::new();
                        let gl_list = gl_accounts.get();
                        if let Some(arr) = d.get("lines").and_then(|v| v.as_array()) {
                            for l in arr {
                                let account_id = l.get("account_id").and_then(|v| v.as_i64()).unwrap_or(0);
                                let mut account_code = l.get("account_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                if account_code.trim().is_empty() && account_id != 0 {
                                    if let Some(a) = gl_list.iter().find(|a| a.id == account_id) {
                                        account_code = a.code.clone();
                                    }
                                }
                                parsed.push(EditJeLine {
                                    account_id,
                                    account_code,
                                    debit: l.get("debit_amount").and_then(|v| v.as_f64())
                                        .or_else(|| l.get("debit").and_then(|v| v.as_f64()))
                                        .unwrap_or(0.0),
                                    credit: l.get("credit_amount").and_then(|v| v.as_f64())
                                        .or_else(|| l.get("credit").and_then(|v| v.as_f64()))
                                        .unwrap_or(0.0),
                                });
                            }
                        }
                        if parsed.is_empty() {
                            parsed.push(EditJeLine::default());
                            parsed.push(EditJeLine::default());
                        }
                        je_edit_lines.set(parsed);
                        je_edit_mode.set(true);
                    } else {
                        je_edit_error.set("Невалиден отговор от сървъра".to_string());
                    }
                }
                Err(e) => je_edit_error.set(format!("Грешка при зареждане: {}", e)),
            }
            je_edit_loading.set(false);
        });
    };

    let persist_je_edit = move |and_post: bool| {
        if je_edit_saving.get_untracked() {
            return;
        }
        let Some(je_id) = reallocate_transaction.get().and_then(|t| t.journal_entry_id) else { return; };
        let lines = je_edit_lines.get();
        let total_debit: f64 = lines.iter().map(|l| l.debit).sum();
        let total_credit: f64 = lines.iter().map(|l| l.credit).sum();
        if (total_debit - total_credit).abs() >= 0.01 || total_debit == 0.0 {
            je_edit_error.set("Записът трябва да е балансиран (дебит = кредит, различни от 0)".to_string());
            return;
        }
        if lines.iter().any(|l| l.account_id == 0) {
            je_edit_error.set("Изберете сметка на всеки ред".to_string());
            return;
        }
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        let status = je_edit_status.get();
        let mut desc = je_edit_description.get();
        if desc.trim().is_empty() {
            desc = "Банкова транзакция".to_string();
        }
        let counterpart_id = je_edit_counterpart_id.get().parse::<i64>().ok();
        let gl_list = gl_accounts.get();
        let tx_meta = reallocate_transaction.get();
        let document_number = tx_meta.as_ref().and_then(|t| t.reference.clone()).filter(|s| !s.trim().is_empty());
        let document_date = tx_meta.as_ref().map(|t| {
            if t.date.len() >= 10 { t.date[..10].to_string() } else { t.date.clone() }
        }).filter(|s| !s.is_empty());
        let lines_json: Vec<serde_json::Value> = lines.iter().map(|l| {
            let account_code = if l.account_code.trim().is_empty() {
                gl_list.iter().find(|a| a.id == l.account_id).map(|a| a.code.clone()).unwrap_or_default()
            } else {
                l.account_code.clone()
            };
            serde_json::json!({
                "account_id": l.account_id,
                "account_code": account_code,
                "debit": l.debit,
                "credit": l.credit,
                "description": ""
            })
        }).collect();
        je_edit_saving.set(true);
        je_edit_error.set(String::new());
        spawn_local(async move {
            if status == "posted" {
                let unpost_url = format!("/api/companies/{}/journal_entries/{}/unpost", company_id, je_id);
                if let Err(e) = api_post(&unpost_url, &serde_json::json!({})).await {
                    je_edit_error.set(format!("Не може да се върне в чернова: {}", e));
                    je_edit_saving.set(false);
                    return;
                }
            }
            let mut je_body = serde_json::json!({
                "description": desc,
                "counterpart_id": counterpart_id,
                "lines": lines_json,
                "total_amount": total_debit,
                "total_vat_amount": 0.0,
                "status": "draft"
            });
            if let Some(dn) = &document_number {
                je_body["document_number"] = serde_json::json!(dn);
            }
            if let Some(dd) = &document_date {
                je_body["document_date"] = serde_json::json!(dd);
            }
            let payload = serde_json::json!({ "journal_entry": je_body });
            let url = format!("/api/companies/{}/journal_entries/{}", company_id, je_id);
            match api_put(&url, &payload).await {
                Ok(resp) => {
                    if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if and_post {
                            let post_url = format!("/api/companies/{}/journal_entries/{}/post", company_id, je_id);
                            match api_post(&post_url, &serde_json::json!({})).await {
                                Ok(post_resp) => {
                                    if post_resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                                        success_msg.set("Записът е запазен и осчетоводен".to_string());
                                        je_edit_status.set("posted".to_string());
                                        je_edit_mode.set(false);
                                        show_reallocate_modal.set(false);
                                        load_transactions();
                                    } else {
                                        let err = post_resp.get("error").and_then(|v| v.as_str()).unwrap_or("Записан е, но не е осчетоводен");
                                        je_edit_error.set(err.to_string());
                                        je_edit_status.set("draft".to_string());
                                        je_edit_mode.set(false);
                                        load_transactions();
                                    }
                                }
                                Err(e) => {
                                    je_edit_error.set(format!("Записан е, но осчетоводяването не успя: {}", e));
                                    je_edit_status.set("draft".to_string());
                                    je_edit_mode.set(false);
                                    load_transactions();
                                }
                            }
                        } else {
                            success_msg.set("Записът е обновен".to_string());
                            je_edit_status.set("draft".to_string());
                            je_edit_mode.set(false);
                            load_transactions();
                        }
                    } else {
                        let err = resp.get("error").or_else(|| resp.get("message"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Грешка при запис");
                        je_edit_error.set(err.to_string());
                    }
                }
                Err(e) => je_edit_error.set(format!("Грешка: {}", e)),
            }
            je_edit_saving.set(false);
        });
    };

    let save_je_edit = move |_| persist_je_edit(false);
    let save_and_post_je = move |_| persist_je_edit(true);

    let post_je = move |_| {
        if je_edit_saving.get_untracked() {
            return;
        }
        let Some(je_id) = reallocate_transaction.get().and_then(|t| t.journal_entry_id) else { return; };
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        je_edit_saving.set(true);
        je_edit_error.set(String::new());
        spawn_local(async move {
            let url = format!("/api/companies/{}/journal_entries/{}/post", company_id, je_id);
            match api_post(&url, &serde_json::json!({})).await {
                Ok(resp) => {
                    if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Записът е осчетоводен".to_string());
                        je_edit_status.set("posted".to_string());
                        load_transactions();
                    } else {
                        let err = resp.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при осчетоводяване");
                        je_edit_error.set(err.to_string());
                    }
                }
                Err(e) => je_edit_error.set(format!("Грешка: {}", e)),
            }
            je_edit_saving.set(false);
        });
    };

    let unpost_je = move |_| {
        if je_edit_saving.get_untracked() {
            return;
        }
        let Some(je_id) = reallocate_transaction.get().and_then(|t| t.journal_entry_id) else { return; };
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        je_edit_saving.set(true);
        je_edit_error.set(String::new());
        spawn_local(async move {
            let url = format!("/api/companies/{}/journal_entries/{}/unpost", company_id, je_id);
            match api_post(&url, &serde_json::json!({})).await {
                Ok(resp) => {
                    if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Записът е върнат в чернова".to_string());
                        je_edit_status.set("draft".to_string());
                        load_transactions();
                    } else {
                        let err = resp.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка");
                        je_edit_error.set(err.to_string());
                    }
                }
                Err(e) => je_edit_error.set(format!("Грешка: {}", e)),
            }
            je_edit_saving.set(false);
        });
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
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
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
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            {move || match current_step.get() {
                // List view — filters stay mounted so date values survive reloads and modals
                0 => view! {
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <h1 style="margin: 0;">"Банкови транзакции"</h1>
                                <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                    <a
                                        href="/bank-accounts"
                                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; text-decoration: none;"
                                    >
                                        "Сметки"
                                    </a>
                                    <a
                                        href="/bank-matching-rules"
                                        style="background: #8e44ad; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; text-decoration: none;"
                                    >
                                        "Правила"
                                    </a>
                                    <button
                                        style="background: #e67e22; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=auto_allocate
                                        disabled=move || loading.get()
                                    >
                                        {move || if loading.get() { "Обработка..." } else { "Автоматична обработка" }}
                                    </button>
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
                                    {move || {
                                        let acc_list = bank_accounts.get();
                                        let selected_acc = filter_account.get();
                                        view! {
                                            <select
                                                style="padding: 10px; border: 1px solid #ddd; border-radius: 4px; min-width: 200px;"
                                                prop:value=selected_acc.clone()
                                                on:change=move |ev| {
                                                    filter_account.set(event_target_value(&ev));
                                                    apply_filters();
                                                }
                                            >
                                                <option value="">"Всички"</option>
                                                {acc_list.iter().map(|acc| {
                                                    let selected = selected_acc == acc.id.to_string();
                                                    view! {
                                                        <option value={acc.id.to_string()} selected=selected>
                                                            {acc.name.clone()}
                                                        </option>
                                                    }
                                                }).collect_view()}
                                            </select>
                                        }
                                    }}
                                </div>
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-size: 0.9em; color: #7f8c8d;">"Статус"</label>
                                    <select
                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                        prop:value=move || filter_status.get()
                                        on:change=move |ev| {
                                            filter_status.set(event_target_value(&ev));
                                            apply_filters();
                                        }
                                    >
                                        <option value="">"Всички"</option>
                                        <option value="unbooked">"Неосчетоводени"</option>
                                        <option value="unallocated">"За обработка"</option>
                                        <option value="allocated">"Обработени"</option>
                                        <option value="posted">"Осчетоводени"</option>
                                        <option value="booked">"Всички със запис"</option>
                                    </select>
                                </div>
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-size: 0.9em; color: #7f8c8d;">"От дата"</label>
                                    <input
                                        type="date"
                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                        prop:value=move || filter_date_from.get()
                                        on:change=move |ev| {
                                            let val = event_target_value(&ev);
                                            if val == filter_date_from.get_untracked() {
                                                return;
                                            }
                                            filter_date_from.set(val);
                                            apply_filters();
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
                                            let val = event_target_value(&ev);
                                            if val == filter_date_to.get_untracked() {
                                                return;
                                            }
                                            filter_date_to.set(val);
                                            apply_filters();
                                        }
                                    />
                                </div>
                            </div>

                            // Transactions table
                            <div style="background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                {move || {
                                    let tx_list = transactions.get();
                                    if loading.get() {
                                    view! {
                                        <div style="padding: 40px; text-align: center; color: #7f8c8d;">
                                            "Зареждане..."
                                        </div>
                                    }.into_any()
                                } else if tx_list.is_empty() {
                                    view! {
                                        <div style="padding: 40px; text-align: center; color: #7f8c8d;">
                                            "Няма транзакции"
                                        </div>
                                    }.into_any()
                                } else {
                                    let visible_ids: Vec<i64> = tx_list.iter().map(|t| t.id).collect();
                                    let selected_n = selected_ids.get().len();
                                    let all_on_page = !visible_ids.is_empty()
                                        && visible_ids.iter().all(|id| selected_ids.get().contains(id));
                                    let total = pagination_info.get().total;
                                    view! {
                                        <div style="padding: 12px 16px; display: flex; gap: 10px; align-items: center; border-bottom: 1px solid #eee; background: #fafafa;">
                                            <span style="color: #7f8c8d; font-size: 13px;">
                                                {if selected_n == 0 {
                                                    format!("{} на страница · {} общо", tx_list.len(), total)
                                                } else {
                                                    format!("Избрани {} от {} на страница", selected_n, tx_list.len())
                                                }}
                                            </span>
                                            <button
                                                style="background: #e74c3c; color: white; border: none; padding: 8px 14px; border-radius: 4px; cursor: pointer;"
                                                disabled=move || selected_ids.get().is_empty() || loading.get()
                                                on:click=delete_selected
                                            >
                                                "Изтрий избраните"
                                            </button>
                                        </div>
                                        <table style="width: 100%; border-collapse: collapse;">
                                            <thead>
                                                <tr style="background: #f8f9fa;">
                                                    <th style="padding: 12px; width: 36px; border-bottom: 2px solid #dee2e6;">
                                                        <input
                                                            type="checkbox"
                                                            prop:checked=all_on_page
                                                            on:change=move |_| {
                                                                if all_on_page {
                                                                    selected_ids.set(std::collections::HashSet::new());
                                                                } else {
                                                                    selected_ids.set(visible_ids.iter().copied().collect());
                                                                }
                                                            }
                                                        />
                                                    </th>
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
                                                    let is_posted = tx.journal_entry_status == "posted";
                                                    let amount_style = if tx.amount >= 0.0 {
                                                        "color: #27ae60; font-weight: bold;"
                                                    } else {
                                                        "color: #e74c3c; font-weight: bold;"
                                                    };
                                                    let date_display = if tx.date.len() >= 10 { tx.date[..10].to_string() } else { tx.date.clone() };

                                                    let row_style = if is_posted {
                                                        "border-bottom: 1px solid #dee2e6; background: #eafaf1;"
                                                    } else if is_booked && !is_allocated {
                                                        "border-bottom: 1px solid #dee2e6; background: #fff8e1;"
                                                    } else {
                                                        "border-bottom: 1px solid #dee2e6;"
                                                    };

                                                    let checked = selected_ids.get().contains(&tx_id);
                                                    view! {
                                                        <tr style=row_style>
                                                            <td style="padding: 12px;">
                                                                <input
                                                                    type="checkbox"
                                                                    prop:checked=checked
                                                                    on:change=move |_| {
                                                                        selected_ids.update(|set| {
                                                                            if set.contains(&tx_id) {
                                                                                set.remove(&tx_id);
                                                                            } else {
                                                                                set.insert(tx_id);
                                                                            }
                                                                        });
                                                                    }
                                                                />
                                                            </td>
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
                                                                    }.into_any()
                                                                } else if !is_allocated {
                                                                    view! {
                                                                        <span style="background: #f39c12; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                            "За обработка"
                                                                        </span>
                                                                    }.into_any()
                                                                } else if is_posted {
                                                                    view! {
                                                                        <span style="background: #1e8449; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                            "Осчетоводен"
                                                                        </span>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span style="background: #3498db; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                            "Обработено"
                                                                        </span>
                                                                    }.into_any()
                                                                }}
                                                            </td>
                                                            <td style="padding: 12px; text-align: center;">
                                                                <div style="display: flex; gap: 5px; justify-content: center; flex-wrap: wrap;">
                                                                    {if !is_booked {
                                                                        view! {
                                                                            <button
                                                                                style="background: #27ae60; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                                                on:click=move |_| open_booking_modal(tx_clone.clone())
                                                                            >
                                                                                "Осчетоводи"
                                                                            </button>
                                                                        }.into_any()
                                                                    } else if !is_allocated {
                                                                        view! {
                                                                            <button
                                                                                style="background: #e67e22; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                                                on:click=move |_| open_reallocate_modal(tx_clone2.clone())
                                                                            >
                                                                                "Обработи"
                                                                            </button>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <button
                                                                                style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                                                on:click=move |_| open_reallocate_modal(tx_clone2.clone())
                                                                            >
                                                                                "Преглед"
                                                                            </button>
                                                                        }.into_any()
                                                                    }}
                                                                    <button
                                                                        style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                                        on:click=move |_| {
                                                                            if web_sys::window().unwrap().confirm_with_message("Изтриване на транзакцията? Свързаната чернова също се маха.").unwrap_or(false) {
                                                                                delete_transaction(tx_id);
                                                                            }
                                                                        }
                                                                    >
                                                                        "Изтрий"
                                                                    </button>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    }.into_any()
                                }
                                }}
                            </div>
                            <div style="background: white; border-radius: 8px; padding: 0 16px; margin-top: 12px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                <Pagination
                                    pagination=pagination_info.into()
                                    on_page_change=on_page_change
                                    on_per_page_change=on_per_page_change
                                />
                            </div>
                        </div>
                    }.into_any(),

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
                                        <li>"MT940/SWIFT"</li>
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
                                            }.into_any()
                                        } else {
                                            view! {
                                                <p style="color: #7f8c8d; margin-top: 15px;">
                                                    "Поддържани формати: XML, CSV, MT940"
                                                </p>
                                            }.into_any()
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
                    }.into_any()
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
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
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
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <span style="background: #27ae60; color: white; padding: 4px 8px; border-radius: 4px; font-size: 0.85em;">
                                                                        "Нова"
                                                                    </span>
                                                                }.into_any()
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
                    }.into_any()
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
                                    }.into_any()
                                } else {
                                    view! {
                                        <p style="color: #7f8c8d; font-size: 0.95em; margin-bottom: 30px;">
                                            "Настройте буферна сметка в Банкови сметки за автоматично осчетоводяване при импорт."
                                        </p>
                                    }.into_any()
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
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
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
                    }.into_any()
                },

                _ => view! { <div></div> }.into_any()
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
                                <div style="background: white; border-radius: 8px; padding: 30px; width: 640px; max-width: 95vw; max-height: 90vh; overflow-y: auto;">
                                    <h2 style="margin: 0 0 20px 0;">"Осчетоводяване на транзакция"</h2>

                                    <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                        <div style="margin-bottom: 10px;">
                                            <strong>"Дата: "</strong>{tx.date.clone()}
                                        </div>
                                        <div style="margin-bottom: 10px;">
                                            <div style="font-weight: bold; margin-bottom: 4px;">"Описание:"</div>
                                            <div style="white-space: pre-wrap; overflow-wrap: anywhere; word-break: break-word; line-height: 1.45;">
                                                {tx.description.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "-".to_string())}
                                            </div>
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
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div>
                                                        <div>"Дт Кореспондираща сметка - "{format!("{:.2}", tx.amount.abs())}</div>
                                                        <div>"Кт Банка (5xx) - "{format!("{:.2}", tx.amount.abs())}</div>
                                                    </div>
                                                }.into_any()
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
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Preview / allocate modal — stays on bank-transactions
            {move || {
                if show_reallocate_modal.get() {
                    let tx = reallocate_transaction.get();
                    let gl_list = gl_accounts.get();
                    let cp_list = counterparts.get();

                    if let Some(tx) = tx {
                        let amount_display = format!("{:+.2} {}", tx.amount, tx.currency);
                        let is_deposit = tx.amount >= 0.0;
                        let can_allocate = tx.is_booked && !tx.is_allocated;
                        let je_label = tx.journal_entry_id.map(|id| format!("#{}", id)).unwrap_or_default();
                        let contra = tx.contra_name.clone().unwrap_or_default();
                        let lines: Vec<JournalLine> = tx.journal_lines.as_ref()
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let heading = if je_label.is_empty() {
                            if can_allocate { "Алокация".to_string() } else { "Преглед".to_string() }
                        } else if can_allocate {
                            format!("Алокация  {}", je_label)
                        } else {
                            format!("Преглед  {}", je_label)
                        };
                        let amount_color = if is_deposit { "#27ae60" } else { "#e74c3c" };
                        let editing = je_edit_mode.get();
                        let loading_edit = je_edit_loading.get();
                        let posted = je_edit_status.get() == "posted";
                        let tx_date = if tx.date.len() >= 10 { tx.date[..10].to_string() } else { tx.date.clone() };
                        let tx_description = tx.description.clone()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "-".to_string());
                        let tx_reference = tx.reference.clone().unwrap_or_default();
                        let heading = if editing {
                            if je_label.is_empty() { "Редакция".to_string() } else { format!("Редакция  {}", je_label) }
                        } else {
                            heading
                        };

                        view! {
                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                                <div style="background: white; border-radius: 8px; padding: 28px; width: 720px; max-width: 95vw; max-height: 90vh; overflow-y: auto;">
                                    <div style="display: flex; justify-content: space-between; align-items: baseline; gap: 12px; margin-bottom: 16px;">
                                        <h2 style="margin: 0;">{heading}</h2>
                                        <div style=format!("font-size: 1.25em; font-weight: 700; color: {};", amount_color)>
                                            {amount_display}
                                        </div>
                                    </div>
                                    <div style="display: flex; gap: 10px; align-items: center; margin-bottom: 16px;">
                                        <span style="color: #7f8c8d; font-size: 13px;">"без ДДС"</span>
                                        <span style=format!(
                                            "padding: 3px 8px; border-radius: 4px; font-size: 12px; color: white; background: {};",
                                            if posted { "#1e8449" } else if can_allocate { "#f39c12" } else { "#3498db" }
                                        )>
                                            {if posted { "Осчетоводен" } else if can_allocate { "За обработка" } else { "Обработено · чернова" }}
                                        </span>
                                    </div>

                                    {move || {
                                        let err = je_edit_error.get();
                                        if err.is_empty() {
                                            view! { <span></span> }.into_any()
                                        } else {
                                            view! {
                                                <div style="background: #fdecea; color: #c0392b; padding: 10px 12px; border-radius: 6px; margin-bottom: 12px;">{err}</div>
                                            }.into_any()
                                        }
                                    }}

                                    <div style="background: #f8f9fa; padding: 12px 14px; border-radius: 6px; margin-bottom: 16px;">
                                        <div style="display: flex; gap: 28px; flex-wrap: wrap; margin-bottom: 10px;">
                                            <div>
                                                <div style="font-size: 12px; color: #7f8c8d; margin-bottom: 4px;">"Дата"</div>
                                                <div style="font-weight: 600;">{tx_date}</div>
                                            </div>
                                            {if !tx_reference.is_empty() {
                                                view! {
                                                    <div>
                                                        <div style="font-size: 12px; color: #7f8c8d; margin-bottom: 4px;">"Референция"</div>
                                                        <div style="font-weight: 600; overflow-wrap: anywhere; word-break: break-word;">{tx_reference.clone()}</div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                        </div>
                                        <div style="font-size: 12px; color: #7f8c8d; margin-bottom: 4px;">"Описание"</div>
                                        <div style="white-space: pre-wrap; overflow-wrap: anywhere; word-break: break-word; line-height: 1.5;">
                                            {tx_description}
                                        </div>
                                    </div>

                                    {if !contra.is_empty() && !editing {
                                        view! {
                                            <div style="background: #f8f9fa; padding: 12px 14px; border-radius: 6px; margin-bottom: 16px;">
                                                <div style="font-size: 12px; color: #7f8c8d; margin-bottom: 4px;">"Контрагент от банката"</div>
                                                <div style="font-weight: 600;">{contra.clone()}</div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}

                                    {if loading_edit {
                                        view! {
                                            <div style="padding: 28px; text-align: center; color: #7f8c8d;">"Зареждане..."</div>
                                        }.into_any()
                                    } else if editing {
                                        let acc_search = je_edit_account_search.get().to_lowercase();
                                        let filtered_gl: Vec<GLAccount> = gl_list.iter()
                                            .filter(|gl| {
                                                if acc_search.is_empty() { return true; }
                                                gl.code.to_lowercase().contains(&acc_search) || gl.name.to_lowercase().contains(&acc_search)
                                            })
                                            .cloned()
                                            .collect();
                                        let cp_search = je_edit_counterpart_search.get().to_lowercase();
                                        let filtered_cp: Vec<CounterpartOption> = cp_list.iter()
                                            .filter(|c| {
                                                if cp_search.is_empty() { return true; }
                                                c.name.to_lowercase().contains(&cp_search)
                                                    || c.eik.as_deref().unwrap_or("").to_lowercase().contains(&cp_search)
                                            })
                                            .cloned()
                                            .take(40)
                                            .collect();
                                        let edit_lines_now = je_edit_lines.get();
                                        let edit_debit: f64 = edit_lines_now.iter().map(|l| l.debit).sum();
                                        let edit_credit: f64 = edit_lines_now.iter().map(|l| l.credit).sum();
                                        let balanced = (edit_debit - edit_credit).abs() < 0.01 && edit_debit != 0.0;

                                        view! {
                                            <div>
                                                <div style="margin-bottom: 14px;">
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 600;">"Контрагент"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="Търсене по име или ЕИК"
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; margin-bottom: 6px;"
                                                        prop:value=move || je_edit_counterpart_search.get()
                                                        on:input=move |ev| je_edit_counterpart_search.set(event_target_value(&ev))
                                                    />
                                                    <select
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                        on:change=move |ev| je_edit_counterpart_id.set(event_target_value(&ev))
                                                    >
                                                        <option value="">"— без контрагент —"</option>
                                                        {filtered_cp.into_iter().map(|c| {
                                                            let selected = je_edit_counterpart_id.get() == c.id.to_string();
                                                            let label = match c.eik.as_ref().filter(|s| !s.is_empty()) {
                                                                Some(eik) => format!("{} ({})", c.name, eik),
                                                                None => c.name.clone(),
                                                            };
                                                            view! {
                                                                <option value={c.id.to_string()} selected=selected>{label}</option>
                                                            }
                                                        }).collect_view()}
                                                    </select>
                                                </div>
                                                <div style="margin-bottom: 8px;">
                                                    <input
                                                        type="text"
                                                        placeholder="Търси сметка по код или име"
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                        prop:value=move || je_edit_account_search.get()
                                                        on:input=move |ev| je_edit_account_search.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <table style="width: 100%; border-collapse: collapse; font-size: 14px; margin-bottom: 8px;">
                                                    <thead>
                                                        <tr style="background: #f8f9fa;">
                                                            <th style="text-align: left; padding: 8px; border-bottom: 2px solid #dee2e6;">"Сметка"</th>
                                                            <th style="text-align: right; padding: 8px; border-bottom: 2px solid #dee2e6; width: 110px;">"Дебит"</th>
                                                            <th style="text-align: right; padding: 8px; border-bottom: 2px solid #dee2e6; width: 110px;">"Кредит"</th>
                                                            <th style="width: 36px; border-bottom: 2px solid #dee2e6;"></th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {edit_lines_now.into_iter().enumerate().map(|(idx, line)| {
                                                            let debit_val = if line.debit > 0.0 { format!("{:.2}", line.debit) } else { String::new() };
                                                            let credit_val = if line.credit > 0.0 { format!("{:.2}", line.credit) } else { String::new() };
                                                            let selected_id = line.account_id.to_string();
                                                            let filtered_gl = filtered_gl.clone();
                                                            view! {
                                                                <tr style="border-bottom: 1px solid #eee;">
                                                                    <td style="padding: 6px;">
                                                                        <select
                                                                            style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                                                            on:change=move |ev| {
                                                                                let val = event_target_value(&ev);
                                                                                let id = val.parse::<i64>().unwrap_or(0);
                                                                                je_edit_lines.update(|rows| {
                                                                                    if let Some(row) = rows.get_mut(idx) {
                                                                                        row.account_id = id;
                                                                                        row.account_code = gl_accounts.get().iter()
                                                                                            .find(|a| a.id == id)
                                                                                            .map(|a| a.code.clone())
                                                                                            .unwrap_or_default();
                                                                                    }
                                                                                });
                                                                            }
                                                                        >
                                                                            <option value="0">"-- сметка --"</option>
                                                                            {filtered_gl.into_iter().map(|gl| {
                                                                                let sel = selected_id == gl.id.to_string();
                                                                                view! {
                                                                                    <option value={gl.id.to_string()} selected=sel>
                                                                                        {format!("{} — {}", gl.code, gl.name)}
                                                                                    </option>
                                                                                }
                                                                            }).collect_view()}
                                                                        </select>
                                                                    </td>
                                                                    <td style="padding: 6px;">
                                                                        <input
                                                                            type="number"
                                                                            step="0.01"
                                                                            style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; text-align: right; box-sizing: border-box;"
                                                                            prop:value=debit_val
                                                                            on:input=move |ev| {
                                                                                let v = event_target_value(&ev).parse::<f64>().unwrap_or(0.0);
                                                                                je_edit_lines.update(|rows| {
                                                                                    if let Some(row) = rows.get_mut(idx) {
                                                                                        row.debit = v;
                                                                                        if v != 0.0 { row.credit = 0.0; }
                                                                                    }
                                                                                });
                                                                            }
                                                                        />
                                                                    </td>
                                                                    <td style="padding: 6px;">
                                                                        <input
                                                                            type="number"
                                                                            step="0.01"
                                                                            style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; text-align: right; box-sizing: border-box;"
                                                                            prop:value=credit_val
                                                                            on:input=move |ev| {
                                                                                let v = event_target_value(&ev).parse::<f64>().unwrap_or(0.0);
                                                                                je_edit_lines.update(|rows| {
                                                                                    if let Some(row) = rows.get_mut(idx) {
                                                                                        row.credit = v;
                                                                                        if v != 0.0 { row.debit = 0.0; }
                                                                                    }
                                                                                });
                                                                            }
                                                                        />
                                                                    </td>
                                                                    <td style="padding: 6px; text-align: center;">
                                                                        <button
                                                                            style="background: transparent; border: none; color: #e74c3c; cursor: pointer; font-size: 18px;"
                                                                            on:click=move |_| {
                                                                                je_edit_lines.update(|rows| {
                                                                                    if rows.len() > 1 { rows.remove(idx); }
                                                                                });
                                                                            }
                                                                        >"×"</button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                                <button
                                                    style="background: #ecf0f1; border: none; padding: 8px 12px; border-radius: 4px; cursor: pointer; margin-bottom: 12px;"
                                                    on:click=move |_| {
                                                        je_edit_lines.update(|rows| rows.push(EditJeLine::default()));
                                                    }
                                                >"+ Ред"</button>
                                                <div style=format!("text-align: right; font-weight: 600; margin-bottom: 16px; color: {};", if balanced { "#27ae60" } else { "#e74c3c" })>
                                                    {format!("Дт {:.2}   Кт {:.2}", edit_debit, edit_credit)}
                                                    {if balanced { "  ✓" } else { "  небалансирано" }}
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div style="margin-bottom: 18px;">
                                                <h4 style="margin: 0 0 8px 0;">"Счетоводни редове"</h4>
                                                <table style="width: 100%; border-collapse: collapse; font-size: 14px;">
                                                    <thead>
                                                        <tr style="background: #f8f9fa;">
                                                            <th style="text-align: left; padding: 8px; border-bottom: 2px solid #dee2e6;">"Сметка"</th>
                                                            <th style="text-align: right; padding: 8px; border-bottom: 2px solid #dee2e6;">"Дебит"</th>
                                                            <th style="text-align: right; padding: 8px; border-bottom: 2px solid #dee2e6;">"Кредит"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {lines.iter().map(|line| {
                                                            let label = gl_line_label(&gl_list, line);
                                                            let debit = line.debit.unwrap_or(0.0);
                                                            let credit = line.credit.unwrap_or(0.0);
                                                            view! {
                                                                <tr style="border-bottom: 1px solid #eee;">
                                                                    <td style="padding: 8px;">{label}</td>
                                                                    <td style="padding: 8px; text-align: right; font-variant-numeric: tabular-nums;">
                                                                        {if debit > 0.0 { format!("{:.2}", debit) } else { String::new() }}
                                                                    </td>
                                                                    <td style="padding: 8px; text-align: right; font-variant-numeric: tabular-nums;">
                                                                        {if credit > 0.0 { format!("{:.2}", credit) } else { String::new() }}
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                                {if lines.is_empty() {
                                                    view! {
                                                        <div style="padding: 12px; color: #7f8c8d;">"Няма редове"</div>
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                        }.into_any()
                                    }}

                                    {if can_allocate && !editing && !loading_edit {
                                        let search = reallocate_account_search.get().to_lowercase();
                                        let filtered_gl: Vec<GLAccount> = gl_list.iter()
                                            .filter(|gl| {
                                                if search.is_empty() { return true; }
                                                gl.code.to_lowercase().contains(&search) || gl.name.to_lowercase().contains(&search)
                                            })
                                            .cloned()
                                            .collect();
                                        let cp_search = reallocate_counterpart_search.get().to_lowercase();
                                        let filtered_cp: Vec<CounterpartOption> = cp_list.iter()
                                            .filter(|c| {
                                                if cp_search.is_empty() { return true; }
                                                c.name.to_lowercase().contains(&cp_search)
                                                    || c.eik.as_deref().unwrap_or("").to_lowercase().contains(&cp_search)
                                            })
                                            .cloned()
                                            .take(40)
                                            .collect();

                                        view! {
                                            <div>
                                                <div style="margin-bottom: 14px;">
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 600;">"Контрагент"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="Търсене по име или ЕИК"
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; margin-bottom: 6px;"
                                                        prop:value=move || reallocate_counterpart_search.get()
                                                        on:input=move |ev| reallocate_counterpart_search.set(event_target_value(&ev))
                                                    />
                                                    <select
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                        on:change=move |ev| reallocate_counterpart_id.set(event_target_value(&ev))
                                                    >
                                                        <option value="">"— без контрагент —"</option>
                                                        {filtered_cp.into_iter().map(|c| {
                                                            let selected = reallocate_counterpart_id.get() == c.id.to_string();
                                                            let label = match c.eik.as_ref().filter(|s| !s.is_empty()) {
                                                                Some(eik) => format!("{} ({})", c.name, eik),
                                                                None => c.name.clone(),
                                                            };
                                                            view! {
                                                                <option value={c.id.to_string()} selected=selected>{label}</option>
                                                            }
                                                        }).collect_view()}
                                                    </select>
                                                </div>
                                                <div style="margin-bottom: 18px;">
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 600;">"Кореспондираща сметка (вместо буфер)"</label>
                                                    <input
                                                        type="text"
                                                        placeholder="Търси сметка по код или име"
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; margin-bottom: 6px;"
                                                        prop:value=move || reallocate_account_search.get()
                                                        on:input=move |ev| reallocate_account_search.set(event_target_value(&ev))
                                                    />
                                                    <select
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                        on:change=move |ev| reallocate_account_id.set(event_target_value(&ev))
                                                    >
                                                        <option value="">"-- Изберете сметка --"</option>
                                                        {filtered_gl.into_iter().map(|gl| {
                                                            let selected = reallocate_account_id.get() == gl.id.to_string();
                                                            view! {
                                                                <option value={gl.id.to_string()} selected=selected>
                                                                    {format!("{} — {}", gl.code, gl.name)}
                                                                </option>
                                                            }
                                                        }).collect_view()}
                                                    </select>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}

                                    <div style="display: flex; gap: 10px; justify-content: flex-end; flex-wrap: wrap;">
                                        <button
                                            style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| {
                                                if je_edit_mode.get() {
                                                    je_edit_mode.set(false);
                                                    je_edit_error.set(String::new());
                                                } else {
                                                    show_reallocate_modal.set(false);
                                                }
                                            }
                                        >
                                            {if editing { "Отказ" } else if can_allocate { "Отказ" } else { "Затвори" }}
                                        </button>
                                        {if can_allocate && !editing {
                                            view! {
                                                <button
                                                    style="background: #e67e22; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                    on:click=do_reallocate
                                                    disabled=move || loading.get() || reallocate_account_id.get().is_empty()
                                                >
                                                    {move || if loading.get() { "Алокиране..." } else { "Алокирай" }}
                                                </button>
                                            }.into_any()
                                        } else if editing {
                                            view! {
                                                <div style="display: flex; gap: 10px;">
                                                    <button
                                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                        on:click=save_je_edit
                                                        disabled=move || je_edit_saving.get()
                                                    >
                                                        {move || if je_edit_saving.get() { "Записване..." } else { "Запази" }}
                                                    </button>
                                                    <button
                                                        style="background: #1e8449; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                        on:click=save_and_post_je
                                                        disabled=move || je_edit_saving.get()
                                                    >
                                                        {move || if je_edit_saving.get() { "Записване..." } else { "Запази и осчетоводи" }}
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else if !loading_edit {
                                            view! {
                                                <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                                    {if posted {
                                                        view! {
                                                            <button
                                                                style="background: #8e44ad; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                                on:click=unpost_je
                                                                disabled=move || je_edit_saving.get()
                                                            >
                                                                {move || if je_edit_saving.get() { "..." } else { "Върни в чернова" }}
                                                            </button>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                                                <button
                                                                    style="background: #f39c12; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                                    on:click=start_je_edit
                                                                    disabled=move || je_edit_loading.get() || je_edit_saving.get()
                                                                >
                                                                    "Редакция"
                                                                </button>
                                                                <button
                                                                    style="background: #1e8449; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                                    on:click=post_je
                                                                    disabled=move || je_edit_saving.get()
                                                                >
                                                                    {move || if je_edit_saving.get() { "..." } else { "Осчетоводи" }}
                                                                </button>
                                                            </div>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
        </Layout>
    }
}

fn gl_line_label(accounts: &[GLAccount], line: &JournalLine) -> String {
    if let Some(id) = line.account_id {
        if let Some(a) = accounts.iter().find(|a| a.id == id) {
            return format!("{} — {}", a.code, a.name);
        }
    }
    if let Some(code) = line.account_code.as_ref().filter(|s| !s.is_empty()) {
        return code.clone();
    }
    format!("#{}", line.account_id.unwrap_or(0))
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}
