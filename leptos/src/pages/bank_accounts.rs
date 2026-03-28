use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete};
use crate::context::use_company_context;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: i64,
    pub name: String,
    pub bank_name: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub currency: String,
    pub gl_account_id: Option<i64>,
    pub gl_account_code: Option<String>,
    pub gl_account_name: Option<String>,
    pub buffer_account_id: Option<i64>,
    pub buffer_account_code: Option<String>,
    pub buffer_account_name: Option<String>,
    pub balance: Option<f64>,
    pub transaction_count: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GLAccount {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[component]
pub fn BankAccounts() -> impl IntoView {
    let ctx = use_company_context();

    let accounts = create_rw_signal(Vec::<BankAccount>::new());
    let gl_accounts = create_rw_signal(Vec::<GLAccount>::new());
    let all_accounts = create_rw_signal(Vec::<GLAccount>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // Modal state
    let show_modal = create_rw_signal(false);
    let editing_id = create_rw_signal(Option::<i64>::None);
    let saving = create_rw_signal(false);

    // Form fields
    let form_name = create_rw_signal(String::new());
    let form_bank_name = create_rw_signal(String::new());
    let form_iban = create_rw_signal(String::new());
    let form_bic = create_rw_signal(String::new());
    let form_currency = create_rw_signal(String::from("EUR"));
    let form_gl_account_id = create_rw_signal(String::new());
    let form_buffer_account_id = create_rw_signal(String::new());

    // Load accounts
    let load_accounts = move || {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        loading.set(true);
        error_msg.set(String::new());

        spawn_local(async move {
            let url = format!("/api/companies/{}/bank_accounts", company_id);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let list: Vec<BankAccount> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        accounts.set(list);
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Load GL accounts
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
                        // Class 5 accounts for GL account (bank accounts in chart)
                        let bank_list: Vec<GLAccount> = list.iter()
                            .filter(|a| a.code.starts_with("5"))
                            .cloned()
                            .collect();
                        gl_accounts.set(bank_list);
                        // All accounts for buffer account selection
                        all_accounts.set(list);
                    }
                }
                Err(_) => {}
            }
        });
    };

    // Initial load
    create_effect(move |_| {
        ctx.selected_company_id.get();
        load_accounts();
        load_gl_accounts();
    });

    // Open create modal
    let open_create_modal = move |_| {
        editing_id.set(None);
        form_name.set(String::new());
        form_bank_name.set(String::new());
        form_iban.set(String::new());
        form_bic.set(String::new());
        form_currency.set(String::from("EUR"));
        form_gl_account_id.set(String::new());
        form_buffer_account_id.set(String::new());
        show_modal.set(true);
    };

    // Open edit modal
    let open_edit_modal = move |account: BankAccount| {
        editing_id.set(Some(account.id));
        form_name.set(account.name);
        form_bank_name.set(account.bank_name.unwrap_or_default());
        form_iban.set(account.iban.unwrap_or_default());
        form_bic.set(account.bic.unwrap_or_default());
        form_currency.set(account.currency);
        form_gl_account_id.set(account.gl_account_id.map(|id| id.to_string()).unwrap_or_default());
        form_buffer_account_id.set(account.buffer_account_id.map(|id| id.to_string()).unwrap_or_default());
        show_modal.set(true);
    };

    // Save account
    let save_account = move |_| {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);
        let name = form_name.get();

        if name.is_empty() {
            error_msg.set("Моля въведете име на сметката".to_string());
            return;
        }

        saving.set(true);
        error_msg.set(String::new());

        let gl_id = form_gl_account_id.get().parse::<i64>().ok();
        let buffer_id = form_buffer_account_id.get().parse::<i64>().ok();

        let payload = serde_json::json!({
            "name": name,
            "bank_name": form_bank_name.get(),
            "iban": form_iban.get(),
            "bic": form_bic.get(),
            "currency": form_currency.get(),
            "gl_account_id": gl_id,
            "buffer_account_id": buffer_id,
        });

        let edit_id = editing_id.get();

        spawn_local(async move {
            let result = if let Some(id) = edit_id {
                let url = format!("/api/companies/{}/bank_accounts/{}", company_id, id);
                api_put(&url, &payload).await
            } else {
                let url = format!("/api/companies/{}/bank_accounts", company_id);
                api_post(&url, &payload).await
            };

            match result {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set(if edit_id.is_some() { "Сметката е обновена" } else { "Сметката е създадена" }.to_string());
                        show_modal.set(false);
                        load_accounts();
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при запазване");
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

    // Delete account
    let delete_account = move |id: i64| {
        let company_id = ctx.selected_company_id.get().unwrap_or(1);

        spawn_local(async move {
            let url = format!("/api/companies/{}/bank_accounts/{}", company_id, id);
            match api_delete(&url).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Сметката е изтрита".to_string());
                        load_accounts();
                    } else {
                        let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при изтриване");
                        error_msg.set(err.to_string());
                    }
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
                <h1 style="margin: 0;">"Банкови сметки"</h1>
                <div style="display: flex; gap: 10px;">
                    <a
                        href="/bank-transactions"
                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; text-decoration: none;"
                    >
                        "Транзакции"
                    </a>
                    <button
                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=open_create_modal
                    >
                        "+ Нова сметка"
                    </button>
                </div>
            </div>

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

            // Accounts table
            <div style="background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                {move || {
                    if loading.get() {
                        view! {
                            <div style="padding: 40px; text-align: center; color: #7f8c8d;">
                                "Зареждане..."
                            </div>
                        }.into_view()
                    } else {
                        let acc_list = accounts.get();
                        if acc_list.is_empty() {
                            view! {
                                <div style="padding: 40px; text-align: center; color: #7f8c8d;">
                                    "Няма банкови сметки"
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <table style="width: 100%; border-collapse: collapse;">
                                    <thead>
                                        <tr style="background: #f8f9fa;">
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Име"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Банка"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"IBAN"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Валута"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"GL сметка"</th>
                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Буферна сметка"</th>
                                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6;">"Действия"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {acc_list.iter().map(|acc| {
                                            let acc_clone = acc.clone();
                                            let acc_id = acc.id;
                                            let tx_count = acc.transaction_count.unwrap_or(0);
                                            let gl_display = if let (Some(code), Some(name)) = (&acc.gl_account_code, &acc.gl_account_name) {
                                                format!("{} - {}", code, name)
                                            } else {
                                                "-".to_string()
                                            };
                                            let buffer_display = if let (Some(code), Some(name)) = (&acc.buffer_account_code, &acc.buffer_account_name) {
                                                format!("{} - {}", code, name)
                                            } else {
                                                "-".to_string()
                                            };

                                            view! {
                                                <tr style="border-bottom: 1px solid #dee2e6;">
                                                    <td style="padding: 12px; font-weight: bold;">{&acc.name}</td>
                                                    <td style="padding: 12px;">{acc.bank_name.clone().unwrap_or("-".to_string())}</td>
                                                    <td style="padding: 12px; font-family: monospace;">{acc.iban.clone().unwrap_or("-".to_string())}</td>
                                                    <td style="padding: 12px;">{&acc.currency}</td>
                                                    <td style="padding: 12px; font-size: 0.9em;">{gl_display}</td>
                                                    <td style="padding: 12px; font-size: 0.9em;">{buffer_display}</td>
                                                    <td style="padding: 12px; text-align: center;">
                                                        <button
                                                            style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-right: 5px;"
                                                            on:click=move |_| open_edit_modal(acc_clone.clone())
                                                        >
                                                            "Редакция"
                                                        </button>
                                                        <button
                                                            style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                            on:click=move |_| {
                                                                if web_sys::window().unwrap().confirm_with_message("Сигурни ли сте?").unwrap_or(false) {
                                                                    delete_account(acc_id);
                                                                }
                                                            }
                                                            disabled=move || tx_count > 0
                                                            title=move || if tx_count > 0 { "Сметката има транзакции" } else { "" }
                                                        >
                                                            "Изтрий"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_view()
                        }
                    }
                }}
            </div>

            // Modal
            {move || {
                if show_modal.get() {
                    let gl_list = gl_accounts.get();
                    let all_list = all_accounts.get();
                    let is_editing = editing_id.get().is_some();

                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 500px; max-height: 90vh; overflow-y: auto;">
                                <h2 style="margin: 0 0 20px 0;">
                                    {if is_editing { "Редакция на сметка" } else { "Нова банкова сметка" }}
                                </h2>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Име на сметката *"</label>
                                    <input
                                        type="text"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em;"
                                        prop:value=move || form_name.get()
                                        on:input=move |ev| form_name.set(event_target_value(&ev))
                                        placeholder="напр. Основна EUR сметка"
                                    />
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Име на банката"</label>
                                    <input
                                        type="text"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em;"
                                        prop:value=move || form_bank_name.get()
                                        on:input=move |ev| form_bank_name.set(event_target_value(&ev))
                                        placeholder="напр. УниКредит Булбанк"
                                    />
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"IBAN"</label>
                                    <input
                                        type="text"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em; font-family: monospace;"
                                        prop:value=move || form_iban.get()
                                        on:input=move |ev| form_iban.set(event_target_value(&ev))
                                        placeholder="напр. BG12UNCR12345678901234"
                                    />
                                </div>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 15px;">
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">"BIC"</label>
                                        <input
                                            type="text"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em; font-family: monospace;"
                                            prop:value=move || form_bic.get()
                                            on:input=move |ev| form_bic.set(event_target_value(&ev))
                                            placeholder="UNCRBGSF"
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Валута"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em;"
                                            on:change=move |ev| form_currency.set(event_target_value(&ev))
                                        >
                                            <option value="EUR" selected=move || form_currency.get() == "EUR">"EUR"</option>
                                            <option value="BGN" selected=move || form_currency.get() == "BGN">"BGN"</option>
                                            <option value="USD" selected=move || form_currency.get() == "USD">"USD"</option>
                                            <option value="GBP" selected=move || form_currency.get() == "GBP">"GBP"</option>
                                            <option value="CHF" selected=move || form_currency.get() == "CHF">"CHF"</option>
                                        </select>
                                    </div>
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Счетоводна сметка (GL)"</label>
                                    <select
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em;"
                                        on:change=move |ev| form_gl_account_id.set(event_target_value(&ev))
                                    >
                                        <option value="" selected=move || form_gl_account_id.get().is_empty()>"-- Без GL сметка --"</option>
                                        {gl_list.iter().map(|gl| {
                                            let selected = form_gl_account_id.get() == gl.id.to_string();
                                            view! {
                                                <option value={gl.id.to_string()} selected=selected>
                                                    {format!("{} - {}", gl.code, gl.name)}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                    <p style="font-size: 0.85em; color: #7f8c8d; margin-top: 5px;">
                                        "Сметка от клас 5 (напр. 503 - Разплащателна сметка)"
                                    </p>
                                </div>

                                <div style="margin-bottom: 20px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Буферна сметка"</label>
                                    <select
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 1em;"
                                        on:change=move |ev| form_buffer_account_id.set(event_target_value(&ev))
                                    >
                                        <option value="" selected=move || form_buffer_account_id.get().is_empty()>"-- Без буферна сметка --"</option>
                                        {all_list.iter().map(|gl| {
                                            let selected = form_buffer_account_id.get() == gl.id.to_string();
                                            view! {
                                                <option value={gl.id.to_string()} selected=selected>
                                                    {format!("{} - {}", gl.code, gl.name)}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                    <p style="font-size: 0.85em; color: #7f8c8d; margin-top: 5px;">
                                        "Временна сметка за неразпределени транзакции (напр. 499). При импорт автоматично се създават записи с буферната сметка като кореспондираща."
                                    </p>
                                </div>

                                <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_modal.set(false)
                                    >
                                        "Отказ"
                                    </button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=save_account
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
                }
            }}
        </Layout>
    }
}
