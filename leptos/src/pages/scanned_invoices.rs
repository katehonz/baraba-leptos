use leptos::prelude::*;
use leptos::ev;
use leptos::html;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{FileReader, ProgressEvent, DragEvent};
use gloo_timers::future::TimeoutFuture;
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete};
use crate::constants::{is_generic_eik, strip_eu_vat_prefix};
use crate::context::use_company_context;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScannedInvoice {
    pub id: i64,
    pub direction: String,
    pub status: String,
    pub vendor_name: Option<String>,
    pub vendor_vat_number: Option<String>,
    pub customer_name: Option<String>,
    pub customer_vat_number: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub subtotal: Option<f64>,
    pub total_tax: Option<f64>,
    pub invoice_total: Option<f64>,
    pub vies_status: Option<String>,
    #[serde(default)]
    pub vies_validation_message: Option<String>,
    #[serde(default)]
    pub vies_company_name: Option<String>,
    #[serde(default)]
    pub vies_company_address: Option<String>,
    pub confidence: Option<f64>,
    pub requires_manual_review: Option<bool>,
    pub manual_review_reason: Option<String>,
    pub counterpart_id: Option<i64>,
    pub counterpart_name: Option<String>,
    pub journal_entry_id: Option<i64>,
    pub vat_period: Option<String>,
    pub has_inventory: Option<bool>,
    pub created_at: Option<String>,
    pub original_file_name: Option<String>,
    pub payment_method: Option<String>,
    pub payment_method_raw: Option<String>,
    pub payment_journal_entry_id: Option<i64>,
}

#[derive(Debug, Clone)]
struct UploadFile {
    name: String,
    content: Vec<u8>,
    status: UploadStatus,
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum UploadStatus {
    Pending,
    Uploading,
    Done,
    Error,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ScanCounterpart {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub eik: Option<String>,
    #[serde(default)]
    pub vat_number: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub counterpart_type: String,
}

#[derive(Debug, Clone, Default)]
struct ViesInfo {
    name: String,
    address: String,
    country_code: String,
    vat_number: String,
}

#[component]
pub fn ScannedInvoices() -> impl IntoView {
    let ctx = use_company_context();

    // Upload state
    let file_input_ref = NodeRef::<html::Input>::new();
    let is_dragging = RwSignal::new(false);
    let upload_queue = RwSignal::new(Vec::<UploadFile>::new());
    let uploading = RwSignal::new(false);

    // Options - load from localStorage if available
    let direction = RwSignal::new(
        get_local_storage("scan_direction").unwrap_or_else(|| String::from("purchase"))
    );
    let vat_period = RwSignal::new(
        get_local_storage("scan_vat_period").unwrap_or_else(get_current_vat_period)
    );

    // Table state
    let invoices = RwSignal::new(Vec::<ScannedInvoice>::new());
    let loading = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());

    // Edit dialog state
    let show_edit_dialog = RwSignal::new(false);
    let edit_invoice = RwSignal::new(ScannedInvoice::default());
    let edit_invoice_index = RwSignal::new(0usize);
    let saving = RwSignal::new(false);
    let paying = RwSignal::new(false);

    // Counterpart + VIES check inside the preview dialog.
    // Local VAT/EIK hit skips VIES. VIES runs only when the party is not in the database.
    let counterparts = RwSignal::new(Vec::<ScanCounterpart>::new());
    let loaded_for_company = RwSignal::new(0i64);
    let check_generation = RwSignal::new(0u64);
    let party_busy = RwSignal::new(false);
    let party_status = RwSignal::new(String::new());
    let party_message = RwSignal::new(String::new());
    let party_matches = RwSignal::new(Vec::<ScanCounterpart>::new());
    let name_hits = RwSignal::new(Vec::<ScanCounterpart>::new());
    let counterpart_query = RwSignal::new(String::new());
    let vies_info = RwSignal::new(ViesInfo::default());
    let adding_counterpart = RwSignal::new(false);

    // Load invoices
    let load_invoices = move || {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        loading.set(true);

        spawn_local(async move {
            let url = format!("/api/companies/{}/scanned_invoices", company_id);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let list: Vec<ScannedInvoice> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        invoices.set(list);
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Initial load
    Effect::new(move |_| {
        load_invoices();
    });

    // File select click
    let on_file_select = move |_| {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    // Process dropped/selected files
    let process_files = move |files: web_sys::FileList| {
        let file_count = files.length();

        for i in 0..file_count {
            if let Some(file) = files.get(i) {
                let fname = file.name();
                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();

                let onload = Closure::wrap(Box::new(move |_: ProgressEvent| {
                    if let Ok(result) = reader_clone.result() {
                        if let Some(array_buffer) = result.dyn_ref::<js_sys::ArrayBuffer>() {
                            let uint8_array = js_sys::Uint8Array::new(array_buffer);
                            let bytes: Vec<u8> = uint8_array.to_vec();

                            upload_queue.update(|queue| {
                                queue.push(UploadFile {
                                    name: fname.clone(),
                                    content: bytes,
                                    status: UploadStatus::Pending,
                                    error: None,
                                });
                            });
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
                let _ = reader.read_as_array_buffer(&file);
            }
        }
    };

    // File input change
    let on_file_change = move |ev: ev::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        if let Some(files) = target.files() {
            process_files(files);
        }
        target.set_value("");
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
                process_files(files);
            }
        }
    };

    // Start upload
    let start_upload = move |_| {
        let queue = upload_queue.get();
        if queue.is_empty() {
            return;
        }

        uploading.set(true);
        error_msg.set(String::new());

        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        let dir = direction.get();
        let period = vat_period.get();

        spawn_local(async move {
            let total = upload_queue.get().len();

            for idx in 0..total {
                // Delay between requests to avoid rate limit
                if idx > 0 {
                    TimeoutFuture::new(3_000).await;
                }

                // Update status
                upload_queue.update(|queue| {
                    if let Some(file) = queue.get_mut(idx) {
                        file.status = UploadStatus::Uploading;
                    }
                });

                let file = upload_queue.get()[idx].clone();
                let base64_content = base64_encode(&file.content);

                let payload = serde_json::json!({
                    "direction": dir,
                    "vat_period": period,
                    "file_content": base64_content,
                    "file_name": file.name,
                });

                let url = format!("/api/companies/{}/scanned_invoices/scan", company_id);

                match api_post(&url, &payload).await {
                    Ok(data) => {
                        if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            upload_queue.update(|queue| {
                                if let Some(f) = queue.get_mut(idx) {
                                    f.status = UploadStatus::Done;
                                }
                            });
                        } else {
                            let err = data.get("error").and_then(|v| v.as_str())
                                .unwrap_or("Грешка").to_string();
                            upload_queue.update(|queue| {
                                if let Some(f) = queue.get_mut(idx) {
                                    f.status = UploadStatus::Error;
                                    f.error = Some(err);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        upload_queue.update(|queue| {
                            if let Some(f) = queue.get_mut(idx) {
                                f.status = UploadStatus::Error;
                                f.error = Some(format!("{}", e));
                            }
                        });
                    }
                }
            }

            uploading.set(false);
            // Reload invoices after upload
            load_invoices();
        });
    };

    // Clear upload queue
    let clear_queue = move |_| {
        upload_queue.set(Vec::new());
    };

    let apply_counterpart = move |invoice_id: i64, counterpart: ScanCounterpart, message: String| {
        edit_invoice.update(|current| {
            if current.id != invoice_id {
                return;
            }
            current.counterpart_id = Some(counterpart.id);
            current.counterpart_name = Some(counterpart.name.clone());
            current.vies_status = Some("valid".to_string());
            current.vies_validation_message = Some("Намерен в базата данни".to_string());
            current.vies_company_name = Some(counterpart.name.clone());
            current.vies_company_address = counterpart.address.clone();
        });
        party_status.set("local".to_string());
        party_message.set(message);
        vies_info.set(ViesInfo::default());
    };

    // Selecting an existing counterpart cancels a VIES call that is still in flight.
    let choose_counterpart = move |id: i64| {
        check_generation.update(|gen| *gen = gen.wrapping_add(1));
        party_busy.set(false);
        let list = counterparts.get_untracked();
        if let Some(counterpart) = list.iter().find(|c| c.id == id).cloned() {
            let invoice_id = edit_invoice.get_untracked().id;
            let name = counterpart.name.clone();
            apply_counterpart(
                invoice_id,
                counterpart,
                format!("Избран контрагент: {}. VIES не се проверява.", name),
            );
        }
    };

    // 1) Match VAT/EIK against counterparts. 2) If already linked, keep it.
    // 3) Only then call VIES.
    let run_party_check = move || {
        let inv = edit_invoice.get_untracked();
        let invoice_id = inv.id;
        let existing_counterpart_id = inv.counterpart_id;
        let (party_name, party_vat) = party_name_and_vat(&inv);
        let company_id = ctx.selected_company_id.get_untracked().unwrap_or(1);
        let gen = check_generation.get_untracked().wrapping_add(1);
        check_generation.set(gen);
        party_busy.set(true);
        party_status.set("checking_local".to_string());
        party_message.set("Проверка в наличните контрагенти...".to_string());
        party_matches.set(Vec::new());
        name_hits.set(Vec::new());
        counterpart_query.set(String::new());
        vies_info.set(ViesInfo::default());

        spawn_local(async move {
            let fresh = loaded_for_company.get_untracked() == company_id;
            let mut list = if fresh {
                counterparts.get_untracked()
            } else {
                Vec::new()
            };
            if !fresh {
                match api_get(&format!("/api/companies/{}/counterparts", company_id)).await {
                    Ok(data) => {
                        if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                            list = arr
                                .iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect();
                        }
                        counterparts.set(list.clone());
                        loaded_for_company.set(company_id);
                    }
                    Err(e) => {
                        if check_generation.get_untracked() == gen {
                            party_status.set("vies_error".to_string());
                            party_message.set(format!("Грешка при зареждане на контрагентите: {}", e));
                            party_busy.set(false);
                        }
                        return;
                    }
                }
            }
            if check_generation.get_untracked() != gen {
                return;
            }

            let matches = exact_counterpart_matches(&list, &party_vat);
            if !matches.is_empty() {
                party_matches.set(matches.clone());
                if matches.len() == 1 {
                    let counterpart = matches[0].clone();
                    let name = counterpart.name.clone();
                    apply_counterpart(
                        invoice_id,
                        counterpart,
                        format!("Намерен в базата: {}. VIES не се проверява.", name),
                    );
                } else {
                    let keep = existing_counterpart_id
                        .filter(|id| matches.iter().any(|c| c.id == *id));
                    edit_invoice.update(|current| {
                        if current.id != invoice_id {
                            return;
                        }
                        if keep.is_none() {
                            current.counterpart_id = None;
                            current.counterpart_name = None;
                        }
                        current.vies_status = Some("valid".to_string());
                        current.vies_validation_message = Some(
                            "Намерени са няколко контрагента по ДДС/ЕИК. VIES не се проверява.".to_string(),
                        );
                    });
                    party_status.set("local".to_string());
                    party_message.set(format!(
                        "Намерени са {} контрагента по ДДС/ЕИК. Изберете. VIES не се проверява.",
                        matches.len()
                    ));
                }
                party_busy.set(false);
                return;
            }

            // Already linked to a real counterpart, and the VAT did not point at a different one.
            if let Some(id) = existing_counterpart_id {
                if let Some(counterpart) = list.iter().find(|c| c.id == id).cloned() {
                    let name = counterpart.name.clone();
                    apply_counterpart(
                        invoice_id,
                        counterpart,
                        format!("Избран е контрагент от базата: {}. VIES не се проверява.", name),
                    );
                    party_busy.set(false);
                    return;
                }
            }

            name_hits.set(name_counterpart_matches(&list, &party_name));

            let normalized = normalize_vat_key(&party_vat);
            let eik = strip_eu_vat_prefix(&normalized);
            if normalized.is_empty() {
                edit_invoice.update(|current| {
                    if current.id != invoice_id {
                        return;
                    }
                    current.vies_status = Some("not_applicable".to_string());
                    current.vies_validation_message = Some("Няма ДДС номер за валидация".to_string());
                });
                party_status.set("no_vat".to_string());
                party_message.set(
                    "Няма го в контрагентите и няма ДДС номер. Изберете контрагент ръчно. VIES не се проверява."
                        .to_string(),
                );
                party_busy.set(false);
                return;
            }
            if is_generic_eik(&eik) {
                edit_invoice.update(|current| {
                    if current.id != invoice_id {
                        return;
                    }
                    current.vies_status = Some("not_applicable".to_string());
                    current.vies_validation_message = Some("Служебен ЕИК — без VIES проверка".to_string());
                });
                party_status.set("generic".to_string());
                party_message.set(
                    "Служебният ЕИК не е реален контрагент. Изберете от списъка. VIES не се проверява.".to_string(),
                );
                party_busy.set(false);
                return;
            }

            let vies_arg = vies_vat_argument(&party_vat);
            party_status.set("checking_vies".to_string());
            party_message.set(
                "Няма такъв контрагент в базата. Проверка във VIES...".to_string(),
            );

            let payload = serde_json::json!({ "vat_number": vies_arg });
            match api_post("/api/vies/check", &payload).await {
                Ok(response) => {
                    if check_generation.get_untracked() != gen {
                        return;
                    }
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let data = response.get("data").cloned().unwrap_or(serde_json::json!({}));
                        let valid = data.get("valid").and_then(|v| v.as_bool()).unwrap_or(false);
                        let info = ViesInfo {
                            name: data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            address: data.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            country_code: data.get("country_code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            vat_number: data.get("vat_number").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        };
                        if valid {
                            let name = info.name.clone();
                            let address = info.address.clone();
                            vies_info.set(info);
                            edit_invoice.update(|current| {
                                if current.id != invoice_id {
                                    return;
                                }
                                current.vies_status = Some("valid".to_string());
                                current.vies_validation_message = Some("Валидиран през VIES".to_string());
                                current.vies_company_name = Some(name.clone());
                                current.vies_company_address = Some(address);
                            });
                            party_status.set("vies_valid".to_string());
                            if name.is_empty() {
                                party_message.set("VIES: ДДС номерът е валиден. Няма го в контрагентите.".to_string());
                            } else {
                                party_message.set(format!(
                                    "VIES: валиден — {}. Няма го в контрагентите.",
                                    name
                                ));
                            }
                        } else {
                            edit_invoice.update(|current| {
                                if current.id != invoice_id {
                                    return;
                                }
                                current.vies_status = Some("invalid".to_string());
                                current.vies_validation_message = Some("ДДС номерът не е валиден във VIES".to_string());
                            });
                            party_status.set("vies_invalid".to_string());
                            party_message.set(
                                "Няма го в контрагентите. VIES: ДДС номерът не е валиден. Може да изберете контрагент ръчно.".to_string(),
                            );
                        }
                    } else {
                        let err = response
                            .get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Грешка при проверка във VIES")
                            .to_string();
                        edit_invoice.update(|current| {
                            if current.id != invoice_id {
                                return;
                            }
                            current.vies_status = Some("error".to_string());
                            current.vies_validation_message = Some(err.clone());
                        });
                        party_status.set("vies_error".to_string());
                        party_message.set(format!("Няма го в контрагентите. VIES: {}", err));
                    }
                }
                Err(e) => {
                    if check_generation.get_untracked() != gen {
                        return;
                    }
                    edit_invoice.update(|current| {
                        if current.id != invoice_id {
                            return;
                        }
                        current.vies_status = Some("error".to_string());
                        current.vies_validation_message = Some(e.clone());
                    });
                    party_status.set("vies_error".to_string());
                    party_message.set(format!("Няма го в контрагентите. VIES: {}", e));
                }
            }
            if check_generation.get_untracked() == gen {
                party_busy.set(false);
            }
        });
    };

    let add_from_vies = move |_| {
        if adding_counterpart.get_untracked() {
            return;
        }
        let info = vies_info.get_untracked();
        let inv = edit_invoice.get_untracked();
        let invoice_id = inv.id;
        let (party_name, party_vat) = party_name_and_vat(&inv);
        let name = if info.name.trim().is_empty() {
            party_name.trim().to_string()
        } else {
            info.name.trim().to_string()
        };
        if name.is_empty() {
            party_status.set("add_error".to_string());
            party_message.set("VIES не върна име. Попълнете името на контрагента и опитайте отново.".to_string());
            return;
        }
        let full_vat = if !info.country_code.is_empty() && !info.vat_number.is_empty() {
            format!("{}{}", info.country_code, info.vat_number)
        } else {
            vies_vat_argument(&party_vat)
        };
        let eik = strip_eu_vat_prefix(&full_vat);
        let cp_type = if inv.direction == "purchase" { "supplier" } else { "customer" };
        let company_id = ctx.selected_company_id.get_untracked().unwrap_or(1);
        let address = info.address.clone();
        let country = info.country_code.clone();
        adding_counterpart.set(true);
        party_message.set("Добавяне на контрагент от VIES...".to_string());

        spawn_local(async move {
            let payload = serde_json::json!({
                "counterpart": {
                    "name": name,
                    "eik": eik,
                    "vat_number": full_vat,
                    "address": address,
                    "country_code": country,
                    "counterpart_type": cp_type,
                    "is_active": true
                }
            });
            match api_post(&format!("/api/companies/{}/counterparts", company_id), &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let id = response.get("data").and_then(|d| d.get("id")).and_then(|v| v.as_i64());
                        if let Some(id) = id {
                            let created = ScanCounterpart {
                                id,
                                name: name.clone(),
                                eik: Some(eik),
                                vat_number: Some(full_vat),
                                address: if address.is_empty() { None } else { Some(address) },
                                country_code: if country.is_empty() { None } else { Some(country) },
                                counterpart_type: cp_type.to_string(),
                            };
                            counterparts.update(|list| {
                                if !list.iter().any(|c| c.id == id) {
                                    list.push(created.clone());
                                }
                            });
                            loaded_for_company.set(company_id);
                            apply_counterpart(
                                invoice_id,
                                created,
                                format!("Добавен от VIES и избран: {}.", name),
                            );
                        } else {
                            party_status.set("add_error".to_string());
                            party_message.set("Контрагентът е създаден, но липсва номер. Презаредете страницата.".to_string());
                        }
                    } else {
                        party_status.set("add_error".to_string());
                        party_message.set("Неуспешно добавяне на контрагент.".to_string());
                    }
                }
                Err(e) => {
                    party_status.set("add_error".to_string());
                    party_message.set(e);
                }
            }
            adding_counterpart.set(false);
        });
    };

    // Open edit dialog
    let open_edit = move |inv: ScannedInvoice| {
        // Find index of this invoice in the list
        let list = invoices.get();
        let idx = list.iter().position(|i| i.id == inv.id).unwrap_or(0);
        edit_invoice_index.set(idx);
        edit_invoice.set(inv);
        show_edit_dialog.set(true);
        run_party_check();
    };

    // Navigate to previous invoice
    let go_prev = move |_| {
        let idx = edit_invoice_index.get();
        let list = invoices.get();
        if idx > 0 && !list.is_empty() {
            let new_idx = idx - 1;
            edit_invoice_index.set(new_idx);
            edit_invoice.set(list[new_idx].clone());
            run_party_check();
        }
    };

    // Navigate to next invoice
    let go_next = move |_| {
        let idx = edit_invoice_index.get();
        let list = invoices.get();
        if !list.is_empty() && idx + 1 < list.len() {
            let new_idx = idx + 1;
            edit_invoice_index.set(new_idx);
            edit_invoice.set(list[new_idx].clone());
            run_party_check();
        }
    };

    let block_if_party_pending = move || -> bool {
        if party_busy.get_untracked() || adding_counterpart.get_untracked() {
            error_msg.set("Изчакайте проверката на контрагента.".to_string());
            return true;
        }
        if edit_invoice.get_untracked().counterpart_id.is_none() && !party_matches.get_untracked().is_empty() {
            error_msg.set("Има повече от един контрагент с този ДДС/ЕИК. Изберете кой да се използва.".to_string());
            return true;
        }
        false
    };

    // Confirm and create journal entry from dialog
    let confirm_invoice = move |_| {
        if block_if_party_pending() {
            return;
        }
        let inv = edit_invoice.get();
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        saving.set(true);
        error_msg.set(String::new());

        spawn_local(async move {
            let url = format!("/api/companies/{}/scanned_invoices/{}", company_id, inv.id);
            match api_put(&url, &scanned_invoice_save_payload(&inv)).await {
                Ok(_) => {}
                Err(e) => {
                    error_msg.set(format!("Грешка при запазване: {}", e));
                    saving.set(false);
                    return;
                }
            }

            let url = format!("/api/companies/{}/scanned_invoices/{}/create_journal_entry", company_id, inv.id);
            match api_post(&url, &serde_json::json!({})).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Осчетоводено успешно".to_string());
                        load_invoices();
                        // Auto-advance to next unprocessed invoice
                        let list = invoices.get();
                        let current_idx = edit_invoice_index.get();
                        // Find next unprocessed
                        if let Some((new_idx, next_inv)) = list.iter().enumerate()
                            .skip(current_idx + 1)
                            .find(|(_, i)| i.status != "processed") {
                            edit_invoice_index.set(new_idx);
                            edit_invoice.set(next_inv.clone());
                        } else {
                            // No more unprocessed, close dialog
                            show_edit_dialog.set(false);
                        }
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка");
                        error_msg.set(err.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            saving.set(false);
        });
    };

    // Save invoice changes
    let save_invoice = move |_| {
        if block_if_party_pending() {
            return;
        }
        let inv = edit_invoice.get();
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        saving.set(true);
        error_msg.set(String::new());

        spawn_local(async move {
            let url = format!("/api/companies/{}/scanned_invoices/{}", company_id, inv.id);
            match api_put(&url, &scanned_invoice_save_payload(&inv)).await {
                Ok(_) => {
                    success_msg.set("Запазено".to_string());
                    load_invoices();
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            saving.set(false);
        });
    };

    // Create journal entry
    let create_journal_entry = move |invoice_id: i64| {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        loading.set(true);

        spawn_local(async move {
            let url = format!("/api/companies/{}/scanned_invoices/{}/create_journal_entry", company_id, invoice_id);
            match api_post(&url, &serde_json::json!({})).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Осчетоводено успешно".to_string());
                        load_invoices();
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
    };

    // Create a separate payment journal entry (cash or card buffer)
    let pay_invoice = move |method: &'static str| {
        if block_if_party_pending() {
            return;
        }
        let inv = edit_invoice.get();
        if inv.payment_journal_entry_id.is_some() {
            error_msg.set("Вече има статия за плащане".to_string());
            return;
        }
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        paying.set(true);
        error_msg.set(String::new());

        spawn_local(async move {
            let mut to_save = inv.clone();
            to_save.payment_method = Some(method.to_string());
            let url = format!("/api/companies/{}/scanned_invoices/{}", company_id, to_save.id);
            if let Err(e) = api_put(&url, &scanned_invoice_save_payload(&to_save)).await {
                error_msg.set(format!("Грешка при запазване: {}", e));
                paying.set(false);
                return;
            }

            let url = format!("/api/companies/{}/scanned_invoices/{}/create_payment", company_id, inv.id);
            match api_post(&url, &serde_json::json!({ "method": method })).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let pay_id = data.get("data")
                            .and_then(|d| d.get("payment_journal_entry_id"))
                            .and_then(|v| v.as_i64());
                        let je_id = data.get("data")
                            .and_then(|d| d.get("journal_entry_id"))
                            .and_then(|v| v.as_i64());
                        let label = if method == "cash" { "в брой" } else { "с карта" };
                        success_msg.set(format!("Създадена е статия за плащане {}{}", label, pay_id.map(|id| format!(" #{}", id)).unwrap_or_default()));
                        edit_invoice.update(|current| {
                            current.status = "processed".to_string();
                            current.payment_method = Some(method.to_string());
                            current.payment_journal_entry_id = pay_id;
                            if je_id.is_some() {
                                current.journal_entry_id = je_id;
                            }
                        });
                        load_invoices();
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при плащане");
                        error_msg.set(err.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            paying.set(false);
        });
    };

    // Delete invoice
    let delete_invoice = move |invoice_id: i64| {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);

        spawn_local(async move {
            let url = format!("/api/companies/{}/scanned_invoices/{}", company_id, invoice_id);
            match api_delete(&url).await {
                Ok(_) => {
                    load_invoices();
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
        });
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <h1 style="margin: 0;">"AI Сканиране на фактури"</h1>
                <button
                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                    on:click=move |_| load_invoices()
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Зареждане..." } else { "Обнови" }}
                </button>
            </div>

            // Messages
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-bottom: 15px;">
                            {err}
                            <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;"
                                on:click=move |_| error_msg.set(String::new())>"✕"</button>
                        </div>
                    }.into_any()
                } else { view! { <span></span> }.into_any() }
            }}
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-bottom: 15px;">
                            {msg}
                            <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;"
                                on:click=move |_| success_msg.set(String::new())>"✕"</button>
                        </div>
                    }.into_any()
                } else { view! { <span></span> }.into_any() }
            }}

            // Hidden file input
            <input
                type="file"
                accept=".pdf,.jpg,.jpeg,.png,.webp"
                multiple=true
                style="display: none;"
                node_ref=file_input_ref
                on:change=on_file_change
            />

            // Upload Card
            <div style="background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                // Options row
                <div style="display: flex; gap: 20px; margin-bottom: 15px; flex-wrap: wrap; align-items: center;">
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-size: 0.9em; color: #7f8c8d;">"Вид документ:"</label>
                        <select
                            style="padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px;"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_local_storage("scan_direction", &val);
                                direction.set(val);
                            }
                        >
                            <option value="purchase" selected=move || direction.get() == "purchase">"Покупка"</option>
                            <option value="sale" selected=move || direction.get() == "sale">"Продажба"</option>
                        </select>
                    </div>
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-size: 0.9em; color: #7f8c8d;">"ДДС период:"</label>
                        <input
                            type="month"
                            style="padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px;"
                            prop:value=move || vat_period.get()
                            on:input=move |ev| {
                                let val = event_target_value(&ev);
                                set_local_storage("scan_vat_period", &val);
                                vat_period.set(val);
                            }
                        />
                    </div>
                </div>

                // Drop zone
                <div
                    style=move || format!(
                        "border: 2px dashed {}; border-radius: 8px; padding: 30px; text-align: center; cursor: pointer; transition: all 0.2s; {}",
                        if is_dragging.get() { "#3498db" } else { "#bdc3c7" },
                        if is_dragging.get() { "background: #ebf5fb;" } else { "" }
                    )
                    on:dragover=on_drag_over
                    on:dragleave=on_drag_leave
                    on:drop=on_drop
                    on:click=on_file_select
                >
                    <div style="font-size: 2.5em; margin-bottom: 10px;">"📄"</div>
                    <div style="font-weight: bold; margin-bottom: 5px;">"Плъзнете файлове тук или кликнете"</div>
                    <div style="color: #7f8c8d; font-size: 0.9em;">"PDF, JPG, PNG, WEBP"</div>
                </div>

                // Upload queue
                {move || {
                    let queue = upload_queue.get();
                    if queue.is_empty() {
                        view! { <span></span> }.into_any()
                    } else {
                        let is_uploading = uploading.get();
                        let done_count = queue.iter().filter(|f| f.status == UploadStatus::Done).count();
                        let error_count = queue.iter().filter(|f| f.status == UploadStatus::Error).count();
                        let all_done = done_count + error_count == queue.len();

                        view! {
                            <div style="margin-top: 15px; border: 1px solid #ddd; border-radius: 4px;">
                                // Progress header
                                <div style="padding: 10px 15px; background: #f8f9fa; border-bottom: 1px solid #ddd; display: flex; justify-content: space-between; align-items: center;">
                                    <span style="font-weight: bold;">
                                        {if is_uploading {
                                            format!("Качване... {}/{}", done_count + error_count, queue.len())
                                        } else if all_done {
                                            format!("Готово: {} успешни, {} грешки", done_count, error_count)
                                        } else {
                                            format!("{} файла за качване", queue.len())
                                        }}
                                    </span>
                                    <div style="display: flex; gap: 10px;">
                                        {if !is_uploading && !all_done {
                                            view! {
                                                <button
                                                    style="background: #27ae60; color: white; border: none; padding: 6px 15px; border-radius: 4px; cursor: pointer;"
                                                    on:click=start_upload
                                                >
                                                    "Започни"
                                                </button>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                        <button
                                            style="background: #95a5a6; color: white; border: none; padding: 6px 15px; border-radius: 4px; cursor: pointer;"
                                            on:click=clear_queue
                                            disabled=is_uploading
                                        >
                                            "Изчисти"
                                        </button>
                                    </div>
                                </div>
                                // File list
                                <div style="max-height: 200px; overflow-y: auto;">
                                    {queue.iter().map(|f| {
                                        let icon = match f.status {
                                            UploadStatus::Pending => "⏳",
                                            UploadStatus::Uploading => "🔄",
                                            UploadStatus::Done => "✅",
                                            UploadStatus::Error => "❌",
                                        };
                                        let bg = match f.status {
                                            UploadStatus::Done => "#e8f5e9",
                                            UploadStatus::Error => "#ffebee",
                                            UploadStatus::Uploading => "#fff3e0",
                                            _ => "white",
                                        };
                                        let err_msg = f.error.clone();

                                        view! {
                                            <div style=format!("padding: 8px 15px; border-bottom: 1px solid #eee; display: flex; align-items: center; gap: 10px; background: {};", bg)>
                                                <span>{icon}</span>
                                                <span style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{f.name.clone()}</span>
                                                {if let Some(err) = err_msg {
                                                    view! { <span style="color: #e74c3c; font-size: 0.85em;">{err}</span> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Invoices Table
            <div style="background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="background: #f8f9fa;">
                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Контрагент"</th>
                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"ДДС №"</th>
                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Фактура"</th>
                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Дата"</th>
                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Период"</th>
                            <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Сума"</th>
                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6;">"Статус"</th>
                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6;">"Действия"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let inv_list = invoices.get();
                            if loading.get() {
                                view! {
                                    <tr><td colspan="8" style="padding: 40px; text-align: center; color: #7f8c8d;">"Зареждане..."</td></tr>
                                }.into_any()
                            } else if inv_list.is_empty() {
                                view! {
                                    <tr><td colspan="8" style="padding: 40px; text-align: center; color: #7f8c8d;">"Няма сканирани фактури"</td></tr>
                                }.into_any()
                            } else {
                                inv_list.into_iter().map(|inv| {
                                    let inv_id = inv.id;
                                    let inv_for_edit = inv.clone();
                                    let name = if inv.direction == "purchase" {
                                        inv.vendor_name.clone().unwrap_or("-".to_string())
                                    } else {
                                        inv.customer_name.clone().unwrap_or("-".to_string())
                                    };
                                    let vat_num = if inv.direction == "purchase" {
                                        inv.vendor_vat_number.clone().unwrap_or("-".to_string())
                                    } else {
                                        inv.customer_vat_number.clone().unwrap_or("-".to_string())
                                    };
                                    let status = inv.status.clone();
                                    let is_processed = status == "processed";
                                    let journal_id = inv.journal_entry_id;

                                    let status_style = match status.as_str() {
                                        "pending" => "background: #f39c12; color: white;",
                                        "validated" => "background: #3498db; color: white;",
                                        "processed" => "background: #27ae60; color: white;",
                                        "rejected" => "background: #e74c3c; color: white;",
                                        _ => "background: #95a5a6; color: white;",
                                    };
                                    let status_text = match status.as_str() {
                                        "pending" => "Чакащ".to_string(),
                                        "validated" => "Валидиран".to_string(),
                                        "processed" => "Осчетоводен".to_string(),
                                        "rejected" => "Отхвърлен".to_string(),
                                        _ => status.clone(),
                                    };

                                    view! {
                                        <tr style="border-bottom: 1px solid #dee2e6;">
                                            <td style="padding: 12px;">{name}</td>
                                            <td style="padding: 12px; font-family: monospace;">{vat_num}</td>
                                            <td style="padding: 12px;">{inv.invoice_number.clone().unwrap_or("-".to_string())}</td>
                                            <td style="padding: 12px;">{inv.invoice_date.clone().map(|d| if d.len() >= 10 { d[..10].to_string() } else { d }).unwrap_or("-".to_string())}</td>
                                            <td style="padding: 12px;">{inv.vat_period.clone().unwrap_or("-".to_string())}</td>
                                            <td style="padding: 12px; text-align: right; font-weight: bold;">{inv.invoice_total.map(|v| format!("{:.2}", v)).unwrap_or("-".to_string())}</td>
                                            <td style="padding: 12px; text-align: center;">
                                                <span style=format!("padding: 4px 10px; border-radius: 4px; font-size: 0.85em; {}", status_style)>
                                                    {status_text}
                                                </span>
                                                {if inv.payment_journal_entry_id.is_some() {
                                                    let pay_label = match inv.payment_method.as_deref() {
                                                        Some("cash") => "в брой",
                                                        Some("card") => "карта",
                                                        _ => "платено",
                                                    };
                                                    view! {
                                                        <div style="margin-top: 4px; font-size: 0.75em; color: #8e44ad;">{pay_label}</div>
                                                    }.into_any()
                                                } else if let Some(method) = inv.payment_method.clone() {
                                                    if method == "cash" || method == "card" {
                                                        view! {
                                                            <div style="margin-top: 4px; font-size: 0.75em; color: #7f8c8d;">
                                                                {if method == "cash" { "разпознато: брой" } else { "разпознато: карта" }}
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                <div style="display: flex; gap: 5px; justify-content: center;">
                                                    // View/Edit button
                                                    <button
                                                        style="background: #3498db; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 0.85em;"
                                                        title="Преглед"
                                                        on:click=move |_| open_edit(inv_for_edit.clone())
                                                    >
                                                        "👁"
                                                    </button>
                                                    // Create journal entry button
                                                    {if !is_processed {
                                                        view! {
                                                            <button
                                                                style="background: #27ae60; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 0.85em;"
                                                                title="Осчетоводи"
                                                                on:click=move |_| create_journal_entry(inv_id)
                                                            >
                                                                "📝"
                                                            </button>
                                                        }.into_any()
                                                    } else if let Some(je_id) = journal_id {
                                                        view! {
                                                            <a
                                                                href=format!("/journal-entries?edit={}", je_id)
                                                                style="background: #3498db; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 0.85em; text-decoration: none; display: inline-block;"
                                                                title=format!("Редактирай запис #{}", je_id)
                                                            >
                                                                {format!("#{}", je_id)}
                                                            </a>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                    // Delete button
                                                    <button
                                                        style="background: #e74c3c; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 0.85em;"
                                                        title="Изтрий"
                                                        on:click=move |_| {
                                                            if web_sys::window().unwrap().confirm_with_message("Изтриване на фактурата?").unwrap_or(false) {
                                                                delete_invoice(inv_id);
                                                            }
                                                        }
                                                    >
                                                        "🗑"
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </tbody>
                </table>
            </div>

            // Edit Dialog
            {move || {
                if show_edit_dialog.get() {
                    let inv = edit_invoice.get_untracked();
                    let is_purchase = inv.direction == "purchase";
                    let confidence = inv.confidence.unwrap_or(0.0);

                    view! {
                        <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; width: 90%; max-width: 700px; max-height: 90vh; overflow-y: auto;">
                                <div style="padding: 15px 20px; border-bottom: 1px solid #ddd; display: flex; justify-content: space-between; align-items: center;">
                                    // Navigation arrows
                                    <div style="display: flex; align-items: center; gap: 10px;">
                                        <button
                                            style="background: #ecf0f1; border: none; padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 1.2em;"
                                            on:click=go_prev
                                            disabled=move || edit_invoice_index.get() == 0
                                            title="Предишна"
                                        >
                                            "◀"
                                        </button>
                                        <span style="font-weight: bold; min-width: 80px; text-align: center;">
                                            {move || format!("{} / {}", edit_invoice_index.get() + 1, invoices.get().len())}
                                        </span>
                                        <button
                                            style="background: #ecf0f1; border: none; padding: 8px 12px; border-radius: 4px; cursor: pointer; font-size: 1.2em;"
                                            on:click=go_next
                                            disabled=move || {
                                                let list = invoices.get();
                                                list.is_empty() || edit_invoice_index.get() + 1 >= list.len()
                                            }
                                            title="Следваща"
                                        >
                                            "▶"
                                        </button>
                                    </div>
                                    <h3 style="margin: 0;">"Преглед на фактура"</h3>
                                    <button
                                        style="background: transparent; border: none; font-size: 1.5em; cursor: pointer; color: #7f8c8d;"
                                        on:click=move |_| show_edit_dialog.set(false)
                                    >"×"</button>
                                </div>
                                <div style="padding: 20px;">
                                    // Confidence
                                    <div style="margin-bottom: 20px; padding: 10px; background: #f8f9fa; border-radius: 4px; text-align: center;">
                                        <span>"Точност: "</span>
                                        <span style=format!("font-weight: bold; color: {};",
                                            if confidence >= 0.8 { "#27ae60" }
                                            else if confidence >= 0.5 { "#f39c12" }
                                            else { "#e74c3c" })>
                                            {format!("{:.1}%", confidence * 100.0)}
                                        </span>
                                    </div>

                                    // Form fields
                                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: bold;">{if is_purchase { "Доставчик" } else { "Клиент" }}</label>
                                            <input
                                                type="text"
                                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                                prop:value=move || {
                                                    let inv = edit_invoice.get();
                                                    if inv.direction == "purchase" { inv.vendor_name.unwrap_or_default() }
                                                    else { inv.customer_name.unwrap_or_default() }
                                                }
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    edit_invoice.update(|inv| {
                                                        if inv.direction == "purchase" { inv.vendor_name = Some(val); }
                                                        else { inv.customer_name = Some(val); }
                                                    });
                                                }
                                            />
                                        </div>
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: bold;">"ДДС номер"</label>
                                            <input
                                                type="text"
                                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                                prop:value=move || {
                                                    let inv = edit_invoice.get();
                                                    if inv.direction == "purchase" { inv.vendor_vat_number.unwrap_or_default() }
                                                    else { inv.customer_vat_number.unwrap_or_default() }
                                                }
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    edit_invoice.update(|inv| {
                                                        if inv.direction == "purchase" { inv.vendor_vat_number = Some(val); }
                                                        else { inv.customer_vat_number = Some(val); }
                                                    });
                                                }
                                                on:change=move |_| run_party_check()
                                            />
                                        </div>
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: bold;">"№ Фактура"</label>
                                            <input
                                                type="text"
                                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                                prop:value=move || edit_invoice.get().invoice_number.unwrap_or_default()
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    edit_invoice.update(|inv| inv.invoice_number = Some(val));
                                                }
                                            />
                                        </div>
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Дата"</label>
                                            <input
                                                type="date"
                                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                                prop:value=move || edit_invoice.get().invoice_date.map(|d| if d.len() >= 10 { d[..10].to_string() } else { d }).unwrap_or_default()
                                                on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    edit_invoice.update(|inv| inv.invoice_date = Some(val));
                                                }
                                            />
                                        </div>
                                    </div>

                                    // Counterpart: local database first, VIES only when missing
                                    <div style="margin-top: 20px; padding: 15px; background: #f4f7fb; border: 1px solid #d6e0ea; border-radius: 8px;">
                                        <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap; margin-bottom: 10px;">
                                            <div style="font-weight: bold;">"Контрагент"</div>
                                            <button
                                                type="button"
                                                style="background: #1a5276; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                on:click=move |_| run_party_check()
                                                disabled=move || party_busy.get() || adding_counterpart.get()
                                            >
                                                {move || if party_busy.get() { "Проверка..." } else { "Провери отново" }}
                                            </button>
                                        </div>
                                        {move || {
                                            let status = party_status.get();
                                            let message = party_message.get();
                                            if message.is_empty() {
                                                return view! { <span></span> }.into_any();
                                            }
                                            let (bg, color) = match status.as_str() {
                                                "local" | "vies_valid" => ("#eafaf1", "#1e8449"),
                                                "checking_local" | "checking_vies" => ("#eaf2f8", "#1a5276"),
                                                "no_vat" | "generic" => ("#fef9e7", "#9a7d0a"),
                                                _ => ("#fdecea", "#c0392b"),
                                            };
                                            view! {
                                                <div style=format!("padding: 10px 12px; border-radius: 4px; background: {}; color: {}; font-size: 0.9em;", bg, color)>
                                                    {message}
                                                </div>
                                            }.into_any()
                                        }}
                                        {move || {
                                            let matches = party_matches.get();
                                            if matches.is_empty() {
                                                return view! { <span></span> }.into_any();
                                            }
                                            let selected = edit_invoice.get().counterpart_id;
                                            view! {
                                                <div style="display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px;">
                                                    {matches.into_iter().map(|c| {
                                                        let id = c.id;
                                                        let label = counterpart_label(&c);
                                                        let is_selected = selected == Some(id);
                                                        let style = if is_selected {
                                                            "background: #1e8449; color: white; border: 1px solid #1e8449; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                        } else {
                                                            "background: white; color: #1a5276; border: 1px solid #1a5276; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                        };
                                                        view! {
                                                            <button type="button" style=style on:click=move |_| choose_counterpart(id)>
                                                                {label}
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }}
                                        {move || {
                                            let status = party_status.get();
                                            let show = matches!(status.as_str(), "checking_vies" | "vies_valid" | "vies_invalid" | "vies_error" | "no_vat" | "generic" | "add_error");
                                            let hits = name_hits.get();
                                            if !show || hits.is_empty() {
                                                return view! { <span></span> }.into_any();
                                            }
                                            view! {
                                                <div style="margin-top: 10px;">
                                                    <div style="font-size: 0.85em; color: #566573; margin-bottom: 6px;">
                                                        "По име в базата — ако изберете някой, VIES не се ползва:"
                                                    </div>
                                                    <div style="display: flex; flex-wrap: wrap; gap: 8px;">
                                                        {hits.into_iter().map(|c| {
                                                            let id = c.id;
                                                            let label = counterpart_label(&c);
                                                            view! {
                                                                <button
                                                                    type="button"
                                                                    style="background: white; color: #1a5276; border: 1px solid #aeb6bf; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                                    on:click=move |_| choose_counterpart(id)
                                                                >
                                                                    {label}
                                                                </button>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }}
                                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 12px;">
                                            <div>
                                                <label style="display: block; margin-bottom: 5px; font-size: 0.85em; color: #566573;">"Търсене"</label>
                                                <input
                                                    type="text"
                                                    placeholder="име / ЕИК / ДДС"
                                                    style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                                    prop:value=move || counterpart_query.get()
                                                    on:input=move |ev| counterpart_query.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <div>
                                                <label style="display: block; margin-bottom: 5px; font-size: 0.85em; color: #566573;">"Избор на контрагент"</label>
                                                <select
                                                    style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                                    on:change=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        if val.is_empty() {
                                                            check_generation.update(|gen| *gen = gen.wrapping_add(1));
                                                            party_busy.set(false);
                                                            edit_invoice.update(|inv| {
                                                                inv.counterpart_id = None;
                                                                inv.counterpart_name = None;
                                                            });
                                                            if party_matches.get_untracked().is_empty() {
                                                                party_status.set("no_vat".to_string());
                                                                party_message.set("Изборът е изчистен. Натиснете „Провери отново“, за да пуснете VIES, ако няма контрагент по ДДС номера.".to_string());
                                                            } else {
                                                                party_status.set("local".to_string());
                                                                party_message.set("Изборът е изчистен. Изберете контрагент. VIES не се проверява.".to_string());
                                                            }
                                                            return;
                                                        }
                                                        if let Ok(id) = val.parse::<i64>() {
                                                            choose_counterpart(id);
                                                        }
                                                    }
                                                >
                                                    <option value="" selected=move || edit_invoice.get().counterpart_id.is_none()>"— изберете —"</option>
                                                    {move || {
                                                        let q = counterpart_query.get().to_lowercase();
                                                        let selected = edit_invoice.get().counterpart_id;
                                                        counterparts.get().into_iter().filter(|c| {
                                                            q.is_empty()
                                                                || c.name.to_lowercase().contains(&q)
                                                                || c.eik.clone().unwrap_or_default().to_lowercase().contains(&q)
                                                                || c.vat_number.clone().unwrap_or_default().to_lowercase().contains(&q)
                                                        }).map(|c| {
                                                            let value = c.id.to_string();
                                                            let label = counterpart_label(&c);
                                                            let is_selected = selected == Some(c.id);
                                                            view! { <option value=value selected=is_selected>{label}</option> }
                                                        }).collect_view()
                                                    }}
                                                </select>
                                            </div>
                                        </div>
                                        {move || {
                                            let inv = edit_invoice.get();
                                            let Some(id) = inv.counterpart_id else {
                                                return view! { <span></span> }.into_any();
                                            };
                                            let list = counterparts.get();
                                            let found = list.iter().find(|c| c.id == id).cloned();
                                            let name = found.as_ref().map(|c| c.name.clone())
                                                .or(inv.counterpart_name.clone())
                                                .unwrap_or_else(|| format!("#{}", id));
                                            let eik = found.as_ref().and_then(|c| c.eik.clone()).unwrap_or_default();
                                            let vat = found.as_ref().and_then(|c| c.vat_number.clone()).unwrap_or_default();
                                            view! {
                                                <div style="margin-top: 10px; font-size: 0.9em; color: #1e8449;">
                                                    {format!("Избран: {}{}{}", name, if eik.is_empty() { String::new() } else { format!(" — ЕИК {}", eik) }, if vat.is_empty() { String::new() } else { format!(" — ДДС {}", vat) })}
                                                </div>
                                            }.into_any()
                                        }}
                                        {move || {
                                            if party_status.get() != "vies_valid" {
                                                return view! { <span></span> }.into_any();
                                            }
                                            let info = vies_info.get();
                                            let busy = adding_counterpart.get();
                                            view! {
                                                <div style="margin-top: 10px; padding: 10px 12px; background: white; border-radius: 4px; border: 1px solid #d5f5e3;">
                                                    {if !info.name.is_empty() {
                                                        view! { <div style="font-weight: bold;">{info.name}</div> }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                    {if !info.address.is_empty() {
                                                        view! { <div style="color: #566573; font-size: 0.9em; margin-top: 4px;">{info.address}</div> }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                    <button
                                                        type="button"
                                                        style="margin-top: 8px; background: #1e8449; color: white; border: none; padding: 8px 12px; border-radius: 4px; cursor: pointer;"
                                                        on:click=add_from_vies
                                                        disabled=busy
                                                    >
                                                        {if busy { "Добавяне..." } else { "Добави като контрагент и избери" }}
                                                    </button>
                                                </div>
                                            }.into_any()
                                        }}
                                    </div>

                                    // Amounts
                                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px; margin-top: 20px;">
                                        <div style="padding: 15px; background: #e8f5e9; border-radius: 8px;">
                                            <label style="display: block; margin-bottom: 5px; color: #7f8c8d; font-size: 0.9em;">"Данъчна основа"</label>
                                            <input
                                                type="text"
                                                inputmode="decimal"
                                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 1.1em; font-weight: bold;"
                                                prop:value=move || edit_invoice.get().subtotal.map(|v| format!("{:.2}", v)).unwrap_or_default()
                                                on:change=move |ev| {
                                                    let val = parse_amount(&event_target_value(&ev));
                                                    edit_invoice.update(|inv| inv.subtotal = Some(val));
                                                }
                                            />
                                        </div>
                                        <div style="padding: 15px; background: #e3f2fd; border-radius: 8px;">
                                            <label style="display: block; margin-bottom: 5px; color: #7f8c8d; font-size: 0.9em;">"ДДС"</label>
                                            <input
                                                type="text"
                                                inputmode="decimal"
                                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 1.1em; font-weight: bold;"
                                                prop:value=move || edit_invoice.get().total_tax.map(|v| format!("{:.2}", v)).unwrap_or_default()
                                                on:change=move |ev| {
                                                    let val = parse_amount(&event_target_value(&ev));
                                                    edit_invoice.update(|inv| inv.total_tax = Some(val));
                                                }
                                            />
                                        </div>
                                        <div style="padding: 15px; background: #fff3e0; border-radius: 8px;">
                                            <label style="display: block; margin-bottom: 5px; color: #7f8c8d; font-size: 0.9em;">"Обща сума"</label>
                                            <input
                                                type="text"
                                                inputmode="decimal"
                                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 1.1em; font-weight: bold;"
                                                prop:value=move || edit_invoice.get().invoice_total.map(|v| format!("{:.2}", v)).unwrap_or_default()
                                                on:change=move |ev| {
                                                    let val = parse_amount(&event_target_value(&ev));
                                                    edit_invoice.update(|inv| inv.invoice_total = Some(val));
                                                }
                                            />
                                        </div>
                                    </div>

                                    {move || {
                                        let inv = edit_invoice.get();
                                        let sub = inv.subtotal.unwrap_or(0.0);
                                        let vat = inv.total_tax.unwrap_or(0.0);
                                        let total = inv.invoice_total.unwrap_or(0.0);
                                        if total <= 0.0 && sub <= 0.0 && vat <= 0.0 {
                                            return view! { <span></span> }.into_any();
                                        }
                                        let check = check_invoice_amounts(sub, vat, total);
                                        if check.ok {
                                            view! {
                                                <div style="margin-top: 12px; padding: 10px 12px; background: #eafaf1; color: #1e8449; border-radius: 4px; font-size: 0.9em;">
                                                    {format!("Контрола: основа {:.2} + ДДС {:.2} = {:.2}  ✓   (за 20% основа = обща сума / 1,20)", sub, vat, if total > 0.0 { total } else { sub + vat })}
                                                </div>
                                            }.into_any()
                                        } else {
                                            let expected_base = check.expected_base_20;
                                            let expected_vat = check.expected_vat_20;
                                            view! {
                                                <div style="margin-top: 12px; padding: 12px; background: #fdecea; color: #c0392b; border-radius: 4px; font-size: 0.9em;">
                                                    <div style="font-weight: bold; margin-bottom: 6px;">"Контрола на сумите — ДДС/основа не съвпадат"</div>
                                                    {check.messages.iter().map(|m| view! { <div>{m.clone()}</div> }).collect_view()}
                                                    <button
                                                        style="margin-top: 10px; background: #c0392b; color: white; border: none; padding: 8px 12px; border-radius: 4px; cursor: pointer;"
                                                        on:click=move |_| {
                                                            edit_invoice.update(|inv| {
                                                                let total = inv.invoice_total.unwrap_or(0.0);
                                                                if total > 0.0 {
                                                                    let (base, vat) = tax_base_from_gross_20(total);
                                                                    inv.subtotal = Some(base);
                                                                    inv.total_tax = Some(vat);
                                                                }
                                                            });
                                                        }
                                                    >
                                                        {format!("Поправи от обща сума: основа = {:.2} (сума/1,20), ДДС = {:.2}", expected_base, expected_vat)}
                                                    </button>
                                                </div>
                                            }.into_any()
                                        }
                                    }}

                                    {move || {
                                        let reason = edit_invoice.get().manual_review_reason.clone().unwrap_or_default();
                                        if reason.is_empty() {
                                            view! { <span></span> }.into_any()
                                        } else {
                                            view! {
                                                <div style="margin-top: 10px; padding: 10px 12px; background: #fff3cd; color: #856404; border-radius: 4px; font-size: 0.9em;">
                                                    {format!("Преглед: {}", reason)}
                                                </div>
                                            }.into_any()
                                        }
                                    }}

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

                                    // Payment method
                                    <div style="margin-top: 20px; padding: 15px; background: #f8f9fa; border-radius: 8px;">
                                        <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
                                            <div>
                                                <div style="font-weight: bold; margin-bottom: 4px;">"Начин на плащане"</div>
                                                <div style="font-size: 0.85em; color: #7f8c8d;">
                                                    {move || {
                                                        let inv = edit_invoice.get();
                                                        if let Some(raw) = inv.payment_method_raw.clone() {
                                                            if !raw.is_empty() { return format!("Разпознато: {}", raw); }
                                                        }
                                                        "AI търси „в брой“, „карта“, „POS“, „терминал“… Може да го смените.".to_string()
                                                    }}
                                                </div>
                                            </div>
                                            <select
                                                style="padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; min-width: 180px;"
                                                on:change=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    edit_invoice.update(|inv| {
                                                        inv.payment_method = if val.is_empty() { None } else { Some(val) };
                                                    });
                                                }
                                            >
                                                <option value="unknown" selected=move || edit_invoice.get().payment_method.as_deref().unwrap_or("unknown") == "unknown">"Не е посочено"</option>
                                                <option value="cash" selected=move || edit_invoice.get().payment_method.as_deref() == Some("cash")>"В брой (каса)"</option>
                                                <option value="card" selected=move || edit_invoice.get().payment_method.as_deref() == Some("card")>"Карта / POS"</option>
                                                <option value="bank" selected=move || edit_invoice.get().payment_method.as_deref() == Some("bank")>"Банков превод"</option>
                                                <option value="credit" selected=move || edit_invoice.get().payment_method.as_deref() == Some("credit")>"Отложено"</option>
                                            </select>
                                        </div>
                                    </div>

                                </div>
                                <div style="padding: 15px 20px; border-top: 1px solid #ddd; display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_edit_dialog.set(false)
                                    >
                                        "Затвори"
                                    </button>
                                    <div style="display: flex; gap: 8px; flex-wrap: wrap; justify-content: flex-end;">
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=save_invoice
                                            disabled=move || saving.get() || paying.get() || party_busy.get() || adding_counterpart.get()
                                        >
                                            "Запази"
                                        </button>
                                        {move || {
                                            let inv = edit_invoice.get();
                                            let busy = saving.get() || paying.get() || party_busy.get() || adding_counterpart.get();
                                            let detected = inv.payment_method.clone().unwrap_or_default();

                                            if let Some(pay_id) = inv.payment_journal_entry_id {
                                                let label = match inv.payment_method.as_deref() {
                                                    Some("cash") => "Платено в брой",
                                                    Some("card") => "Платено с карта",
                                                    _ => "Платено",
                                                };
                                                view! {
                                                    <a
                                                        href=format!("/journal-entries?edit={}", pay_id)
                                                        style="background: #8e44ad; color: white; border: none; padding: 10px 16px; border-radius: 4px; text-decoration: none; font-weight: bold;"
                                                        title="Отвори статията за плащане"
                                                    >
                                                        {format!("{} #{}", label, pay_id)}
                                                    </a>
                                                }.into_any()
                                            } else {
                                                let cash_border = if detected == "cash" { "box-shadow: 0 0 0 2px #e67e22;" } else { "" };
                                                let card_border = if detected == "card" { "box-shadow: 0 0 0 2px #8e44ad;" } else { "" };
                                                view! {
                                                    <button
                                                        style=format!("background: #e67e22; color: white; border: none; padding: 10px 16px; border-radius: 4px; cursor: pointer; font-weight: bold; {}", cash_border)
                                                        on:click=move |_| pay_invoice("cash")
                                                        disabled=busy
                                                        title="Нова статия: Дт доставчик/клиент, Кт каса (или обратно при продажба)"
                                                    >
                                                        {if busy && paying.get() { "..." } else { "Плащане в брой" }}
                                                    </button>
                                                    <button
                                                        style=format!("background: #8e44ad; color: white; border: none; padding: 10px 16px; border-radius: 4px; cursor: pointer; font-weight: bold; {}", card_border)
                                                        on:click=move |_| pay_invoice("card")
                                                        disabled=busy
                                                        title="Нова статия през буферната сметка за карта"
                                                    >
                                                        {if busy && paying.get() { "..." } else { "Плащане с карта" }}
                                                    </button>
                                                }.into_any()
                                            }
                                        }}
                                        {move || {
                                            let inv = edit_invoice.get();
                                            if inv.status != "processed" {
                                                view! {
                                                    <button
                                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-weight: bold;"
                                                        on:click=confirm_invoice
                                                        disabled=move || saving.get() || paying.get() || party_busy.get() || adding_counterpart.get()
                                                    >
                                                        {move || if saving.get() { "..." } else { "Потвърди и осчетоводи ▶" }}
                                                    </button>
                                                }.into_any()
                                            } else if let Some(je_id) = inv.journal_entry_id {
                                                view! {
                                                    <a
                                                        href=format!("/journal-entries?edit={}", je_id)
                                                        style="color: #27ae60; font-weight: bold; padding: 10px; text-decoration: none;"
                                                    >
                                                        {format!("✓ Фактура #{}", je_id)}
                                                    </a>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <span style="color: #27ae60; font-weight: bold; padding: 10px;">
                                                        "✓ Осчетоводено"
                                                    </span>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
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

fn scanned_invoice_save_payload(inv: &ScannedInvoice) -> serde_json::Value {
    let invoice_date = inv.invoice_date.as_ref().map(|d| {
        if d.len() >= 10 { d[..10].to_string() } else { d.clone() }
    });
    serde_json::json!({
        "vendor_name": inv.vendor_name,
        "vendor_vat_number": inv.vendor_vat_number,
        "customer_name": inv.customer_name,
        "customer_vat_number": inv.customer_vat_number,
        "invoice_number": inv.invoice_number,
        "invoice_date": invoice_date,
        "subtotal": inv.subtotal,
        "total_tax": inv.total_tax,
        "invoice_total": inv.invoice_total,
        "payment_method": inv.payment_method,
        "counterpart_id": inv.counterpart_id,
        "vies_status": inv.vies_status,
        "vies_validation_message": inv.vies_validation_message,
        "vies_company_name": inv.vies_company_name,
        "vies_company_address": inv.vies_company_address,
    })
}

fn party_name_and_vat(inv: &ScannedInvoice) -> (String, String) {
    if inv.direction == "purchase" {
        (
            inv.vendor_name.clone().unwrap_or_default(),
            inv.vendor_vat_number.clone().unwrap_or_default(),
        )
    } else {
        (
            inv.customer_name.clone().unwrap_or_default(),
            inv.customer_vat_number.clone().unwrap_or_default(),
        )
    }
}

fn normalize_vat_key(raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_whitespace() && *c != '.' && *c != '-')
        .collect::<String>()
        .to_uppercase()
}

fn vies_vat_argument(raw: &str) -> String {
    let cleaned = normalize_vat_key(raw);
    let mut chars = cleaned.chars();
    let first = chars.next();
    let second = chars.next();
    if matches!((first, second), (Some(a), Some(b)) if a.is_ascii_alphabetic() && b.is_ascii_alphabetic()) {
        cleaned
    } else if cleaned.is_empty() {
        String::new()
    } else {
        format!("BG{}", cleaned)
    }
}

fn counterpart_vat_hit(counterpart: &ScanCounterpart, cleaned: &str, eik: &str) -> bool {
    let mut keys = Vec::new();
    if let Some(value) = counterpart.vat_number.as_deref() {
        let normalized = normalize_vat_key(value);
        if !normalized.is_empty() {
            let stripped = strip_eu_vat_prefix(&normalized);
            keys.push(normalized);
            if !stripped.is_empty() {
                keys.push(stripped);
            }
        }
    }
    if let Some(value) = counterpart.eik.as_deref() {
        let stripped = strip_eu_vat_prefix(value);
        if !stripped.is_empty() {
            keys.push(stripped);
        }
    }
    keys.into_iter().any(|key| {
        if is_generic_eik(&key) {
            return false;
        }
        key == cleaned || (!eik.is_empty() && key == eik)
    })
}

fn exact_counterpart_matches(list: &[ScanCounterpart], vat_raw: &str) -> Vec<ScanCounterpart> {
    let cleaned = normalize_vat_key(vat_raw);
    if cleaned.is_empty() {
        return Vec::new();
    }
    let eik = strip_eu_vat_prefix(&cleaned);
    if !eik.is_empty() && is_generic_eik(&eik) {
        return Vec::new();
    }
    list.iter()
        .filter(|counterpart| counterpart_vat_hit(counterpart, &cleaned, &eik))
        .cloned()
        .collect()
}

fn fold_party_name(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn name_counterpart_matches(list: &[ScanCounterpart], party_name: &str) -> Vec<ScanCounterpart> {
    let query = fold_party_name(party_name);
    if query.chars().count() < 4 {
        return Vec::new();
    }
    list.iter()
        .filter(|counterpart| {
            let name = fold_party_name(&counterpart.name);
            !name.is_empty() && (name.contains(&query) || query.contains(&name))
        })
        .take(8)
        .cloned()
        .collect()
}

fn counterpart_label(counterpart: &ScanCounterpart) -> String {
    let ident = counterpart
        .eik
        .clone()
        .filter(|value| !value.is_empty())
        .or_else(|| counterpart.vat_number.clone().filter(|value| !value.is_empty()));
    match ident {
        Some(ident) => format!("{} — {}", ident, counterpart.name),
        None => counterpart.name.clone(),
    }
}

fn tax_base_from_gross_20(gross: f64) -> (f64, f64) {
    let base = ((gross / 1.2) * 100.0).round() / 100.0;
    let vat = ((gross - base) * 100.0).round() / 100.0;
    (base, vat)
}

struct InvoiceAmountCheck {
    ok: bool,
    messages: Vec<String>,
    expected_base_20: f64,
    expected_vat_20: f64,
}

fn check_invoice_amounts(subtotal: f64, vat: f64, total: f64) -> InvoiceAmountCheck {
    let gross = if total > 0.0 { total } else { subtotal + vat };
    let (expected_base_20, expected_vat_20) = tax_base_from_gross_20(gross);
    let mut messages = Vec::new();
    if (subtotal + vat - gross).abs() > 0.05 {
        messages.push(format!(
            "Основа ({:.2}) + ДДС ({:.2}) ≠ обща сума ({:.2})",
            subtotal, vat, gross
        ));
    }
    if vat.abs() > 0.005 {
        let vat20 = ((subtotal * 0.20) * 100.0).round() / 100.0;
        let vat9 = ((subtotal * 0.09) * 100.0).round() / 100.0;
        if (vat - vat20).abs() > 0.05 && (vat - vat9).abs() > 0.05 {
            messages.push(format!(
                "ДДС не е 20% или 9% от основата. За 20% от общата сума: основа = {:.2} (сума/1,20), ДДС = {:.2}",
                expected_base_20, expected_vat_20
            ));
        }
    }
    InvoiceAmountCheck {
        ok: messages.is_empty(),
        messages,
        expected_base_20,
        expected_vat_20,
    }
}

fn parse_amount(raw: &str) -> f64 {
    let s = raw.trim().replace('\u{00a0}', "").replace(' ', "");
    if s.is_empty() {
        return 0.0;
    }
    let normalized = if s.contains(',') && s.contains('.') {
        if s.rfind(',').unwrap_or(0) > s.rfind('.').unwrap_or(0) {
            s.replace('.', "").replace(',', ".")
        } else {
            s.replace(',', "")
        }
    } else {
        s.replace(',', ".")
    };
    normalized.parse().unwrap_or(0.0)
}

fn get_current_vat_period() -> String {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = now.get_month() + 1;
    format!("{:04}-{:02}", year, month)
}

fn get_local_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|storage| storage.get_item(key).ok())
        .flatten()
}

fn set_local_storage(key: &str, value: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item(key, value);
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod party_match_tests {
    use super::*;

    fn sample(id: i64, name: &str, eik: &str, vat: &str) -> ScanCounterpart {
        ScanCounterpart {
            id,
            name: name.to_string(),
            eik: Some(eik.to_string()),
            vat_number: Some(vat.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn matches_vat_with_or_without_country_prefix() {
        let list = vec![sample(1, "Алфа ЕООД", "131272801", "BG131272801")];
        assert_eq!(exact_counterpart_matches(&list, "bg 131272801").len(), 1);
        assert_eq!(exact_counterpart_matches(&list, "131272801").len(), 1);
        assert!(exact_counterpart_matches(&list, "BG999999999").is_empty());
    }

    #[test]
    fn generic_eik_is_not_treated_as_a_counterpart_hit() {
        let list = vec![sample(1, "Неизвестен", "999999999", "BG999999999")];
        assert!(exact_counterpart_matches(&list, "999999999").is_empty());
        assert!(exact_counterpart_matches(&list, "BG999999999").is_empty());
    }

    #[test]
    fn name_suggestions_need_at_least_four_characters() {
        let list = vec![sample(2, "Бета ООД", "111111111", "BG111111111")];
        assert_eq!(name_counterpart_matches(&list, "бета оод").len(), 1);
        assert!(name_counterpart_matches(&list, "аб").is_empty());
    }

    #[test]
    fn digits_only_vat_is_sent_to_vies_as_bulgarian() {
        assert_eq!(vies_vat_argument("131272801"), "BG131272801");
        assert_eq!(vies_vat_argument("it12345678901"), "IT12345678901");
    }
}
