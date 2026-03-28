use leptos::*;
use crate::api::{set_token, set_selected_company_id, API_BASE};

#[component]
pub fn VerifyEmail() -> impl IntoView {
    let status = create_rw_signal(String::from("loading")); // loading, success, error
    let message = create_rw_signal(String::new());
    let company_id = create_rw_signal(Option::<i64>::None);

    // Get token from URL
    create_effect(move |_| {
        let window = web_sys::window().unwrap();
        let search = window.location().search().unwrap_or_default();

        // Parse token from query string
        let token = search
            .strip_prefix("?")
            .and_then(|s| {
                s.split('&')
                    .find(|p| p.starts_with("token="))
                    .map(|p| p.strip_prefix("token=").unwrap_or("").to_string())
            });

        if let Some(token) = token {
            verify_email(token, status, message, company_id);
        } else {
            status.set("error".to_string());
            message.set("Невалиден линк за потвърждение".to_string());
        }
    });

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

            // Planet edge
            <div style="position:absolute; bottom:-35%; right:-20%; width:700px; height:700px; border-radius:50%; background:radial-gradient(circle at 30% 30%, #12141f, #0a0c14 40%, #050608); box-shadow:-15px -15px 80px rgba(30,50,100,0.08); pointer-events:none;"></div>

            <div style="
                position: relative; z-index: 10;
                background: rgba(8, 8, 14, 0.88);
                backdrop-filter: blur(25px);
                -webkit-backdrop-filter: blur(25px);
                border-radius: 24px;
                border: 1px solid rgba(255, 255, 255, 0.06);
                padding: 40px;
                max-width: 450px;
                width: 100%;
                margin: 20px;
                text-align: center;
                box-shadow: 0 25px 60px -12px rgba(0, 0, 0, 0.8);
            ">
                // Loading state
                {move || {
                    if status.get() == "loading" {
                        view! {
                            <div>
                                <div style="
                                    width: 60px; height: 60px; margin: 0 auto 20px;
                                    border: 3px solid rgba(217, 119, 6, 0.2);
                                    border-top: 3px solid #d97706;
                                    border-radius: 50%;
                                    animation: spin 1s linear infinite;
                                "></div>
                                <h2 style="color: white; margin: 0 0 10px 0;">"Потвърждаване..."</h2>
                                <p style="color: rgba(255, 255, 255, 0.5);">"Моля, изчакайте докато потвърдим вашия email."</p>
                            </div>
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    }
                }}

                // Success state
                {move || {
                    if status.get() == "success" {
                        view! {
                            <div>
                                <div style="
                                    width: 70px; height: 70px; margin: 0 auto 20px;
                                    background: rgba(34, 197, 94, 0.15);
                                    border-radius: 50%; display: flex; align-items: center; justify-content: center;
                                    font-size: 35px;
                                ">"✓"</div>
                                <h2 style="color: #4ade80; margin: 0 0 10px 0;">"Email потвърден!"</h2>
                                <p style="color: rgba(255, 255, 255, 0.5); margin-bottom: 25px;">
                                    {message.get()}
                                </p>
                                <a
                                    href="/dashboard"
                                    style="
                                        display: inline-block; padding: 14px 35px;
                                        background: linear-gradient(135deg, #10b981 0%, #059669 100%);
                                        border-radius: 12px; color: white; text-decoration: none;
                                        font-weight: 600; font-size: 15px;
                                    "
                                >"Към таблото"</a>
                            </div>
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    }
                }}

                // Error state
                {move || {
                    if status.get() == "error" {
                        view! {
                            <div>
                                <div style="
                                    width: 70px; height: 70px; margin: 0 auto 20px;
                                    background: rgba(239, 68, 68, 0.15);
                                    border-radius: 50%; display: flex; align-items: center; justify-content: center;
                                    font-size: 35px;
                                ">"✗"</div>
                                <h2 style="color: #f87171; margin: 0 0 10px 0;">"Грешка"</h2>
                                <p style="color: rgba(255, 255, 255, 0.5); margin-bottom: 25px;">
                                    {message.get()}
                                </p>
                                <div style="display: flex; gap: 10px; justify-content: center;">
                                    <a
                                        href="/login"
                                        style="
                                            display: inline-block; padding: 12px 25px;
                                            background: rgba(255, 255, 255, 0.06);
                                            border: 1px solid rgba(255, 255, 255, 0.12);
                                            border-radius: 10px; color: white; text-decoration: none;
                                            font-weight: 500;
                                        "
                                    >"Към вход"</a>
                                    <a
                                        href="/register"
                                        style="
                                            display: inline-block; padding: 12px 25px;
                                            background: linear-gradient(135deg, #b45309, #f59e0b);
                                            border-radius: 10px; color: white; text-decoration: none;
                                            font-weight: 500;
                                        "
                                    >"Регистрация"</a>
                                </div>
                            </div>
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    }
                }}
            </div>

            <style>
                "@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
                @keyframes twinkle { 0%, 100% { opacity: 0.15; transform: scale(1); } 50% { opacity: 1; transform: scale(1.3); } }"
            </style>
        </div>
    }
}

fn verify_email(
    token: String,
    status: RwSignal<String>,
    message: RwSignal<String>,
    company_id: RwSignal<Option<i64>>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{}/api/auth/verify_email?token={}", API_BASE, token);

        match gloo_net::http::Request::get(&url).send().await {
            Ok(response) => {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            // Save token if provided (auto-login)
                            if let Some(auth_token) = data.get("token").and_then(|v| v.as_str()) {
                                set_token(auth_token);
                            }

                            // Save company ID if available
                            if let Some(user) = data.get("user") {
                                // User is verified, they can now login
                            }

                            status.set("success".to_string());
                            message.set("Вашият акаунт е активиран успешно!".to_string());
                        } else {
                            status.set("error".to_string());
                            let msg = data.get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Грешка при потвърждение");
                            message.set(msg.to_string());
                        }
                    }
                    Err(e) => {
                        status.set("error".to_string());
                        message.set(format!("Грешка: {}", e));
                    }
                }
            }
            Err(e) => {
                status.set("error".to_string());
                message.set(format!("Грешка при свързване: {}", e));
            }
        }
    });
}
