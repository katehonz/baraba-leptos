use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use crate::api::{api_delete, api_get, api_post, api_put, get_selected_company_id};
use crate::context::{use_company_context, use_user_context};

#[derive(Clone, Debug, Default, Deserialize)]
struct NumberCatalogItem {
    code: String,
    name: String,
    #[serde(default)]
    tax: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct BoundType {
    family: String,
    document_type: String,
    name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct SequenceRow {
    id: i64,
    code: String,
    name: String,
    #[serde(default)]
    is_tax: bool,
    #[serde(default)]
    next_formatted: String,
    #[serde(default)]
    issued_max: i64,
    #[serde(default)]
    deletable: bool,
    #[serde(default)]
    document_types: Vec<BoundType>,
}

fn pad10(n: i64) -> String {
    format!("{:010}", n)
}

fn parse_next(raw: &str) -> Result<i64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Въведете следващ номер".into());
    }
    if !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err("Номерът трябва да съдържа само цифри".into());
    }
    if trimmed.len() > 10 {
        return Err("Номерът е най-много 10 цифри".into());
    }
    let n: i64 = trimmed.parse().map_err(|_| "Невалиден номер".to_string())?;
    if n < 1 {
        return Err("Номерът трябва да е поне 0000000001".into());
    }
    Ok(n)
}

fn is_warehouse(seq: &SequenceRow) -> bool {
    if seq.document_types.is_empty() {
        return seq.code == "WH_INTERNAL";
    }
    seq.document_types.iter().all(|t| t.family == "warehouse")
}

fn invoice_kinds(seq: &SequenceRow) -> Vec<String> {
    seq.document_types
        .iter()
        .filter(|t| t.family == "invoice_sale")
        .map(|t| t.document_type.clone())
        .collect()
}

fn owner_name(sequences: &[SequenceRow], code: &str) -> String {
    sequences
        .iter()
        .find(|seq| invoice_kinds(seq).iter().any(|kind| kind == code))
        .map(|seq| seq.name.clone())
        .unwrap_or_else(|| "без интервал".into())
}

#[component]
pub fn DocumentNumbersTab() -> impl IntoView {
    let ctx = use_company_context();
    let user_ctx = use_user_context();
    let loading = RwSignal::new(true);
    let saving = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let sequences = RwSignal::new(Vec::<SequenceRow>::new());
    let catalog = RwSignal::new(Vec::<NumberCatalogItem>::new());
    let reload_tick = RwSignal::new(0i32);
    let new_name = RwSignal::new(String::new());
    let new_next = RwSignal::new(String::from("0000000001"));
    let new_types = RwSignal::new(Vec::<String>::new());

    let company_id = move || {
        ctx.selected_company_id
            .get()
            .filter(|id| *id > 0)
            .or_else(get_selected_company_id)
            .unwrap_or(0)
    };

    let can_write = move || {
        let cid = company_id();
        if cid <= 0 {
            return false;
        }
        if user_ctx.is_super_admin() {
            return true;
        }
        matches!(user_ctx.role_for_company(cid).as_deref(), Some("admin" | "accountant"))
    };

    Effect::new(move |_| {
        let cid = company_id();
        let _tick = reload_tick.get();
        new_name.set(String::new());
        new_next.set("0000000001".into());
        new_types.set(Vec::new());
        if cid <= 0 {
            sequences.set(Vec::new());
            catalog.set(Vec::new());
            loading.set(false);
            return;
        }
        loading.set(true);
        error_msg.set(String::new());
        spawn_local(async move {
            match api_get(&format!("/api/companies/{}/sequences", cid)).await {
                Ok(data) => {
                    let rows = data
                        .get("data")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect()
                        })
                        .unwrap_or_default();
                    let items = data
                        .get("catalog")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect()
                        })
                        .unwrap_or_default();
                    sequences.set(rows);
                    catalog.set(items);
                }
                Err(e) => error_msg.set(e),
            }
            loading.set(false);
        });
    });

    let create_interval = move |_| {
        let cid = company_id();
        let name = new_name.get().trim().to_string();
        let types = new_types.get();
        if name.is_empty() {
            error_msg.set("Въведете име на интервала".into());
            return;
        }
        if types.is_empty() {
            error_msg.set("Изберете поне един вид документ".into());
            return;
        }
        let next_number = match parse_next(&new_next.get()) {
            Ok(n) => n,
            Err(e) => {
                error_msg.set(e);
                return;
            }
        };
        error_msg.set(String::new());
        success_msg.set(String::new());
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "name": name,
                "next_number": next_number,
                "document_types": types,
            });
            match api_post(&format!("/api/companies/{}/sequences", cid), &body).await {
                Ok(_) => {
                    success_msg.set("Интервалът е създаден. Избраните документи вече ползват тази номерация.".into());
                    reload_tick.update(|n| *n += 1);
                }
                Err(e) => error_msg.set(e),
            }
            saving.set(false);
        });
    };

    view! {
        <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 860px;">
            <h3 style="margin: 0 0 8px 0; color: #2c3e50;">"Номерация на документи"</h3>
            <p style="color: #7f8c8d; margin: 0 0 18px 0; font-size: 14px; line-height: 1.5;">
                "Изберете група документи и им дайте отделен интервал. Номерът е 10 цифри с водещи нули, например "
                <strong>"0000000001"</strong>
                ". Проформата не е данъчен документ и има свой интервал. Фактура, дебитно и кредитно известие ползват един, протоколите и отчетите — друг."
            </p>

            {move || {
                let err = error_msg.get();
                if err.is_empty() {
                    view! { <span></span> }.into_any()
                } else {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 12px 15px; border-radius: 8px; margin-bottom: 16px;">
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
                        <div style="background: #27ae60; color: white; padding: 12px 15px; border-radius: 8px; margin-bottom: 16px;">
                            {msg}
                        </div>
                    }.into_any()
                }
            }}

            {move || {
                if loading.get() {
                    return view! { <p style="color: #7f8c8d;">"Зареждане..."</p> }.into_any();
                }
                let cid = company_id();
                let write = can_write();
                let rows = sequences.get();
                view! {
                    <div>
                        {rows.into_iter().map(|seq| {
                            view! {
                                <SequenceCard
                                    seq=seq
                                    can_write=write
                                    company_id=cid
                                    reload_tick=reload_tick
                                    error_msg=error_msg
                                    success_msg=success_msg
                                    saving=saving
                                />
                            }
                        }).collect_view()}

                        <Show when=move || can_write()>
                            <div style="border: 1px dashed #3498db; border-radius: 8px; padding: 18px; margin-top: 8px;">
                                <h4 style="margin: 0 0 6px 0; color: #2c3e50;">"Нов интервал"</h4>
                                <p style="margin: 0 0 14px 0; color: #7f8c8d; font-size: 13px;">
                                    "Маркирайте видовете, които минават в него. Те напускат досегашния си интервал."
                                </p>
                                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 14px;">
                                    <label style="font-size: 14px;">
                                        "Име"
                                        <input
                                            type="text"
                                            placeholder="Например: Протоколи по чл. 117"
                                            style="display: block; width: 100%; margin-top: 6px; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || new_name.get()
                                            on:input=move |ev| new_name.set(event_target_value(&ev))
                                        />
                                    </label>
                                    <label style="font-size: 14px;">
                                        "Начален номер"
                                        <input
                                            type="text"
                                            inputmode="numeric"
                                            maxlength="10"
                                            style="display: block; width: 100%; margin-top: 6px; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-variant-numeric: tabular-nums;"
                                            prop:value=move || new_next.get()
                                            on:input=move |ev| new_next.set(event_target_value(&ev))
                                        />
                                    </label>
                                </div>
                                <p style="margin: 0 0 12px 0; color: #34495e; font-size: 13px;">
                                    {move || match parse_next(&new_next.get()) {
                                        Ok(n) => format!("Първият номер ще бъде {}", pad10(n)),
                                        Err(_) => "Въведете число от 1 до 10 цифри".into(),
                                    }}
                                </p>
                                <div style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 14px;">
                                    {move || {
                                        let selected = new_types.get();
                                        let rows = sequences.get();
                                        catalog.get().into_iter().map(|item| {
                                            let code = item.code.clone();
                                            let checked = selected.iter().any(|c| c == &code);
                                            let owner = owner_name(&rows, &code);
                                            let code_for_change = code.clone();
                                            view! {
                                                <label style="display: flex; gap: 10px; align-items: flex-start; font-size: 14px; line-height: 1.4;">
                                                    <input
                                                        type="checkbox"
                                                        style="margin-top: 3px;"
                                                        prop:checked=checked
                                                        on:change=move |_| {
                                                            let code = code_for_change.clone();
                                                            new_types.update(|list| {
                                                                if let Some(i) = list.iter().position(|c| c == &code) {
                                                                    list.remove(i);
                                                                } else {
                                                                    list.push(code);
                                                                }
                                                            });
                                                        }
                                                    />
                                                    <span>
                                                        <strong>{item.name.clone()}</strong>
                                                        {if item.tax { "" } else { " — не е данъчен документ" }}
                                                        <span style="color: #7f8c8d;">{" · сега в "}{owner}</span>
                                                    </span>
                                                </label>
                                            }
                                        }).collect_view()
                                    }}
                                </div>
                                {move || {
                                    let selected = new_types.get();
                                    let mixed = selected.iter().any(|c| c == "PROFORMA") && selected.len() > 1;
                                    if mixed {
                                        view! {
                                            <p style="background: #fff6e5; color: #8a5a00; padding: 10px 12px; border-radius: 6px; font-size: 13px;">
                                                "Проформата не е данъчен документ. Ако я сложите при фактурите, ще дели номерата им."
                                            </p>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }
                                }}
                                <button
                                    style="background: #3498db; color: white; border: none; padding: 12px 22px; border-radius: 4px; cursor: pointer;"
                                    on:click=create_interval
                                    disabled=move || saving.get()
                                >
                                    {move || if saving.get() { "Запазване..." } else { "Създай интервал" }}
                                </button>
                            </div>
                        </Show>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn SequenceCard(
    seq: SequenceRow,
    can_write: bool,
    company_id: i64,
    reload_tick: RwSignal<i32>,
    error_msg: RwSignal<String>,
    success_msg: RwSignal<String>,
    saving: RwSignal<bool>,
) -> impl IntoView {
    let sequence_id = seq.id;
    let warehouse = is_warehouse(&seq);
    let tax_series = seq.is_tax;
    let deletable = seq.deletable;
    let kinds = invoice_kinds(&seq);
    let mixed = kinds.iter().any(|k| k == "PROFORMA") && kinds.len() > 1;
    let type_names = seq
        .document_types
        .iter()
        .map(|t| t.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let issued_label = if seq.issued_max > 0 {
        format!("Последен издаден номер: {}", pad10(seq.issued_max))
    } else {
        "Все още няма издаден номер в този интервал".into()
    };
    let badge = if warehouse {
        "Склад"
    } else if tax_series {
        "Данъчен интервал"
    } else {
        "Не е данъчен документ"
    };
    let name = RwSignal::new(seq.name);
    let next_text = RwSignal::new(seq.next_formatted);

    let save = move |_| {
        let clean = name.get().trim().to_string();
        let next_number = match parse_next(&next_text.get()) {
            Ok(n) => n,
            Err(e) => {
                error_msg.set(e);
                return;
            }
        };
        error_msg.set(String::new());
        success_msg.set(String::new());
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "name": clean,
                "next_number": next_number,
            });
            match api_put(&format!("/api/companies/{}/sequences/{}", company_id, sequence_id), &body).await {
                Ok(_) => {
                    success_msg.set("Интервалът е запазен".into());
                    reload_tick.update(|n| *n += 1);
                }
                Err(e) => error_msg.set(e),
            }
            saving.set(false);
        });
    };

    let remove = move |_| {
        error_msg.set(String::new());
        success_msg.set(String::new());
        saving.set(true);
        spawn_local(async move {
            match api_delete(&format!("/api/companies/{}/sequences/{}", company_id, sequence_id)).await {
                Ok(_) => {
                    success_msg.set("Празният интервал е изтрит".into());
                    reload_tick.update(|n| *n += 1);
                }
                Err(e) => error_msg.set(e),
            }
            saving.set(false);
        });
    };

    view! {
        <div style="border: 1px solid #e5e7eb; border-radius: 8px; padding: 16px; margin-bottom: 14px;">
            <div style="display: flex; justify-content: space-between; gap: 12px; align-items: center; margin-bottom: 12px; flex-wrap: wrap;">
                <span style=format!(
                    "font-size: 12px; font-weight: 600; padding: 4px 8px; border-radius: 999px; {}",
                    if warehouse {
                        "background: #eef2ff; color: #3730a3;"
                    } else if tax_series {
                        "background: #e8f6ef; color: #1e7a46;"
                    } else {
                        "background: #f4f4f5; color: #3f3f46;"
                    }
                )>
                    {badge}
                </span>
                <Show when=move || deletable && can_write>
                    <button
                        style="background: transparent; color: #c0392b; border: none; cursor: pointer; font-size: 13px;"
                        on:click=remove
                        disabled=move || saving.get()
                    >
                        "Изтрий празния интервал"
                    </button>
                </Show>
            </div>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px;">
                <label style="font-size: 14px;">
                    "Име"
                    <input
                        type="text"
                        style="display: block; width: 100%; margin-top: 6px; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                        disabled=!can_write
                    />
                </label>
                <label style="font-size: 14px;">
                    "Следващ номер"
                    <input
                        type="text"
                        inputmode="numeric"
                        maxlength="10"
                        style="display: block; width: 100%; margin-top: 6px; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; font-variant-numeric: tabular-nums;"
                        prop:value=move || next_text.get()
                        on:input=move |ev| next_text.set(event_target_value(&ev))
                        disabled=!can_write
                    />
                </label>
            </div>
            <p style="margin: 10px 0 0 0; color: #34495e; font-size: 13px;">
                {move || match parse_next(&next_text.get()) {
                    Ok(n) => format!("Ще се издаде като {}", pad10(n)),
                    Err(e) => e,
                }}
            </p>
            <p style="margin: 6px 0 0 0; color: #7f8c8d; font-size: 13px;">{issued_label}</p>
            <p style="margin: 8px 0 0 0; font-size: 14px;">
                <span style="color: #7f8c8d;">"Документи: "</span>
                {if type_names.is_empty() { "няма — преместени са в друг интервал".to_string() } else { type_names }}
            </p>
            <Show when=move || mixed>
                <p style="background: #fff6e5; color: #8a5a00; padding: 10px 12px; border-radius: 6px; font-size: 13px; margin: 10px 0 0 0;">
                    "Проформата е в интервал с данъчни документи и дели номерата им."
                </p>
            </Show>
            <Show when=move || can_write>
                <button
                    style="margin-top: 12px; background: #27ae60; color: white; border: none; padding: 10px 18px; border-radius: 4px; cursor: pointer;"
                    on:click=save
                    disabled=move || saving.get()
                >
                    {move || if saving.get() { "Запазване..." } else { "Запази интервала" }}
                </button>
            </Show>
        </div>
    }
}
