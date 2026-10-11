use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete, company_api_url};
use crate::constants::{is_generic_eik, strip_eu_vat_prefix};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Counterpart {
    pub id: i64,
    pub name: String,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub counterpart_type: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViesResult {
    pub valid: bool,
    pub name: String,
    pub address: String,
    pub country_code: String,
    pub vat_number: String,
    #[serde(default)]
    pub eik: String,
}

fn counterpart_eik_value(eik: &str, vat: &str) -> String {
    let from_eik = strip_eu_vat_prefix(eik);
    if from_eik.is_empty() {
        strip_eu_vat_prefix(vat)
    } else {
        from_eik
    }
}

fn find_duplicate_by_eik(
    list: &[Counterpart],
    eik: &str,
    exclude_id: Option<i64>,
) -> Option<Counterpart> {
    if eik.is_empty() || is_generic_eik(eik) {
        return None;
    }
    list.iter()
        .find(|c| {
            if exclude_id.map(|id| c.id == id).unwrap_or(false) {
                return false;
            }
            c.eik
                .as_ref()
                .map(|stored| strip_eu_vat_prefix(stored) == eik)
                .unwrap_or(false)
        })
        .cloned()
}

fn duplicate_eik_message(existing: &Counterpart, eik: &str) -> String {
    format!(
        "Контрагент с ЕИК {} вече съществува: {}",
        eik, existing.name
    )
}

#[component]
pub fn Counterparts() -> impl IntoView {
    let counterparts = RwSignal::new(Vec::<Counterpart>::new());
    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let search_query = RwSignal::new(String::new());

    // New counterpart modal
    let show_new_modal = RwSignal::new(false);
    let saving = RwSignal::new(false);

    // Form fields
    let form_vat_number = RwSignal::new(String::new());
    let form_name = RwSignal::new(String::new());
    let form_eik = RwSignal::new(String::new());
    let form_address = RwSignal::new(String::new());
    let form_city = RwSignal::new(String::new());
    let form_country_code = RwSignal::new(String::from("BG"));
    let form_email = RwSignal::new(String::new());
    let form_phone = RwSignal::new(String::new());
    let form_type = RwSignal::new(String::from("both"));

    // VIES lookup
    let vies_loading = RwSignal::new(false);
    let vies_result = RwSignal::new(Option::<ViesResult>::None);
    let vies_error = RwSignal::new(String::new());
    let duplicate_msg = RwSignal::new(String::new());
    let form_error = RwSignal::new(String::new());

    // Delete modal
    let show_delete_modal = RwSignal::new(false);
    let delete_counterpart = RwSignal::new(Option::<Counterpart>::None);
    let deleting = RwSignal::new(false);

    // Edit modal
    let show_edit_modal = RwSignal::new(false);
    let edit_counterpart = RwSignal::new(Option::<Counterpart>::None);

    // Load counterparts
    let reload = move || {
        loading.set(true);
        spawn_local(async move {
            match api_get(&format!("{}/counterparts", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let cps: Vec<Counterpart> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        counterparts.set(cps);
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
        reload();
    });

    // Reset form
    let reset_form = move || {
        form_vat_number.set(String::new());
        form_name.set(String::new());
        form_eik.set(String::new());
        form_address.set(String::new());
        form_city.set(String::new());
        form_country_code.set("BG".to_string());
        form_email.set(String::new());
        form_phone.set(String::new());
        form_type.set("both".to_string());
        vies_result.set(None);
        vies_error.set(String::new());
        duplicate_msg.set(String::new());
        form_error.set(String::new());
    };

    // VIES lookup
    let do_vies_lookup = move |_| {
        let vat = form_vat_number.get();
        if vat.len() < 3 {
            vies_error.set("ДДС номерът трябва да е поне 3 символа".to_string());
            return;
        }

        vies_loading.set(true);
        vies_error.set(String::new());
        vies_result.set(None);

        spawn_local(async move {
            let payload = serde_json::json!({ "vat_number": vat });
            match api_post("/api/vies/check", &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Some(data) = response.get("data") {
                            if let Ok(result) = serde_json::from_value::<ViesResult>(data.clone()) {
                                if result.valid {
                                    let eik = if !result.eik.is_empty() {
                                        strip_eu_vat_prefix(&result.eik)
                                    } else {
                                        strip_eu_vat_prefix(&result.vat_number)
                                    };
                                    form_name.set(result.name.clone());
                                    form_address.set(result.address.clone());
                                    form_country_code.set(result.country_code.clone());
                                    form_eik.set(eik.clone());
                                    if !result.country_code.is_empty() && !result.vat_number.is_empty() {
                                        form_vat_number.set(format!(
                                            "{}{}",
                                            result.country_code, result.vat_number
                                        ));
                                    }
                                    duplicate_msg.set(String::new());
                                    if let Some(existing) = find_duplicate_by_eik(
                                        &counterparts.get(),
                                        &eik,
                                        None,
                                    ) {
                                        duplicate_msg.set(duplicate_eik_message(&existing, &eik));
                                    }
                                    vies_result.set(Some(result));
                                } else {
                                    vies_error.set("ДДС номерът не е валиден".to_string());
                                }
                            }
                        }
                    } else {
                        let err = response.get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("Грешка при проверка");
                        vies_error.set(err.to_string());
                    }
                }
                Err(e) => {
                    vies_error.set(format!("Грешка: {}", e));
                }
            }
            vies_loading.set(false);
        });
    };

    // Save counterpart
    let save_counterpart = move |_| {
        let name = form_name.get();
        if name.is_empty() {
            form_error.set("Името е задължително".to_string());
            error_msg.set("Името е задължително".to_string());
            return;
        }

        let eik = counterpart_eik_value(&form_eik.get(), &form_vat_number.get());
        form_eik.set(eik.clone());
        if let Some(existing) = find_duplicate_by_eik(&counterparts.get(), &eik, None) {
            let msg = duplicate_eik_message(&existing, &eik);
            duplicate_msg.set(msg.clone());
            form_error.set(msg.clone());
            error_msg.set(msg);
            return;
        }

        saving.set(true);
        error_msg.set(String::new());
        form_error.set(String::new());

        let payload = serde_json::json!({
            "counterpart": {
                "name": name,
                "eik": eik,
                "vat_number": form_vat_number.get(),
                "address": form_address.get(),
                "city": form_city.get(),
                "country_code": form_country_code.get(),
                "email": form_email.get(),
                "phone": form_phone.get(),
                "counterpart_type": form_type.get(),
                "is_active": true
            }
        });

        spawn_local(async move {
            match api_post(&format!("{}/counterparts", company_api_url()), &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Контрагентът е създаден успешно".to_string());
                        show_new_modal.set(false);
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
                        let msg = format!("Грешка: {}", err);
                        error_msg.set(msg.clone());
                        form_error.set(msg);
                    }
                }
                Err(e) => {
                    let msg = format!("Грешка: {}", e);
                    error_msg.set(msg.clone());
                    form_error.set(msg);
                }
            }
            saving.set(false);
        });
    };

    // Delete counterpart
    let do_delete = move |_| {
        if let Some(cp) = delete_counterpart.get() {
            deleting.set(true);
            let cp_id = cp.id;
            spawn_local(async move {
                match api_delete(&format!("{}/counterparts/{}", company_api_url(), cp_id)).await {
                    Ok(response) => {
                        if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            success_msg.set("Контрагентът е изтрит успешно".to_string());
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
                delete_counterpart.set(None);
                deleting.set(false);
            });
        }
    };

    // Open edit modal
    let open_edit = move |cp: Counterpart| {
        form_name.set(cp.name.clone());
        form_eik.set(cp.eik.clone().unwrap_or_default());
        form_vat_number.set(cp.vat_number.clone().unwrap_or_default());
        form_address.set(cp.address.clone().unwrap_or_default());
        form_city.set(cp.city.clone().unwrap_or_default());
        form_country_code.set(cp.country_code.clone().unwrap_or_else(|| "BG".to_string()));
        form_email.set(cp.email.clone().unwrap_or_default());
        form_phone.set(cp.phone.clone().unwrap_or_default());
        form_type.set(cp.counterpart_type.clone());
        duplicate_msg.set(String::new());
        form_error.set(String::new());
        edit_counterpart.set(Some(cp));
        show_edit_modal.set(true);
    };

    // Save edit
    let save_edit = move |_| {
        if let Some(cp) = edit_counterpart.get() {
            let name = form_name.get();
            if name.is_empty() {
                form_error.set("Името е задължително".to_string());
                error_msg.set("Името е задължително".to_string());
                return;
            }

            let eik = counterpart_eik_value(&form_eik.get(), &form_vat_number.get());
            form_eik.set(eik.clone());
            if let Some(existing) = find_duplicate_by_eik(&counterparts.get(), &eik, Some(cp.id)) {
                let msg = duplicate_eik_message(&existing, &eik);
                duplicate_msg.set(msg.clone());
                form_error.set(msg.clone());
                error_msg.set(msg);
                return;
            }

            saving.set(true);
            error_msg.set(String::new());
            form_error.set(String::new());
            let cp_id = cp.id;

            let payload = serde_json::json!({
                "counterpart": {
                    "name": name,
                    "eik": eik,
                    "vat_number": form_vat_number.get(),
                    "address": form_address.get(),
                    "city": form_city.get(),
                    "country_code": form_country_code.get(),
                    "email": form_email.get(),
                    "phone": form_phone.get(),
                    "counterpart_type": form_type.get(),
                    "is_active": true
                }
            });

            spawn_local(async move {
                match api_put(&format!("{}/counterparts/{}", company_api_url(), cp_id), &payload).await {
                    Ok(response) => {
                        if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            success_msg.set("Контрагентът е редактиран успешно".to_string());
                            show_edit_modal.set(false);
                            edit_counterpart.set(None);
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
                            let msg = format!("Грешка: {}", err);
                            error_msg.set(msg.clone());
                            form_error.set(msg);
                        }
                    }
                    Err(e) => {
                        let msg = format!("Грешка: {}", e);
                        error_msg.set(msg.clone());
                        form_error.set(msg);
                    }
                }
                saving.set(false);
            });
        }
    };

    // Filtered counterparts
    let filtered = move || {
        let query = search_query.get().to_lowercase();
        counterparts.get()
            .into_iter()
            .filter(|c| {
                if query.is_empty() {
                    return true;
                }
                c.name.to_lowercase().contains(&query) ||
                c.eik.as_ref().map(|e| e.to_lowercase().contains(&query)).unwrap_or(false) ||
                c.vat_number.as_ref().map(|v| v.to_lowercase().contains(&query)).unwrap_or(false) ||
                c.city.as_ref().map(|ct| ct.to_lowercase().contains(&query)).unwrap_or(false)
            })
            .collect::<Vec<_>>()
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <h1>"Контрагенти"</h1>
                <button
                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                    on:click=move |_| show_new_modal.set(true)
                >
                    "+ Нов контрагент"
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

            // Search
            <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                <input
                    type="text"
                    placeholder="Търсене по име, ЕИК, ДДС номер, град..."
                    style="width: 100%; padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                    on:input=move |ev| search_query.set(event_target_value(&ev))
                    prop:value=move || search_query.get()
                />
            </div>

            // Loading
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

            // Table
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_any();
                }

                let list = filtered();
                if list.is_empty() {
                    return view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 1.1em;">
                                {if counterparts.get().is_empty() {
                                    "Няма добавени контрагенти"
                                } else {
                                    "Няма намерени резултати"
                                }}
                            </p>
                        </div>
                    }.into_any();
                }

                view! {
                    <div style="background: white; border-radius: 8px; margin-top: 20px; overflow: hidden;">
                        <table style="width: 100%; border-collapse: collapse;">
                            <thead style="background: #34495e; color: white;">
                                <tr>
                                    <th style="padding: 12px; text-align: left;">"Име"</th>
                                    <th style="padding: 12px; text-align: left;">"ЕИК"</th>
                                    <th style="padding: 12px; text-align: left;">"ДДС номер"</th>
                                    <th style="padding: 12px; text-align: left;">"Град"</th>
                                    <th style="padding: 12px; text-align: left;">"Тип"</th>
                                    <th style="padding: 12px; text-align: center; width: 120px;">"Действия"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {list.into_iter().map(|cp| {
                                    let cp_for_delete = cp.clone();
                                    let cp_for_edit = cp.clone();
                                    let type_label = match cp.counterpart_type.as_str() {
                                        "customer" => "Клиент",
                                        "supplier" => "Доставчик",
                                        "both" => "И двете",
                                        _ => "Друго",
                                    };
                                    view! {
                                        <tr style="border-bottom: 1px solid #ecf0f1;">
                                            <td style="padding: 12px;">
                                                <div style="font-weight: 500;">{cp.name.clone()}</div>
                                                {cp.email.as_ref().map(|e| view! {
                                                    <div style="color: #7f8c8d; font-size: 12px;">{e.clone()}</div>
                                                })}
                                            </td>
                                            <td style="padding: 12px; font-family: monospace;">
                                                {cp.eik.clone().unwrap_or_else(|| "-".to_string())}
                                            </td>
                                            <td style="padding: 12px; font-family: monospace;">
                                                {cp.vat_number.clone().unwrap_or_else(|| "-".to_string())}
                                            </td>
                                            <td style="padding: 12px;">
                                                {cp.city.clone().unwrap_or_else(|| "-".to_string())}
                                            </td>
                                            <td style="padding: 12px;">
                                                <span style="display: inline-block; padding: 4px 8px; border-radius: 12px; font-size: 11px; background: #ecf0f1; color: #34495e;">
                                                    {type_label}
                                                </span>
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                <button
                                                    style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-right: 5px;"
                                                    title="Редактиране"
                                                    on:click=move |_| {
                                                        open_edit(cp_for_edit.clone());
                                                    }
                                                >
                                                    "✏"
                                                </button>
                                                <button
                                                    style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                    title="Изтриване"
                                                    on:click=move |_| {
                                                        delete_counterpart.set(Some(cp_for_delete.clone()));
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
                }.into_any()
            }}

            // New counterpart modal
            {move || {
                if show_new_modal.get() {
                    let vies_res = vies_result.get();
                    let vies_err = vies_error.get();
                    let is_vies_loading = vies_loading.get();

                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: flex-start; justify-content: center; z-index: 1000; padding: 20px; overflow-y: auto;">
                            <div style="background: white; border-radius: 8px; padding: 30px; max-width: 700px; width: 100%; margin: 20px 0;">
                                <h3 style="margin-top: 0;">"Нов контрагент"</h3>

                                {move || {
                                    let err = form_error.get();
                                    if err.is_empty() {
                                        view! { <span></span> }.into_any()
                                    } else {
                                        view! {
                                            <div style="background: #f8d7da; color: #721c24; padding: 12px; border-radius: 4px; margin-bottom: 15px;">
                                                {err}
                                            </div>
                                        }.into_any()
                                    }
                                }}

                                // VIES Lookup section
                                <div style="background: #f8f9fa; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                                    <label style="display: block; font-size: 14px; font-weight: 500; margin-bottom: 10px;">
                                        "🔍 Търсене по ДДС номер (VIES)"
                                    </label>
                                    <p style="margin: 0 0 10px 0; font-size: 12px; color: #7f8c8d;">
                                        "Буквите BG, IT, ES и др. се премахват; номерът се записва в полето ЕИК."
                                    </p>
                                    <div style="display: flex; gap: 10px;">
                                        <input
                                            type="text"
                                            placeholder="напр. BG123456789"
                                            style="flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_vat_number.set(event_target_value(&ev))
                                            prop:value=move || form_vat_number.get()
                                        />
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            disabled=move || is_vies_loading
                                            on:click=do_vies_lookup
                                        >
                                            {move || if vies_loading.get() { "Търсене..." } else { "Провери" }}
                                        </button>
                                    </div>

                                    // VIES result
                                    {if let Some(res) = vies_res {
                                        let stored_eik = if res.eik.is_empty() {
                                            res.vat_number.clone()
                                        } else {
                                            res.eik.clone()
                                        };
                                        view! {
                                            <div style="margin-top: 15px; padding: 15px; background: #d4edda; border-radius: 4px; color: #155724;">
                                                <strong>"✓ Валиден ДДС номер"</strong>
                                                <div style="margin-top: 5px;">"Име: "{res.name}</div>
                                                <div>"Адрес: "{res.address}</div>
                                                <div>"ЕИК: "{stored_eik}" (без "{res.country_code}")"</div>
                                            </div>
                                        }.into_any()
                                    } else if !vies_err.is_empty() {
                                        view! {
                                            <div style="margin-top: 15px; padding: 15px; background: #f8d7da; border-radius: 4px; color: #721c24;">
                                                {vies_err}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    {move || {
                                        let dup = duplicate_msg.get();
                                        if dup.is_empty() {
                                            view! { <span></span> }.into_any()
                                        } else {
                                            view! {
                                                <div style="margin-top: 15px; padding: 15px; background: #fff3cd; border-radius: 4px; color: #856404;">
                                                    {dup}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>

                                // Form fields
                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Име *"</label>
                                        <input
                                            type="text"
                                            placeholder="Име на фирмата"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_name.set(event_target_value(&ev))
                                            prop:value=move || form_name.get()
                                        />
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"ЕИК"</label>
                                        <input
                                            type="text"
                                            placeholder="123456789"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| {
                                                form_eik.set(event_target_value(&ev));
                                                duplicate_msg.set(String::new());
                                                form_error.set(String::new());
                                            }
                                            prop:value=move || form_eik.get()
                                        />
                                        <div style="font-size: 11px; color: #7f8c8d; margin-top: 4px;">
                                            "Извън ЕС / неизвестни: 999999999 или 9999999999999 (може да се повтаря)"
                                        </div>
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Държава"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            prop:value=move || form_country_code.get()
                                            on:change=move |ev| form_country_code.set(event_target_value(&ev))
                                        >
                                            <option value="BG">"България"</option>
                                            <option value="DE">"Германия"</option>
                                            <option value="AT">"Австрия"</option>
                                            <option value="BE">"Белгия"</option>
                                            <option value="FR">"Франция"</option>
                                            <option value="IT">"Италия"</option>
                                            <option value="ES">"Испания"</option>
                                            <option value="NL">"Нидерландия"</option>
                                            <option value="PL">"Полша"</option>
                                            <option value="RO">"Румъния"</option>
                                            <option value="GR">"Гърция"</option>
                                            <option value="CZ">"Чехия"</option>
                                            <option value="HU">"Унгария"</option>
                                            <option value="PT">"Португалия"</option>
                                            <option value="SE">"Швеция"</option>
                                            <option value="IE">"Ирландия"</option>
                                            <option value="HR">"Хърватия"</option>
                                            <option value="SK">"Словакия"</option>
                                            <option value="SI">"Словения"</option>
                                            <option value="DK">"Дания"</option>
                                            <option value="FI">"Финландия"</option>
                                            <option value="LT">"Литва"</option>
                                            <option value="LV">"Латвия"</option>
                                            <option value="EE">"Естония"</option>
                                            <option value="CY">"Кипър"</option>
                                            <option value="LU">"Люксембург"</option>
                                            <option value="MT">"Малта"</option>
                                            <option value="XX">"Извън ЕС"</option>
                                        </select>
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Град"</label>
                                        <input
                                            type="text"
                                            placeholder="София"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_city.set(event_target_value(&ev))
                                            prop:value=move || form_city.get()
                                        />
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Тип"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| form_type.set(event_target_value(&ev))
                                        >
                                            <option value="both" selected=move || form_type.get() == "both">"Клиент и Доставчик"</option>
                                            <option value="customer" selected=move || form_type.get() == "customer">"Само Клиент"</option>
                                            <option value="supplier" selected=move || form_type.get() == "supplier">"Само Доставчик"</option>
                                        </select>
                                    </div>

                                    <div style="grid-column: span 2;">
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Адрес"</label>
                                        <input
                                            type="text"
                                            placeholder="Улица, номер, етаж..."
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_address.set(event_target_value(&ev))
                                            prop:value=move || form_address.get()
                                        />
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Email"</label>
                                        <input
                                            type="email"
                                            placeholder="email@example.com"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_email.set(event_target_value(&ev))
                                            prop:value=move || form_email.get()
                                        />
                                    </div>

                                    <div>
                                        <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Телефон"</label>
                                        <input
                                            type="tel"
                                            placeholder="+359 888 123 456"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:input=move |ev| form_phone.set(event_target_value(&ev))
                                            prop:value=move || form_phone.get()
                                        />
                                    </div>
                                </div>

                                // Action buttons
                                <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 25px;">
                                    <button
                                        style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| {
                                            show_new_modal.set(false);
                                            reset_form();
                                        }
                                    >
                                        "Отказ"
                                    </button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        disabled=move || saving.get() || !duplicate_msg.get().is_empty()
                                        on:click=save_counterpart
                                    >
                                        {move || if saving.get() { "Запазване..." } else { "Запази" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Delete confirmation modal
            {move || {
                if show_delete_modal.get() {
                    if let Some(cp) = delete_counterpart.get() {
                        view! {
                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                                <div style="background: white; border-radius: 8px; padding: 30px; max-width: 500px; width: 90%;">
                                    <h3 style="margin-top: 0; color: #e74c3c;">"Потвърждение за изтриване"</h3>

                                    <p>"Сигурни ли сте, че искате да изтриете контрагента:"</p>
                                    <p style="font-weight: 500; font-size: 1.1em;">{cp.name.clone()}</p>

                                    <p style="color: #e74c3c; font-size: 13px;">"Това действие не може да бъде отменено!"</p>

                                    <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px;">
                                        <button
                                            style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| {
                                                show_delete_modal.set(false);
                                                delete_counterpart.set(None);
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
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Edit counterpart modal
            {move || {
                if show_edit_modal.get() {
                    if let Some(cp) = edit_counterpart.get() {
                        view! {
                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: flex-start; justify-content: center; z-index: 1000; padding: 20px; overflow-y: auto;">
                                <div style="background: white; border-radius: 8px; padding: 30px; max-width: 700px; width: 100%; margin: 20px 0;">
                                    <h3 style="margin-top: 0;">"Редактиране на контрагент"</h3>
                                    <p style="color: #7f8c8d; margin-bottom: 20px;">{cp.name.clone()}</p>

                                    {move || {
                                        let err = form_error.get();
                                        if err.is_empty() {
                                            view! { <span></span> }.into_any()
                                        } else {
                                            view! {
                                                <div style="background: #f8d7da; color: #721c24; padding: 12px; border-radius: 4px; margin-bottom: 15px;">
                                                    {err}
                                                </div>
                                            }.into_any()
                                        }
                                    }}

                                    // Form fields
                                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                        <div style="grid-column: span 2;">
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Име *"</label>
                                            <input
                                                type="text"
                                                placeholder="Име на фирмата"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:input=move |ev| form_name.set(event_target_value(&ev))
                                                prop:value=move || form_name.get()
                                            />
                                        </div>

                                        <div>
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"ЕИК"</label>
                                            <input
                                                type="text"
                                                placeholder="123456789"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:input=move |ev| {
                                                    form_eik.set(event_target_value(&ev));
                                                    duplicate_msg.set(String::new());
                                                    form_error.set(String::new());
                                                }
                                                prop:value=move || form_eik.get()
                                            />
                                            <div style="font-size: 11px; color: #7f8c8d; margin-top: 4px;">
                                                "Извън ЕС / неизвестни: 999999999 или 9999999999999 (може да се повтаря)"
                                            </div>
                                        </div>

                                        <div>
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"ДДС номер"</label>
                                            <input
                                                type="text"
                                                placeholder="BG123456789"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:input=move |ev| form_vat_number.set(event_target_value(&ev))
                                                prop:value=move || form_vat_number.get()
                                            />
                                        </div>

                                        <div>
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Държава"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                prop:value=move || form_country_code.get()
                                                on:change=move |ev| form_country_code.set(event_target_value(&ev))
                                            >
                                                <option value="BG">"България"</option>
                                                <option value="DE">"Германия"</option>
                                                <option value="AT">"Австрия"</option>
                                                <option value="BE">"Белгия"</option>
                                                <option value="FR">"Франция"</option>
                                                <option value="IT">"Италия"</option>
                                                <option value="ES">"Испания"</option>
                                                <option value="NL">"Нидерландия"</option>
                                                <option value="PL">"Полша"</option>
                                                <option value="RO">"Румъния"</option>
                                                <option value="GR">"Гърция"</option>
                                                <option value="CZ">"Чехия"</option>
                                                <option value="HU">"Унгария"</option>
                                                <option value="PT">"Португалия"</option>
                                                <option value="SE">"Швеция"</option>
                                                <option value="IE">"Ирландия"</option>
                                                <option value="HR">"Хърватия"</option>
                                                <option value="SK">"Словакия"</option>
                                                <option value="SI">"Словения"</option>
                                                <option value="DK">"Дания"</option>
                                                <option value="FI">"Финландия"</option>
                                                <option value="LT">"Литва"</option>
                                                <option value="LV">"Латвия"</option>
                                                <option value="EE">"Естония"</option>
                                                <option value="CY">"Кипър"</option>
                                                <option value="LU">"Люксембург"</option>
                                                <option value="MT">"Малта"</option>
                                                <option value="XX">"Извън ЕС"</option>
                                            </select>
                                        </div>

                                        <div>
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Град"</label>
                                            <input
                                                type="text"
                                                placeholder="София"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:input=move |ev| form_city.set(event_target_value(&ev))
                                                prop:value=move || form_city.get()
                                            />
                                        </div>

                                        <div>
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Тип"</label>
                                            <select
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:change=move |ev| form_type.set(event_target_value(&ev))
                                            >
                                                <option value="both" selected=move || form_type.get() == "both">"Клиент и Доставчик"</option>
                                                <option value="customer" selected=move || form_type.get() == "customer">"Само Клиент"</option>
                                                <option value="supplier" selected=move || form_type.get() == "supplier">"Само Доставчик"</option>
                                            </select>
                                        </div>

                                        <div style="grid-column: span 2;">
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Адрес"</label>
                                            <input
                                                type="text"
                                                placeholder="Улица, номер, етаж..."
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:input=move |ev| form_address.set(event_target_value(&ev))
                                                prop:value=move || form_address.get()
                                            />
                                        </div>

                                        <div>
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Email"</label>
                                            <input
                                                type="email"
                                                placeholder="email@example.com"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:input=move |ev| form_email.set(event_target_value(&ev))
                                                prop:value=move || form_email.get()
                                            />
                                        </div>

                                        <div>
                                            <label style="display: block; font-size: 13px; color: #7f8c8d; margin-bottom: 5px;">"Телефон"</label>
                                            <input
                                                type="tel"
                                                placeholder="+359 888 123 456"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:input=move |ev| form_phone.set(event_target_value(&ev))
                                                prop:value=move || form_phone.get()
                                            />
                                        </div>
                                    </div>

                                    // Action buttons
                                    <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 25px;">
                                        <button
                                            style="background: #ecf0f1; color: #34495e; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=move |_| {
                                                show_edit_modal.set(false);
                                                edit_counterpart.set(None);
                                                reset_form();
                                            }
                                        >
                                            "Отказ"
                                        </button>
                                        <button
                                            style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            disabled=move || saving.get() || !duplicate_msg.get().is_empty()
                                            on:click=save_edit
                                        >
                                            {move || if saving.get() { "Запазване..." } else { "Запази" }}
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
        </Layout>
    }
}
