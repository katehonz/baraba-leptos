use leptos::*;
use leptos_router::use_navigate;
use crate::api::{api_post_auth, set_token};
use crate::models::{LoginRequest, UserWrapper};

#[component]
pub fn Login() -> impl IntoView {
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let error = create_rw_signal(String::new());
    let loading = create_rw_signal(false);
    let needs_verification = create_rw_signal(false);
    let resend_loading = create_rw_signal(false);
    let resend_message = create_rw_signal(String::new());
    let navigate = use_navigate();

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
                    } else if let Some(token) = data["token"].as_str() {
                        set_token(token);
                        navigate("/dashboard", Default::default());
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

    // SVG: Imperial Star Destroyer
    let star_destroyer_svg = r##"<svg viewBox="0 0 400 140" xmlns="http://www.w3.org/2000/svg">
<polygon points="0,70 300,25 370,18 395,20 400,25 400,35 385,38 400,50 400,90 385,102 400,105 400,115 395,120 370,122 300,115" fill="rgba(130,135,145,0.18)"/>
<polygon points="280,25 300,8 330,5 345,8 350,25" fill="rgba(140,145,155,0.22)"/>
<rect x="305" y="8" width="20" height="12" rx="1" fill="rgba(120,125,135,0.18)"/>
<circle cx="310" cy="5" r="4" fill="rgba(130,135,145,0.18)"/>
<circle cx="335" cy="5" r="4" fill="rgba(130,135,145,0.18)"/>
<rect x="396" y="32" width="10" height="76" rx="2" fill="rgba(80,140,255,0.12)"/>
<rect x="398" y="40" width="6" height="60" rx="1" fill="rgba(120,170,255,0.15)"/>
<line x1="50" y1="55" x2="350" y2="27" stroke="rgba(100,105,115,0.08)" stroke-width="0.5"/>
<line x1="50" y1="85" x2="350" y2="113" stroke="rgba(100,105,115,0.08)" stroke-width="0.5"/>
<line x1="150" y1="50" x2="350" y2="30" stroke="rgba(100,105,115,0.06)" stroke-width="0.3"/>
<rect x="200" y="60" width="40" height="20" rx="2" fill="rgba(60,65,75,0.12)" stroke="rgba(100,105,115,0.08)" stroke-width="0.3"/>
</svg>"##;

    // SVG: TIE Fighter
    let tie_fighter_svg = r##"<svg viewBox="0 0 50 60" xmlns="http://www.w3.org/2000/svg">
<polygon points="2,2 8,0 8,60 2,58" fill="rgba(150,155,165,0.3)"/>
<polygon points="42,0 48,2 48,58 42,60" fill="rgba(150,155,165,0.3)"/>
<circle cx="25" cy="30" r="9" fill="rgba(140,145,155,0.22)" stroke="rgba(160,165,175,0.28)" stroke-width="0.5"/>
<circle cx="25" cy="30" r="4" fill="rgba(80,130,200,0.12)"/>
<line x1="8" y1="30" x2="16" y2="30" stroke="rgba(150,155,165,0.3)" stroke-width="2"/>
<line x1="34" y1="30" x2="42" y2="30" stroke="rgba(150,155,165,0.3)" stroke-width="2"/>
</svg>"##;

    // SVG: X-Wing
    let xwing_svg = r##"<svg viewBox="0 0 60 70" xmlns="http://www.w3.org/2000/svg">
<polygon points="27,2 33,2 35,65 25,65" fill="rgba(180,185,195,0.22)"/>
<line x1="30" y1="18" x2="3" y2="3" stroke="rgba(180,185,195,0.22)" stroke-width="2.5"/>
<line x1="30" y1="18" x2="57" y2="3" stroke="rgba(180,185,195,0.22)" stroke-width="2.5"/>
<line x1="30" y1="48" x2="3" y2="63" stroke="rgba(180,185,195,0.22)" stroke-width="2.5"/>
<line x1="30" y1="48" x2="57" y2="63" stroke="rgba(180,185,195,0.22)" stroke-width="2.5"/>
<circle cx="3" cy="3" r="2.5" fill="rgba(255,80,80,0.3)"/>
<circle cx="57" cy="3" r="2.5" fill="rgba(255,80,80,0.3)"/>
<circle cx="3" cy="63" r="2.5" fill="rgba(255,80,80,0.3)"/>
<circle cx="57" cy="63" r="2.5" fill="rgba(255,80,80,0.3)"/>
<ellipse cx="30" cy="22" rx="4" ry="7" fill="rgba(80,130,200,0.18)"/>
</svg>"##;

    // SVG: Space Station (Death Star silhouette)
    let station_svg = r##"<svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
<circle cx="100" cy="100" r="92" fill="rgba(55,58,68,0.07)" stroke="rgba(100,105,115,0.1)" stroke-width="0.8"/>
<ellipse cx="100" cy="60" rx="75" ry="10" fill="none" stroke="rgba(100,105,115,0.05)" stroke-width="0.5"/>
<ellipse cx="100" cy="102" rx="91" ry="8" fill="none" stroke="rgba(100,105,115,0.09)" stroke-width="1.5"/>
<ellipse cx="100" cy="140" rx="70" ry="8" fill="none" stroke="rgba(100,105,115,0.05)" stroke-width="0.5"/>
<circle cx="60" cy="60" r="28" fill="rgba(50,55,65,0.07)" stroke="rgba(100,105,115,0.08)" stroke-width="0.5"/>
<circle cx="60" cy="60" r="14" fill="rgba(80,200,80,0.03)"/>
<circle cx="60" cy="60" r="6" fill="rgba(80,255,80,0.02)"/>
<line x1="15" y1="85" x2="185" y2="85" stroke="rgba(100,105,115,0.04)" stroke-width="0.3"/>
<line x1="20" y1="120" x2="180" y2="120" stroke="rgba(100,105,115,0.04)" stroke-width="0.3"/>
<line x1="100" y1="8" x2="100" y2="95" stroke="rgba(100,105,115,0.03)" stroke-width="0.3"/>
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
            // Dense starfield
            <div style="position: absolute; inset: 0; pointer-events: none; overflow: hidden;">
                {(0..120).map(|i| {
                    let size = match i % 10 {
                        0 => 3,
                        1 | 2 => 2,
                        _ => 1,
                    };
                    let left = ((i as u32).wrapping_mul(7919) % 1000) as f32 / 10.0;
                    let top = ((i as u32).wrapping_mul(6271) % 1000) as f32 / 10.0;
                    let delay = (i % 8) as f32 * 0.35;
                    let duration = 2.5 + (i % 5) as f32 * 0.6;
                    let opacity = match size {
                        3 => 0.9,
                        2 => 0.5,
                        _ => 0.25,
                    };
                    let color = match i % 20 {
                        0 => "#aaccff",
                        7 => "#ffeedd",
                        14 => "#ffddaa",
                        _ => "#ffffff",
                    };
                    view! {
                        <div style=format!(
                            "position:absolute; width:{}px; height:{}px; background:{}; border-radius:50%; left:{}%; top:{}%; opacity:{}; animation:twinkle {}s ease-in-out {}s infinite;",
                            size, size, color, left, top, opacity, duration, delay
                        )></div>
                    }
                }).collect_view()}
            </div>

            // Nebula wisps
            <div style="
                position: absolute; width: 500px; height: 350px; pointer-events: none;
                background: radial-gradient(ellipse, rgba(30,15,50,0.2) 0%, transparent 70%);
                top: 5%; right: 10%;
            "></div>
            <div style="
                position: absolute; width: 400px; height: 250px; pointer-events: none;
                background: radial-gradient(ellipse, rgba(15,25,45,0.15) 0%, transparent 70%);
                bottom: 10%; left: 5%;
            "></div>

            // Planet edge (bottom-right)
            <div style="
                position: absolute;
                bottom: -35%;
                right: -20%;
                width: 700px;
                height: 700px;
                border-radius: 50%;
                background: radial-gradient(circle at 30% 30%, #12141f 0%, #0a0c14 40%, #050608 100%);
                box-shadow: -15px -15px 80px rgba(30,50,100,0.08), inset 5px 5px 40px rgba(0,0,0,0.6);
                pointer-events: none;
            "></div>

            // Space Station (far background, top-left)
            <div style="
                position: absolute; top: 3%; left: 3%;
                width: 220px; height: 220px;
                opacity: 0.6;
                animation: stationRotate 400s linear infinite;
                pointer-events: none;
            " inner_html=station_svg></div>

            // Star Destroyer 1 (large, drifting left)
            <div style="
                position: absolute; top: 10%;
                width: 380px;
                animation: driftLeft 100s linear infinite;
                pointer-events: none;
                opacity: 0.9;
            " inner_html=star_destroyer_svg></div>

            // Star Destroyer 2 (smaller, drifting right, flipped)
            <div style="
                position: absolute; bottom: 22%;
                width: 200px;
                animation: driftRight 130s linear infinite;
                pointer-events: none;
                opacity: 0.5;
                transform: scaleX(-1);
            " inner_html=star_destroyer_svg></div>

            // TIE Fighters (3 ships, drifting left at different speeds)
            {vec![
                ("top: 25%; width: 35px;", "driftLeft 50s linear 5s infinite"),
                ("top: 18%; width: 28px;", "driftLeft 45s linear 12s infinite"),
                ("top: 32%; width: 22px;", "driftLeft 55s linear 25s infinite"),
            ].into_iter().map(|(pos, anim)| {
                let tie = tie_fighter_svg;
                view! {
                    <div style=format!(
                        "position:absolute; {} animation:{}; pointer-events:none;",
                        pos, anim
                    ) inner_html=tie></div>
                }
            }).collect_view()}

            // X-Wings (2 ships, drifting right)
            {vec![
                ("bottom: 35%; width: 40px;", "driftRight 55s linear 8s infinite"),
                ("bottom: 40%; width: 32px;", "driftRight 50s linear 20s infinite"),
            ].into_iter().map(|(pos, anim)| {
                let xw = xwing_svg;
                view! {
                    <div style=format!(
                        "position:absolute; {} animation:{}; pointer-events:none;",
                        pos, anim
                    ) inner_html=xw></div>
                }
            }).collect_view()}

            // Laser bolts (red = Imperial, green = Rebel)
            {vec![
                ("top:22%; height:2px; width:25px; background:linear-gradient(90deg, transparent, #ff3333, #ff5555, transparent);", "laserFlyLeft 4s linear 2s infinite"),
                ("top:28%; height:2px; width:20px; background:linear-gradient(90deg, transparent, #ff3333, #ff5555, transparent);", "laserFlyLeft 3.5s linear 6s infinite"),
                ("top:15%; height:2px; width:18px; background:linear-gradient(90deg, transparent, #ff3333, transparent);", "laserFlyLeft 3s linear 10s infinite"),
                ("bottom:36%; height:2px; width:22px; background:linear-gradient(90deg, transparent, #44ff44, #66ff66, transparent);", "laserFlyRight 3.8s linear 4s infinite"),
                ("bottom:42%; height:2px; width:20px; background:linear-gradient(90deg, transparent, #44ff44, transparent);", "laserFlyRight 3.2s linear 9s infinite"),
                ("bottom:33%; height:2px; width:16px; background:linear-gradient(90deg, transparent, #44ff44, transparent);", "laserFlyRight 4.2s linear 15s infinite"),
            ].into_iter().map(|(style, anim)| {
                view! {
                    <div style=format!(
                        "position:absolute; {} border-radius:1px; animation:{}; pointer-events:none;",
                        style, anim
                    )></div>
                }
            }).collect_view()}

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
                                            }.into_view()
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }
                                    }}
                                </div>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
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

                    // Form
                    <form on:submit=handle_submit>
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
                                }.into_view()
                            } else {
                                view! { <span>"Вход"</span> }.into_view()
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

                    // Register link
                    <div style="text-align: center;">
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
                    50% { opacity: 1; transform: scale(1.3); }
                }
                @keyframes driftLeft {
                    0% { transform: translateX(100vw); }
                    100% { transform: translateX(-120vw); }
                }
                @keyframes driftRight {
                    0% { transform: translateX(-120vw); }
                    100% { transform: translateX(100vw); }
                }
                @keyframes laserFlyLeft {
                    0% { transform: translateX(100vw); opacity: 0; }
                    5% { opacity: 1; }
                    95% { opacity: 1; }
                    100% { transform: translateX(-100vw); opacity: 0; }
                }
                @keyframes laserFlyRight {
                    0% { transform: translateX(-100vw); opacity: 0; }
                    5% { opacity: 1; }
                    95% { opacity: 1; }
                    100% { transform: translateX(100vw); opacity: 0; }
                }
                @keyframes stationRotate {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }
                @keyframes spin {
                    to { transform: rotate(360deg); }
                }
                @keyframes eyeGlow {
                    0%, 100% { opacity: 0.7; }
                    50% { opacity: 1; }
                }
                .eye-glow {
                    animation: eyeGlow 3s ease-in-out infinite;
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
