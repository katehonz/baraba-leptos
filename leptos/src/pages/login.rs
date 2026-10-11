use leptos::prelude::*;
use leptos::ev;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use crate::api::{api_post_auth, api_get_public, set_token};
use crate::models::{LoginRequest, UserWrapper};

#[component]
pub fn Login() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let needs_verification = RwSignal::new(false);
    let resend_loading = RwSignal::new(false);
    let resend_message = RwSignal::new(String::new());
    let registration_enabled = RwSignal::new(true);
    let needs_2fa = RwSignal::new(false);
    let challenge_token = RwSignal::new(String::new());
    let totp_code = RwSignal::new(String::new());
    let navigate = use_navigate();

    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(data) = api_get_public("/api/auth/registration_status").await {
                if let Some(enabled) = data.get("registration_enabled").and_then(|v| v.as_bool()) {
                    registration_enabled.set(enabled);
                }
            }
        });
    });

    let navigate_totp = navigate.clone();

    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let email_val = email.get();
        let password_val = password.get();

        if email_val.is_empty() || password_val.is_empty() {
            error.set("Моля въведете email и парола".to_string());
            return;
        }

        loading.set(true);
        error.set(String::new());
        needs_verification.set(false);
        needs_2fa.set(false);
        resend_message.set(String::new());

        let navigate = navigate.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let request = UserWrapper {
                user: LoginRequest {
                    email: email_val,
                    password: password_val
                }
            };

            match api_post_auth("/api/auth/signin", &request).await {
                Ok(data) => {
                    if data.get("requires_verification").and_then(|v| v.as_bool()).unwrap_or(false) {
                        needs_verification.set(true);
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Моля, потвърдете вашия email.");
                        error.set(msg.to_string());
                    } else if data.get("requires_2fa").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Some(challenge) = data.get("challenge_token").and_then(|v| v.as_str()) {
                            challenge_token.set(challenge.to_string());
                            totp_code.set(String::new());
                            needs_2fa.set(true);
                        } else {
                            error.set("Неуспешно стартиране на двуфакторната защита".to_string());
                        }
                    } else if let Some(token) = data.get("token").and_then(|v| v.as_str()) {
                        set_token(token);
                        navigate("/dashboard", Default::default());
                    } else if let Some(msg) = data.get("message").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                        error.set(msg.to_string());
                    } else {
                        error.set("Невалиден email или парола".to_string());
                    }
                }
                Err(e) => {
                    error.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    let resend_verification = move |_| {
        let email_val = email.get();
        if email_val.is_empty() {
            return;
        }

        resend_loading.set(true);
        resend_message.set(String::new());

        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({ "email": email_val });

            match api_post_auth("/api/auth/resend_verification", &body).await {
                Ok(data) => {
                    let msg = data.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Email изпратен.");
                    resend_message.set(msg.to_string());
                }
                Err(e) => {
                    resend_message.set(format!("Грешка: {}", e));
                }
            }
            resend_loading.set(false);
        });
    };

    let submit_totp = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let code = totp_code.get();
        let challenge = challenge_token.get();
        if code.trim().is_empty() {
            error.set("Въведете код от приложението или код за възстановяване".to_string());
            return;
        }
        loading.set(true);
        error.set(String::new());
        let navigate = navigate_totp.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({
                "challenge_token": challenge,
                "code": code,
            });
            match api_post_auth("/api/auth/signin/totp", &body).await {
                Ok(data) => {
                    if let Some(token) = data.get("token").and_then(|v| v.as_str()) {
                        set_token(token);
                        navigate("/dashboard", Default::default());
                    } else {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Невалиден код");
                        error.set(msg.to_string());
                    }
                }
                Err(e) => error.set(format!("Грешка: {}", e)),
            }
            loading.set(false);
        });
    };

    let back_to_password = move |_| {
        needs_2fa.set(false);
        challenge_token.set(String::new());
        totp_code.set(String::new());
        error.set(String::new());
    };

    // SVG: Hooded vagabond (Jawa-style space scavenger)
    let vagabond_svg = r##"<svg viewBox="0 0 80 90" xmlns="http://www.w3.org/2000/svg" style="filter:drop-shadow(0 0 15px rgba(255,170,0,0.3))">
<path d="M18,42 Q16,18 40,8 Q64,18 62,42 L68,84 Q40,90 12,84 Z" fill="#1a1510"/>
<path d="M24,40 Q23,20 40,13 Q57,20 56,40 L54,48 Q40,52 26,48 Z" fill="#0a0806"/>
<path d="M22,48 L17,82" stroke="rgba(50,40,25,0.4)" stroke-width="0.7" fill="none"/>
<path d="M58,48 L63,82" stroke="rgba(50,40,25,0.4)" stroke-width="0.7" fill="none"/>
<path d="M33,52 L31,80" stroke="rgba(40,32,20,0.25)" stroke-width="0.4" fill="none"/>
<path d="M47,52 L49,80" stroke="rgba(40,32,20,0.25)" stroke-width="0.4" fill="none"/>
<path d="M40,50 L40,82" stroke="rgba(40,32,20,0.2)" stroke-width="0.3" fill="none"/>
<path d="M26,56 Q40,60 54,56" stroke="rgba(80,65,40,0.6)" stroke-width="1.5" fill="none"/>
<circle cx="40" cy="58" r="2" fill="rgba(120,100,60,0.5)"/>
<ellipse cx="34" cy="33" rx="4.5" ry="3" fill="#ffaa00" class="eye-glow"/>
<ellipse cx="46" cy="33" rx="4.5" ry="3" fill="#ffaa00" class="eye-glow"/>
<ellipse cx="34" cy="33" rx="2.5" ry="1.5" fill="#ffcc44"/>
<ellipse cx="46" cy="33" rx="2.5" ry="1.5" fill="#ffcc44"/>
<ellipse cx="34" cy="33" rx="8" ry="5.5" fill="rgba(255,170,0,0.2)"/>
<ellipse cx="46" cy="33" rx="8" ry="5.5" fill="rgba(255,170,0,0.2)"/>
</svg>"##;

    view! {
        <div style="
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            background: radial-gradient(ellipse at 20% 40%, #0d0b12 0%, #060608 40%, #030304 100%);
            position: relative;
            overflow: hidden;
        ">
            // Simple starfield (30 stars, lighter)
            <div style="position: absolute; inset: 0; pointer-events: none; overflow: hidden;">
                {(0..30).map(|i| {
                    let size = if i % 3 == 0 { 2 } else { 1 };
                    let left = ((i as u32).wrapping_mul(7919) % 1000) as f32 / 10.0;
                    let top = ((i as u32).wrapping_mul(6271) % 1000) as f32 / 10.0;
                    let delay = (i % 6) as f32 * 0.5;
                    let duration = 3.0 + (i % 3) as f32 * 0.5;
                    view! {
                        <div style=format!(
                            "position:absolute; width:{}px; height:{}px; background:#fff; border-radius:50%; left:{}%; top:{}%; opacity:0.3; animation:twinkle {}s ease-in-out {}s infinite;",
                            size, size, left, top, duration, delay
                        )></div>
                    }
                }).collect_view()}
            </div>

            // Subtle nebula
            <div style="
                position: absolute; width: 400px; height: 300px; pointer-events: none;
                background: radial-gradient(ellipse, rgba(30,15,50,0.15) 0%, transparent 70%);
                top: 10%; right: 15%;
            "></div>

            // Login card
            <div style="
                position: relative;
                z-index: 10;
                width: 100%;
                max-width: 420px;
                margin: 20px;
            ">
                // Glass card (darker)
                <div style="
                    background: rgba(8, 8, 14, 0.88);
                    backdrop-filter: blur(25px);
                    -webkit-backdrop-filter: blur(25px);
                    border-radius: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.06);
                    padding: 40px;
                    box-shadow: 0 25px 60px -12px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.03);
                ">
                    // Logo/Brand
                    <div style="text-align: center; margin-bottom: 35px;">
                        <div style="
                            width: 80px;
                            height: 90px;
                            margin: 0 auto 15px;
                        " inner_html=vagabond_svg></div>
                        <h1 style="
                            font-size: 28px;
                            font-weight: 700;
                            color: white;
                            margin: 0 0 6px 0;
                            letter-spacing: -0.5px;
                        ">"Baraba"</h1>
                        <p style="
                            color: rgba(255, 170, 0, 0.7);
                            font-size: 13px;
                            margin: 0 0 4px 0;
                            font-style: italic;
                            letter-spacing: 0.5px;
                        ">"Изкуството на заблудата"</p>
                        <p style="
                            color: rgba(255, 255, 255, 0.35);
                            font-size: 12px;
                            margin: 0;
                        ">"Счетоводна система"</p>
                    </div>

                    // Verification needed message
                    {move || {
                        if needs_verification.get() {
                            view! {
                                <div style="
                                    background: rgba(59, 130, 246, 0.12);
                                    border: 1px solid rgba(59, 130, 246, 0.25);
                                    color: #93c5fd;
                                    padding: 16px;
                                    border-radius: 12px;
                                    margin-bottom: 20px;
                                    font-size: 14px;
                                ">
                                    <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 12px;">
                                        <span style="font-size: 20px;">"📧"</span>
                                        <span style="font-weight: 600;">"Email не е потвърден"</span>
                                    </div>
                                    <p style="margin: 0 0 12px 0; color: rgba(255, 255, 255, 0.6);">
                                        "Моля, проверете вашата поща и кликнете на линка за потвърждение."
                                    </p>
                                    <button
                                        style="
                                            padding: 10px 20px;
                                            background: rgba(59, 130, 246, 0.25);
                                            border: 1px solid rgba(59, 130, 246, 0.4);
                                            border-radius: 8px;
                                            color: white;
                                            font-size: 13px;
                                            cursor: pointer;
                                            width: 100%;
                                        "
                                        on:click=resend_verification
                                        disabled=move || resend_loading.get()
                                    >
                                        {move || if resend_loading.get() { "Изпращане..." } else { "Изпрати отново email" }}
                                    </button>
                                    {move || {
                                        let msg = resend_message.get();
                                        if !msg.is_empty() {
                                            view! {
                                                <p style="margin: 10px 0 0 0; font-size: 13px; color: #4ade80;">
                                                    {msg}
                                                </p>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}

                    // Error message
                    {move || {
                        let err = error.get();
                        if err.is_empty() || needs_verification.get() {
                            None
                        } else {
                            Some(view! {
                                <div style="
                                    background: rgba(239, 68, 68, 0.15);
                                    border: 1px solid rgba(239, 68, 68, 0.25);
                                    color: #fca5a5;
                                    padding: 12px 16px;
                                    border-radius: 12px;
                                    margin-bottom: 20px;
                                    font-size: 14px;
                                    display: flex;
                                    align-items: center;
                                    gap: 10px;
                                ">
                                    <span style="font-size: 18px;">"!"</span>
                                    {err}
                                </div>
                            })
                        }
                    }}

                    <form
                        on:submit=submit_totp
                        style=move || if needs_2fa.get() { "" } else { "display: none;" }
                    >
                                <p style="color: rgba(255, 255, 255, 0.7); font-size: 14px; margin: 0 0 16px 0;">
                                    "Въведете 6-цифрения код от приложението или код за възстановяване."
                                </p>
                                <div style="margin-bottom: 20px;">
                                    <label style="
                                        display: block;
                                        color: rgba(255, 255, 255, 0.6);
                                        font-size: 13px;
                                        font-weight: 500;
                                        margin-bottom: 8px;
                                        text-transform: uppercase;
                                        letter-spacing: 0.5px;
                                    ">"Код"</label>
                                    <input
                                        type="text"
                                        autocomplete="one-time-code"
                                        placeholder="123456"
                                        class="cosmic-input"
                                        style="
                                            width: 100%;
                                            padding: 14px 16px;
                                            background: rgba(255, 255, 255, 0.05);
                                            border: 1px solid rgba(255, 255, 255, 0.1);
                                            border-radius: 12px;
                                            color: white;
                                            font-size: 18px;
                                            letter-spacing: 2px;
                                            box-sizing: border-box;
                                            outline: none;
                                        "
                                        prop:value=move || totp_code.get()
                                        on:input=move |e| totp_code.set(event_target_value(&e))
                                    />
                                </div>
                                <button
                                    type="submit"
                                    style="
                                        width: 100%;
                                        padding: 14px;
                                        background: linear-gradient(135deg, #b45309 0%, #d97706 50%, #f59e0b 100%);
                                        border: none;
                                        border-radius: 12px;
                                        color: white;
                                        font-size: 16px;
                                        font-weight: 600;
                                        cursor: pointer;
                                        text-transform: uppercase;
                                        letter-spacing: 1px;
                                    "
                                    disabled=move || loading.get()
                                >
                                    {move || if loading.get() { "Проверка..." } else { "Потвърди" }}
                                </button>
                                <button
                                    type="button"
                                    style="
                                        width: 100%;
                                        margin-top: 12px;
                                        padding: 12px;
                                        background: transparent;
                                        border: none;
                                        color: rgba(255, 255, 255, 0.5);
                                        font-size: 13px;
                                        cursor: pointer;
                                    "
                                    on:click=back_to_password
                                >
                                    "Назад към паролата"
                                </button>
                    </form>

                    // Form
                    <form
                        on:submit=handle_submit
                        style=move || if needs_2fa.get() { "display: none;" } else { "" }
                    >
                        <div style="margin-bottom: 20px;">
                            <label style="
                                display: block;
                                color: rgba(255, 255, 255, 0.6);
                                font-size: 13px;
                                font-weight: 500;
                                margin-bottom: 8px;
                                text-transform: uppercase;
                                letter-spacing: 0.5px;
                            ">"Email"</label>
                            <input
                                type="email"
                                placeholder="you@example.com"
                                class="cosmic-input"
                                style="
                                    width: 100%;
                                    padding: 14px 16px;
                                    background: rgba(255, 255, 255, 0.05);
                                    border: 1px solid rgba(255, 255, 255, 0.1);
                                    border-radius: 12px;
                                    color: white;
                                    font-size: 15px;
                                    box-sizing: border-box;
                                    transition: all 0.2s ease;
                                    outline: none;
                                "
                                prop:value=move || email.get()
                                on:input=move |e| email.set(event_target_value(&e))
                            />
                        </div>

                        <div style="margin-bottom: 25px;">
                            <label style="
                                display: block;
                                color: rgba(255, 255, 255, 0.6);
                                font-size: 13px;
                                font-weight: 500;
                                margin-bottom: 8px;
                                text-transform: uppercase;
                                letter-spacing: 0.5px;
                            ">"Парола"</label>
                            <input
                                type="password"
                                placeholder="********"
                                class="cosmic-input"
                                style="
                                    width: 100%;
                                    padding: 14px 16px;
                                    background: rgba(255, 255, 255, 0.05);
                                    border: 1px solid rgba(255, 255, 255, 0.1);
                                    border-radius: 12px;
                                    color: white;
                                    font-size: 15px;
                                    box-sizing: border-box;
                                    transition: all 0.2s ease;
                                    outline: none;
                                "
                                prop:value=move || password.get()
                                on:input=move |e| password.set(event_target_value(&e))
                            />
                        </div>

                        // Forgot password link
                        <div style="text-align: right; margin-bottom: 25px;">
                            <a
                                href="/forgot-password"
                                style="
                                    color: rgba(255, 255, 255, 0.5);
                                    font-size: 13px;
                                    text-decoration: none;
                                    transition: color 0.2s ease;
                                "
                            >"Забравена парола?"</a>
                        </div>

                        // Submit button
                        <button
                            type="submit"
                            style="
                                width: 100%;
                                padding: 14px;
                                background: linear-gradient(135deg, #b45309 0%, #d97706 50%, #f59e0b 100%);
                                border: none;
                                border-radius: 12px;
                                color: white;
                                font-size: 16px;
                                font-weight: 600;
                                cursor: pointer;
                                transition: all 0.3s ease;
                                box-shadow: 0 4px 20px rgba(217, 119, 6, 0.35);
                                text-transform: uppercase;
                                letter-spacing: 1px;
                            "
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() {
                                view! {
                                    <span style="display: flex; align-items: center; justify-content: center; gap: 10px;">
                                        <span style="
                                            width: 18px;
                                            height: 18px;
                                            border: 2px solid rgba(255,255,255,0.3);
                                            border-top-color: white;
                                            border-radius: 50%;
                                            animation: spin 0.8s linear infinite;
                                        "></span>
                                        "Влизане..."
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span>"Вход"</span> }.into_any()
                            }}
                        </button>
                    </form>

                    // Divider
                    <div style="
                        display: flex;
                        align-items: center;
                        margin: 30px 0;
                        gap: 15px;
                    ">
                        <div style="flex: 1; height: 1px; background: rgba(255, 255, 255, 0.08);"></div>
                        <span style="color: rgba(255, 255, 255, 0.3); font-size: 12px; text-transform: uppercase; letter-spacing: 1px;">"или"</span>
                        <div style="flex: 1; height: 1px; background: rgba(255, 255, 255, 0.08);"></div>
                    </div>

                    // Register link or disabled message
                    <div style="text-align: center;">
                        {move || {
                            if registration_enabled.get() {
                                view! {
                                    <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0;">
                                        "Нямате акаунт? "
                                        <a
                                            href="/register"
                                            style="
                                                color: #fbbf24;
                                                text-decoration: none;
                                                font-weight: 600;
                                                transition: color 0.2s ease;
                                            "
                                        >"Регистрация"</a>
                                    </p>
                                }.into_any()
                            } else {
                                view! {
                                    <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0;">
                                        "Регистрацията е временно спряна заради злонамерени действия."
                                    </p>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Footer
                <p style="
                    text-align: center;
                    color: rgba(255, 255, 255, 0.15);
                    font-size: 12px;
                    margin-top: 25px;
                ">"2025 Baraba. Всички права запазени."</p>
            </div>

            // CSS animations
            <style>
                "@keyframes twinkle {
                    0%, 100% { opacity: 0.15; transform: scale(1); }
                    50% { opacity: 0.6; transform: scale(1.2); }
                }
                @keyframes spin {
                    to { transform: rotate(360deg); }
                }
                .eye-glow {
                    animation: eyeGlow 3s ease-in-out infinite;
                }
                @keyframes eyeGlow {
                    0%, 100% { opacity: 0.7; }
                    50% { opacity: 1; }
                }
                .cosmic-input:focus {
                    border-color: rgba(255, 170, 0, 0.4) !important;
                    background: rgba(255, 255, 255, 0.08) !important;
                    box-shadow: 0 0 15px rgba(255, 170, 0, 0.1);
                }
                .cosmic-input::placeholder {
                    color: rgba(255, 255, 255, 0.3);
                }"
            </style>
        </div>
    }
}
