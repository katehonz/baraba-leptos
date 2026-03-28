use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MovementMapping {
    pub id: i64,
    #[serde(alias = "movement_type_code", alias = "asset_transaction_type", alias = "cash_movement_type")]
    pub type_code: String,
    #[serde(alias = "movement_type_name", alias = "transaction_type_name")]
    pub type_name: String,
    pub debit_account: String,
    pub credit_account: String,
    pub debit_analytical: Option<String>,
    pub credit_analytical: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MovementType {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingCategory {
    Stock,
    Asset,
    Cash,
}

impl MappingCategory {
    fn api_path(&self) -> &'static str {
        match self {
            MappingCategory::Stock => "movement_mappings",
            MappingCategory::Asset => "asset_mappings",
            MappingCategory::Cash => "cash_mappings",
        }
    }

    fn type_field(&self) -> &'static str {
        match self {
            MappingCategory::Stock => "movement_type_code",
            MappingCategory::Asset => "asset_transaction_type",
            MappingCategory::Cash => "cash_movement_type",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            MappingCategory::Stock => "Материални запаси",
            MappingCategory::Asset => "Дълготрайни активи",
            MappingCategory::Cash => "Парични средства",
        }
    }
}

#[component]
pub fn SaftMovementMappings() -> impl IntoView {
    // Tab state
    let active_tab = create_rw_signal(MappingCategory::Stock);

    let loading = create_rw_signal(true);
    let saving = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    let mappings = create_rw_signal(Vec::<MovementMapping>::new());
    let movement_types = create_rw_signal(Vec::<MovementType>::new());

    // Form signals
    let show_form = create_rw_signal(false);
    let editing_id = create_rw_signal(Option::<i64>::None);
    let form_movement_type = create_rw_signal(String::new());
    let form_debit_account = create_rw_signal(String::new());
    let form_credit_account = create_rw_signal(String::new());
    let form_debit_analytical = create_rw_signal(String::new());
    let form_credit_analytical = create_rw_signal(String::new());
    let form_description = create_rw_signal(String::new());

    // Load mappings for current tab
    let load_data = move || {
        loading.set(true);
        error_msg.set(String::new());
        let category = active_tab.get();
        let api_path = category.api_path();

        spawn_local(async move {
            match api_get(&format!("/api/companies/1/saft/{}", api_path)).await {
                Ok(data) => {
                    if let Some(items) = data.get("data").and_then(|v| v.as_array()) {
                        let parsed: Vec<MovementMapping> = items.iter().filter_map(|v| {
                            // Parse with flexible field names
                            let mut m: MovementMapping = serde_json::from_value(v.clone()).ok()?;
                            // Normalize field names based on category
                            if let Some(code) = v.get("movement_type_code").and_then(|v| v.as_str()) {
                                m.type_code = code.to_string();
                            }
                            if let Some(code) = v.get("asset_transaction_type").and_then(|v| v.as_str()) {
                                m.type_code = code.to_string();
                            }
                            if let Some(code) = v.get("cash_movement_type").and_then(|v| v.as_str()) {
                                m.type_code = code.to_string();
                            }
                            if let Some(name) = v.get("movement_type_name").and_then(|v| v.as_str()) {
                                m.type_name = name.to_string();
                            }
                            if let Some(name) = v.get("transaction_type_name").and_then(|v| v.as_str()) {
                                m.type_name = name.to_string();
                            }
                            Some(m)
                        }).collect();
                        mappings.set(parsed);
                    }
                    // Parse movement types (different keys for each category)
                    let types_key = match category {
                        MappingCategory::Stock => "movement_types",
                        MappingCategory::Asset => "transaction_types",
                        MappingCategory::Cash => "movement_types",
                    };
                    if let Some(types) = data.get(types_key).and_then(|v| v.as_object()) {
                        let mut parsed: Vec<MovementType> = types.iter().map(|(code, name)| {
                            MovementType {
                                code: code.clone(),
                                name: name.as_str().unwrap_or("").to_string(),
                            }
                        }).collect();
                        parsed.sort_by(|a, b| a.code.cmp(&b.code));
                        movement_types.set(parsed);
                    }
                }
                Err(e) => error_msg.set(format!("Грешка при зареждане: {}", e)),
            }
            loading.set(false);
        });
    };

    // Initial load and reload on tab change
    create_effect(move |_| {
        let _ = active_tab.get();
        load_data();
    });

    // Clear form
    let clear_form = move || {
        editing_id.set(None);
        form_movement_type.set(String::new());
        form_debit_account.set(String::new());
        form_credit_account.set(String::new());
        form_debit_analytical.set(String::new());
        form_credit_analytical.set(String::new());
        form_description.set(String::new());
    };

    // Edit mapping
    let edit_mapping = move |mapping: MovementMapping| {
        editing_id.set(Some(mapping.id));
        form_movement_type.set(mapping.type_code);
        form_debit_account.set(mapping.debit_account);
        form_credit_account.set(mapping.credit_account);
        form_debit_analytical.set(mapping.debit_analytical.unwrap_or_default());
        form_credit_analytical.set(mapping.credit_analytical.unwrap_or_default());
        form_description.set(mapping.description.unwrap_or_default());
        show_form.set(true);
    };

    // Save mapping
    let save_mapping = move |_| {
        saving.set(true);
        error_msg.set(String::new());
        let category = active_tab.get();
        let api_path = category.api_path();
        let type_field = category.type_field();

        spawn_local(async move {
            let mut body = serde_json::json!({
                "debit_account": form_debit_account.get(),
                "credit_account": form_credit_account.get(),
                "debit_analytical": if form_debit_analytical.get().is_empty() { None } else { Some(form_debit_analytical.get()) },
                "credit_analytical": if form_credit_analytical.get().is_empty() { None } else { Some(form_credit_analytical.get()) },
                "description": if form_description.get().is_empty() { None } else { Some(form_description.get()) },
                "is_active": true
            });
            body[type_field] = serde_json::Value::String(form_movement_type.get());

            let result = if let Some(id) = editing_id.get() {
                api_put(&format!("/api/companies/1/saft/{}/{}", api_path, id), &body).await
            } else {
                api_post(&format!("/api/companies/1/saft/{}", api_path), &body).await
            };

            match result {
                Ok(_) => {
                    success_msg.set("Запазено".to_string());
                    clear_form();
                    show_form.set(false);
                    load_data();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Delete mapping
    let delete_mapping = move |id: i64| {
        let api_path = active_tab.get().api_path();
        spawn_local(async move {
            match api_delete(&format!("/api/companies/1/saft/{}/{}", api_path, id)).await {
                Ok(_) => {
                    success_msg.set("Изтрито".to_string());
                    load_data();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Create defaults
    let create_defaults = move |_| {
        saving.set(true);
        let api_path = active_tab.get().api_path();
        spawn_local(async move {
            match api_post(&format!("/api/companies/1/saft/{}/create_defaults", api_path), &serde_json::json!({})).await {
                Ok(_) => {
                    success_msg.set("Стандартните настройки са създадени".to_string());
                    load_data();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Auto-hide success messages
    create_effect(move |_| {
        if !success_msg.get().is_empty() {
            set_timeout(move || success_msg.set(String::new()), std::time::Duration::from_secs(3));
        }
    });

    // Group mappings by type code
    let grouped_mappings = move || {
        let items = mappings.get();
        let mut groups: std::collections::HashMap<String, Vec<MovementMapping>> = std::collections::HashMap::new();
        for m in items {
            groups.entry(m.type_code.clone()).or_default().push(m);
        }
        groups
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <h1 style="margin: 0;">"SAF-T Кореспонденции на сметки"</h1>
                <div style="display: flex; gap: 10px;">
                    <button
                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=move |_| { clear_form(); show_form.set(true); }
                    >
                        "+ Добави"
                    </button>
                    <button
                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=create_defaults
                        disabled=move || saving.get() || !mappings.get().is_empty()
                    >
                        "Зареди стандартни"
                    </button>
                </div>
            </div>

            // Tabs
            <div style="display: flex; gap: 0; margin-bottom: 20px; border-bottom: 2px solid #ddd;">
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == MappingCategory::Stock {
                            "background: #3498db; color: white; border-radius: 8px 8px 0 0;"
                        } else {
                            "background: transparent; color: #7f8c8d;"
                        }
                    )
                    on:click=move |_| active_tab.set(MappingCategory::Stock)
                >
                    "📦 Материални запаси"
                </button>
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == MappingCategory::Asset {
                            "background: #3498db; color: white; border-radius: 8px 8px 0 0;"
                        } else {
                            "background: transparent; color: #7f8c8d;"
                        }
                    )
                    on:click=move |_| active_tab.set(MappingCategory::Asset)
                >
                    "🏭 Дълготрайни активи"
                </button>
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == MappingCategory::Cash {
                            "background: #3498db; color: white; border-radius: 8px 8px 0 0;"
                        } else {
                            "background: transparent; color: #7f8c8d;"
                        }
                    )
                    on:click=move |_| active_tab.set(MappingCategory::Cash)
                >
                    "💰 Парични средства"
                </button>
            </div>

            <p style="color: #7f8c8d; margin-bottom: 20px;">
                {move || format!(
                    "Дефинирайте кои Дт/Кт кореспонденции се отнасят към кой тип движение за {}. Можете да използвате wildcards (напр. 302* за всички сметки започващи с 302).",
                    active_tab.get().title()
                )}
            </p>

            // Messages
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                            {err}
                            <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;" on:click=move |_| error_msg.set(String::new())>"×"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                            {msg}
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Form Modal
            {move || if show_form.get() {
                let category = active_tab.get();
                let type_label = match category {
                    MappingCategory::Stock => "Тип движение",
                    MappingCategory::Asset => "Тип транзакция",
                    MappingCategory::Cash => "Тип плащане",
                };

                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                        <div style="background: white; padding: 30px; border-radius: 8px; width: 500px; max-width: 90%;">
                            <h2 style="margin-top: 0;">
                                {move || if editing_id.get().is_some() { "Редактиране" } else { "Нова кореспонденция" }}
                            </h2>

                            <div style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px; font-weight: 500;">{type_label}</label>
                                <select
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                    on:change=move |ev| form_movement_type.set(event_target_value(&ev))
                                >
                                    <option value="">"-- Изберете --"</option>
                                    {move || movement_types.get().into_iter().map(|t| {
                                        let code = t.code.clone();
                                        let selected = form_movement_type.get() == code;
                                        view! {
                                            <option value=code.clone() selected=selected>
                                                {format!("{} - {}", t.code, t.name)}
                                            </option>
                                        }
                                    }).collect_view()}
                                </select>
                            </div>

                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 15px;">
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Дебит сметка"</label>
                                    <input
                                        type="text"
                                        placeholder="302, 302*, 30*"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || form_debit_account.get()
                                        on:input=move |ev| form_debit_account.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Кредит сметка"</label>
                                    <input
                                        type="text"
                                        placeholder="401, 401*, 40*"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || form_credit_account.get()
                                        on:input=move |ev| form_credit_account.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>

                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 15px;">
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #7f8c8d;">"Дебит аналитична (опция)"</label>
                                    <input
                                        type="text"
                                        placeholder="3021"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || form_debit_analytical.get()
                                        on:input=move |ev| form_debit_analytical.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #7f8c8d;">"Кредит аналитична (опция)"</label>
                                    <input
                                        type="text"
                                        placeholder="4011"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || form_credit_analytical.get()
                                        on:input=move |ev| form_credit_analytical.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>

                            <div style="margin-bottom: 20px;">
                                <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Описание"</label>
                                <input
                                    type="text"
                                    placeholder="Материали от доставчици"
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                    prop:value=move || form_description.get()
                                    on:input=move |ev| form_description.set(event_target_value(&ev))
                                />
                            </div>

                            <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                <button
                                    style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=move |_| show_form.set(false)
                                >
                                    "Отказ"
                                </button>
                                <button
                                    style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                    on:click=save_mapping
                                    disabled=move || saving.get()
                                >
                                    {move || if saving.get() { "Запазване..." } else { "Запази" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}

            // Loading
            {move || if loading.get() {
                view! {
                    <div style="background: white; border-radius: 8px; padding: 40px; text-align: center;">
                        <p style="color: #7f8c8d;">"Зареждане..."</p>
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}

            // Mappings Table grouped by type
            {move || if !loading.get() {
                let groups = grouped_mappings();
                let category = active_tab.get();
                if groups.is_empty() {
                    view! {
                        <div style="background: white; border-radius: 8px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d; font-size: 18px;">"Няма настроени кореспонденции"</p>
                            <p style="color: #bdc3c7;">
                                {format!("Натиснете \"Зареди стандартни\" за да заредите примерни настройки за {}.", category.title())}
                            </p>
                        </div>
                    }.into_view()
                } else {
                    let mut sorted_codes: Vec<String> = groups.keys().cloned().collect();
                    sorted_codes.sort_by(|a, b| {
                        let a_num: i32 = a.parse().unwrap_or(999);
                        let b_num: i32 = b.parse().unwrap_or(999);
                        a_num.cmp(&b_num)
                    });

                    view! {
                        <div>
                            {sorted_codes.into_iter().map(|code| {
                                let items = groups.get(&code).cloned().unwrap_or_default();
                                let type_name = items.first().map(|i| i.type_name.clone()).unwrap_or_default();

                                view! {
                                    <div style="background: white; border-radius: 8px; margin-bottom: 20px; overflow: hidden;">
                                        <div style="background: #3498db; color: white; padding: 12px 20px; font-weight: 600;">
                                            {format!("{} - {}", code, type_name)}
                                        </div>
                                        <table style="width: 100%; border-collapse: collapse;">
                                            <thead>
                                                <tr style="background: #ecf0f1;">
                                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #ddd;">"Дт сметка"</th>
                                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #ddd;">"Кт сметка"</th>
                                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #ddd;">"Описание"</th>
                                                    <th style="padding: 12px; text-align: center; border-bottom: 1px solid #ddd; width: 100px;">"Действия"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {items.into_iter().map(|m| {
                                                    let m_clone = m.clone();
                                                    let m_id = m.id;
                                                    view! {
                                                        <tr style="border-bottom: 1px solid #eee;">
                                                            <td style="padding: 12px;">
                                                                <span style="font-family: monospace; background: #f8f9fa; padding: 2px 6px; border-radius: 3px;">
                                                                    {&m.debit_account}
                                                                </span>
                                                                {m.debit_analytical.as_ref().map(|a| view! {
                                                                    <span style="color: #7f8c8d; margin-left: 5px; font-size: 12px;">
                                                                        {format!("({})", a)}
                                                                    </span>
                                                                })}
                                                            </td>
                                                            <td style="padding: 12px;">
                                                                <span style="font-family: monospace; background: #f8f9fa; padding: 2px 6px; border-radius: 3px;">
                                                                    {&m.credit_account}
                                                                </span>
                                                                {m.credit_analytical.as_ref().map(|a| view! {
                                                                    <span style="color: #7f8c8d; margin-left: 5px; font-size: 12px;">
                                                                        {format!("({})", a)}
                                                                    </span>
                                                                })}
                                                            </td>
                                                            <td style="padding: 12px; color: #7f8c8d;">
                                                                {m.description.as_ref().unwrap_or(&String::new()).clone()}
                                                            </td>
                                                            <td style="padding: 12px; text-align: center;">
                                                                <button
                                                                    style="background: none; border: none; cursor: pointer; color: #3498db; margin-right: 8px;"
                                                                    title="Редактирай"
                                                                    on:click=move |_| edit_mapping(m_clone.clone())
                                                                >
                                                                    "✎"
                                                                </button>
                                                                <button
                                                                    style="background: none; border: none; cursor: pointer; color: #e74c3c;"
                                                                    title="Изтрий"
                                                                    on:click=move |_| delete_mapping(m_id)
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
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }
            } else {
                view! { <span></span> }.into_view()
            }}
        </Layout>
    }
}
