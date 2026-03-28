use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileReader, ProgressEvent};
use crate::components::Layout;
use crate::api::api_post_raw;
use crate::context::use_company_context;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreviewSummary {
    pub contractors_count: i32,
    pub documents_count: i32,
    pub total_net_amount: f64,
    pub total_vat_amount: f64,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreviewContractor {
    pub name: String,
    pub eik: String,
    pub vat_number: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreviewDocument {
    pub document_number: String,
    pub document_date: String,
    pub reason: String,
    pub net_amount: f64,
    pub vat_amount: f64,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreviewResponse {
    pub success: bool,
    pub import_type: Option<String>,
    pub summary: Option<PreviewSummary>,
    pub contractors: Option<Vec<PreviewContractor>>,
    pub documents: Option<Vec<PreviewDocument>>,
    pub error: Option<String>,
}

#[component]
pub fn ControlisyImport() -> impl IntoView {
    let current_step = create_rw_signal(1);
    let file_name = create_rw_signal(String::new());
    let file_content = create_rw_signal(String::new());
    let file_input_ref = create_node_ref::<html::Input>();
    let loading = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());
    let preview_data = create_rw_signal(PreviewResponse::default());

    // Validation counts
    let valid_count = create_rw_signal(0);
    let warning_count = create_rw_signal(0);
    let error_count = create_rw_signal(0);

    // Import results
    let imported_contractors = create_rw_signal(0);
    let imported_documents = create_rw_signal(0);
    let imported_journal_entries = create_rw_signal(0);
    let imported_vat_journal_entries = create_rw_signal(0);

    // File counts (how many were in the XML)
    let file_contractors_count = create_rw_signal(0);
    let file_documents_count = create_rw_signal(0);

    // Errors and warnings from import
    let import_errors = create_rw_signal(Vec::<String>::new());
    let import_warnings = create_rw_signal(Vec::<String>::new());

    let ctx = use_company_context();

    let on_file_select = move |_| {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    let on_file_change = move |ev: ev::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        if let Some(files) = target.files() {
            if let Some(file) = files.get(0) {
                file_name.set(file.name());

                // Read file content
                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();

                let onload = Closure::wrap(Box::new(move |_: ProgressEvent| {
                    if let Ok(result) = reader_clone.result() {
                        if let Some(text) = result.as_string() {
                            file_content.set(text);
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();

                let _ = reader.read_as_text(&file);
            }
        }
    };

    let fetch_preview = move |_| {
        let content = file_content.get();
        if content.is_empty() {
            error_msg.set("Моля изберете файл".to_string());
            return;
        }

        loading.set(true);
        error_msg.set(String::new());

        let company_id = match ctx.selected_company_id.get() {
            Some(id) => id,
            None => {
                error_msg.set("Моля изберете фирма".to_string());
                loading.set(false);
                return;
            }
        };

        spawn_local(async move {
            match api_post_raw(&format!("/api/companies/{}/controlisy/preview", company_id), &content).await {
                Ok(data) => {
                    if let Ok(preview) = serde_json::from_value::<PreviewResponse>(data) {
                        if preview.success {
                            // Set validation counts based on preview
                            if let Some(ref summary) = preview.summary {
                                valid_count.set(summary.documents_count);
                            }
                            preview_data.set(preview);
                            current_step.set(2);
                        } else {
                            error_msg.set(preview.error.unwrap_or("Грешка при обработка".to_string()));
                        }
                    } else {
                        error_msg.set("Невалиден отговор от сървъра".to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    let do_import = move |_| {
        let content = file_content.get();
        if content.is_empty() {
            return;
        }

        loading.set(true);
        error_msg.set(String::new());

        let company_id = match ctx.selected_company_id.get() {
            Some(id) => id,
            None => {
                error_msg.set("Моля изберете фирма".to_string());
                loading.set(false);
                return;
            }
        };

        spawn_local(async move {
            match api_post_raw(&format!("/api/companies/{}/controlisy/import", company_id), &content).await {
                Ok(data) => {
                    // File counts (how many were in the XML)
                    if let Some(count) = data.get("contractors_in_file").and_then(|v| v.as_i64()) {
                        file_contractors_count.set(count as i32);
                    }
                    if let Some(count) = data.get("documents_in_file").and_then(|v| v.as_i64()) {
                        file_documents_count.set(count as i32);
                    }

                    // Collect errors
                    let errors: Vec<String> = data.get("errors")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect())
                        .unwrap_or_default();
                    import_errors.set(errors.clone());

                    // Collect warnings
                    let warnings: Vec<String> = data.get("warnings")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect())
                        .unwrap_or_default();
                    import_warnings.set(warnings);

                    if let Some(success) = data.get("success").and_then(|v| v.as_bool()) {
                        if success || errors.is_empty() {
                            // contractors_created + contractors_updated
                            let created = data.get("contractors_created").and_then(|v| v.as_i64()).unwrap_or(0);
                            let updated = data.get("contractors_updated").and_then(|v| v.as_i64()).unwrap_or(0);
                            imported_contractors.set((created + updated) as i32);

                            if let Some(imported) = data.get("documents_imported").and_then(|v| v.as_i64()) {
                                imported_documents.set(imported as i32);
                            }
                            if let Some(je) = data.get("journal_entries").and_then(|v| v.as_i64()) {
                                imported_journal_entries.set(je as i32);
                            }
                            if let Some(vje) = data.get("vat_journal_entries").and_then(|v| v.as_i64()) {
                                imported_vat_journal_entries.set(vje as i32);
                            }
                            current_step.set(4);
                        } else {
                            error_msg.set(errors.join(", "));
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

    view! {
        <Layout>
            <h1>"Импорт от Controlisy"</h1>

            // Hidden file input
            <input
                type="file"
                accept=".json,.xml"
                style="display: none;"
                node_ref=file_input_ref
                on:change=on_file_change
            />

            // Step indicator
            <div style="display: flex; gap: 20px; margin: 30px 0;">
                <div style=move || format!("flex: 1; text-align: center; padding: 15px; border-radius: 8px; {}",
                    if current_step.get() >= 1 { "background: #3498db; color: white;" } else { "background: #ecf0f1;" })>
                    <div style="font-size: 1.5em; font-weight: bold;">"1"</div>
                    <div>"Качване"</div>
                </div>
                <div style=move || format!("flex: 1; text-align: center; padding: 15px; border-radius: 8px; {}",
                    if current_step.get() >= 2 { "background: #3498db; color: white;" } else { "background: #ecf0f1;" })>
                    <div style="font-size: 1.5em; font-weight: bold;">"2"</div>
                    <div>"Преглед"</div>
                </div>
                <div style=move || format!("flex: 1; text-align: center; padding: 15px; border-radius: 8px; {}",
                    if current_step.get() >= 3 { "background: #3498db; color: white;" } else { "background: #ecf0f1;" })>
                    <div style="font-size: 1.5em; font-weight: bold;">"3"</div>
                    <div>"Валидация"</div>
                </div>
                <div style=move || format!("flex: 1; text-align: center; padding: 15px; border-radius: 8px; {}",
                    if current_step.get() >= 4 { "background: #3498db; color: white;" } else { "background: #ecf0f1;" })>
                    <div style="font-size: 1.5em; font-weight: bold;">"4"</div>
                    <div>"Импорт"</div>
                </div>
            </div>

            // Error message
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! { <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">{err}</div> }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}

            <div style="background: white; border-radius: 8px; padding: 40px; text-align: center;">
                {move || match current_step.get() {
                    1 => view! {
                        <div>
                            <div style="border: 2px dashed #bdc3c7; border-radius: 8px; padding: 60px; margin-bottom: 20px;">
                                <div style="font-size: 3em;">"📁"</div>
                                <h3>"Плъзнете файл тук"</h3>
                                <p style="color: #7f8c8d;">"или"</p>
                                <button
                                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=on_file_select
                                >
                                    "Изберете файл"
                                </button>
                                {move || {
                                    let name = file_name.get();
                                    if !name.is_empty() {
                                        view! { <p style="color: #27ae60; margin-top: 10px; font-weight: bold;">{format!("Избран файл: {}", name)}</p> }.into_view()
                                    } else {
                                        view! { <p style="color: #7f8c8d; margin-top: 10px;">"Поддържани формати: .json, .xml"</p> }.into_view()
                                    }
                                }}
                            </div>
                            <button
                                style="background: #27ae60; color: white; border: none; padding: 12px 30px; border-radius: 4px; cursor: pointer;"
                                on:click=fetch_preview
                                disabled=move || loading.get() || file_content.get().is_empty()
                            >
                                {move || if loading.get() { "Зареждане..." } else { "Продължи" }}
                            </button>
                        </div>
                    }.into_view(),
                    2 => view! {
                        <div>
                            <h3>"Преглед на данните"</h3>
                            {move || {
                                let data = preview_data.get();
                                if let Some(summary) = data.summary {
                                    view! {
                                        <div>
                                            <p style="font-size: 1.2em; margin: 20px 0;">
                                                <strong>"Контрагенти: "</strong>{summary.contractors_count}
                                                " | "
                                                <strong>"Документи: "</strong>{summary.documents_count}
                                            </p>
                                            <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px; margin: 20px 0; text-align: left;">
                                                <div style="padding: 15px; background: #e8f5e9; border-radius: 8px;">
                                                    <div style="color: #7f8c8d; font-size: 0.9em;">"Нето"</div>
                                                    <div style="font-size: 1.3em; font-weight: bold;">{format!("{:.2} EUR", summary.total_net_amount)}</div>
                                                </div>
                                                <div style="padding: 15px; background: #e3f2fd; border-radius: 8px;">
                                                    <div style="color: #7f8c8d; font-size: 0.9em;">"ДДС"</div>
                                                    <div style="font-size: 1.3em; font-weight: bold;">{format!("{:.2} EUR", summary.total_vat_amount)}</div>
                                                </div>
                                                <div style="padding: 15px; background: #fff3e0; border-radius: 8px;">
                                                    <div style="color: #7f8c8d; font-size: 0.9em;">"Общо"</div>
                                                    <div style="font-size: 1.3em; font-weight: bold;">{format!("{:.2} EUR", summary.total_amount)}</div>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <p style="color: #7f8c8d;">"Няма данни"</p> }.into_view()
                                }
                            }}
                            <div style="margin-top: 20px;">
                                <button
                                    style="background: #95a5a6; color: white; border: none; padding: 12px 30px; border-radius: 4px; cursor: pointer; margin-right: 10px;"
                                    on:click=move |_| current_step.set(1)
                                >
                                    "Назад"
                                </button>
                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 12px 30px; border-radius: 4px; cursor: pointer;"
                                    on:click=move |_| current_step.set(3)
                                >
                                    "Продължи"
                                </button>
                            </div>
                        </div>
                    }.into_view(),
                    3 => view! {
                        <div>
                            <h3>"Валидация"</h3>
                            <div style="display: flex; justify-content: center; gap: 30px; margin: 20px 0;">
                                <div style="color: #27ae60; font-size: 1.2em;">
                                    "✓ Валидни: "{move || valid_count.get()}
                                </div>
                                <div style="color: #f39c12; font-size: 1.2em;">
                                    "⚠ Предупреждения: "{move || warning_count.get()}
                                </div>
                                <div style="color: #e74c3c; font-size: 1.2em;">
                                    "✗ Грешки: "{move || error_count.get()}
                                </div>
                            </div>
                            <div style="margin-top: 20px;">
                                <button
                                    style="background: #95a5a6; color: white; border: none; padding: 12px 30px; border-radius: 4px; cursor: pointer; margin-right: 10px;"
                                    on:click=move |_| current_step.set(2)
                                >
                                    "Назад"
                                </button>
                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 12px 30px; border-radius: 4px; cursor: pointer;"
                                    on:click=do_import
                                    disabled=move || loading.get()
                                >
                                    {move || if loading.get() { "Импортиране..." } else { "Импортирай" }}
                                </button>
                            </div>
                        </div>
                    }.into_view(),
                    4 => view! {
                        <div>
                            <div style="font-size: 4em; color: #27ae60;">"✓"</div>
                            <h3>"Импортът е завършен!"</h3>

                            // Summary grid
                            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px; margin: 20px auto; max-width: 600px; text-align: left;">
                                <div style="padding: 15px; background: #e8f5e9; border-radius: 8px;">
                                    <div style="color: #7f8c8d; font-size: 0.9em;">"Контрагенти"</div>
                                    <div style="font-size: 1.3em; font-weight: bold;">
                                        {move || imported_contractors.get()}" / "{move || file_contractors_count.get()}
                                    </div>
                                    <div style="color: #7f8c8d; font-size: 0.8em;">"(импортирани / във файла)"</div>
                                </div>
                                <div style="padding: 15px; background: #e3f2fd; border-radius: 8px;">
                                    <div style="color: #7f8c8d; font-size: 0.9em;">"Документи"</div>
                                    <div style="font-size: 1.3em; font-weight: bold;">
                                        {move || imported_documents.get()}" / "{move || file_documents_count.get()}
                                    </div>
                                    <div style="color: #7f8c8d; font-size: 0.8em;">"(импортирани / във файла)"</div>
                                </div>
                                <div style="padding: 15px; background: #fff3e0; border-radius: 8px;">
                                    <div style="color: #7f8c8d; font-size: 0.9em;">"Счетоводни записи"</div>
                                    <div style="font-size: 1.3em; font-weight: bold;">
                                        {move || imported_journal_entries.get()}
                                    </div>
                                </div>
                                <div style="padding: 15px; background: #fce4ec; border-radius: 8px;">
                                    <div style="color: #7f8c8d; font-size: 0.9em;">"ДДС записи"</div>
                                    <div style="font-size: 1.3em; font-weight: bold;">
                                        {move || imported_vat_journal_entries.get()}
                                    </div>
                                </div>
                            </div>

                            // Show errors if any
                            {move || {
                                let errors = import_errors.get();
                                if !errors.is_empty() {
                                    view! {
                                        <div style="background: #ffebee; border: 1px solid #ef5350; border-radius: 8px; padding: 15px; margin: 15px auto; max-width: 600px; text-align: left;">
                                            <div style="color: #c62828; font-weight: bold; margin-bottom: 10px;">"Грешки ("{errors.len()}"):"</div>
                                            <ul style="margin: 0; padding-left: 20px; color: #c62828;">
                                                {errors.iter().take(10).map(|e| view! { <li>{e.clone()}</li> }).collect_view()}
                                            </ul>
                                            {if errors.len() > 10 {
                                                view! { <p style="color: #7f8c8d; margin-top: 10px;">"... и още "{errors.len() - 10}" грешки"</p> }.into_view()
                                            } else {
                                                view! { <span></span> }.into_view()
                                            }}
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}

                            // Show warnings if any
                            {move || {
                                let warnings = import_warnings.get();
                                if !warnings.is_empty() {
                                    view! {
                                        <div style="background: #fff8e1; border: 1px solid #ffb300; border-radius: 8px; padding: 15px; margin: 15px auto; max-width: 600px; text-align: left;">
                                            <div style="color: #f57f17; font-weight: bold; margin-bottom: 10px;">"Предупреждения ("{warnings.len()}"):"</div>
                                            <ul style="margin: 0; padding-left: 20px; color: #f57f17;">
                                                {warnings.iter().take(10).map(|w| view! { <li>{w.clone()}</li> }).collect_view()}
                                            </ul>
                                            {if warnings.len() > 10 {
                                                view! { <p style="color: #7f8c8d; margin-top: 10px;">"... и още "{warnings.len() - 10}" предупреждения"</p> }.into_view()
                                            } else {
                                                view! { <span></span> }.into_view()
                                            }}
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}

                            <button
                                style="background: #3498db; color: white; border: none; padding: 12px 30px; border-radius: 4px; cursor: pointer; margin-top: 20px;"
                                on:click=move |_| {
                                    current_step.set(1);
                                    file_name.set(String::new());
                                    file_content.set(String::new());
                                    preview_data.set(PreviewResponse::default());
                                    imported_contractors.set(0);
                                    imported_documents.set(0);
                                    imported_journal_entries.set(0);
                                    imported_vat_journal_entries.set(0);
                                    file_contractors_count.set(0);
                                    file_documents_count.set(0);
                                    import_errors.set(Vec::new());
                                    import_warnings.set(Vec::new());
                                }
                            >
                                "Нов импорт"
                            </button>
                        </div>
                    }.into_view(),
                    _ => view! { <div></div> }.into_view()
                }}
            </div>
        </Layout>
    }
}
