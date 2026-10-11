use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::components::{Layout, Pagination, PaginationInfo};
use crate::api::{api_get, api_delete, api_post, api_put, company_api_url, require_company_id};
use crate::context::use_company_context;
use gloo_timers::future::TimeoutFuture;
use js_sys;
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i64,
    pub entry_date: String,
    pub description: String,
    pub reference: Option<String>,
    pub status: String,
    pub document_number: Option<String>,
    pub total_amount: Option<f64>,
    pub total_vat_amount: Option<f64>,
    pub vat_period: Option<String>,
    pub counterpart_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalLine {
    pub id: i64,
    pub account_id: i64,
    pub account_code: Option<String>,
    pub debit_amount: f64,
    pub credit_amount: f64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalEntryDetail {
    pub id: i64,
    pub entry_date: String,
    pub document_date: Option<String>,
    pub description: String,
    pub reference: Option<String>,
    pub status: String,
    pub counterpart_id: Option<i64>,
    pub lines: Vec<JournalLine>,
    pub total_debit: f64,
    pub total_credit: f64,
    #[serde(default)]
    pub document_number: Option<String>,
    #[serde(default)]
    pub payment_journal_entry_id: Option<i64>,
    #[serde(default)]
    pub is_payment_entry: Option<bool>,
    #[serde(default)]
    pub payment_direction: Option<String>,
    #[serde(default)]
    pub can_create_payment: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Counterpart {
    pub id: i64,
    pub name: String,
    pub vat_number: Option<String>,
    pub counterpart_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Currency {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLineInput {
    pub account_code: String,
    pub debit: f64,
    pub credit: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJournalEntryPayload {
    pub entry_date: String,
    pub document_date: Option<String>,
    pub description: String,
    pub reference: Option<String>,
    pub status: String,
    pub document_number: Option<String>,
    pub vat_period: Option<String>,
    pub vat_purchase_operation: Option<String>,
    pub vat_sales_operation: Option<String>,
    pub lines: String, // JSON string of lines
    pub counterpart_id: Option<i64>,
    pub total_amount: Option<f64>,
    pub total_vat_amount: Option<f64>,
    pub journal_type: Option<String>,
}

fn format_date(date_str: &str) -> String {
    // Parse ISO date and format as DD.MM.YYYY
    if let Some(date_part) = date_str.split('T').next() {
        let parts: Vec<&str> = date_part.split('-').collect();
        if parts.len() == 3 {
            return format!("{}.{}.{}", parts[2], parts[1], parts[0]);
        }
    }
    date_str.to_string()
}

fn get_date_part(date_str: &str) -> String {
    date_str.split('T').next().unwrap_or("").to_string()
}

fn document_type_options(entry_type: &str, selected: &str) -> impl IntoView {
    crate::constants::doc_types_for(entry_type).iter().map(|entry| {
        let code = entry.code.to_string();
        let label = if entry.code.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            format!("{} — {}", entry.code, entry.name)
        } else {
            entry.name.to_string()
        };
        let is_selected = selected == entry.code;
        view! { <option value=code selected=is_selected>{label}</option> }
    }).collect_view()
}

fn doc_type_for_switch(entry_type: &str, current: &str) -> String {
    let types = crate::constants::doc_types_for(entry_type);
    if types.iter().any(|entry| entry.code == current) {
        current.to_string()
    } else if entry_type == "NON_VAT" {
        "OTHER".to_string()
    } else {
        "01".to_string()
    }
}

fn query_param(search: &str, key: &str) -> Option<String> {
    let s = search.strip_prefix('?').unwrap_or(search);
    s.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next()?;
        let v = parts.next().unwrap_or("");
        if k == key && !v.is_empty() {
            Some(v.to_string())
        } else {
            None
        }
    })
}

fn serialize_journal_lines(lines: &[JournalLineInput], accounts: &[Account]) -> String {
    let enriched: Vec<serde_json::Value> = lines.iter()
        .filter(|l| !l.account_code.trim().is_empty())
        .map(|l| {
            let account_id = accounts.iter().find(|a| a.code == l.account_code).map(|a| a.id);
            serde_json::json!({
                "account_id": account_id,
                "account_code": l.account_code,
                "debit": l.debit,
                "credit": l.credit,
                "description": l.description,
            })
        })
        .collect();
    serde_json::to_string(&enriched).unwrap_or_else(|_| "[]".to_string())
}

fn get_status_badge(status: &str) -> (&'static str, &'static str) {
    match status {
        "draft" => ("Чернова", "#f39c12"),
        "posted" => ("Осчетоводен", "#27ae60"),
        "voided" => ("Анулиран", "#e74c3c"),
        _ => ("Неизвестен", "#7f8c8d"),
    }
}

#[component]
pub fn JournalEntries() -> impl IntoView {
    let company_ctx = use_company_context();
    let entries = RwSignal::new(Vec::<JournalEntry>::new());
    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let search_query = RwSignal::new(String::new());
    let search_seq = RwSignal::new(0u64);
    let selected_status = RwSignal::new(String::new());
    let date_from = RwSignal::new(String::new());
    let date_to = RwSignal::new(String::new());
    // Pagination state
    let current_page = RwSignal::new(1i32);
    let per_page = RwSignal::new(25i32);
    let pagination_info = RwSignal::new(PaginationInfo::default());

    let deleting = RwSignal::new(false);

    // Bulk selection state
    let selected_ids = RwSignal::new(std::collections::HashSet::<i64>::new());
    let bulk_posting = RwSignal::new(false);
    let bulk_unposting = RwSignal::new(false);
    let bulk_deleting = RwSignal::new(false);

    // Selection modal states (for new entry)
    let show_counterpart_modal = RwSignal::new(false);
    let counterpart_search = RwSignal::new(String::new());
    let show_account_modal = RwSignal::new(false);
    let account_modal_line_idx = RwSignal::new(0usize);
    let account_search = RwSignal::new(String::new());

    // Selection modal states (for edit entry)
    let show_edit_counterpart_modal = RwSignal::new(false);
    let edit_counterpart_search = RwSignal::new(String::new());
    let show_edit_account_modal = RwSignal::new(false);
    let edit_account_modal_line_idx = RwSignal::new(0usize);
    let edit_account_search = RwSignal::new(String::new());

    // View modal state
    let show_view_modal = RwSignal::new(false);
    let view_entry_id = RwSignal::new(0i64);
    let view_entry_detail = RwSignal::new(Option::<JournalEntryDetail>::None);
    let view_loading = RwSignal::new(false);
    let view_posting = RwSignal::new(false);
    let view_paying = RwSignal::new(false);

    // Single entry delete modal
    let show_single_delete_modal = RwSignal::new(false);
    let single_delete_entry = RwSignal::new(Option::<JournalEntry>::None);

    // Edit modal state
    let show_edit_modal = RwSignal::new(false);
    let edit_entry = RwSignal::new(Option::<JournalEntry>::None);
    let edit_entry_id = RwSignal::new(0i64);
    let edit_saving = RwSignal::new(false);
    let edit_loading = RwSignal::new(false);

    // Edit form fields (same as new entry)
    let edit_entry_type = RwSignal::new(String::from("PURCHASE"));
    let edit_document_date = RwSignal::new(String::new());
    let edit_entry_date = RwSignal::new(String::new());
    let edit_description = RwSignal::new(String::new());
    let edit_document_number = RwSignal::new(String::new());
    let edit_vat_period = RwSignal::new(String::new());
    let edit_vat_operation = RwSignal::new(String::from("PURCHASE_CREDIT"));
    let edit_document_type = RwSignal::new(String::from("01"));
    let edit_counterpart_id = RwSignal::new(String::new());
    let edit_currency = RwSignal::new(String::from("EUR"));
    let edit_lines = RwSignal::new(vec![
        JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
        JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
    ]);

    // New entry modal state
    let show_new_entry_modal = RwSignal::new(false);
    let accounts = RwSignal::new(Vec::<Account>::new());
    let counterparts = RwSignal::new(Vec::<Counterpart>::new());
    let currencies = RwSignal::new(Vec::<Currency>::new());
    let saving = RwSignal::new(false);

    // Form fields
    let entry_type = RwSignal::new(String::from("PURCHASE")); // PURCHASE, SALE, NON_VAT
    let entry_document_date = RwSignal::new(String::new()); // Дата на документа
    let entry_date = RwSignal::new(String::new()); // Счетоводна дата (will be set on mount)
    let entry_description = RwSignal::new(String::new());
    let entry_document_number = RwSignal::new(String::new());
    let entry_vat_period = RwSignal::new(String::new()); // Will be set on mount
    let entry_vat_operation = RwSignal::new(String::from("PURCHASE_CREDIT")); // VAT deal type
    let document_type = RwSignal::new(String::from("01")); // Invoice type code
    let entry_counterpart_id = RwSignal::new(String::new()); // Selected counterpart
    let entry_currency = RwSignal::new(String::from("EUR")); // Selected currency

    // Journal lines - stored as JSON string for simplicity
    let entry_lines = RwSignal::new(vec![
        JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
        JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
    ]);
    let entry_line_count = create_memo(move |_| entry_lines.with(|l| l.len()));
    let edit_line_count = create_memo(move |_| edit_lines.with(|l| l.len()));

    // Function to reload entries
    let reload_entries = move || {
        loading.set(true);
        let page = current_page.get();
        let pp = per_page.get();
        let status = selected_status.get();
        let q = search_query.get();
        let from = date_from.get();
        let to = date_to.get();

        spawn_local(async move {
            let mut url = format!("{}/journal_entries?page={}&per_page={}", company_api_url(), page, pp);
            if !status.is_empty() {
                url.push_str(&format!("&status={}", status));
            }
            if !q.is_empty() {
                url.push_str(&format!("&q={}", urlencoding::encode(&q)));
            }
            if !from.is_empty() {
                url.push_str(&format!("&date_from={}", urlencoding::encode(&from)));
            }
            if !to.is_empty() {
                url.push_str(&format!("&date_to={}", urlencoding::encode(&to)));
            }

            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let ents: Vec<JournalEntry> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        entries.set(ents);
                    }
                    // Parse pagination
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
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Debounced server search so counterpart / document number match across pages
    let trigger_search = move || {
        current_page.set(1);
        let seq = search_seq.get_untracked() + 1;
        search_seq.set(seq);
        spawn_local(async move {
            TimeoutFuture::new(300).await;
            if search_seq.get_untracked() == seq {
                reload_entries();
            }
        });
    };

    // Bulk post selected entries
    let bulk_post_entries = move || {
        let ids: Vec<i64> = selected_ids.get().iter().copied().collect();
        if ids.is_empty() {
            return;
        }

        bulk_posting.set(true);
        error_msg.set(String::new());
        success_msg.set(String::new());

        spawn_local(async move {
            match api_post(&format!("{}/journal_entries/bulk_post", company_api_url()), &serde_json::json!({ "ids": ids })).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let count = response.get("posted_count").and_then(|v| v.as_i64()).unwrap_or(ids.len() as i64);
                        success_msg.set(format!("Успешно осчетоводени {} записа", count));
                        selected_ids.set(std::collections::HashSet::new());
                        reload_entries();
                    } else {
                        let err = response.get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("Неизвестна грешка");
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при осчетоводяване: {}", e));
                }
            }
            bulk_posting.set(false);
        });
    };

    // Bulk unpost selected entries (return to draft)
    let bulk_unpost_entries = move || {
        let ids: Vec<i64> = selected_ids.get().iter().copied().collect();
        if ids.is_empty() {
            return;
        }

        bulk_unposting.set(true);
        error_msg.set(String::new());
        success_msg.set(String::new());

        spawn_local(async move {
            match api_post(&format!("{}/journal_entries/bulk_unpost", company_api_url()), &serde_json::json!({ "ids": ids })).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let count = response.get("unposted_count").and_then(|v| v.as_i64()).unwrap_or(ids.len() as i64);
                        success_msg.set(format!("Успешно върнати в чернова {} записа", count));
                        selected_ids.set(std::collections::HashSet::new());
                        reload_entries();
                    } else {
                        let err = response.get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("Неизвестна грешка");
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при връщане в чернова: {}", e));
                }
            }
            bulk_unposting.set(false);
        });
    };

    // Load accounts for the form
    let load_accounts = move || {
        spawn_local(async move {
            match api_get(&format!("{}/accounts?per_page=1000", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let accs: Vec<Account> = arr.iter()
                            .filter_map(|v| {
                                let id = v.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
                                let code = v.get("code").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
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

    // Load counterparts
    let load_counterparts = move || {
        spawn_local(async move {
            match api_get(&format!("{}/counterparts", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let cps: Vec<Counterpart> = arr.iter()
                            .filter_map(|v| {
                                let id = v.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
                                let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                let vat_number = v.get("vat_number").and_then(|x| x.as_str()).map(|s| s.to_string());
                                let counterpart_type = v.get("counterpart_type").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                Some(Counterpart { id, name, vat_number, counterpart_type })
                            })
                            .collect();
                        counterparts.set(cps);
                    }
                }
                Err(_) => {}
            }
        });
    };

    // Load currencies
    let load_currencies = move || {
        spawn_local(async move {
            match api_get("/api/currencies").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let curs: Vec<Currency> = arr.iter()
                            .filter_map(|v| {
                                let id = v.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
                                let code = v.get("code").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                let symbol = v.get("symbol").and_then(|x| x.as_str()).map(|s| s.to_string());
                                Some(Currency { id, code, name, symbol })
                            })
                            .collect();
                        currencies.set(curs);
                    }
                }
                Err(_) => {}
            }
        });
    };

    // Reset form to initial state
    let reset_form = move || {
        entry_type.set("PURCHASE".to_string());
        // Keep current date values
        entry_description.set(String::new());
        entry_document_number.set(String::new());
        entry_vat_operation.set("PURCHASE_CREDIT".to_string());
        document_type.set("01".to_string());
        entry_lines.set(vec![
            JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
            JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
        ]);
    };

    // Save new entry
    let save_entry = move |_| {
        if saving.get_untracked() {
            return;
        }
        error_msg.set(String::new());

        if let Err(e) = require_company_id() {
            error_msg.set(e);
            return;
        }

        let desc = entry_description.get().trim().to_string();
        if desc.is_empty() {
            error_msg.set("Моля, попълнете описание на операцията".to_string());
            return;
        }
        let edate = entry_date.get();
        if edate.is_empty() {
            error_msg.set("Моля, попълнете счетоводна дата".to_string());
            return;
        }

        let lines = entry_lines.get();
        let has_account = lines.iter().any(|l| !l.account_code.trim().is_empty());
        if !has_account {
            error_msg.set("Моля, добавете поне един ред със сметка".to_string());
            return;
        }

        let total_debit: f64 = lines.iter().map(|l| l.debit).sum();
        let total_credit: f64 = lines.iter().map(|l| l.credit).sum();
        if (total_debit - total_credit).abs() >= 0.01 || total_debit == 0.0 {
            error_msg.set("Записът трябва да е балансиран (дебит = кредит, различни от 0)".to_string());
            return;
        }

        saving.set(true);

        let lines_json = serialize_journal_lines(&lines, &accounts.get());

        let etype = entry_type.get();
        let vat_purchase = if etype == "PURCHASE" {
            Some(format!("{}|{}", document_type.get(), entry_vat_operation.get()))
        } else {
            None
        };
        let vat_sales = if etype == "SALE" {
            Some(format!("{}|{}", document_type.get(), entry_vat_operation.get()))
        } else {
            None
        };

        // Calculate total_amount and total_vat_amount from lines
        let total_amount: f64 = lines.iter().map(|l| l.debit).sum();
        let total_vat_amount: f64 = lines.iter()
            .filter(|l| l.account_code.starts_with("453"))
            .map(|l| if l.debit != 0.0 { l.debit } else { l.credit })
            .sum();

        let counterpart = entry_counterpart_id.get();
        let counterpart_id_val = if counterpart.is_empty() {
            None
        } else {
            counterpart.parse::<i64>().ok()
        };

        let payload = NewJournalEntryPayload {
            entry_date: edate,
            document_date: {
                let dd = entry_document_date.get();
                if dd.is_empty() { None } else { Some(dd) }
            },
            description: desc,
            reference: None,
            status: "draft".to_string(),
            document_number: {
                let dn = entry_document_number.get();
                if dn.is_empty() { None } else { Some(dn) }
            },
            vat_period: if etype != "NON_VAT" {
                let vp = entry_vat_period.get();
                if vp.is_empty() { None } else { Some(vp) }
            } else {
                None
            },
            vat_purchase_operation: vat_purchase,
            vat_sales_operation: vat_sales,
            lines: lines_json,
            counterpart_id: counterpart_id_val,
            total_amount: Some(total_amount),
            total_vat_amount: if etype != "NON_VAT" { Some(total_vat_amount) } else { Some(0.0) },
            journal_type: if etype == "NON_VAT" { Some(document_type.get()) } else { None },
        };

        spawn_local(async move {
            let wrapped = serde_json::json!({ "journal_entry": &payload });
            match api_post(&format!("{}/journal_entries", company_api_url()), &wrapped).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Записът е създаден успешно".to_string());
                        show_new_entry_modal.set(false);
                        reset_form();
                        reload_entries();
                    } else {
                        let err = response.get("message")
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| {
                                response.get("errors")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter()
                                        .filter_map(|e| e.get("messages").and_then(|m| m.as_array()))
                                        .flatten()
                                        .filter_map(|m| m.as_str())
                                        .collect::<Vec<_>>()
                                        .join(", "))
                            })
                            .unwrap_or_else(|| "Неизвестна грешка".to_string());
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при създаване: {}", e));
                }
            }
            saving.set(false);
        });
    };

    // Calculate totals
    let total_debit = move || {
        entry_lines.get().iter().map(|l| l.debit).sum::<f64>()
    };

    let total_credit = move || {
        entry_lines.get().iter().map(|l| l.credit).sum::<f64>()
    };

    let is_balanced = move || {
        let d = total_debit();
        let c = total_credit();
        (d - c).abs() < 0.01 && d != 0.0
    };

    // Signal to store edit ID from URL (will be processed after load_entry_for_edit is defined)
    let url_edit_id = RwSignal::new(Option::<i64>::None);
    let url_view_id = RwSignal::new(Option::<i64>::None);

    // Pagination callbacks
    let on_page_change = Callback::new(move |page: i32| {
        current_page.set(page);
        reload_entries();
    });

    let on_per_page_change = Callback::new(move |pp: i32| {
        per_page.set(pp);
        current_page.set(1);
        reload_entries();
    });

    // Fetch entries when company changes (multi-company) and on mount
    Effect::new(move |_| {
        // Track active company — re-run when user switches firm
        let company_id = company_ctx.selected_company_id.get();
        if company_id.is_none() {
            // Wait until company context finishes loading / selects a firm
            return;
        }

        current_page.set(1);
        selected_ids.set(std::collections::HashSet::new());
        show_new_entry_modal.set(false);
        show_edit_modal.set(false);

        reload_entries();
        load_accounts();
        load_counterparts();
        load_currencies();

        // Set today's date using js_sys
        let date = js_sys::Date::new_0();
        let iso = date.to_iso_string().as_string().unwrap_or_default();
        if iso.len() >= 10 {
            entry_document_date.set(iso[..10].to_string());
            entry_date.set(iso[..10].to_string());
        }
        if iso.len() >= 7 {
            entry_vat_period.set(iso[..7].to_string());
        }

        // ?edit=ID opens the editor (scanned invoices). ?id= / ?view= opens preview (bank JE links).
        if let Some(window) = web_sys::window() {
            if let Ok(search) = window.location().search() {
                if let Some(edit_id) = query_param(&search, "edit").and_then(|s| s.parse::<i64>().ok()) {
                    url_edit_id.set(Some(edit_id));
                } else if let Some(view_id) = query_param(&search, "id")
                    .or_else(|| query_param(&search, "view"))
                    .and_then(|s| s.parse::<i64>().ok())
                {
                    url_view_id.set(Some(view_id));
                }
            }
        }
    });

    // Filtered entries (status/date are client-side; text search is server-side)
    let filtered_entries = move || {
        let status_filter = selected_status.get();
        let from = date_from.get();
        let to = date_to.get();

        entries.get()
            .into_iter()
            .filter(|e| {
                // Status filter
                if !status_filter.is_empty() && e.status != status_filter {
                    return false;
                }
                // Date from filter
                if !from.is_empty() {
                    let entry_date = get_date_part(&e.entry_date);
                    if entry_date < from {
                        return false;
                    }
                }
                // Date to filter
                if !to.is_empty() {
                    let entry_date = get_date_part(&e.entry_date);
                    if entry_date > to {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<_>>()
    };

    // Bulk delete selected draft entries
    let bulk_delete_entries = move || {
        let ids: Vec<i64> = selected_ids.get().iter()
            .copied()
            .filter(|id| {
                entries.get().iter().any(|e| e.id == *id && e.status == "draft")
            })
            .collect();
        if ids.is_empty() {
            return;
        }

        bulk_deleting.set(true);
        error_msg.set(String::new());
        success_msg.set(String::new());

        spawn_local(async move {
            match api_post(&format!("{}/journal_entries/bulk_delete", company_api_url()), &serde_json::json!({ "ids": ids })).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let count = response.get("deleted_count").and_then(|v| v.as_i64()).unwrap_or(ids.len() as i64);
                        success_msg.set(format!("Успешно изтрити {} записа", count));
                        selected_ids.set(std::collections::HashSet::new());
                        reload_entries();
                    } else {
                        let err = response.get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("Неизвестна грешка");
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при изтриване: {}", e));
                }
            }
            bulk_deleting.set(false);
        });
    };

    // Load entry details for view modal
    let load_entry_detail = move |entry_id: i64| {
        view_loading.set(true);
        view_entry_detail.set(None);
        spawn_local(async move {
            match api_get(&format!("{}/journal_entries/{}", company_api_url(), entry_id)).await {
                Ok(data) => {
                    if let Some(detail_data) = data.get("data") {
                        if let Ok(detail) = serde_json::from_value::<JournalEntryDetail>(detail_data.clone()) {
                            view_entry_detail.set(Some(detail));
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            view_loading.set(false);
        });
    };

    Effect::new(move |_| {
        if let Some(id) = url_view_id.get() {
            url_view_id.set(None);
            view_entry_id.set(id);
            show_view_modal.set(true);
            load_entry_detail(id);
        }
    });

    // Delete single entry
    let do_delete_single = move |_| {
        if let Some(entry) = single_delete_entry.get() {
            deleting.set(true);
            let entry_id = entry.id;
            spawn_local(async move {
                match api_delete(&format!("{}/journal_entries/{}", company_api_url(), entry_id)).await {
                    Ok(response) => {
                        if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            success_msg.set("Записът е изтрит успешно".to_string());
                            reload_entries();
                        } else {
                            let err = response.get("error")
                                .and_then(|e| e.as_str())
                                .unwrap_or("Неизвестна грешка");
                            error_msg.set(format!("Грешка: {}", err));
                        }
                    }
                    Err(e) => {
                        error_msg.set(format!("Грешка при изтриване: {}", e));
                    }
                }
                show_single_delete_modal.set(false);
                single_delete_entry.set(None);
                deleting.set(false);
            });
        }
    };

    // Post entry (change status to posted)
    let post_entry = move |entry_id: i64| {
        if view_posting.get() || view_paying.get() {
            return;
        }
        view_posting.set(true);
        error_msg.set(String::new());
        spawn_local(async move {
            match api_post(&format!("{}/journal_entries/{}/post", company_api_url(), entry_id), &serde_json::json!({})).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Записът е осчетоводен успешно".to_string());
                        reload_entries();
                        show_view_modal.set(false);
                    } else {
                        let err = response.get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("Неизвестна грешка");
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при осчетоводяване: {}", e));
                }
            }
            view_posting.set(false);
        });
    };

    // Separate cash/card payment JE — same logic as AI scanning
    let pay_entry = move |method: &'static str| {
        let entry_id = view_entry_id.get();
        if entry_id == 0 || view_posting.get() || view_paying.get() {
            return;
        }
        view_paying.set(true);
        error_msg.set(String::new());
        success_msg.set(String::new());
        spawn_local(async move {
            match api_post(
                &format!("{}/journal_entries/{}/create_payment", company_api_url(), entry_id),
                &serde_json::json!({ "method": method }),
            ).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let pay_id = data.get("data")
                            .and_then(|d| d.get("payment_journal_entry_id"))
                            .and_then(|v| v.as_i64());
                        let label = if method == "cash" { "в брой" } else { "с карта" };
                        success_msg.set(format!(
                            "Създадена е статия за плащане {}{}",
                            label,
                            pay_id.map(|id| format!(" #{}", id)).unwrap_or_default()
                        ));
                        view_entry_detail.update(|current| {
                            if let Some(d) = current {
                                d.payment_journal_entry_id = pay_id;
                                d.can_create_payment = Some(false);
                            }
                        });
                        reload_entries();
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при плащане");
                        error_msg.set(err.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            view_paying.set(false);
        });
    };

    // Load entry for editing
    let load_entry_for_edit = move |entry_id: i64| {
        edit_loading.set(true);
        edit_entry_id.set(entry_id);
        spawn_local(async move {
            match api_get(&format!("{}/journal_entries/{}", company_api_url(), entry_id)).await {
                Ok(data) => {
                    if let Some(detail_data) = data.get("data") {
                        // Parse entry details
                        let entry_date_val = detail_data.get("entry_date")
                            .and_then(|v| v.as_str())
                            .map(|s| get_date_part(s))
                            .unwrap_or_default();
                        let description_val = detail_data.get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let doc_number = detail_data.get("document_number")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let vat_period_raw = detail_data.get("vat_period")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        // Convert VAT period from DD.MM.YYYY to YYYY-MM if needed
                        let vat_period_val = if vat_period_raw.contains('.') {
                            // Format: DD.MM.YYYY -> YYYY-MM
                            let parts: Vec<&str> = vat_period_raw.split('.').collect();
                            if parts.len() >= 3 {
                                format!("{}-{:0>2}", parts[2], parts[1])
                            } else {
                                vat_period_raw.clone()
                            }
                        } else {
                            vat_period_raw.clone()
                        };

                        // Load VAT operation fields to determine entry type
                        let vat_purchase_op = detail_data.get("vat_purchase_operation")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let vat_sales_op = detail_data.get("vat_sales_operation")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        // Determine entry type based on VAT operations
                        // Only PURCHASE or SALE if there's an actual VAT operation set
                        let entry_type = if !vat_purchase_op.is_empty() {
                            "PURCHASE".to_string()
                        } else if !vat_sales_op.is_empty() {
                            "SALE".to_string()
                        } else {
                            // No VAT operation = NON_VAT (even if vat_period exists)
                            "NON_VAT".to_string()
                        };

                        // Parse VAT operation code and document type from operation string (format: "doctype|operation")
                        let (doc_type, vat_op) = if !vat_purchase_op.is_empty() {
                            let parts: Vec<&str> = vat_purchase_op.split('|').collect();
                            (parts.get(0).unwrap_or(&"01").to_string(), parts.get(1).unwrap_or(&"PURCHASE_CREDIT").to_string())
                        } else if !vat_sales_op.is_empty() {
                            let parts: Vec<&str> = vat_sales_op.split('|').collect();
                            (parts.get(0).unwrap_or(&"01").to_string(), parts.get(1).unwrap_or(&"DOMESTIC_SALE").to_string())
                        } else {
                            let stored = detail_data.get("journal_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("OTHER");
                            let code = if crate::constants::is_non_vat_doc(stored) { stored } else { "OTHER" };
                            (code.to_string(), "PURCHASE_CREDIT".to_string())
                        };

                        // Load counterpart_id
                        let counterpart_id_val = detail_data.get("counterpart_id")
                            .and_then(|v| v.as_i64())
                            .map(|id| id.to_string())
                            .unwrap_or_default();

                        let document_date_val = detail_data.get("document_date")
                            .and_then(|v| v.as_str())
                            .map(|s| get_date_part(s))
                            .unwrap_or_else(|| entry_date_val.clone());
                        edit_document_date.set(document_date_val);
                        edit_entry_date.set(entry_date_val.clone());
                        edit_description.set(description_val.clone());
                        edit_document_number.set(doc_number.clone());
                        edit_vat_period.set(vat_period_val.clone());
                        edit_entry_type.set(entry_type);
                        edit_document_type.set(doc_type);
                        edit_vat_operation.set(vat_op);
                        edit_counterpart_id.set(counterpart_id_val);

                        // Set edit_entry for the modal
                        let status_val = detail_data.get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("draft")
                            .to_string();
                        edit_entry.set(Some(JournalEntry {
                            id: entry_id,
                            entry_date: entry_date_val,
                            description: description_val,
                            reference: None,
                            status: status_val,
                            document_number: Some(doc_number),
                            total_amount: None,
                            total_vat_amount: None,
                            vat_period: Some(vat_period_val),
                            counterpart_name: None,
                        }));

                        // Parse lines - try to get account_code directly, fallback to lookup by account_id
                        if let Some(lines_arr) = detail_data.get("lines").and_then(|v| v.as_array()) {
                            let accs = accounts.get();
                            let parsed_lines: Vec<JournalLineInput> = lines_arr.iter()
                                .filter_map(|l| {
                                    // First try to get account_code directly from response
                                    let mut account_code = l.get("account_code")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_default();

                                    // If no account_code, try to look up by account_id
                                    if account_code.is_empty() {
                                        let account_id = l.get("account_id").and_then(|v| v.as_i64()).unwrap_or(0);
                                        account_code = accs.iter()
                                            .find(|a| a.id == account_id)
                                            .map(|a| a.code.clone())
                                            .unwrap_or_default();
                                    }

                                    let debit = l.get("debit_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let credit = l.get("credit_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let desc = l.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    Some(JournalLineInput {
                                        account_code,
                                        debit,
                                        credit,
                                        description: desc,
                                    })
                                })
                                .collect();
                            if !parsed_lines.is_empty() {
                                edit_lines.set(parsed_lines);
                            } else {
                                edit_lines.set(vec![
                                    JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
                                    JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
                                ]);
                            }
                        }
                        // Show the edit modal
                        show_edit_modal.set(true);
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            edit_loading.set(false);
        });
    };

    // Process URL edit ID after load_entry_for_edit is defined
    Effect::new(move |_| {
        if let Some(id) = url_edit_id.get() {
            // Only process once
            url_edit_id.set(None);
            load_entry_for_edit(id);
        }
    });

    // Reset edit form
    let reset_edit_form = move || {
        edit_entry_type.set("PURCHASE".to_string());
        edit_description.set(String::new());
        edit_document_number.set(String::new());
        edit_vat_operation.set("PURCHASE_CREDIT".to_string());
        edit_document_type.set("01".to_string());
        edit_counterpart_id.set(String::new());
        edit_currency.set("BGN".to_string());
        edit_lines.set(vec![
            JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
            JournalLineInput { account_code: String::new(), debit: 0.0, credit: 0.0, description: String::new() },
        ]);
    };

    // Calculate totals for edit form
    let edit_total_debit = move || {
        edit_lines.get().iter().map(|l| l.debit).sum::<f64>()
    };

    let edit_total_credit = move || {
        edit_lines.get().iter().map(|l| l.credit).sum::<f64>()
    };

    let edit_is_balanced = move || {
        let d = edit_total_debit();
        let c = edit_total_credit();
        (d - c).abs() < 0.01 && d != 0.0
    };

    // Save edited entry
    let save_edit = move |_| {
        if edit_saving.get_untracked() {
            return;
        }
        error_msg.set(String::new());

        let desc = edit_description.get().trim().to_string();
        if desc.is_empty() {
            error_msg.set("Моля, попълнете описание на операцията".to_string());
            return;
        }
        let edate = edit_entry_date.get();
        if edate.is_empty() {
            error_msg.set("Моля, попълнете счетоводна дата".to_string());
            return;
        }

        let lines = edit_lines.get();
        let total_debit: f64 = lines.iter().map(|l| l.debit).sum();
        let total_credit: f64 = lines.iter().map(|l| l.credit).sum();
        if (total_debit - total_credit).abs() >= 0.01 || total_debit == 0.0 {
            error_msg.set("Записът трябва да е балансиран (дебит = кредит, различни от 0)".to_string());
            return;
        }

        edit_saving.set(true);

        let lines_json = serialize_journal_lines(&lines, &accounts.get());

        let etype = edit_entry_type.get();
        let vat_purchase = if etype == "PURCHASE" {
            Some(format!("{}|{}", edit_document_type.get(), edit_vat_operation.get()))
        } else {
            None
        };
        let vat_sales = if etype == "SALE" {
            Some(format!("{}|{}", edit_document_type.get(), edit_vat_operation.get()))
        } else {
            None
        };

        // Calculate total_amount and total_vat_amount from lines
        let total_amount: f64 = lines.iter().map(|l| l.debit).sum();
        let total_vat_amount: f64 = lines.iter()
            .filter(|l| l.account_code.starts_with("453"))
            .map(|l| if l.debit != 0.0 { l.debit } else { l.credit })
            .sum();

        let counterpart = edit_counterpart_id.get();
        let counterpart_id_val = if counterpart.is_empty() {
            None
        } else {
            counterpart.parse::<i64>().ok()
        };

        let payload = NewJournalEntryPayload {
            entry_date: edate,
            document_date: {
                let dd = edit_document_date.get();
                if dd.is_empty() { None } else { Some(dd) }
            },
            description: desc,
            reference: None,
            status: "draft".to_string(),
            document_number: {
                let dn = edit_document_number.get();
                if dn.is_empty() { None } else { Some(dn) }
            },
            vat_period: if etype != "NON_VAT" {
                let vp = edit_vat_period.get();
                if vp.is_empty() { None } else { Some(vp) }
            } else {
                None
            },
            vat_purchase_operation: vat_purchase,
            vat_sales_operation: vat_sales,
            lines: lines_json,
            counterpart_id: counterpart_id_val,
            total_amount: Some(total_amount),
            total_vat_amount: if etype != "NON_VAT" { Some(total_vat_amount) } else { Some(0.0) },
            journal_type: if etype == "NON_VAT" { Some(edit_document_type.get()) } else { None },
        };

        let entry_id = edit_entry_id.get();
        spawn_local(async move {
            let wrapped = serde_json::json!({ "journal_entry": &payload });
            match api_put(&format!("{}/journal_entries/{}", company_api_url(), entry_id), &wrapped).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Записът е обновен успешно".to_string());
                        reload_entries();
                        show_edit_modal.set(false);
                        edit_entry.set(None);
                    } else {
                        let err = response.get("message")
                            .or_else(|| response.get("error"))
                            .and_then(|e| e.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| {
                                response.get("errors")
                                    .and_then(|e| e.as_array())
                                    .map(|arr| arr.iter()
                                        .filter_map(|e| e.get("messages").and_then(|m| m.as_array()))
                                        .flatten()
                                        .filter_map(|m| m.as_str())
                                        .collect::<Vec<_>>()
                                        .join(", "))
                            })
                            .unwrap_or_else(|| format!("Неизвестна грешка: {:?}", response));
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при обновяване: {}", e));
                }
            }
            edit_saving.set(false);
        });
    };

    // Status counts
    let status_counts = move || {
        let ents = entries.get();
        let mut counts = std::collections::HashMap::new();
        for e in &ents {
            *counts.entry(e.status.clone()).or_insert(0) += 1;
        }
        counts
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Счетоводни записи"</h1>
                <div style="display: flex; gap: 10px;">
                    <button
                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=move |_| show_new_entry_modal.set(true)
                    >
                        "+ Нов запис"
                    </button>
                </div>
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
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
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
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Search and filters
            <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                <div style="display: flex; gap: 15px; flex-wrap: wrap; align-items: center;">
                    // Search input
                    <div style="flex: 1; min-width: 200px;">
                        <input
                            type="text"
                            placeholder="Търсене по описание, контрагент или № документ..."
                            style="width: 100%; padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                            on:input=move |ev| {
                                search_query.set(event_target_value(&ev));
                                trigger_search();
                            }
                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                if ev.key() == "Enter" {
                                    current_page.set(1);
                                    search_seq.update(|n| *n += 1);
                                    reload_entries();
                                }
                            }
                            prop:value=move || search_query.get()
                        />
                    </div>

                    // Date from
                    <div style="display: flex; align-items: center; gap: 5px;">
                        <label style="color: #7f8c8d; font-size: 13px;">"От:"</label>
                        <input
                            type="date"
                            style="padding: 8px 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                            on:input=move |ev| {
                                date_from.set(event_target_value(&ev));
                                trigger_search();
                            }
                            prop:value=move || date_from.get()
                        />
                    </div>

                    // Date to
                    <div style="display: flex; align-items: center; gap: 5px;">
                        <label style="color: #7f8c8d; font-size: 13px;">"До:"</label>
                        <input
                            type="date"
                            style="padding: 8px 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                            on:input=move |ev| {
                                date_to.set(event_target_value(&ev));
                                trigger_search();
                            }
                            prop:value=move || date_to.get()
                        />
                    </div>

                    // Clear filters
                    <button
                        style="padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; background: #ecf0f1; color: #34495e; border: none;"
                        on:click=move |_| {
                            search_query.set(String::new());
                            date_from.set(String::new());
                            date_to.set(String::new());
                            selected_status.set(String::new());
                            current_page.set(1);
                            search_seq.update(|n| *n += 1);
                            reload_entries();
                        }
                    >
                        "Изчисти филтри"
                    </button>
                </div>

                // Status filter buttons
                <div style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 15px;">
                    <button
                        style=move || format!(
                            "padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; {}",
                            if selected_status.get().is_empty() {
                                "background: #34495e; color: white; border: none;"
                            } else {
                                "background: white; color: #34495e; border: 1px solid #ddd;"
                            }
                        )
                        on:click=move |_| selected_status.set(String::new())
                    >
                        "Всички ("{move || entries.get().len()}")"
                    </button>
                    {move || {
                        let counts = status_counts();
                        let statuses = vec![
                            ("draft", "Чернови", "#f39c12"),
                            ("posted", "Осчетоводени", "#27ae60"),
                            ("voided", "Анулирани", "#e74c3c"),
                        ];
                        statuses.into_iter().map(|(s, label, color)| {
                            let count = counts.get(s).copied().unwrap_or(0);
                            let s_clone = s.to_string();
                            let s_clone2 = s.to_string();
                            let color = color.to_string();
                            view! {
                                <button
                                    style=move || format!(
                                        "padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; {}",
                                        if selected_status.get() == s_clone {
                                            format!("background: {}; color: white; border: none;", color)
                                        } else {
                                            "background: white; color: #34495e; border: 1px solid #ddd;".to_string()
                                        }
                                    )
                                    on:click=move |_| selected_status.set(s_clone2.clone())
                                >
                                    {label}" ("{count}")"
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>

            // New entry modal
            {move || {
                if show_new_entry_modal.get() {
                    let accs = accounts.get();
                    let current_type = entry_type.get();
                    let current_vat_op = entry_vat_operation.get();
                    let current_doc_type = document_type.get();

                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: flex-start; justify-content: center; z-index: 1000; padding: 20px; overflow-y: auto;">
                            <div style="background: white; border-radius: 8px; padding: 30px; max-width: 1000px; width: 100%; margin: 20px 0;">
                                // Header with entry type toggle
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                    <div>
                                        <h3 style="margin: 0;">"Нов счетоводен запис"</h3>
                                        {move || {
                                            company_ctx.selected_company().map(|c| {
                                                view! {
                                                    <p style="margin: 4px 0 0 0; font-size: 13px; color: #7f8c8d;">
                                                        "Фирма: "
                                                        <strong style="color: #2c3e50;">{c.name}</strong>
                                                    </p>
                                                }
                                            })
                                        }}
                                    </div>
                                    <div style="display: flex; gap: 5px;">
                                        <button
                                            style=move || format!(
                                                "padding: 8px 16px; border-radius: 4px; cursor: pointer; border: none; {}",
                                                if entry_type.get() == "PURCHASE" { "background: #3498db; color: white;" } else { "background: #ecf0f1; color: #34495e;" }
                                            )
                                            on:click=move |_| {
                                                entry_type.set("PURCHASE".to_string());
                                                entry_vat_operation.set("PURCHASE_CREDIT".to_string());
                                                let dt = document_type.get_untracked();
                                                document_type.set(doc_type_for_switch("PURCHASE", &dt));
                                            }
                                        >
                                            "🛒 Покупки"
                                        </button>
                                        <button
                                            style=move || format!(
                                                "padding: 8px 16px; border-radius: 4px; cursor: pointer; border: none; {}",
                                                if entry_type.get() == "SALE" { "background: #27ae60; color: white;" } else { "background: #ecf0f1; color: #34495e;" }
                                            )
                                            on:click=move |_| {
                                                entry_type.set("SALE".to_string());
                                                entry_vat_operation.set("SALES_20".to_string());
                                                let dt = document_type.get_untracked();
                                                document_type.set(doc_type_for_switch("SALE", &dt));
                                            }
                                        >
                                            "💰 Продажби"
                                        </button>
                                        <button
                                            style=move || format!(
                                                "padding: 8px 16px; border-radius: 4px; cursor: pointer; border: none; {}",
                                                if entry_type.get() == "NON_VAT" { "background: #7f8c8d; color: white;" } else { "background: #ecf0f1; color: #34495e;" }
                                            )
                                            on:click=move |_| {
                                                entry_type.set("NON_VAT".to_string());
                                                let dt = document_type.get_untracked();
                                                document_type.set(doc_type_for_switch("NON_VAT", &dt));
                                            }
                                        >
                                            "📋 Без ДДС"
                                        </button>
                                    </div>
                                </div>

                                // Header fields row
                                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-bottom: 20px;">
                                    // Document type
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Тип документ"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| document_type.set(event_target_value(&ev))
                                        >
                                            {document_type_options(&current_type, &current_doc_type)}
                                        </select>
                                    </div>

                                    // Document number
                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"№ на документ"</label>
                                        <input
                                            type="text"
                                            placeholder="0000000001"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| entry_document_number.set(event_target_value(&ev))
                                            prop:value=move || entry_document_number.get()
                                        />
                                    </div>

                                    // Document date
                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Дата на документа"</label>
                                        <input
                                            type="date"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                entry_document_date.set(val.clone());
                                                entry_date.set(val.clone());
                                                if val.len() >= 7 {
                                                    entry_vat_period.set(val[..7].to_string());
                                                }
                                            }
                                            prop:value=move || entry_document_date.get()
                                        />
                                    </div>

                                    // Accounting date
                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Счетоводна дата"</label>
                                        <input
                                            type="date"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                entry_date.set(val.clone());
                                                if val.len() >= 7 {
                                                    entry_vat_period.set(val[..7].to_string());
                                                }
                                            }
                                            prop:value=move || entry_date.get()
                                        />
                                    </div>

                                    // VAT Period (only for VAT entries)
                                    {if current_type != "NON_VAT" {
                                        view! {
                                            <div>
                                                <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"ДДС период"</label>
                                                <input
                                                    type="month"
                                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                    on:input=move |ev| entry_vat_period.set(event_target_value(&ev))
                                                    prop:value=move || entry_vat_period.get()
                                                />
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>

                                // Counterpart and Currency row
                                <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 15px; margin-bottom: 20px;">
                                    // Counterpart with modal selection
                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Контрагент"</label>
                                        <div style="display: flex; gap: 5px;">
                                            <button
                                                type="button"
                                                style="flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 4px; background: white; text-align: left; cursor: pointer;"
                                                on:click=move |_| {
                                                    counterpart_search.set(String::new());
                                                    show_counterpart_modal.set(true);
                                                }
                                            >
                                                {move || {
                                                    let cp_id = entry_counterpart_id.get();
                                                    if cp_id.is_empty() {
                                                        "-- Изберете контрагент --".to_string()
                                                    } else {
                                                        counterparts.get().iter()
                                                            .find(|cp| cp.id.to_string() == cp_id)
                                                            .map(|cp| format!("{}{}", cp.name, cp.vat_number.as_ref().map(|v| format!(" ({})", v)).unwrap_or_default()))
                                                            .unwrap_or("-- Изберете контрагент --".to_string())
                                                    }
                                                }}
                                            </button>
                                            // Clear button
                                            {move || {
                                                if !entry_counterpart_id.get().is_empty() {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            style="padding: 10px 12px; border: 1px solid #e74c3c; border-radius: 4px; background: #e74c3c; color: white; cursor: pointer;"
                                                            on:click=move |_| entry_counterpart_id.set(String::new())
                                                        >
                                                            "✕"
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }
                                            }}
                                        </div>
                                    </div>

                                    // Currency
                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Валута"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| entry_currency.set(event_target_value(&ev))
                                        >
                                            {move || currencies.get().into_iter().map(|cur| {
                                                let selected = entry_currency.get() == cur.code;
                                                let display = format!("{} - {}", cur.code, cur.name);
                                                view! {
                                                    <option value={cur.code.clone()} selected=selected>{display}</option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                </div>

                                // Description (required)
                                <div style="margin-bottom: 20px;">
                                    <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">
                                        "Описание "
                                        <span style="color: #e74c3c;">"*"</span>
                                    </label>
                                    <input
                                        type="text"
                                        placeholder="Напр. Заплати 02-2026, Фактура за услуги..."
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                        on:input=move |ev| entry_description.set(event_target_value(&ev))
                                        prop:value=move || entry_description.get()
                                    />
                                </div>

                                // Inline form errors (visible inside modal)
                                {move || {
                                    let err = error_msg.get();
                                    if !err.is_empty() && show_new_entry_modal.get() {
                                        view! {
                                            <div style="background: #fdecea; color: #c0392b; padding: 12px 15px; border-radius: 6px; margin-bottom: 15px; border: 1px solid #f5c6cb; font-size: 14px;">
                                                {err}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }
                                }}

                                // VAT Operation panel (only for VAT entries)
                                {if current_type == "PURCHASE" {
                                    view! {
                                        <div style="background: #f8f9fa; border-radius: 8px; padding: 15px; margin-bottom: 20px;">
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 8px;">"ДДС операция (Покупки)"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:change=move |ev| entry_vat_operation.set(event_target_value(&ev))
                                            >
                                                <option value="PURCHASE_CREDIT" selected={current_vat_op == "PURCHASE_CREDIT"}>"С право на пълен данъчен кредит (Кол. 10)"</option>
                                                <option value="PURCHASE_NO_CREDIT" selected={current_vat_op == "PURCHASE_NO_CREDIT"}>"Без право на данъчен кредит (Кол. 9)"</option>
                                                <option value="PURCHASE_COL_12" selected={current_vat_op == "PURCHASE_COL_12"}>"С право на частичен ДК (Кол. 12)"</option>
                                                <option value="ICA" selected={current_vat_op == "ICA"}>"ВОП - Вътреобщностно придобиване (Кол. 10, 14)"</option>
                                                <option value="PURCHASE_COL_14" selected={current_vat_op == "PURCHASE_COL_14"}>"Покупки по чл. 82, ал. 2-5 (Кол. 14)"</option>
                                                <option value="PURCHASE_COL_15" selected={current_vat_op == "PURCHASE_COL_15"}>"Внос без право на ДК (Кол. 15)"</option>
                                            </select>
                                        </div>
                                    }.into_any()
                                } else if current_type == "SALE" {
                                    view! {
                                        <div style="background: #f8f9fa; border-radius: 8px; padding: 15px; margin-bottom: 20px;">
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 8px;">"ДДС операция (Продажби)"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:change=move |ev| entry_vat_operation.set(event_target_value(&ev))
                                            >
                                                <option value="SALES_20" selected={current_vat_op == "SALES_20"}>"Сделки със ставка 20% (Кол. 11)"</option>
                                                <option value="SALES_COL_17" selected={current_vat_op == "SALES_COL_17"}>"Сделки със ставка 9% (Кол. 17)"</option>
                                                <option value="EXPORT_ZERO" selected={current_vat_op == "EXPORT_ZERO"}>"Износ - ставка 0% (Кол. 19)"</option>
                                                <option value="ICS" selected={current_vat_op == "ICS"}>"ВОД - Вътреобщностна доставка (Кол. 20)"</option>
                                                <option value="SALES_COL_21" selected={current_vat_op == "SALES_COL_21"}>"Доставки на стоки с монтаж (Кол. 21)"</option>
                                                <option value="SALES_COL_22" selected={current_vat_op == "SALES_COL_22"}>"Доставки на услуги по чл. 21 (Кол. 22)"</option>
                                                <option value="SALES_COL_24" selected={current_vat_op == "SALES_COL_24"}>"Освободени доставки (Кол. 24)"</option>
                                            </select>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}

                                // Journal lines
                                <h4 style="margin-bottom: 10px;">"Редове"</h4>
                                <div style="border: 1px solid #ddd; border-radius: 8px; overflow: hidden;">
                                    // Header row
                                    <div style="display: grid; grid-template-columns: 2fr 1fr 1fr 40px; gap: 10px; padding: 10px; background: #34495e; color: white; font-size: 13px;">
                                        <div>"Сметка"</div>
                                        <div style="text-align: right;">"Дебит"</div>
                                        <div style="text-align: right;">"Кредит"</div>
                                        <div></div>
                                    </div>

                                    // Lines - use memo for count to avoid recreating DOM on value changes
                                    {move || {
                                        let count = entry_line_count.get();
                                        (0..count).map(|idx| {
                                            view! {
                                                <div style="display: grid; grid-template-columns: 2fr 1fr 1fr 40px; gap: 10px; padding: 10px; border-bottom: 1px solid #eee;">
                                                    // Account button - opens modal
                                                    <button
                                                        type="button"
                                                        style="padding: 8px; border: 1px solid #ddd; border-radius: 4px; background: white; text-align: left; cursor: pointer;"
                                                        on:click=move |_| {
                                                            account_modal_line_idx.set(idx);
                                                            account_search.set(String::new());
                                                            show_account_modal.set(true);
                                                        }
                                                    >
                                                        {move || {
                                                            entry_lines.with(|lines| {
                                                                let code = lines.get(idx).map(|l| l.account_code.clone()).unwrap_or_default();
                                                                if code.is_empty() {
                                                                    "-- Изберете сметка --".to_string()
                                                                } else {
                                                                    accounts.get().iter()
                                                                        .find(|a| a.code == code)
                                                                        .map(|a| format!("{} - {}", a.code, a.name))
                                                                        .unwrap_or(code)
                                                                }
                                                            })
                                                        }}
                                                    </button>

                                                    // Debit input
                                                    <input
                                                        type="text"
                                                        inputmode="decimal"
                                                        placeholder="0.00"
                                                        style="padding: 8px; border: 1px solid #ddd; border-radius: 4px; text-align: right;"
                                                        prop:value=move || {
                                                            entry_lines.with(|lines| {
                                                                lines.get(idx).map(|l| if l.debit != 0.0 { l.debit.to_string() } else { String::new() }).unwrap_or_default()
                                                            })
                                                        }
                                                        on:change=move |ev| {
                                                            let val: f64 = event_target_value(&ev).replace(',', ".").parse().unwrap_or(0.0);
                                                            entry_lines.update(|lines| {
                                                                if let Some(l) = lines.get_mut(idx) {
                                                                    l.debit = val;
                                                                    if val != 0.0 { l.credit = 0.0; }
                                                                }
                                                            });
                                                        }
                                                    />

                                                    // Credit input
                                                    <input
                                                        type="text"
                                                        inputmode="decimal"
                                                        placeholder="0.00"
                                                        style="padding: 8px; border: 1px solid #ddd; border-radius: 4px; text-align: right;"
                                                        prop:value=move || {
                                                            entry_lines.with(|lines| {
                                                                lines.get(idx).map(|l| if l.credit != 0.0 { l.credit.to_string() } else { String::new() }).unwrap_or_default()
                                                            })
                                                        }
                                                        on:change=move |ev| {
                                                            let val: f64 = event_target_value(&ev).replace(',', ".").parse().unwrap_or(0.0);
                                                            entry_lines.update(|lines| {
                                                                if let Some(l) = lines.get_mut(idx) {
                                                                    l.credit = val;
                                                                    if val != 0.0 { l.debit = 0.0; }
                                                                }
                                                            });
                                                        }
                                                    />

                                                    // Delete button
                                                    <button
                                                        style="background: #e74c3c; color: white; border: none; border-radius: 4px; cursor: pointer; padding: 6px;"
                                                        on:click=move |_| {
                                                            entry_lines.update(|lines| {
                                                                if lines.len() > 2 {
                                                                    lines.remove(idx);
                                                                }
                                                            });
                                                        }
                                                        disabled=move || entry_lines.with(|lines| lines.len() <= 2)
                                                    >
                                                        "×"
                                                    </button>
                                                </div>
                                            }
                                        }).collect_view()
                                    }}

                                    // Add line button
                                    <div style="padding: 10px;">
                                        <button
                                            style="background: #ecf0f1; color: #34495e; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| {
                                                entry_lines.update(|lines| {
                                                    lines.push(JournalLineInput {
                                                        account_code: String::new(),
                                                        debit: 0.0,
                                                        credit: 0.0,
                                                        description: String::new(),
                                                    });
                                                });
                                            }
                                        >
                                            "+ Добави ред"
                                        </button>
                                    </div>
                                </div>

                                // Totals
                                <div style="display: flex; justify-content: flex-end; gap: 30px; margin-top: 15px; padding: 15px; background: #f8f9fa; border-radius: 8px;">
                                    <div style=move || format!("color: {}", if is_balanced() { "#27ae60" } else { "#e74c3c" })>
                                        <strong>"Общо Дебит: "</strong>{move || format!("{:.2} €", total_debit())}
                                    </div>
                                    <div style=move || format!("color: {}", if is_balanced() { "#27ae60" } else { "#e74c3c" })>
                                        <strong>"Общо Кредит: "</strong>{move || format!("{:.2} €", total_credit())}
                                    </div>
                                    {move || {
                                        if !is_balanced() {
                                            let diff = total_debit() - total_credit();
                                            view! {
                                                <div style="color: #e74c3c;">
                                                    <strong>"Разлика: "</strong>{format!("{:.2} €", diff.abs())}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span style="color: #27ae60;">"✓ Балансирано"</span> }.into_any()
                                        }
                                    }}
                                </div>

                                // Action buttons
                                <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px;">
                                    <button
                                        style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| {
                                            show_new_entry_modal.set(false);
                                            reset_form();
                                        }
                                    >
                                        "Отказ"
                                    </button>
                                    <button
                                        style=move || format!(
                                            "color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: {}; background: {};",
                                            if is_balanced() && !saving.get() { "pointer" } else { "not-allowed" },
                                            if is_balanced() { "#27ae60" } else { "#95a5a6" }
                                        )
                                        disabled=move || !is_balanced() || saving.get()
                                        on:click=save_entry
                                    >
                                        {move || if saving.get() { "Запазване..." } else { "Запази като чернова" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // View entry modal
            {move || {
                if show_view_modal.get() {
                    let entry_id = view_entry_id.get();
                    let detail = view_entry_detail.get();
                    let is_loading = view_loading.get();

                    let entry_status = view_entry_detail.get()
                        .map(|d| d.status.clone())
                        .or_else(|| {
                            entries.get().iter()
                                .find(|e| e.id == entry_id)
                                .map(|e| e.status.clone())
                        })
                        .unwrap_or_default();
                    let can_post = entry_status == "draft";

                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; max-width: 700px; width: 90%; max-height: 80vh; overflow-y: auto;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                    <h3 style="margin: 0;">"Преглед на запис #"{entry_id}</h3>
                                    <button
                                        style="background: transparent; border: none; font-size: 24px; cursor: pointer; color: #7f8c8d;"
                                        on:click=move |_| {
                                            show_view_modal.set(false);
                                            view_entry_detail.set(None);
                                        }
                                    >"×"</button>
                                </div>

                                {if is_loading {
                                    view! {
                                        <div style="text-align: center; padding: 40px;">
                                            <p style="color: #7f8c8d;">"Зареждане..."</p>
                                        </div>
                                    }.into_any()
                                } else if let Some(d) = detail {
                                    let (status_label, status_color) = get_status_badge(&entry_status);
                                    view! {
                                        <div>
                                            // Header info
                                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 20px;">
                                                <div>
                                                    <label style="font-size: 12px; color: #7f8c8d;">"Дата на документа"</label>
                                                    <p style="margin: 5px 0; font-weight: 500;">{
                                                        d.document_date.as_ref().map(|dd| format_date(dd)).unwrap_or_else(|| format_date(&d.entry_date))
                                                    }</p>
                                                </div>
                                                <div>
                                                    <label style="font-size: 12px; color: #7f8c8d;">"Счетоводна дата"</label>
                                                    <p style="margin: 5px 0; font-weight: 500;">{format_date(&d.entry_date)}</p>
                                                </div>
                                                <div>
                                                    <label style="font-size: 12px; color: #7f8c8d;">"Статус"</label>
                                                    <p style="margin: 5px 0;">
                                                        <span style=format!(
                                                            "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                            status_color
                                                        )>
                                                            {status_label}
                                                        </span>
                                                    </p>
                                                </div>
                                            </div>

                                            <div style="margin-bottom: 20px;">
                                                <label style="font-size: 12px; color: #7f8c8d;">"Описание"</label>
                                                <p style="margin: 5px 0; font-weight: 500;">{d.description.clone()}</p>
                                            </div>

                                            {d.reference.as_ref().map(|r| view! {
                                                <div style="margin-bottom: 20px;">
                                                    <label style="font-size: 12px; color: #7f8c8d;">"Референция"</label>
                                                    <p style="margin: 5px 0; font-family: monospace;">{r.clone()}</p>
                                                </div>
                                            })}

                                            {
                                                let cps = counterparts.get();
                                                d.counterpart_id.map(|cid| {
                                                    let cp_name = cps.iter()
                                                        .find(|c| c.id == cid)
                                                        .map(|c| c.name.clone())
                                                        .unwrap_or_else(|| format!("ID: {}", cid));
                                                    view! {
                                                        <div style="margin-bottom: 20px;">
                                                            <label style="font-size: 12px; color: #7f8c8d;">"Контрагент"</label>
                                                            <p style="margin: 5px 0; font-weight: 500;">{cp_name}</p>
                                                        </div>
                                                    }
                                                })
                                            }

                                            // Lines table
                                            <h4 style="margin-bottom: 10px;">"Редове"</h4>
                                            <div style="border: 1px solid #ddd; border-radius: 8px; overflow: hidden;">
                                                <div style="display: grid; grid-template-columns: 1fr 1fr 1fr 2fr; gap: 10px; padding: 10px; background: #34495e; color: white; font-size: 13px;">
                                                    <div>"Сметка"</div>
                                                    <div style="text-align: right;">"Дебит"</div>
                                                    <div style="text-align: right;">"Кредит"</div>
                                                    <div>"Описание"</div>
                                                </div>
                                                {
                                                    let accs = accounts.get();
                                                    d.lines.iter().map(|line| {
                                                        // Try to get account code from line, fallback to lookup by id
                                                        let account_display = line.account_code.clone()
                                                            .filter(|c| !c.is_empty())
                                                            .or_else(|| {
                                                                accs.iter()
                                                                    .find(|a| a.id == line.account_id)
                                                                    .map(|a| a.code.clone())
                                                            })
                                                            .unwrap_or_else(|| line.account_id.to_string());

                                                        view! {
                                                            <div style="display: grid; grid-template-columns: 1fr 1fr 1fr 2fr; gap: 10px; padding: 10px; border-bottom: 1px solid #eee;">
                                                                <div style="font-family: monospace;">{account_display}</div>
                                                                <div style="text-align: right; font-family: monospace;">
                                                                    {if line.debit_amount != 0.0 { format!("{:.2}", line.debit_amount) } else { "-".to_string() }}
                                                                </div>
                                                                <div style="text-align: right; font-family: monospace;">
                                                                    {if line.credit_amount != 0.0 { format!("{:.2}", line.credit_amount) } else { "-".to_string() }}
                                                                </div>
                                                                <div style="color: #7f8c8d; font-size: 13px;">
                                                                    {line.description.clone().unwrap_or_else(|| "-".to_string())}
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect_view()
                                                }
                                            </div>

                                            // Totals
                                            <div style="display: flex; justify-content: flex-end; gap: 30px; margin-top: 15px; padding: 15px; background: #f8f9fa; border-radius: 8px;">
                                                <div>
                                                    <strong>"Общо Дебит: "</strong>{format!("{:.2} €", d.total_debit)}
                                                </div>
                                                <div>
                                                    <strong>"Общо Кредит: "</strong>{format!("{:.2} €", d.total_credit)}
                                                </div>
                                            </div>

                                            {move || {
                                                let err = error_msg.get();
                                                if err.is_empty() {
                                                    view! { <span></span> }.into_any()
                                                } else {
                                                    view! {
                                                        <div style="margin-top: 16px; background: #fdecea; color: #c0392b; padding: 10px 12px; border-radius: 4px; font-size: 0.9em;">
                                                            {err}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                            {move || {
                                                let msg = success_msg.get();
                                                if msg.is_empty() {
                                                    view! { <span></span> }.into_any()
                                                } else {
                                                    view! {
                                                        <div style="margin-top: 16px; background: #eafaf1; color: #1e8449; padding: 10px 12px; border-radius: 4px; font-size: 0.9em;">
                                                            {msg}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}

                                            // Action buttons — same cash/card payments as AI scanning
                                            <div style="display: flex; gap: 8px; justify-content: flex-end; margin-top: 20px; flex-wrap: wrap; align-items: center;">
                                                <button
                                                    style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                    on:click=move |_| {
                                                        show_view_modal.set(false);
                                                        view_entry_detail.set(None);
                                                    }
                                                >
                                                    "Затвори"
                                                </button>
                                                {move || {
                                                    let detail = view_entry_detail.get();
                                                    let busy = view_posting.get() || view_paying.get();
                                                    let is_payment = detail.as_ref().and_then(|d| d.is_payment_entry).unwrap_or(false);
                                                    let can_pay = detail.as_ref().and_then(|d| d.can_create_payment).unwrap_or(false) && !is_payment;
                                                    let pay_id = detail.as_ref().and_then(|d| d.payment_journal_entry_id);

                                                    if let Some(pay_id) = pay_id {
                                                        view! {
                                                            <a
                                                                href=format!("/journal-entries?edit={}", pay_id)
                                                                style="background: #8e44ad; color: white; border: none; padding: 10px 16px; border-radius: 4px; text-decoration: none; font-weight: bold;"
                                                                title="Отвори статията за плащане"
                                                            >
                                                                {format!("Платено #{}", pay_id)}
                                                            </a>
                                                        }.into_any()
                                                    } else if can_pay {
                                                        view! {
                                                            <>
                                                                <button
                                                                    style="background: #e67e22; color: white; border: none; padding: 10px 16px; border-radius: 4px; cursor: pointer; font-weight: bold;"
                                                                    on:click=move |_| pay_entry("cash")
                                                                    disabled=busy
                                                                    title="Нова статия: Дт доставчик/клиент, Кт каса (или обратно при продажба)"
                                                                >
                                                                    {if busy && view_paying.get() { "..." } else { "Плащане в брой" }}
                                                                </button>
                                                                <button
                                                                    style="background: #8e44ad; color: white; border: none; padding: 10px 16px; border-radius: 4px; cursor: pointer; font-weight: bold;"
                                                                    on:click=move |_| pay_entry("card")
                                                                    disabled=busy
                                                                    title="Нова статия през буферната сметка за карта"
                                                                >
                                                                    {if busy && view_paying.get() { "..." } else { "Плащане с карта" }}
                                                                </button>
                                                            </>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }
                                                }}
                                                {if can_post {
                                                    view! {
                                                        <button
                                                            style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                            on:click=move |_| post_entry(entry_id)
                                                            disabled=move || view_posting.get() || view_paying.get()
                                                        >
                                                            {move || if view_posting.get() { "..." } else { "✓ Осчетоводи" }}
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="text-align: center; padding: 40px;">
                                            <p style="color: #e74c3c;">"Грешка при зареждане на данните"</p>
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Single delete confirmation modal
            {move || {
                if show_single_delete_modal.get() {
                    if let Some(entry) = single_delete_entry.get() {
                        view! {
                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                                <div style="background: white; border-radius: 8px; padding: 30px; max-width: 500px; width: 90%;">
                                    <h3 style="margin-top: 0; color: #e74c3c;">"Потвърждение за изтриване"</h3>

                                    <div style="background: #fff3e0; border: 1px solid #ffb300; border-radius: 4px; padding: 15px; margin-bottom: 15px;">
                                        <p style="margin: 0;">"Сигурни ли сте, че искате да изтриете този запис?"</p>
                                        <ul style="margin: 10px 0 0 0; padding-left: 20px; color: #7f8c8d; font-size: 13px;">
                                            <li>"№ "{entry.id}</li>
                                            <li>"Дата: "{format_date(&entry.entry_date)}</li>
                                            <li>"Описание: "{entry.description.clone()}</li>
                                        </ul>
                                    </div>

                                    <p style="color: #e74c3c; font-size: 13px;">"Това действие не може да бъде отменено!"</p>

                                    <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                        <button
                                            style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| {
                                                show_single_delete_modal.set(false);
                                                single_delete_entry.set(None);
                                            }
                                        >
                                            "Отказ"
                                        </button>
                                        <button
                                            style="background: #e74c3c; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            disabled=move || deleting.get()
                                            on:click=do_delete_single
                                        >
                                            {move || if deleting.get() { "Изтриване..." } else { "Изтрий" }}
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

            // Edit entry modal (full form like new entry)
            {move || {
                if show_edit_modal.get() {
                    if let Some(entry) = edit_entry.get() {
                        let accs = accounts.get();
                        let current_type = edit_entry_type.get();
                        let current_vat_op = edit_vat_operation.get();
                        let current_doc_type = edit_document_type.get();
                        let is_loading = edit_loading.get();

                        view! {
                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: flex-start; justify-content: center; z-index: 1000; padding: 20px; overflow-y: auto;">
                                <div style="background: white; border-radius: 8px; padding: 30px; max-width: 1000px; width: 100%; margin: 20px 0;">
                                    // Header with entry type toggle
                                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                        <h3 style="margin: 0;">"Редактиране на запис #"{entry.id}</h3>
                                        <div style="display: flex; gap: 5px;">
                                            <button
                                                style=move || format!(
                                                    "padding: 8px 16px; border-radius: 4px; cursor: pointer; border: none; {}",
                                                    if edit_entry_type.get() == "PURCHASE" { "background: #3498db; color: white;" } else { "background: #ecf0f1; color: #34495e;" }
                                                )
                                                on:click=move |_| {
                                                    edit_entry_type.set("PURCHASE".to_string());
                                                    edit_vat_operation.set("PURCHASE_CREDIT".to_string());
                                                    let dt = edit_document_type.get_untracked();
                                                    edit_document_type.set(doc_type_for_switch("PURCHASE", &dt));
                                                }
                                            >
                                                "🛒 Покупки"
                                            </button>
                                            <button
                                                style=move || format!(
                                                    "padding: 8px 16px; border-radius: 4px; cursor: pointer; border: none; {}",
                                                    if edit_entry_type.get() == "SALE" { "background: #27ae60; color: white;" } else { "background: #ecf0f1; color: #34495e;" }
                                                )
                                                on:click=move |_| {
                                                    edit_entry_type.set("SALE".to_string());
                                                    edit_vat_operation.set("SALES_20".to_string());
                                                    let dt = edit_document_type.get_untracked();
                                                    edit_document_type.set(doc_type_for_switch("SALE", &dt));
                                                }
                                            >
                                                "💰 Продажби"
                                            </button>
                                            <button
                                                style=move || format!(
                                                    "padding: 8px 16px; border-radius: 4px; cursor: pointer; border: none; {}",
                                                    if edit_entry_type.get() == "NON_VAT" { "background: #7f8c8d; color: white;" } else { "background: #ecf0f1; color: #34495e;" }
                                                )
                                                on:click=move |_| {
                                                    edit_entry_type.set("NON_VAT".to_string());
                                                    let dt = edit_document_type.get_untracked();
                                                    edit_document_type.set(doc_type_for_switch("NON_VAT", &dt));
                                                }
                                            >
                                                "📋 Без ДДС"
                                            </button>
                                        </div>
                                    </div>

                                    {if is_loading {
                                        view! {
                                            <div style="text-align: center; padding: 40px;">
                                                <p style="color: #7f8c8d;">"Зареждане..."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div>
                                                // Header fields row
                                                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-bottom: 20px;">
                                                    // Document type
                                                    <div style="grid-column: span 2;">
                                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Тип документ"</label>
                                                        <select
                                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                            on:change=move |ev| edit_document_type.set(event_target_value(&ev))
                                                        >
                                                            {document_type_options(&current_type, &current_doc_type)}
                                                        </select>
                                                    </div>

                                                    // Document number
                                                    <div>
                                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"№ на документ"</label>
                                                        <input
                                                            type="text"
                                                            placeholder="0000000001"
                                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                            on:input=move |ev| edit_document_number.set(event_target_value(&ev))
                                                            prop:value=move || edit_document_number.get()
                                                        />
                                                    </div>

                                                    // Document date
                                                    <div>
                                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Дата на документа"</label>
                                                        <input
                                                            type="date"
                                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                            on:input=move |ev| {
                                                                let val = event_target_value(&ev);
                                                                edit_document_date.set(val.clone());
                                                                edit_entry_date.set(val.clone());
                                                                if val.len() >= 7 {
                                                                    edit_vat_period.set(val[..7].to_string());
                                                                }
                                                            }
                                                            prop:value=move || edit_document_date.get()
                                                        />
                                                    </div>

                                                    // Accounting date
                                                    <div>
                                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Счетоводна дата"</label>
                                                        <input
                                                            type="date"
                                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                            on:input=move |ev| {
                                                                let val = event_target_value(&ev);
                                                                edit_entry_date.set(val.clone());
                                                                if val.len() >= 7 {
                                                                    edit_vat_period.set(val[..7].to_string());
                                                                }
                                                            }
                                                            prop:value=move || edit_entry_date.get()
                                                        />
                                                    </div>

                                                    // VAT Period (only for VAT entries)
                                                    {if current_type != "NON_VAT" {
                                                        view! {
                                                            <div>
                                                                <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"ДДС период"</label>
                                                                <input
                                                                    type="month"
                                                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                                    on:input=move |ev| edit_vat_period.set(event_target_value(&ev))
                                                                    prop:value=move || edit_vat_period.get()
                                                                />
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                </div>

                                                // Counterpart and Currency row
                                                <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 15px; margin-bottom: 20px;">
                                                    // Counterpart with modal selection
                                                    <div>
                                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Контрагент"</label>
                                                        <div style="display: flex; gap: 5px;">
                                                            <button
                                                                type="button"
                                                                style="flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 4px; background: white; text-align: left; cursor: pointer;"
                                                                on:click=move |_| {
                                                                    edit_counterpart_search.set(String::new());
                                                                    show_edit_counterpart_modal.set(true);
                                                                }
                                                            >
                                                                {move || {
                                                                    let cp_id = edit_counterpart_id.get();
                                                                    if cp_id.is_empty() {
                                                                        "-- Изберете контрагент --".to_string()
                                                                    } else {
                                                                        counterparts.get().iter()
                                                                            .find(|cp| cp.id.to_string() == cp_id)
                                                                            .map(|cp| format!("{}{}", cp.name, cp.vat_number.as_ref().map(|v| format!(" ({})", v)).unwrap_or_default()))
                                                                            .unwrap_or("-- Изберете контрагент --".to_string())
                                                                    }
                                                                }}
                                                            </button>
                                                            // Clear button
                                                            {move || {
                                                                if !edit_counterpart_id.get().is_empty() {
                                                                    view! {
                                                                        <button
                                                                            type="button"
                                                                            style="padding: 10px 12px; border: 1px solid #e74c3c; border-radius: 4px; background: #e74c3c; color: white; cursor: pointer;"
                                                                            on:click=move |_| edit_counterpart_id.set(String::new())
                                                                        >
                                                                            "✕"
                                                                        </button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }
                                                            }}
                                                        </div>
                                                    </div>

                                                    // Currency
                                                    <div>
                                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Валута"</label>
                                                        <select
                                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                            on:change=move |ev| edit_currency.set(event_target_value(&ev))
                                                        >
                                                            {move || currencies.get().into_iter().map(|cur| {
                                                                let selected = edit_currency.get() == cur.code;
                                                                let display = format!("{} - {}", cur.code, cur.name);
                                                                view! {
                                                                    <option value={cur.code.clone()} selected=selected>{display}</option>
                                                                }
                                                            }).collect_view()}
                                                        </select>
                                                    </div>
                                                </div>

                                                // Description
                                                <div style="margin-bottom: 20px;">
                                                    <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">
                                                        "Описание "
                                                        <span style="color: #e74c3c;">"*"</span>
                                                    </label>
                                                    <input
                                                        type="text"
                                                        placeholder="Напр. Заплати 02-2026, Фактура за услуги..."
                                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                        on:input=move |ev| edit_description.set(event_target_value(&ev))
                                                        prop:value=move || edit_description.get()
                                                    />
                                                </div>

                                                // Inline form errors (visible inside edit modal)
                                                {move || {
                                                    let err = error_msg.get();
                                                    if !err.is_empty() && show_edit_modal.get() {
                                                        view! {
                                                            <div style="background: #fdecea; color: #c0392b; padding: 12px 15px; border-radius: 6px; margin-bottom: 15px; border: 1px solid #f5c6cb; font-size: 14px;">
                                                                {err}
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }
                                                }}

                                                // VAT Operation panel (only for VAT entries)
                                                {if current_type == "PURCHASE" {
                                                    view! {
                                                        <div style="background: #f8f9fa; border-radius: 8px; padding: 15px; margin-bottom: 20px;">
                                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 8px;">"ДДС операция (Покупки)"</label>
                                                            <select
                                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                                on:change=move |ev| edit_vat_operation.set(event_target_value(&ev))
                                                            >
                                                                <option value="PURCHASE_CREDIT" selected={current_vat_op == "PURCHASE_CREDIT"}>"С право на пълен данъчен кредит (Кол. 10)"</option>
                                                                <option value="PURCHASE_NO_CREDIT" selected={current_vat_op == "PURCHASE_NO_CREDIT"}>"Без право на данъчен кредит (Кол. 9)"</option>
                                                                <option value="PURCHASE_COL_12" selected={current_vat_op == "PURCHASE_COL_12"}>"С право на частичен ДК (Кол. 12)"</option>
                                                                <option value="ICA" selected={current_vat_op == "ICA"}>"ВОП - Вътреобщностно придобиване (Кол. 10, 14)"</option>
                                                                <option value="PURCHASE_COL_14" selected={current_vat_op == "PURCHASE_COL_14"}>"Покупки по чл. 82, ал. 2-5 (Кол. 14)"</option>
                                                                <option value="PURCHASE_COL_15" selected={current_vat_op == "PURCHASE_COL_15"}>"Внос без право на ДК (Кол. 15)"</option>
                                                            </select>
                                                        </div>
                                                    }.into_any()
                                                } else if current_type == "SALE" {
                                                    view! {
                                                        <div style="background: #f8f9fa; border-radius: 8px; padding: 15px; margin-bottom: 20px;">
                                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 8px;">"ДДС операция (Продажби)"</label>
                                                            <select
                                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                                on:change=move |ev| edit_vat_operation.set(event_target_value(&ev))
                                                            >
                                                                <option value="SALES_20" selected={current_vat_op == "SALES_20"}>"Сделки със ставка 20% (Кол. 11)"</option>
                                                                <option value="SALES_COL_17" selected={current_vat_op == "SALES_COL_17"}>"Сделки със ставка 9% (Кол. 17)"</option>
                                                                <option value="EXPORT_ZERO" selected={current_vat_op == "EXPORT_ZERO"}>"Износ - ставка 0% (Кол. 19)"</option>
                                                                <option value="ICS" selected={current_vat_op == "ICS"}>"ВОД - Вътреобщностна доставка (Кол. 20)"</option>
                                                                <option value="SALES_COL_21" selected={current_vat_op == "SALES_COL_21"}>"Доставки на стоки с монтаж (Кол. 21)"</option>
                                                                <option value="SALES_COL_22" selected={current_vat_op == "SALES_COL_22"}>"Доставки на услуги по чл. 21 (Кол. 22)"</option>
                                                                <option value="SALES_COL_24" selected={current_vat_op == "SALES_COL_24"}>"Освободени доставки (Кол. 24)"</option>
                                                            </select>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}

                                                // Journal lines
                                                <h4 style="margin-bottom: 10px;">"Редове"</h4>
                                                <div style="border: 1px solid #ddd; border-radius: 8px; overflow: hidden;">
                                                    // Header row
                                                    <div style="display: grid; grid-template-columns: 2fr 1fr 1fr 40px; gap: 10px; padding: 10px; background: #34495e; color: white; font-size: 13px;">
                                                        <div>"Сметка"</div>
                                                        <div style="text-align: right;">"Дебит"</div>
                                                        <div style="text-align: right;">"Кредит"</div>
                                                        <div></div>
                                                    </div>

                                                    // Lines - use memo for count to avoid recreating DOM on value changes
                                                    {move || {
                                                        let count = edit_line_count.get();
                                                        (0..count).map(|idx| {
                                                            view! {
                                                                <div style="display: grid; grid-template-columns: 2fr 1fr 1fr 40px; gap: 10px; padding: 10px; border-bottom: 1px solid #eee;">
                                                                    // Account button - opens modal
                                                                    <button
                                                                        type="button"
                                                                        style="padding: 8px; border: 1px solid #ddd; border-radius: 4px; background: white; text-align: left; cursor: pointer;"
                                                                        on:click=move |_| {
                                                                            edit_account_modal_line_idx.set(idx);
                                                                            edit_account_search.set(String::new());
                                                                            show_edit_account_modal.set(true);
                                                                        }
                                                                    >
                                                                        {move || {
                                                                            edit_lines.with(|lines| {
                                                                                let code = lines.get(idx).map(|l| l.account_code.clone()).unwrap_or_default();
                                                                                if code.is_empty() {
                                                                                    "-- Изберете сметка --".to_string()
                                                                                } else {
                                                                                    accounts.get().iter()
                                                                                        .find(|a| a.code == code)
                                                                                        .map(|a| format!("{} - {}", a.code, a.name))
                                                                                        .unwrap_or(code)
                                                                                }
                                                                            })
                                                                        }}
                                                                    </button>

                                                                    // Debit input
                                                                    <input
                                                                        type="text"
                                                                        inputmode="decimal"
                                                                        placeholder="0.00"
                                                                        style="padding: 8px; border: 1px solid #ddd; border-radius: 4px; text-align: right;"
                                                                        prop:value=move || {
                                                                            edit_lines.with(|lines| {
                                                                                lines.get(idx).map(|l| if l.debit != 0.0 { l.debit.to_string() } else { String::new() }).unwrap_or_default()
                                                                            })
                                                                        }
                                                                        on:change=move |ev| {
                                                                            let val: f64 = event_target_value(&ev).replace(',', ".").parse().unwrap_or(0.0);
                                                                            edit_lines.update(|lines| {
                                                                                if let Some(l) = lines.get_mut(idx) {
                                                                                    l.debit = val;
                                                                                    if val != 0.0 { l.credit = 0.0; }
                                                                                }
                                                                            });
                                                                        }
                                                                    />

                                                                    // Credit input
                                                                    <input
                                                                        type="text"
                                                                        inputmode="decimal"
                                                                        placeholder="0.00"
                                                                        style="padding: 8px; border: 1px solid #ddd; border-radius: 4px; text-align: right;"
                                                                        prop:value=move || {
                                                                            edit_lines.with(|lines| {
                                                                                lines.get(idx).map(|l| if l.credit != 0.0 { l.credit.to_string() } else { String::new() }).unwrap_or_default()
                                                                            })
                                                                        }
                                                                        on:change=move |ev| {
                                                                            let val: f64 = event_target_value(&ev).replace(',', ".").parse().unwrap_or(0.0);
                                                                            edit_lines.update(|lines| {
                                                                                if let Some(l) = lines.get_mut(idx) {
                                                                                    l.credit = val;
                                                                                    if val != 0.0 { l.debit = 0.0; }
                                                                                }
                                                                            });
                                                                        }
                                                                    />

                                                                    // Delete button
                                                                    <button
                                                                        style="background: #e74c3c; color: white; border: none; border-radius: 4px; cursor: pointer; padding: 6px;"
                                                                        on:click=move |_| {
                                                                            edit_lines.update(|lines| {
                                                                                if lines.len() > 2 {
                                                                                    lines.remove(idx);
                                                                                }
                                                                            });
                                                                        }
                                                                        disabled=move || edit_lines.with(|lines| lines.len() <= 2)
                                                                    >
                                                                        "×"
                                                                    </button>
                                                                </div>
                                                            }
                                                        }).collect_view()
                                                    }}

                                                    // Add line button
                                                    <div style="padding: 10px;">
                                                        <button
                                                            style="background: #ecf0f1; color: #34495e; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                                            on:click=move |_| {
                                                                edit_lines.update(|lines| {
                                                                    lines.push(JournalLineInput {
                                                                        account_code: String::new(),
                                                                        debit: 0.0,
                                                                        credit: 0.0,
                                                                        description: String::new(),
                                                                    });
                                                                });
                                                            }
                                                        >
                                                            "+ Добави ред"
                                                        </button>
                                                    </div>
                                                </div>

                                                // Totals
                                                <div style="display: flex; justify-content: flex-end; gap: 30px; margin-top: 15px; padding: 15px; background: #f8f9fa; border-radius: 8px;">
                                                    <div style=move || format!("color: {}", if edit_is_balanced() { "#27ae60" } else { "#e74c3c" })>
                                                        <strong>"Общо Дебит: "</strong>{move || format!("{:.2} €", edit_total_debit())}
                                                    </div>
                                                    <div style=move || format!("color: {}", if edit_is_balanced() { "#27ae60" } else { "#e74c3c" })>
                                                        <strong>"Общо Кредит: "</strong>{move || format!("{:.2} €", edit_total_credit())}
                                                    </div>
                                                    {move || {
                                                        if !edit_is_balanced() {
                                                            let diff = edit_total_debit() - edit_total_credit();
                                                            view! {
                                                                <div style="color: #e74c3c;">
                                                                    <strong>"Разлика: "</strong>{format!("{:.2} €", diff.abs())}
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span style="color: #27ae60;">"✓ Балансирано"</span> }.into_any()
                                                        }
                                                    }}
                                                </div>

                                                // Action buttons
                                                <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px;">
                                                    <button
                                                        style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                                        on:click=move |_| {
                                                            show_edit_modal.set(false);
                                                            edit_entry.set(None);
                                                            reset_edit_form();
                                                        }
                                                    >
                                                        "Отказ"
                                                    </button>
                                                    <button
                                                        style=move || format!(
                                                            "color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: {}; background: {};",
                                                            if edit_is_balanced() && !edit_saving.get() { "pointer" } else { "not-allowed" },
                                                            if edit_is_balanced() { "#3498db" } else { "#95a5a6" }
                                                        )
                                                        disabled=move || !edit_is_balanced() || edit_saving.get()
                                                        on:click=save_edit
                                                    >
                                                        {move || if edit_saving.get() { "Запазване..." } else { "Запази" }}
                                                    </button>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }}
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

            // Loading state
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

            // Entries list
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_any();
                }

                let filtered = filtered_entries();
                let total_filtered = filtered.len();

                if total_filtered == 0 {
                    return view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 1.1em;">
                                {if entries.get().is_empty() {
                                    "Няма счетоводни записи"
                                } else {
                                    "Няма намерени записи по зададените критерии"
                                }}
                            </p>
                        </div>
                    }.into_any();
                }

                view! {
                    <div>
                        <p style="color: #7f8c8d; margin-top: 15px; font-size: 14px;">
                            "Показани: "{total_filtered}" / "{entries.get().len()}" записа"
                        </p>

                        // Bulk actions bar
                        {move || {
                            let selected_count = selected_ids.get().len();
                            let draft_entries: Vec<_> = entries.get().into_iter()
                                .filter(|e| e.status == "draft" && selected_ids.get().contains(&e.id))
                                .collect();
                            let posted_entries: Vec<_> = entries.get().into_iter()
                                .filter(|e| e.status == "posted" && selected_ids.get().contains(&e.id))
                                .collect();
                            let can_post = !draft_entries.is_empty();
                            let can_unpost = !posted_entries.is_empty();
                            let can_delete = !draft_entries.is_empty();

                            if selected_count > 0 {
                                view! {
                                    <div style="background: #3498db; color: white; padding: 12px 20px; border-radius: 8px; margin-top: 15px; display: flex; justify-content: space-between; align-items: center;">
                                        <span style="font-weight: 500;">
                                            "Избрани: "{selected_count}" записа"
                                            {if draft_entries.len() > 0 && draft_entries.len() < selected_count {
                                                format!(" ({} чернови)", draft_entries.len())
                                            } else {
                                                String::new()
                                            }}
                                            {if posted_entries.len() > 0 && posted_entries.len() < selected_count {
                                                format!(" ({} осчетоводени)", posted_entries.len())
                                            } else {
                                                String::new()
                                            }}
                                        </span>
                                        <div style="display: flex; gap: 10px;">
                                            <button
                                                style=move || format!(
                                                    "padding: 8px 16px; border-radius: 4px; border: none; font-weight: 500; {}",
                                                    if can_post && !bulk_posting.get() {
                                                        "background: #27ae60; color: white; cursor: pointer;"
                                                    } else {
                                                        "background: #95a5a6; color: #ecf0f1; cursor: not-allowed;"
                                                    }
                                                )
                                                disabled=move || !can_post || bulk_posting.get()
                                                on:click=move |_| bulk_post_entries()
                                            >
                                                {move || if bulk_posting.get() { "Осчетоводяване..." } else { "Осчетоводи избраните" }}
                                            </button>
                                            <button
                                                style=move || format!(
                                                    "padding: 8px 16px; border-radius: 4px; border: none; font-weight: 500; {}",
                                                    if can_unpost && !bulk_unposting.get() {
                                                        "background: #e67e22; color: white; cursor: pointer;"
                                                    } else {
                                                        "background: #95a5a6; color: #ecf0f1; cursor: not-allowed;"
                                                    }
                                                )
                                                disabled=move || !can_unpost || bulk_unposting.get()
                                                on:click=move |_| bulk_unpost_entries()
                                            >
                                                {move || if bulk_unposting.get() { "Връщане..." } else { "Върни в чернова" }}
                                            </button>
                                            <button
                                                style=move || format!(
                                                    "padding: 8px 16px; border-radius: 4px; border: none; font-weight: 500; {}",
                                                    if can_delete && !bulk_deleting.get() {
                                                        "background: #e74c3c; color: white; cursor: pointer;"
                                                    } else {
                                                        "background: #95a5a6; color: #ecf0f1; cursor: not-allowed;"
                                                    }
                                                )
                                                disabled=move || !can_delete || bulk_deleting.get()
                                                on:click=move |_| bulk_delete_entries()
                                            >
                                                {move || if bulk_deleting.get() { "Изтриване..." } else { "Изтрий избраните" }}
                                            </button>
                                            <button
                                                style="background: transparent; color: white; border: 1px solid white; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                                on:click=move |_| selected_ids.set(std::collections::HashSet::new())
                                            >
                                                "Изчисти селекцията"
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}

                        <div style="background: white; border-radius: 8px; margin-top: 15px; overflow: hidden;">
                            <table style="width: 100%; border-collapse: collapse;">
                                <thead style="background: #34495e; color: white;">
                                    <tr>
                                        <th style="padding: 12px; text-align: center; width: 40px;">
                                            <input
                                                type="checkbox"
                                                style="width: 18px; height: 18px; cursor: pointer;"
                                                prop:checked=move || {
                                                    let all_selectable_ids: std::collections::HashSet<_> = entries.get()
                                                        .iter()
                                                        .filter(|e| e.status == "draft" || e.status == "posted")
                                                        .map(|e| e.id)
                                                        .collect();
                                                    !all_selectable_ids.is_empty() && all_selectable_ids.iter().all(|id| selected_ids.get().contains(id))
                                                }
                                                on:change=move |_| {
                                                    let all_selectable_ids: std::collections::HashSet<_> = entries.get()
                                                        .iter()
                                                        .filter(|e| e.status == "draft" || e.status == "posted")
                                                        .map(|e| e.id)
                                                        .collect();
                                                    let current = selected_ids.get();
                                                    if all_selectable_ids.iter().all(|id| current.contains(id)) {
                                                        // Deselect all
                                                        selected_ids.set(std::collections::HashSet::new());
                                                    } else {
                                                        // Select all
                                                        selected_ids.set(all_selectable_ids);
                                                    }
                                                }
                                            />
                                        </th>
                                        <th style="padding: 12px; text-align: left; width: 60px;">"№"</th>
                                        <th style="padding: 12px; text-align: left; width: 100px;">"Дата"</th>
                                        <th style="padding: 12px; text-align: left;">"Описание"</th>
                                        <th style="padding: 12px; text-align: left; width: 150px;">"Референция"</th>
                                        <th style="padding: 12px; text-align: right; width: 120px;">"Сума"</th>
                                        <th style="padding: 12px; text-align: center; width: 120px;">"Статус"</th>
                                        <th style="padding: 12px; text-align: center; width: 140px;">"Действия"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {filtered.into_iter().map(|entry| {
                                        let (status_label, status_color) = get_status_badge(&entry.status);
                                        let amount = entry.total_amount.unwrap_or(0.0);
                                        let entry_id_for_checkbox = entry.id;
                                        let can_select = entry.status == "draft" || entry.status == "posted";

                                        view! {
                                            <tr style="border-bottom: 1px solid #ecf0f1;">
                                                <td style="padding: 12px; text-align: center;">
                                                    {if can_select {
                                                        view! {
                                                            <input
                                                                type="checkbox"
                                                                style="width: 18px; height: 18px; cursor: pointer;"
                                                                prop:checked=move || selected_ids.get().contains(&entry_id_for_checkbox)
                                                                on:change=move |_| {
                                                                    let mut current = selected_ids.get();
                                                                    if current.contains(&entry_id_for_checkbox) {
                                                                        current.remove(&entry_id_for_checkbox);
                                                                    } else {
                                                                        current.insert(entry_id_for_checkbox);
                                                                    }
                                                                    selected_ids.set(current);
                                                                }
                                                            />
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span style="color: #bdc3c7;">"-"</span>
                                                        }.into_any()
                                                    }}
                                                </td>
                                                <td style="padding: 12px; color: #7f8c8d;">
                                                    {entry.id}
                                                </td>
                                                <td style="padding: 12px; font-family: monospace;">
                                                    {format_date(&entry.entry_date)}
                                                </td>
                                                <td style="padding: 12px;">
                                                    <div style="font-weight: 500;">{entry.description.clone()}</div>
                                                    {entry.document_number.as_ref().map(|doc| {
                                                        view! {
                                                            <div style="color: #7f8c8d; font-size: 12px;">
                                                                "Док: "{doc.clone()}
                                                            </div>
                                                        }
                                                    })}
                                                    {entry.counterpart_name.as_ref().map(|name| {
                                                        view! {
                                                            <div style="color: #95a5a6; font-size: 12px;">
                                                                {name.clone()}
                                                            </div>
                                                        }
                                                    })}
                                                </td>
                                                <td style="padding: 12px; font-family: monospace; color: #7f8c8d; font-size: 13px;">
                                                    {entry.reference.clone().unwrap_or_else(|| "-".to_string())}
                                                </td>
                                                <td style="padding: 12px; text-align: right; font-family: monospace;">
                                                    {if amount != 0.0 {
                                                        format!("{:.2} €", amount)
                                                    } else {
                                                        "-".to_string()
                                                    }}
                                                </td>
                                                <td style="padding: 12px; text-align: center;">
                                                    <span style=format!(
                                                        "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                        status_color
                                                    )>
                                                        {status_label}
                                                    </span>
                                                </td>
                                                <td style="padding: 12px; text-align: center;">
                                                    {
                                                        let entry_id = entry.id;
                                                        let entry_for_delete = entry.clone();
                                                        let entry_for_edit = entry.clone();
                                                        let can_modify = entry.status == "draft";
                                                        view! {
                                                            <>
                                                                <button
                                                                    style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-right: 5px; font-size: 12px;"
                                                                    title="Преглед"
                                                                    on:click=move |_| {
                                                                        error_msg.set(String::new());
                                                                        success_msg.set(String::new());
                                                                        view_entry_id.set(entry_id);
                                                                        show_view_modal.set(true);
                                                                        load_entry_detail(entry_id);
                                                                    }
                                                                >
                                                                    "👁"
                                                                </button>
                                                                <button
                                                                    style=move || format!(
                                                                        "border: none; padding: 6px 12px; border-radius: 4px; margin-right: 5px; font-size: 12px; {}",
                                                                        if can_modify { "background: #f39c12; color: white; cursor: pointer;" } else { "background: #bdc3c7; color: #7f8c8d; cursor: not-allowed;" }
                                                                    )
                                                                    disabled=move || !can_modify
                                                                    on:click={
                                                                        let e = entry_for_edit.clone();
                                                                        move |_| {
                                                                            if can_modify {
                                                                                edit_entry.set(Some(e.clone()));
                                                                                edit_document_date.set(get_date_part(&e.entry_date));
                                                                                edit_entry_date.set(get_date_part(&e.entry_date));
                                                                                edit_description.set(e.description.clone());
                                                                                edit_document_number.set(e.document_number.clone().unwrap_or_default());
                                                                                edit_vat_period.set(e.vat_period.clone().unwrap_or_default());
                                                                                show_edit_modal.set(true);
                                                                                load_entry_for_edit(e.id);
                                                                            }
                                                                        }
                                                                    }
                                                                >
                                                                    "✏"
                                                                </button>
                                                                <button
                                                                    style=move || format!(
                                                                        "border: none; padding: 6px 12px; border-radius: 4px; font-size: 12px; {}",
                                                                        if can_modify { "background: #e74c3c; color: white; cursor: pointer;" } else { "background: #bdc3c7; color: #7f8c8d; cursor: not-allowed;" }
                                                                    )
                                                                    disabled=move || !can_modify
                                                                    on:click={
                                                                        let e = entry_for_delete.clone();
                                                                        move |_| {
                                                                            if can_modify {
                                                                                single_delete_entry.set(Some(e.clone()));
                                                                                show_single_delete_modal.set(true);
                                                                            }
                                                                        }
                                                                    }
                                                                >
                                                                    "🗑"
                                                                </button>
                                                            </>
                                                        }
                                                    }
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
                }.into_any()
            }}

            // Counterpart Selection Modal
            {move || {
                if show_counterpart_modal.get() {
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000;">
                            <div style="background: white; border-radius: 8px; padding: 20px; width: 500px; max-height: 80vh; overflow-y: auto;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                    <h3 style="margin: 0;">"Избор на контрагент"</h3>
                                    <button
                                        style="background: transparent; border: none; font-size: 24px; cursor: pointer; color: #7f8c8d;"
                                        on:click=move |_| show_counterpart_modal.set(false)
                                    >"×"</button>
                                </div>

                                // Search input
                                <input
                                    type="text"
                                    placeholder="Търсене по име или ЕИК..."
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 15px; box-sizing: border-box;"
                                    prop:value=move || counterpart_search.get()
                                    on:input=move |ev| counterpart_search.set(event_target_value(&ev))
                                />

                                // Results list
                                <div style="border: 1px solid #ddd; border-radius: 4px; max-height: 400px; overflow-y: auto;">
                                    {move || {
                                        let search = counterpart_search.get().to_lowercase();
                                        let filtered: Vec<_> = counterparts.get().into_iter()
                                            .filter(|cp| {
                                                if search.is_empty() { return true; }
                                                cp.name.to_lowercase().contains(&search) ||
                                                cp.vat_number.as_ref().map(|v| v.to_lowercase().contains(&search)).unwrap_or(false)
                                            })
                                            .collect();
                                        if filtered.is_empty() {
                                            view! {
                                                <div style="padding: 20px; text-align: center; color: #7f8c8d;">"Няма намерени контрагенти"</div>
                                            }.into_any()
                                        } else {
                                            filtered.into_iter().map(|cp| {
                                                let id = cp.id;
                                                let display = format!("{}{}", cp.name, cp.vat_number.map(|v| format!(" ({})", v)).unwrap_or_default());
                                                view! {
                                                    <div
                                                        style="padding: 12px; cursor: pointer; border-bottom: 1px solid #eee; transition: background 0.2s;"
                                                        on:click=move |_| {
                                                            entry_counterpart_id.set(id.to_string());
                                                            show_counterpart_modal.set(false);
                                                        }
                                                        on:mouseenter=move |ev| {
                                                            if let Some(target) = ev.target() {
                                                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                                                    let _ = el.style().set_property("background", "#f0f0f0");
                                                                }
                                                            }
                                                        }
                                                        on:mouseleave=move |ev| {
                                                            if let Some(target) = ev.target() {
                                                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                                                    let _ = el.style().set_property("background", "white");
                                                                }
                                                            }
                                                        }
                                                    >
                                                        {display}
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Account Selection Modal
            {move || {
                if show_account_modal.get() {
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000;">
                            <div style="background: white; border-radius: 8px; padding: 20px; width: 500px; max-height: 80vh; overflow-y: auto;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                    <h3 style="margin: 0;">"Избор на сметка"</h3>
                                    <button
                                        style="background: transparent; border: none; font-size: 24px; cursor: pointer; color: #7f8c8d;"
                                        on:click=move |_| show_account_modal.set(false)
                                    >"×"</button>
                                </div>

                                // Search input
                                <input
                                    type="text"
                                    placeholder="Търсене по код или име..."
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 15px; box-sizing: border-box;"
                                    prop:value=move || account_search.get()
                                    on:input=move |ev| account_search.set(event_target_value(&ev))
                                />

                                // Results list
                                <div style="border: 1px solid #ddd; border-radius: 4px; max-height: 400px; overflow-y: auto;">
                                    {move || {
                                        let search = account_search.get().to_lowercase();
                                        let filtered: Vec<_> = accounts.get().into_iter()
                                            .filter(|acc| {
                                                if search.is_empty() { return true; }
                                                acc.code.to_lowercase().contains(&search) ||
                                                acc.name.to_lowercase().contains(&search)
                                            })
                                            .collect();
                                        let line_idx = account_modal_line_idx.get();
                                        if filtered.is_empty() {
                                            view! {
                                                <div style="padding: 20px; text-align: center; color: #7f8c8d;">"Няма намерени сметки"</div>
                                            }.into_any()
                                        } else {
                                            filtered.into_iter().map(|acc| {
                                                let code = acc.code.clone();
                                                let display = format!("{} - {}", acc.code, acc.name);
                                                view! {
                                                    <div
                                                        style="padding: 12px; cursor: pointer; border-bottom: 1px solid #eee; transition: background 0.2s;"
                                                        on:click=move |_| {
                                                            let c = code.clone();
                                                            entry_lines.update(|lines| {
                                                                if let Some(l) = lines.get_mut(line_idx) {
                                                                    l.account_code = c;
                                                                }
                                                            });
                                                            show_account_modal.set(false);
                                                        }
                                                        on:mouseenter=move |ev| {
                                                            if let Some(target) = ev.target() {
                                                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                                                    let _ = el.style().set_property("background", "#f0f0f0");
                                                                }
                                                            }
                                                        }
                                                        on:mouseleave=move |ev| {
                                                            if let Some(target) = ev.target() {
                                                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                                                    let _ = el.style().set_property("background", "white");
                                                                }
                                                            }
                                                        }
                                                    >
                                                        {display}
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Edit Counterpart Selection Modal
            {move || {
                if show_edit_counterpart_modal.get() {
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000;">
                            <div style="background: white; border-radius: 8px; padding: 20px; width: 500px; max-height: 80vh; overflow-y: auto;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                    <h3 style="margin: 0;">"Избор на контрагент"</h3>
                                    <button
                                        style="background: transparent; border: none; font-size: 24px; cursor: pointer; color: #7f8c8d;"
                                        on:click=move |_| show_edit_counterpart_modal.set(false)
                                    >"×"</button>
                                </div>

                                // Search input
                                <input
                                    type="text"
                                    placeholder="Търсене по име или ЕИК..."
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 15px; box-sizing: border-box;"
                                    prop:value=move || edit_counterpart_search.get()
                                    on:input=move |ev| edit_counterpart_search.set(event_target_value(&ev))
                                />

                                // Results list
                                <div style="border: 1px solid #ddd; border-radius: 4px; max-height: 400px; overflow-y: auto;">
                                    {move || {
                                        let search = edit_counterpart_search.get().to_lowercase();
                                        let filtered: Vec<_> = counterparts.get().into_iter()
                                            .filter(|cp| {
                                                if search.is_empty() { return true; }
                                                cp.name.to_lowercase().contains(&search) ||
                                                cp.vat_number.as_ref().map(|v| v.to_lowercase().contains(&search)).unwrap_or(false)
                                            })
                                            .collect();
                                        if filtered.is_empty() {
                                            view! {
                                                <div style="padding: 20px; text-align: center; color: #7f8c8d;">"Няма намерени контрагенти"</div>
                                            }.into_any()
                                        } else {
                                            filtered.into_iter().map(|cp| {
                                                let id = cp.id;
                                                let display = format!("{}{}", cp.name, cp.vat_number.map(|v| format!(" ({})", v)).unwrap_or_default());
                                                view! {
                                                    <div
                                                        style="padding: 12px; cursor: pointer; border-bottom: 1px solid #eee; transition: background 0.2s;"
                                                        on:click=move |_| {
                                                            edit_counterpart_id.set(id.to_string());
                                                            show_edit_counterpart_modal.set(false);
                                                        }
                                                        on:mouseenter=move |ev| {
                                                            if let Some(target) = ev.target() {
                                                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                                                    let _ = el.style().set_property("background", "#f0f0f0");
                                                                }
                                                            }
                                                        }
                                                        on:mouseleave=move |ev| {
                                                            if let Some(target) = ev.target() {
                                                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                                                    let _ = el.style().set_property("background", "white");
                                                                }
                                                            }
                                                        }
                                                    >
                                                        {display}
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Edit Account Selection Modal
            {move || {
                if show_edit_account_modal.get() {
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000;">
                            <div style="background: white; border-radius: 8px; padding: 20px; width: 500px; max-height: 80vh; overflow-y: auto;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                    <h3 style="margin: 0;">"Избор на сметка"</h3>
                                    <button
                                        style="background: transparent; border: none; font-size: 24px; cursor: pointer; color: #7f8c8d;"
                                        on:click=move |_| show_edit_account_modal.set(false)
                                    >"×"</button>
                                </div>

                                // Search input
                                <input
                                    type="text"
                                    placeholder="Търсене по код или име..."
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 15px; box-sizing: border-box;"
                                    prop:value=move || edit_account_search.get()
                                    on:input=move |ev| edit_account_search.set(event_target_value(&ev))
                                />

                                // Results list
                                <div style="border: 1px solid #ddd; border-radius: 4px; max-height: 400px; overflow-y: auto;">
                                    {move || {
                                        let search = edit_account_search.get().to_lowercase();
                                        let filtered: Vec<_> = accounts.get().into_iter()
                                            .filter(|acc| {
                                                if search.is_empty() { return true; }
                                                acc.code.to_lowercase().contains(&search) ||
                                                acc.name.to_lowercase().contains(&search)
                                            })
                                            .collect();
                                        let line_idx = edit_account_modal_line_idx.get();
                                        if filtered.is_empty() {
                                            view! {
                                                <div style="padding: 20px; text-align: center; color: #7f8c8d;">"Няма намерени сметки"</div>
                                            }.into_any()
                                        } else {
                                            filtered.into_iter().map(|acc| {
                                                let code = acc.code.clone();
                                                let display = format!("{} - {}", acc.code, acc.name);
                                                view! {
                                                    <div
                                                        style="padding: 12px; cursor: pointer; border-bottom: 1px solid #eee; transition: background 0.2s;"
                                                        on:click=move |_| {
                                                            let c = code.clone();
                                                            edit_lines.update(|lines| {
                                                                if let Some(l) = lines.get_mut(line_idx) {
                                                                    l.account_code = c;
                                                                }
                                                            });
                                                            show_edit_account_modal.set(false);
                                                        }
                                                        on:mouseenter=move |ev| {
                                                            if let Some(target) = ev.target() {
                                                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                                                    let _ = el.style().set_property("background", "#f0f0f0");
                                                                }
                                                            }
                                                        }
                                                        on:mouseleave=move |ev| {
                                                            if let Some(target) = ev.target() {
                                                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                                                    let _ = el.style().set_property("background", "white");
                                                                }
                                                            }
                                                        }
                                                    >
                                                        {display}
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    }}
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
