use leptos::*;
use leptos_router::use_query_map;
use crate::api::{api_get, api_post_auth};

#[component]
pub fn ResetPassword() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.get().get("token").cloned().unwrap_or_default();

    let password = create_rw_signal(String::new());
    let confirm_password = create_rw_signal(String::new());
    let error = create_rw_signal(String::new());
    let success = create_rw_signal(false);
    let loading = create_rw_signal(false);
    let validating = create_rw_signal(true);
    let token_valid = create_rw_signal(false);
    let user_email = create_rw_signal(String::new());

    // Validate token on load
    create_effect(move |_| {
        let t = token();
        if t.is_empty() {
            validating.set(false);
            error.set("Липсващ токен за възстановяване".to_string());
            return;
        }

        spawn_local(async move {
            let url = format!("/api/auth/reset_password/validate?token={}", t);
            match api_get(&url).await {
                Ok(data) => {
                    if data.get("valid").and_then(|v| v.as_bool()).unwrap_or(false) {
                        token_valid.set(true);
                        if let Some(email) = data.get("email").and_then(|v| v.as_str()) {
                            user_email.set(email.to_string());
                        }
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Невалиден или изтекъл токен");
                        error.set(msg.to_string());
                    }
                }
                Err(_) => {
                    error.set("Грешка при проверка на токена".to_string());
                }
            }
            validating.set(false);
        });
    });

    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        if password.get().len() < 8 {
            error.set("Паролата трябва да е поне 8 символа".to_string());
            return;
        }
        if password.get() != confirm_password.get() {
            error.set("Паролите не съвпадат".to_string());
            return;
        }

        loading.set(true);
        error.set(String::new());

        let t = token();
        let pass = password.get();

        spawn_local(async move {
            let body = serde_json::json!({
                "token": t,
                "password": pass,
                "password_confirmation": pass,
            });

            match api_post_auth("/api/auth/reset_password", &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success.set(true);
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Грешка при смяна на парола");
                        error.set(msg.to_string());
                    }
                }
                Err(e) => {
                    error.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

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
            // Starfield
            <div style="position: absolute; inset: 0; pointer-events: none; overflow: hidden;">
                {(0..120).map(|i| {
                    let size = match i % 10 { 0 => 3, 1 | 2 => 2, _ => 1 };
                    let left = ((i as u32).wrapping_mul(7919) % 1000) as f32 / 10.0;
                    let top = ((i as u32).wrapping_mul(6271) % 1000) as f32 / 10.0;
                    let delay = (i % 8) as f32 * 0.35;
                    let duration = 2.5 + (i % 5) as f32 * 0.6;
                    let opacity = match size { 3 => 0.9, 2 => 0.5, _ => 0.25 };
                    let color = match i % 20 { 0 => "#aaccff", 7 => "#ffeedd", 14 => "#ffddaa", _ => "#ffffff" };
                    view! {
                        <div style=format!(
                            "position:absolute; width:{}px; height:{}px; background:{}; border-radius:50%; left:{}%; top:{}%; opacity:{}; animation:twinkle {}s ease-in-out {}s infinite;",
                            size, size, color, left, top, opacity, duration, delay
                        )></div>
                    }
                }).collect_view()}
            </div>

            // Nebula wisps
            <div style="position:absolute; width:500px; height:350px; pointer-events:none; background:radial-gradient(ellipse, rgba(30,15,50,0.2), transparent 70%); top:5%; right:10%;"></div>
            <div style="position:absolute; width:400px; height:250px; pointer-events:none; background:radial-gradient(ellipse, rgba(15,25,45,0.15), transparent 70%); bottom:10%; left:5%;"></div>

            // Planet edge
            <div style="position:absolute; bottom:-35%; right:-20%; width:700px; height:700px; border-radius:50%; background:radial-gradient(circle at 30% 30%, #12141f, #0a0c14 40%, #050608); box-shadow:-15px -15px 80px rgba(30,50,100,0.08); pointer-events:none;"></div>

            <div style="position: relative; z-index: 10; width: 100%; max-width: 420px; margin: 20px;">
                <div style="
                    background: rgba(8, 8, 14, 0.88);
                    backdrop-filter: blur(25px);
                    -webkit-backdrop-filter: blur(25px);
                    border-radius: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.06);
                    padding: 40px;
                    box-shadow: 0 25px 60px -12px rgba(0, 0, 0, 0.8);
                ">
                    // Header
                    <div style="text-align: center; margin-bottom: 30px;">
                        <div style="
                            width: 70px; height: 70px; margin: 0 auto 15px;
                            background: rgba(16, 185, 129, 0.12);
                            border-radius: 50%; display: flex; align-items: center; justify-content: center;
                            font-size: 30px; box-shadow: 0 10px 40px rgba(16, 185, 129, 0.15);
                        ">"🔐"</div>
                        <h1 style="font-size: 24px; font-weight: 700; color: white; margin: 0 0 5px 0;">"Нова парола"</h1>
                        <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0;">
                            "Въведете новата си парола"
                        </p>
                    </div>

                    // Loading state
                    {move || {
                        if validating.get() {
                            view! {
                                <div style="text-align: center; padding: 20px;">
                                    <p style="color: rgba(255, 255, 255, 0.5);">"Проверка на токена..."</p>
                                </div>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }
                    }}

                    // Success state
                    {move || {
                        if success.get() {
                            view! {
                                <div style="text-align: center; padding: 20px 0;">
                                    <div style="
                                        width: 60px; height: 60px; margin: 0 auto 20px;
                                        background: rgba(34, 197, 94, 0.15);
                                        border-radius: 50%; display: flex; align-items: center; justify-content: center;
                                        font-size: 30px;
                                    ">"✓"</div>
                                    <h3 style="color: #4ade80; margin: 0 0 10px 0;">"Паролата е променена!"</h3>
                                    <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0 0 20px 0;">
                                        "Можете да влезете с новата си парола."
                                    </p>
                                    <a
                                        href="/login"
                                        style="
                                            display: inline-block; padding: 12px 30px;
                                            background: linear-gradient(135deg, #b45309, #f59e0b);
                                            border-radius: 10px; color: white; text-decoration: none;
                                            font-weight: 600;
                                        "
                                    >"Към вход"</a>
                                </div>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }
                    }}

                    // Error state (invalid token)
                    {move || {
                        let err = error.get();
                        if !validating.get() && !token_valid.get() && !err.is_empty() && !success.get() {
                            view! {
                                <div style="text-align: center; padding: 20px 0;">
                                    <div style="
                                        width: 60px; height: 60px; margin: 0 auto 20px;
                                        background: rgba(239, 68, 68, 0.15);
                                        border-radius: 50%; display: flex; align-items: center; justify-content: center;
                                        font-size: 30px;
                                    ">"✕"</div>
                                    <h3 style="color: #f87171; margin: 0 0 10px 0;">"Невалиден линк"</h3>
                                    <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0 0 20px 0;">
                                        {err}
                                    </p>
                                    <a
                                        href="/forgot-password"
                                        style="
                                            display: inline-block; padding: 12px 30px;
                                            background: linear-gradient(135deg, #b45309, #f59e0b);
                                            border-radius: 10px; color: white; text-decoration: none;
                                            font-weight: 600;
                                        "
                                    >"Заяви нов линк"</a>
                                </div>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }
                    }}

                    // Form (only if token is valid)
                    {move || {
                        if validating.get() || !token_valid.get() || success.get() {
                            return view! { <span></span> }.into_view();
                        }

                        let email = user_email.get();

                        view! {
                            <div>
                                // Show email
                                {if !email.is_empty() {
                                    view! {
                                        <div style="
                                            background: rgba(217, 119, 6, 0.08);
                                            border: 1px solid rgba(217, 119, 6, 0.2);
                                            border-radius: 10px; padding: 12px; margin-bottom: 20px;
                                            text-align: center;
                                        ">
                                            <span style="color: rgba(255, 255, 255, 0.5); font-size: 13px;">"Акаунт: "</span>
                                            <span style="color: white; font-weight: 500;">{email}</span>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }}

                                // Form error
                                {move || {
                                    let err = error.get();
                                    if !err.is_empty() && token_valid.get() {
                                        view! {
                                            <div style="
                                                background: rgba(239, 68, 68, 0.15);
                                                border: 1px solid rgba(239, 68, 68, 0.25);
                                                color: #fca5a5; padding: 12px 16px; border-radius: 12px;
                                                margin-bottom: 20px; font-size: 14px;
                                            ">{err}</div>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }
                                }}

                                <form on:submit=handle_submit>
                                    <div style="margin-bottom: 15px;">
                                        <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Нова парола"</label>
                                        <input
                                            type="password"
                                            placeholder="Минимум 8 символа"
                                            class="cosmic-input"
                                            style="
                                                width: 100%; padding: 14px; background: rgba(255, 255, 255, 0.05);
                                                border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 12px;
                                                color: white; font-size: 15px; box-sizing: border-box; outline: none;
                                            "
                                            prop:value=move || password.get()
                                            on:input=move |e| password.set(event_target_value(&e))
                                        />
                                    </div>

                                    <div style="margin-bottom: 25px;">
                                        <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Потвърди парола"</label>
                                        <input
                                            type="password"
                                            placeholder="Повтори паролата"
                                            class="cosmic-input"
                                            style="
                                                width: 100%; padding: 14px; background: rgba(255, 255, 255, 0.05);
                                                border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 12px;
                                                color: white; font-size: 15px; box-sizing: border-box; outline: none;
                                            "
                                            prop:value=move || confirm_password.get()
                                            on:input=move |e| confirm_password.set(event_target_value(&e))
                                        />
                                    </div>

                                    <button
                                        type="submit"
                                        style="
                                            width: 100%; padding: 14px;
                                            background: linear-gradient(135deg, #10b981 0%, #059669 100%);
                                            border: none; border-radius: 12px; color: white; font-size: 15px;
                                            font-weight: 600; cursor: pointer; text-transform: uppercase; letter-spacing: 1px;
                                            box-shadow: 0 4px 15px rgba(16, 185, 129, 0.3);
                                        "
                                        disabled=move || loading.get()
                                    >
                                        {move || if loading.get() { "Запазване..." } else { "Запази нова парола" }}
                                    </button>
                                </form>
                            </div>
                        }.into_view()
                    }}
                </div>
            </div>

            <style>
                "@keyframes twinkle {
                    0%, 100% { opacity: 0.15; transform: scale(1); }
                    50% { opacity: 1; transform: scale(1.3); }
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
