use leptos::*;
use crate::api::api_post_auth;

#[component]
pub fn ForgotPassword() -> impl IntoView {
    let email = create_rw_signal(String::new());
    let error = create_rw_signal(String::new());
    let success = create_rw_signal(false);
    let loading = create_rw_signal(false);

    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        if email.get().is_empty() {
            error.set("Моля въведете email".to_string());
            return;
        }

        loading.set(true);
        error.set(String::new());

        let email_val = email.get();

        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({ "email": email_val });

            match api_post_auth("/api/auth/forgot_password", &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(true) {
                        success.set(true);
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Грешка при изпращане");
                        error.set(msg.to_string());
                    }
                }
                Err(_) => {
                    // Even on error, show success to prevent email enumeration
                    success.set(true);
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
                            background: rgba(217, 119, 6, 0.15);
                            border-radius: 50%; display: flex; align-items: center; justify-content: center;
                            font-size: 30px; box-shadow: 0 10px 40px rgba(217, 119, 6, 0.15);
                        ">"?"</div>
                        <h1 style="font-size: 24px; font-weight: 700; color: white; margin: 0 0 5px 0;">"Забравена парола"</h1>
                        <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0;">
                            "Въведете email за възстановяване"
                        </p>
                    </div>

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
                                    ">"✉"</div>
                                    <h3 style="color: #4ade80; margin: 0 0 10px 0;">"Email изпратен!"</h3>
                                    <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0 0 20px 0;">
                                        "Ако съществува акаунт с този email, ще получите линк за възстановяване на паролата."
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

                    // Form
                    {move || {
                        if success.get() {
                            return view! { <span></span> }.into_view();
                        }

                        view! {
                            <div>
                                // Error
                                {move || {
                                    let err = error.get();
                                    if !err.is_empty() {
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
                                    <div style="margin-bottom: 25px;">
                                        <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Email"</label>
                                        <input
                                            type="email"
                                            placeholder="you@example.com"
                                            class="cosmic-input"
                                            style="
                                                width: 100%; padding: 14px; background: rgba(255, 255, 255, 0.05);
                                                border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 12px;
                                                color: white; font-size: 15px; box-sizing: border-box; outline: none;
                                            "
                                            prop:value=move || email.get()
                                            on:input=move |e| email.set(event_target_value(&e))
                                        />
                                    </div>

                                    <button
                                        type="submit"
                                        style="
                                            width: 100%; padding: 14px;
                                            background: linear-gradient(135deg, #b45309 0%, #d97706 50%, #f59e0b 100%);
                                            border: none; border-radius: 12px; color: white; font-size: 15px;
                                            font-weight: 600; cursor: pointer; text-transform: uppercase; letter-spacing: 1px;
                                            box-shadow: 0 4px 15px rgba(217, 119, 6, 0.35);
                                        "
                                        disabled=move || loading.get()
                                    >
                                        {move || if loading.get() { "Изпращане..." } else { "Изпрати линк" }}
                                    </button>
                                </form>

                                <div style="text-align: center; margin-top: 25px;">
                                    <a href="/login" style="color: rgba(255, 255, 255, 0.5); font-size: 14px; text-decoration: none;">
                                        "← Обратно към вход"
                                    </a>
                                </div>
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
