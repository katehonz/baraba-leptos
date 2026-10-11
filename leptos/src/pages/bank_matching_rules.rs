use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete};
use crate::context::use_company_context;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchingRule {
    pub id: i64,
    pub name: String,
    pub keywords: String,
    pub direction: String,
    pub priority: i32,
    pub is_active: bool,
    pub account_id: i64,
    pub account_code: Option<String>,
    pub account_name: Option<String>,
    pub counterpart_id: Option<i64>,
    pub counterpart_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GLAccount {
    pub id: i64,
    pub code: String,
    pub name: String,
}

fn direction_label(d: &str) -> &'static str {
    match d {
        "debit" => "Изходящи (−)",
        "credit" => "Входящи (+)",
        _ => "Всички",
    }
}

#[component]
pub fn BankMatchingRules() -> impl IntoView {
    let ctx = use_company_context();

    let rules = RwSignal::new(Vec::<MatchingRule>::new());
    let gl_accounts = RwSignal::new(Vec::<GLAccount>::new());
    let loading = RwSignal::new(true);
    let saving = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());

    let show_modal = RwSignal::new(false);
    let editing_id = RwSignal::new(Option::<i64>::None);
    let form_name = RwSignal::new(String::new());
    let form_keywords = RwSignal::new(String::new());
    let form_direction = RwSignal::new(String::from("any"));
    let form_priority = RwSignal::new(String::from("10"));
    let form_account_id = RwSignal::new(String::new());
    let form_active = RwSignal::new(true);

    let load_rules = move || {
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        if company_id == 0 {
            error_msg.set("Няма избрана фирма".to_string());
            loading.set(false);
            return;
        }
        loading.set(true);
        spawn_local(async move {
            let url = format!("/api/companies/{}/bank_matching_rules", company_id);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let list: Vec<MatchingRule> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        rules.set(list);
                    }
                }
                Err(e) => error_msg.set(format!("Грешка при зареждане: {}", e)),
            }
            loading.set(false);
        });
    };

    let load_gl = move || {
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        if company_id == 0 { return; }
        spawn_local(async move {
            let url = format!("/api/companies/{}/accounts?per_page=1000", company_id);
            if let Ok(data) = api_get(&url).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    let list: Vec<GLAccount> = arr.iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect();
                    gl_accounts.set(list);
                }
            }
        });
    };

    Effect::new(move |_| {
        ctx.selected_company_id.get();
        load_rules();
        load_gl();
    });

    let open_create = move |_| {
        editing_id.set(None);
        form_name.set(String::new());
        form_keywords.set(String::new());
        form_direction.set(String::from("any"));
        form_priority.set(String::from("10"));
        form_account_id.set(String::new());
        form_active.set(true);
        error_msg.set(String::new());
        show_modal.set(true);
    };

    let open_edit = move |rule: MatchingRule| {
        editing_id.set(Some(rule.id));
        form_name.set(rule.name);
        form_keywords.set(rule.keywords);
        form_direction.set(rule.direction);
        form_priority.set(rule.priority.to_string());
        form_account_id.set(rule.account_id.to_string());
        form_active.set(rule.is_active);
        error_msg.set(String::new());
        show_modal.set(true);
    };

    let save_rule = move |_| {
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        let name = form_name.get();
        let keywords = form_keywords.get();
        if name.trim().is_empty() {
            error_msg.set("Въведете име на правилото".to_string());
            return;
        }
        if keywords.trim().is_empty() {
            error_msg.set("Въведете поне една ключова дума".to_string());
            return;
        }
        let account_id = match form_account_id.get().parse::<i64>() {
            Ok(id) if id > 0 => id,
            _ => {
                error_msg.set("Изберете счетоводна сметка".to_string());
                return;
            }
        };

        saving.set(true);
        error_msg.set(String::new());
        let edit_id = editing_id.get();
        let payload = serde_json::json!({
            "bank_matching_rule": {
                "name": name.trim(),
                "keywords": keywords.trim(),
                "direction": form_direction.get(),
                "priority": form_priority.get().parse::<i32>().unwrap_or(10),
                "account_id": account_id,
                "is_active": form_active.get(),
            }
        });

        spawn_local(async move {
            let result = if let Some(id) = edit_id {
                api_put(&format!("/api/companies/{}/bank_matching_rules/{}", company_id, id), &payload).await
            } else {
                api_post(&format!("/api/companies/{}/bank_matching_rules", company_id), &payload).await
            };
            match result {
                Ok(data) if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) => {
                    success_msg.set(if edit_id.is_some() { "Правилото е обновено".into() } else { "Правилото е създадено".into() });
                    show_modal.set(false);
                    load_rules();
                }
                Ok(_) => error_msg.set("Грешка при запазване".to_string()),
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    let delete_rule = move |id: i64| {
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        spawn_local(async move {
            match api_delete(&format!("/api/companies/{}/bank_matching_rules/{}", company_id, id)).await {
                Ok(_) => {
                    success_msg.set("Правилото е изтрито".to_string());
                    load_rules();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    let seed_defaults = move |_| {
        let company_id = ctx.selected_company_id.get().unwrap_or(0);
        saving.set(true);
        spawn_local(async move {
            match api_post(&format!("/api/companies/{}/bank_matching_rules/create_defaults", company_id), &serde_json::json!({})).await {
                Ok(data) => {
                    let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Готово");
                    success_msg.set(msg.to_string());
                    load_rules();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; flex-wrap: wrap; gap: 10px;">
                <h1 style="margin: 0;">"Правила за банкови транзакции"</h1>
                <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                    <a href="/bank-transactions" style="background: #3498db; color: white; padding: 10px 20px; border-radius: 4px; text-decoration: none;">
                        "Към транзакции"
                    </a>
                    <button
                        style="background: #8e44ad; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=seed_defaults
                        disabled=move || saving.get()
                    >
                        "Стандартни правила"
                    </button>
                    <button
                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=open_create
                    >
                        "+ Ново правило"
                    </button>
                </div>
            </div>

            <div style="background: #eef6ff; border: 1px solid #c5ddf5; padding: 14px 16px; border-radius: 8px; margin-bottom: 20px; color: #2c3e50; line-height: 1.5;">
                "Ключовите думи се търсят в описанието и в името на контрагента. "
                <b>"Не е нужен AI ключ."</b>
                " Работи с кирилица и латиница — напр. "
                <code>"POS, ПОС"</code>
                " или "
                <code>"теглене, ATM, withdrawal"</code>
                ". По-висок приоритет печели при няколко съвпадения."
            </div>

            {move || {
                let msg = success_msg.get();
                if msg.is_empty() { view! { <span></span> }.into_any() } else {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 12px; border-radius: 8px; margin-bottom: 16px;">
                            {msg}
                        </div>
                    }.into_any()
                }
            }}
            {move || {
                let err = error_msg.get();
                if err.is_empty() || show_modal.get() { view! { <span></span> }.into_any() } else {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 12px; border-radius: 8px; margin-bottom: 16px;">
                            {err}
                        </div>
                    }.into_any()
                }
            }}

            <div style="background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.08);">
                {move || {
                    if loading.get() {
                        view! { <div style="padding: 40px; text-align: center; color: #7f8c8d;">"Зареждане..."</div> }.into_any()
                    } else {
                        let list = rules.get();
                        if list.is_empty() {
                            view! {
                                <div style="padding: 40px; text-align: center; color: #7f8c8d;">
                                    "Няма правила. Натиснете „Стандартни правила“ или добавете свое."
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <table style="width: 100%; border-collapse: collapse;">
                                    <thead>
                                        <tr style="background: #f8f9fa;">
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Име"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Ключови думи"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Посока"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Сметка"</th>
                                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6;">"Приоритет"</th>
                                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6;">"Действия"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {list.iter().map(|rule| {
                                            let rule_clone = rule.clone();
                                            let rule_id = rule.id;
                                            let acc = match (&rule.account_code, &rule.account_name) {
                                                (Some(c), Some(n)) => format!("{} - {}", c, n),
                                                _ => rule.account_id.to_string(),
                                            };
                                            let kw = rule.keywords.clone();
                                            let inactive = !rule.is_active;
                                            view! {
                                                <tr style=if inactive { "border-bottom: 1px solid #dee2e6; opacity: 0.55;" } else { "border-bottom: 1px solid #dee2e6;" }>
                                                    <td style="padding: 12px; font-weight: bold;">{rule.name.clone()}</td>
                                                    <td style="padding: 12px; font-size: 0.9em; max-width: 280px;">{kw}</td>
                                                    <td style="padding: 12px;">{direction_label(&rule.direction)}</td>
                                                    <td style="padding: 12px; font-size: 0.9em;">{acc}</td>
                                                    <td style="padding: 12px; text-align: center;">{rule.priority}</td>
                                                    <td style="padding: 12px; text-align: center; white-space: nowrap;">
                                                        <button
                                                            style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-right: 5px;"
                                                            on:click=move |_| open_edit(rule_clone.clone())
                                                        >"Редакция"</button>
                                                        <button
                                                            style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                            on:click=move |_| {
                                                                if web_sys::window().unwrap().confirm_with_message("Изтриване на правилото?").unwrap_or(false) {
                                                                    delete_rule(rule_id);
                                                                }
                                                            }
                                                        >"Изтрий"</button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }
                    }
                }}
            </div>

            {move || {
                if !show_modal.get() {
                    view! { <span></span> }.into_any()
                } else {
                    let gl_list = gl_accounts.get();
                    let is_editing = editing_id.get().is_some();
                    view! {
                        <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 28px; width: 560px; max-height: 90vh; overflow-y: auto;">
                                <h2 style="margin: 0 0 16px 0;">
                                    {if is_editing { "Редакция на правило" } else { "Ново правило" }}
                                </h2>
                                {move || {
                                    let err = error_msg.get();
                                    if err.is_empty() { view! { <span></span> }.into_any() } else {
                                        view! {
                                            <div style="background: #e74c3c; color: white; padding: 10px; border-radius: 4px; margin-bottom: 12px;">{err}</div>
                                        }.into_any()
                                    }
                                }}
                                <label style="display: block; font-weight: bold; margin-bottom: 6px;">"Име *"</label>
                                <input
                                    type="text"
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 12px; box-sizing: border-box;"
                                    prop:value=move || form_name.get()
                                    on:input=move |ev| form_name.set(event_target_value(&ev))
                                    placeholder="напр. POS покупки"
                                />
                                <label style="display: block; font-weight: bold; margin-bottom: 6px;">"Ключови думи *"</label>
                                <textarea
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 6px; min-height: 80px; box-sizing: border-box; font-family: inherit;"
                                    prop:value=move || form_keywords.get()
                                    on:input=move |ev| form_keywords.set(event_target_value(&ev))
                                    placeholder="POS, ПОС, pos purchase, покупка с карта"
                                />
                                <p style="font-size: 0.85em; color: #7f8c8d; margin: 0 0 12px 0;">
                                    "Разделете със запетая. Сложете и кирилица, и латиница — банките пишат различно."
                                </p>
                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px;">
                                    <div>
                                        <label style="display: block; font-weight: bold; margin-bottom: 6px;">"Посока"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| form_direction.set(event_target_value(&ev))
                                        >
                                            <option value="any" selected=move || form_direction.get() == "any">"Всички"</option>
                                            <option value="debit" selected=move || form_direction.get() == "debit">"Изходящи (−)"</option>
                                            <option value="credit" selected=move || form_direction.get() == "credit">"Входящи (+)"</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label style="display: block; font-weight: bold; margin-bottom: 6px;">"Приоритет"</label>
                                        <input
                                            type="number"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || form_priority.get()
                                            on:input=move |ev| form_priority.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                                <label style="display: block; font-weight: bold; margin-bottom: 6px;">"Счетоводна сметка *"</label>
                                <select
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 16px;"
                                    on:change=move |ev| form_account_id.set(event_target_value(&ev))
                                >
                                    <option value="" selected=move || form_account_id.get().is_empty()>"-- Изберете сметка --"</option>
                                    {gl_list.iter().map(|gl| {
                                        let selected = form_account_id.get() == gl.id.to_string();
                                        view! {
                                            <option value={gl.id.to_string()} selected=selected>
                                                {format!("{} - {}", gl.code, gl.name)}
                                            </option>
                                        }
                                    }).collect_view()}
                                </select>
                                <label style="display: flex; align-items: center; gap: 8px; margin-bottom: 18px;">
                                    <input
                                        type="checkbox"
                                        checked=move || form_active.get()
                                        on:change=move |ev| form_active.set(event_target_checked(&ev))
                                    />
                                    "Активно"
                                </label>
                                <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_modal.set(false)
                                    >"Отказ"</button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=save_rule
                                        disabled=move || saving.get()
                                    >{move || if saving.get() { "Запазване..." } else { "Запази" }}</button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </Layout>
    }
}
