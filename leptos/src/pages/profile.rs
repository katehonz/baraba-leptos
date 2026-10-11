use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, download_text_file};

#[component]
pub fn Profile() -> impl IntoView {
    let loading = RwSignal::new(true);
    let saving = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());

    // Profile fields
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let created_at = RwSignal::new(String::new());

    // Password change fields
    let current_password = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let password_saving = RwSignal::new(false);
    let password_error = RwSignal::new(String::new());
    let password_success = RwSignal::new(String::new());

    // Active tab: 0=Profile, 1=Password, 2=Two-factor
    let active_tab = RwSignal::new(0);

    let otp_enabled = RwSignal::new(false);
    let otp_pending = RwSignal::new(false);
    let recovery_remaining = RwSignal::new(0_i64);
    let setup_password = RwSignal::new(String::new());
    let setup_qr = RwSignal::new(String::new());
    let setup_secret = RwSignal::new(String::new());
    let setup_codes = RwSignal::new(Vec::<String>::new());
    let confirm_code = RwSignal::new(String::new());
    let disable_password = RwSignal::new(String::new());
    let disable_code = RwSignal::new(String::new());
    let regen_password = RwSignal::new(String::new());
    let regen_code = RwSignal::new(String::new());
    let twofa_busy = RwSignal::new(false);
    let twofa_error = RwSignal::new(String::new());
    let twofa_success = RwSignal::new(String::new());

    // Load user profile
    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(data) = api_get("/api/auth/me").await {
                if let Some(user) = data.get("data") {
                    first_name.set(user.get("first_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    last_name.set(user.get("last_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    email.set(user.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    created_at.set(user.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    otp_enabled.set(user.get("otp_enabled").and_then(|v| v.as_bool()).unwrap_or(false));
                    otp_pending.set(user.get("otp_pending").and_then(|v| v.as_bool()).unwrap_or(false));
                    recovery_remaining.set(user.get("recovery_codes_remaining").and_then(|v| v.as_i64()).unwrap_or(0));
                }
            }
            loading.set(false);
        });
    });

    // Auto-hide messages
    Effect::new(move |_| {
        if !success_msg.get().is_empty() {
            set_timeout(move || success_msg.set(String::new()), std::time::Duration::from_secs(3));
        }
    });
    Effect::new(move |_| {
        if !password_success.get().is_empty() {
            set_timeout(move || password_success.set(String::new()), std::time::Duration::from_secs(3));
        }
    });
    Effect::new(move |_| {
        if !twofa_success.get().is_empty() {
            set_timeout(move || twofa_success.set(String::new()), std::time::Duration::from_secs(5));
        }
    });

    // Save profile
    let save_profile = move |_| {
        saving.set(true);
        error_msg.set(String::new());
        success_msg.set(String::new());

        let first = first_name.get();
        let last = last_name.get();
        let mail = email.get();

        spawn_local(async move {
            let body = serde_json::json!({
                "first_name": first,
                "last_name": last,
                "email": mail,
            });

            match api_put("/api/auth/profile", &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Профилът е обновен успешно".to_string());
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Грешка при запис");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            saving.set(false);
        });
    };

    // Change password
    let change_password = move |_| {
        password_saving.set(true);
        password_error.set(String::new());
        password_success.set(String::new());

        let current = current_password.get();
        let new_pass = new_password.get();
        let confirm = confirm_password.get();

        // Client-side validation
        if current.is_empty() {
            password_error.set("Въведете текущата парола".to_string());
            password_saving.set(false);
            return;
        }
        if new_pass.len() < 8 {
            password_error.set("Новата парола трябва да е поне 8 символа".to_string());
            password_saving.set(false);
            return;
        }
        if new_pass != confirm {
            password_error.set("Паролите не съвпадат".to_string());
            password_saving.set(false);
            return;
        }

        spawn_local(async move {
            let body = serde_json::json!({
                "current_password": current,
                "new_password": new_pass,
                "confirm_password": confirm,
            });

            match api_put("/api/auth/password", &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        password_success.set("Паролата е променена успешно".to_string());
                        current_password.set(String::new());
                        new_password.set(String::new());
                        confirm_password.set(String::new());
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Грешка при смяна на парола");
                        password_error.set(msg.to_string());
                    }
                }
                Err(e) => {
                    password_error.set(format!("Грешка: {}", e));
                }
            }
            password_saving.set(false);
        });
    };

    let start_twofa = move |_| {
        let password = setup_password.get();
        if password.is_empty() {
            twofa_error.set("Въведете паролата си".to_string());
            return;
        }
        twofa_busy.set(true);
        twofa_error.set(String::new());
        twofa_success.set(String::new());
        spawn_local(async move {
            let body = serde_json::json!({ "password": password });
            match api_post("/api/auth/two_factor/setup", &body).await {
                Ok(data) => {
                    setup_qr.set(data.get("qr_image").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    setup_secret.set(data.get("secret").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    let codes = data.get("recovery_codes").and_then(|v| v.as_array()).map(|arr| {
                        arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>()
                    }).unwrap_or_default();
                    recovery_remaining.set(codes.len() as i64);
                    setup_codes.set(codes);
                    otp_pending.set(true);
                    setup_password.set(String::new());
                    twofa_success.set("Сканирайте QR кода и изтеглете кодовете за възстановяване.".to_string());
                }
                Err(e) => twofa_error.set(e),
            }
            twofa_busy.set(false);
        });
    };

    let confirm_twofa = move |_| {
        let code = confirm_code.get();
        if code.trim().is_empty() {
            twofa_error.set("Въведете 6-цифрения код от приложението".to_string());
            return;
        }
        twofa_busy.set(true);
        twofa_error.set(String::new());
        twofa_success.set(String::new());
        spawn_local(async move {
            let body = serde_json::json!({ "code": code });
            match api_post("/api/auth/two_factor/confirm", &body).await {
                Ok(_) => {
                    otp_enabled.set(true);
                    otp_pending.set(false);
                    setup_qr.set(String::new());
                    setup_secret.set(String::new());
                    confirm_code.set(String::new());
                    twofa_success.set("Двуфакторната защита е включена. Изтеглете кодовете, преди да напуснете страницата.".to_string());
                }
                Err(e) => twofa_error.set(e),
            }
            twofa_busy.set(false);
        });
    };

    let download_codes = move |_| {
        let codes = setup_codes.get();
        if codes.is_empty() {
            twofa_error.set("Няма кодове за изтегляне. Генерирайте нови.".to_string());
            return;
        }
        let text = recovery_codes_text(&email.get(), &codes);
        download_text_file("baraba-kodove-za-vazstanovyavane.txt", &text, "text/plain");
    };

    let regenerate_codes = move |_| {
        let password = regen_password.get();
        let code = regen_code.get();
        if password.is_empty() || code.trim().is_empty() {
            twofa_error.set("Въведете парола и код от приложението или код за възстановяване".to_string());
            return;
        }
        twofa_busy.set(true);
        twofa_error.set(String::new());
        twofa_success.set(String::new());
        spawn_local(async move {
            let body = serde_json::json!({ "password": password, "code": code });
            match api_post("/api/auth/two_factor/recovery_codes", &body).await {
                Ok(data) => {
                    let codes = data.get("recovery_codes").and_then(|v| v.as_array()).map(|arr| {
                        arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>()
                    }).unwrap_or_default();
                    recovery_remaining.set(codes.len() as i64);
                    setup_codes.set(codes);
                    regen_password.set(String::new());
                    regen_code.set(String::new());
                    twofa_success.set("Новите кодове заменят старите. Изтеглете ги сега.".to_string());
                }
                Err(e) => twofa_error.set(e),
            }
            twofa_busy.set(false);
        });
    };

    let disable_twofa = move |_| {
        let password = disable_password.get();
        let code = disable_code.get();
        if password.is_empty() || code.trim().is_empty() {
            twofa_error.set("Въведете парола и код, за да изключите защитата".to_string());
            return;
        }
        twofa_busy.set(true);
        twofa_error.set(String::new());
        twofa_success.set(String::new());
        spawn_local(async move {
            let body = serde_json::json!({ "password": password, "code": code });
            match api_post("/api/auth/two_factor/disable", &body).await {
                Ok(_) => {
                    otp_enabled.set(false);
                    otp_pending.set(false);
                    recovery_remaining.set(0);
                    setup_codes.set(Vec::new());
                    setup_qr.set(String::new());
                    setup_secret.set(String::new());
                    disable_password.set(String::new());
                    disable_code.set(String::new());
                    twofa_success.set("Двуфакторната защита е изключена.".to_string());
                }
                Err(e) => twofa_error.set(e),
            }
            twofa_busy.set(false);
        });
    };

    view! {
        <Layout>
            <h1 style="color: #2c3e50; margin-bottom: 20px;">"Моят профил"</h1>

            // Tabs
            <div style="display: flex; gap: 0; border-bottom: 2px solid #3498db; flex-wrap: wrap;">
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 14px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == 0 {
                            "background: #3498db; color: white; border-radius: 4px 4px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set(0)
                >
                    "👤 Профил"
                </button>
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 14px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == 1 {
                            "background: #3498db; color: white; border-radius: 4px 4px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set(1)
                >
                    "🔐 Смяна на парола"
                </button>
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 14px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == 2 {
                            "background: #3498db; color: white; border-radius: 4px 4px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set(2)
                >
                    "🛡️ Двуфакторна защита"
                </button>
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

            // Profile Tab
            {move || {
                if loading.get() || active_tab.get() != 0 {
                    return view! { <span></span> }.into_any();
                }

                view! {
                    <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 30px; max-width: 500px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                        // Success message
                        {move || {
                            let success = success_msg.get();
                            if !success.is_empty() {
                                view! {
                                    <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                        {success}
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
                                    <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                        {err}
                                        <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;" on:click=move |_| error_msg.set(String::new())>"×"</button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}

                        // Name fields
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 15px;">
                            <div>
                                <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                                    "Име"
                                </label>
                                <input
                                    type="text"
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                    prop:value=move || first_name.get()
                                    on:input=move |e| first_name.set(event_target_value(&e))
                                />
                            </div>
                            <div>
                                <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                                    "Фамилия"
                                </label>
                                <input
                                    type="text"
                                    style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                    prop:value=move || last_name.get()
                                    on:input=move |e| last_name.set(event_target_value(&e))
                                />
                            </div>
                        </div>

                        // Email
                        <div style="margin-bottom: 15px;">
                            <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                                "Email"
                            </label>
                            <input
                                type="email"
                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                prop:value=move || email.get()
                                on:input=move |e| email.set(event_target_value(&e))
                            />
                        </div>

                        // Created at (read only)
                        <div style="margin-bottom: 20px;">
                            <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                                "Регистриран на"
                            </label>
                            <div style="padding: 10px; background: #f8f9fa; border: 1px solid #ddd; border-radius: 4px; color: #7f8c8d;">
                                {move || created_at.get()}
                            </div>
                        </div>

                        // Save button
                        <button
                            style="width: 100%; padding: 12px; background: #3498db; border: none; border-radius: 4px; color: white; font-size: 14px; font-weight: 600; cursor: pointer;"
                            on:click=save_profile
                            disabled=move || saving.get()
                        >
                            {move || if saving.get() { "Запазване..." } else { "Запази промените" }}
                        </button>
                    </div>
                }.into_any()
            }}

            // Password Tab
            {move || {
                if loading.get() || active_tab.get() != 1 {
                    return view! { <span></span> }.into_any();
                }

                view! {
                    <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 30px; max-width: 500px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                        // Success message
                        {move || {
                            let success = password_success.get();
                            if !success.is_empty() {
                                view! {
                                    <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                        {success}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}

                        // Error message
                        {move || {
                            let err = password_error.get();
                            if !err.is_empty() {
                                view! {
                                    <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                        {err}
                                        <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;" on:click=move |_| password_error.set(String::new())>"×"</button>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}

                        // Current password
                        <div style="margin-bottom: 15px;">
                            <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                                "Текуща парола"
                            </label>
                            <input
                                type="password"
                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                prop:value=move || current_password.get()
                                on:input=move |e| current_password.set(event_target_value(&e))
                            />
                        </div>

                        // New password
                        <div style="margin-bottom: 15px;">
                            <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                                "Нова парола"
                            </label>
                            <input
                                type="password"
                                placeholder="Минимум 8 символа"
                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                prop:value=move || new_password.get()
                                on:input=move |e| new_password.set(event_target_value(&e))
                            />
                        </div>

                        // Confirm password
                        <div style="margin-bottom: 20px;">
                            <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">
                                "Потвърди новата парола"
                            </label>
                            <input
                                type="password"
                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                prop:value=move || confirm_password.get()
                                on:input=move |e| confirm_password.set(event_target_value(&e))
                            />
                        </div>

                        // Change password button
                        <button
                            style="width: 100%; padding: 12px; background: #e67e22; border: none; border-radius: 4px; color: white; font-size: 14px; font-weight: 600; cursor: pointer;"
                            on:click=change_password
                            disabled=move || password_saving.get()
                        >
                            {move || if password_saving.get() { "Промяна..." } else { "Промени паролата" }}
                        </button>
                    </div>
                }.into_any()
            }}

            // Two-factor tab
            {move || {
                if loading.get() || active_tab.get() != 2 {
                    return view! { <span></span> }.into_any();
                }

                let field = "width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;";
                view! {
                    <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 30px; max-width: 640px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                        <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 16px;">
                            <strong style="color: #2c3e50;">"Статус:"</strong>
                            {move || if otp_enabled.get() {
                                view! { <span style="background: #27ae60; color: white; padding: 4px 10px; border-radius: 12px; font-size: 13px;">"Включена"</span> }.into_any()
                            } else if otp_pending.get() {
                                view! { <span style="background: #f39c12; color: white; padding: 4px 10px; border-radius: 12px; font-size: 13px;">"Чака потвърждение"</span> }.into_any()
                            } else {
                                view! { <span style="background: #95a5a6; color: white; padding: 4px 10px; border-radius: 12px; font-size: 13px;">"Изключена"</span> }.into_any()
                            }}
                        </div>
                        <p style="color: #7f8c8d; margin-top: 0;">
                            "При включена защита входът иска и 6-цифрен код от приложение като Google Authenticator, Aegis, Authy или 1Password."
                        </p>

                        {move || {
                            let success = twofa_success.get();
                            if success.is_empty() {
                                view! { <span></span> }.into_any()
                            } else {
                                view! { <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">{success}</div> }.into_any()
                            }
                        }}
                        {move || {
                            let err = twofa_error.get();
                            if err.is_empty() {
                                view! { <span></span> }.into_any()
                            } else {
                                view! {
                                    <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                        {err}
                                        <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;" on:click=move |_| twofa_error.set(String::new())>"×"</button>
                                    </div>
                                }.into_any()
                            }
                        }}

                        {move || {
                            let codes = setup_codes.get();
                            if codes.is_empty() {
                                return view! { <span></span> }.into_any();
                            }
                            view! {
                                <div style="background: #fff8e6; border: 1px solid #f0d58c; border-radius: 8px; padding: 16px; margin-bottom: 20px;">
                                    <strong style="color: #8a6d1d;">"Кодове за възстановяване"</strong>
                                    <p style="color: #6b5a24; margin: 8px 0 12px;">
                                        "Всеки код е за еднократна употреба. След като напуснете страницата, те няма да се покажат отново. Изтеглете ги сега."
                                    </p>
                                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 12px;">
                                        {codes.into_iter().map(|code| view! {
                                            <div style="font-family: ui-monospace, SFMono-Regular, Menlo, monospace; background: white; border: 1px solid #f0d58c; border-radius: 4px; padding: 8px; text-align: center;">
                                                {code}
                                            </div>
                                        }).collect_view()}
                                    </div>
                                    <button
                                        style="padding: 10px 16px; background: #8a6d1d; border: none; border-radius: 4px; color: white; font-weight: 600; cursor: pointer;"
                                        on:click=download_codes
                                    >
                                        "Изтегли кодовете"
                                    </button>
                                </div>
                            }.into_any()
                        }}

                        {move || {
                            if otp_enabled.get() || !setup_qr.get().is_empty() {
                                return view! { <span></span> }.into_any();
                            }
                            view! {
                                <div>
                                    {move || if otp_pending.get() {
                                        view! { <p style="color: #e67e22;">"Настройката не е завършена. Въведете паролата, за да генерирате нов QR код и нови кодове."</p> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">"Парола"</label>
                                    <input
                                        type="password"
                                        style=field
                                        prop:value=move || setup_password.get()
                                        on:input=move |e| setup_password.set(event_target_value(&e))
                                    />
                                    <button
                                        style="width: 100%; margin-top: 15px; padding: 12px; background: #27ae60; border: none; border-radius: 4px; color: white; font-size: 14px; font-weight: 600; cursor: pointer;"
                                        on:click=start_twofa
                                        disabled=move || twofa_busy.get()
                                    >
                                        {move || if twofa_busy.get() { "Генериране..." } else { "Включи двуфакторна защита" }}
                                    </button>
                                </div>
                            }.into_any()
                        }}

                        {move || {
                            let qr = setup_qr.get();
                            if otp_enabled.get() || qr.is_empty() {
                                return view! { <span></span> }.into_any();
                            }
                            let secret = grouped_secret(&setup_secret.get());
                            view! {
                                <div>
                                    <p style="color: #34495e; font-weight: 500;">"1. Сканирайте QR кода"</p>
                                    <img
                                        src=qr
                                        alt="QR код за двуфакторна защита"
                                        width="240"
                                        height="240"
                                        style="width: 240px; height: 240px; image-rendering: pixelated; border: 1px solid #eee; border-radius: 8px;"
                                    />
                                    <p style="color: #7f8c8d; font-size: 13px;">"Ако не можете да сканирате, въведете ключа ръчно:"</p>
                                    <div style="font-family: ui-monospace, SFMono-Regular, Menlo, monospace; background: #f8f9fa; border: 1px solid #ddd; border-radius: 4px; padding: 10px; letter-spacing: 1px;">
                                        {secret}
                                    </div>
                                    <p style="color: #34495e; font-weight: 500; margin-top: 18px;">"2. Въведете 6-цифрения код от приложението"</p>
                                    <input
                                        type="text"
                                        inputmode="numeric"
                                        autocomplete="one-time-code"
                                        maxlength="6"
                                        placeholder="123456"
                                        style=field
                                        prop:value=move || confirm_code.get()
                                        on:input=move |e| confirm_code.set(event_target_value(&e))
                                    />
                                    <button
                                        style="width: 100%; margin-top: 15px; padding: 12px; background: #3498db; border: none; border-radius: 4px; color: white; font-size: 14px; font-weight: 600; cursor: pointer;"
                                        on:click=confirm_twofa
                                        disabled=move || twofa_busy.get()
                                    >
                                        {move || if twofa_busy.get() { "Проверка..." } else { "Потвърди и включи" }}
                                    </button>
                                </div>
                            }.into_any()
                        }}

                        {move || {
                            if !otp_enabled.get() {
                                return view! { <span></span> }.into_any();
                            }
                            view! {
                                <div>
                                    <p style="color: #34495e;">
                                        "Оставащи кодове за възстановяване: "
                                        <strong>{move || recovery_remaining.get().to_string()}</strong>
                                    </p>
                                    <h3 style="color: #2c3e50; font-size: 16px;">"Нови кодове за възстановяване"</h3>
                                    <p style="color: #7f8c8d; font-size: 13px;">"Новите кодове анулират предишните. Нужни са паролата и текущ код."</p>
                                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
                                        <input type="password" placeholder="Парола" style=field prop:value=move || regen_password.get() on:input=move |e| regen_password.set(event_target_value(&e)) />
                                        <input type="text" placeholder="Код" style=field prop:value=move || regen_code.get() on:input=move |e| regen_code.set(event_target_value(&e)) />
                                    </div>
                                    <button
                                        style="margin-top: 12px; padding: 10px 16px; background: #8a6d1d; border: none; border-radius: 4px; color: white; font-weight: 600; cursor: pointer;"
                                        on:click=regenerate_codes
                                        disabled=move || twofa_busy.get()
                                    >
                                        "Генерирай и покажи нови кодове"
                                    </button>

                                    <h3 style="color: #c0392b; font-size: 16px; margin-top: 28px;">"Изключи двуфакторната защита"</h3>
                                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
                                        <input type="password" placeholder="Парола" style=field prop:value=move || disable_password.get() on:input=move |e| disable_password.set(event_target_value(&e)) />
                                        <input type="text" placeholder="Код" style=field prop:value=move || disable_code.get() on:input=move |e| disable_code.set(event_target_value(&e)) />
                                    </div>
                                    <button
                                        style="margin-top: 12px; padding: 10px 16px; background: #e74c3c; border: none; border-radius: 4px; color: white; font-weight: 600; cursor: pointer;"
                                        on:click=disable_twofa
                                        disabled=move || twofa_busy.get()
                                    >
                                        "Изключи"
                                    </button>
                                </div>
                            }.into_any()
                        }}
                    </div>
                }.into_any()
            }}
        </Layout>
    }
}

fn grouped_secret(secret: &str) -> String {
    secret
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

fn recovery_codes_text(email: &str, codes: &[String]) -> String {
    let mut text = format!(
        "Кодове за възстановяване — Baraba\nАкаунт: {}\n\nЗапазете тези кодове на сигурно място. Всеки код може да се използва само веднъж, ако нямате достъп до приложението за удостоверяване.\n\n",
        email
    );
    for code in codes {
        text.push_str(code);
        text.push('\n');
    }
    text
}
