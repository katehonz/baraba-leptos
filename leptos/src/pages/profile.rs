use leptos::*;
use crate::components::Layout;
use crate::api::{api_get, api_put};

#[component]
pub fn Profile() -> impl IntoView {
    let loading = create_rw_signal(true);
    let saving = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // Profile fields
    let first_name = create_rw_signal(String::new());
    let last_name = create_rw_signal(String::new());
    let email = create_rw_signal(String::new());
    let created_at = create_rw_signal(String::new());

    // Password change fields
    let current_password = create_rw_signal(String::new());
    let new_password = create_rw_signal(String::new());
    let confirm_password = create_rw_signal(String::new());
    let password_saving = create_rw_signal(false);
    let password_error = create_rw_signal(String::new());
    let password_success = create_rw_signal(String::new());

    // Active tab: 0=Profile, 1=Password
    let active_tab = create_rw_signal(0);

    // Load user profile
    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(data) = api_get("/api/auth/me").await {
                if let Some(user) = data.get("data") {
                    first_name.set(user.get("first_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    last_name.set(user.get("last_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    email.set(user.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    created_at.set(user.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string());
                }
            }
            loading.set(false);
        });
    });

    // Auto-hide messages
    create_effect(move |_| {
        if !success_msg.get().is_empty() {
            set_timeout(move || success_msg.set(String::new()), std::time::Duration::from_secs(3));
        }
    });
    create_effect(move |_| {
        if !password_success.get().is_empty() {
            set_timeout(move || password_success.set(String::new()), std::time::Duration::from_secs(3));
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

    view! {
        <Layout>
            <h1 style="color: #2c3e50; margin-bottom: 20px;">"Моят профил"</h1>

            // Tabs
            <div style="display: flex; gap: 0; border-bottom: 2px solid #3498db;">
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
            </div>

            // Loading
            {move || {
                if loading.get() {
                    view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d;">"Зареждане..."</p>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Profile Tab
            {move || {
                if loading.get() || active_tab.get() != 0 {
                    return view! { <span></span> }.into_view();
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
                                        <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;" on:click=move |_| error_msg.set(String::new())>"×"</button>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <span></span> }.into_view()
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
                }.into_view()
            }}

            // Password Tab
            {move || {
                if loading.get() || active_tab.get() != 1 {
                    return view! { <span></span> }.into_view();
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
                                }.into_view()
                            } else {
                                view! { <span></span> }.into_view()
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
                                }.into_view()
                            } else {
                                view! { <span></span> }.into_view()
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
                }.into_view()
            }}
        </Layout>
    }
}
