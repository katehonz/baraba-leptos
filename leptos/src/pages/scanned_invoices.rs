use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{FileReader, ProgressEvent, DragEvent};
use gloo_timers::future::TimeoutFuture;
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete};
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
    pub confidence: Option<f64>,
    pub requires_manual_review: Option<bool>,
    pub counterpart_id: Option<i64>,
    pub counterpart_name: Option<String>,
    pub journal_entry_id: Option<i64>,
    pub vat_period: Option<String>,
    pub has_inventory: Option<bool>,
    pub created_at: Option<String>,
    pub original_file_name: Option<String>,
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

#[component]
pub fn ScannedInvoices() -> impl IntoView {
    let ctx = use_company_context();

    // Upload state
    let file_input_ref = create_node_ref::<html::Input>();
    let is_dragging = create_rw_signal(false);
    let upload_queue = create_rw_signal(Vec::<UploadFile>::new());
    let uploading = create_rw_signal(false);

    // Options - load from localStorage if available
    let direction = create_rw_signal(
        get_local_storage("scan_direction").unwrap_or_else(|| String::from("purchase"))
    );
    let vat_period = create_rw_signal(
        get_local_storage("scan_vat_period").unwrap_or_else(get_current_vat_period)
    );

    // Table state
    let invoices = create_rw_signal(Vec::<ScannedInvoice>::new());
    let loading = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // Edit dialog state
    let show_edit_dialog = create_rw_signal(false);
    let edit_invoice = create_rw_signal(ScannedInvoice::default());
    let edit_invoice_index = create_rw_signal(0usize);
    let saving = create_rw_signal(false);

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
    create_effect(move |_| {
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

    // Open edit dialog
    let open_edit = move |inv: ScannedInvoice| {
        // Find index of this invoice in the list
        let list = invoices.get();
        let idx = list.iter().position(|i| i.id == inv.id).unwrap_or(0);
        edit_invoice_index.set(idx);
        edit_invoice.set(inv);
        show_edit_dialog.set(true);
    };

    // Navigate to previous invoice
    let go_prev = move |_| {
        let idx = edit_invoice_index.get();
        let list = invoices.get();
        if idx > 0 && !list.is_empty() {
            let new_idx = idx - 1;
            edit_invoice_index.set(new_idx);
            edit_invoice.set(list[new_idx].clone());
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
        }
    };

    // Confirm and create journal entry from dialog
    let confirm_invoice = move |_| {
        let inv = edit_invoice.get();
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        saving.set(true);

        spawn_local(async move {
            // First save any changes
            let payload = serde_json::json!({
                "vendor_name": inv.vendor_name,
                "vendor_vat_number": inv.vendor_vat_number,
                "customer_name": inv.customer_name,
                "customer_vat_number": inv.customer_vat_number,
                "invoice_number": inv.invoice_number,
                "subtotal": inv.subtotal,
                "total_tax": inv.total_tax,
                "invoice_total": inv.invoice_total,
            });

            let url = format!("/api/companies/{}/scanned_invoices/{}", company_id, inv.id);
            let _ = api_put(&url, &payload).await;

            // Then create journal entry
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
        let inv = edit_invoice.get();
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        saving.set(true);

        spawn_local(async move {
            let payload = serde_json::json!({
                "vendor_name": inv.vendor_name,
                "vendor_vat_number": inv.vendor_vat_number,
                "customer_name": inv.customer_name,
                "customer_vat_number": inv.customer_vat_number,
                "invoice_number": inv.invoice_number,
                "subtotal": inv.subtotal,
                "total_tax": inv.total_tax,
                "invoice_total": inv.invoice_total,
            });

            let url = format!("/api/companies/{}/scanned_invoices/{}", company_id, inv.id);
            match api_put(&url, &payload).await {
                Ok(_) => {
                    success_msg.set("Запазено".to_string());
                    show_edit_dialog.set(false);
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
                    }.into_view()
                } else { view! { <span></span> }.into_view() }
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
                    }.into_view()
                } else { view! { <span></span> }.into_view() }
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
                        view! { <span></span> }.into_view()
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
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
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
                                                <span style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{&f.name}</span>
                                                {if let Some(err) = err_msg {
                                                    view! { <span style="color: #e74c3c; font-size: 0.85em;">{err}</span> }.into_view()
                                                } else {
                                                    view! { <span></span> }.into_view()
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_view()
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
                                }.into_view()
                            } else if inv_list.is_empty() {
                                view! {
                                    <tr><td colspan="8" style="padding: 40px; text-align: center; color: #7f8c8d;">"Няма сканирани фактури"</td></tr>
                                }.into_view()
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
                                                        }.into_view()
                                                    } else if let Some(je_id) = journal_id {
                                                        view! {
                                                            <a
                                                                href=format!("/journal-entries?edit={}", je_id)
                                                                style="background: #3498db; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 0.85em; text-decoration: none; display: inline-block;"
                                                                title=format!("Редактирай запис #{}", je_id)
                                                            >
                                                                {format!("#{}", je_id)}
                                                            </a>
                                                        }.into_view()
                                                    } else {
                                                        view! { <span></span> }.into_view()
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
                                }).collect_view()
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
                                                    let val: f64 = event_target_value(&ev).replace(',', ".").parse().unwrap_or(0.0);
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
                                                    let val: f64 = event_target_value(&ev).replace(',', ".").parse().unwrap_or(0.0);
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
                                                    let val: f64 = event_target_value(&ev).replace(',', ".").parse().unwrap_or(0.0);
                                                    edit_invoice.update(|inv| inv.invoice_total = Some(val));
                                                }
                                            />
                                        </div>
                                    </div>

                                </div>
                                <div style="padding: 15px 20px; border-top: 1px solid #ddd; display: flex; justify-content: space-between; align-items: center;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_edit_dialog.set(false)
                                    >
                                        "Затвори"
                                    </button>
                                    <div style="display: flex; gap: 10px;">
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=save_invoice
                                            disabled=move || saving.get()
                                        >
                                            "Запази"
                                        </button>
                                        {move || {
                                            let inv = edit_invoice.get();
                                            if inv.status != "processed" {
                                                view! {
                                                    <button
                                                        style="background: #27ae60; color: white; border: none; padding: 10px 25px; border-radius: 4px; cursor: pointer; font-weight: bold;"
                                                        on:click=confirm_invoice
                                                        disabled=move || saving.get()
                                                    >
                                                        {move || if saving.get() { "..." } else { "Потвърди и осчетоводи ▶" }}
                                                    </button>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <span style="color: #27ae60; font-weight: bold; padding: 10px;">
                                                        "✓ Осчетоводено"
                                                    </span>
                                                }.into_view()
                                            }
                                        }}
                                    </div>
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
