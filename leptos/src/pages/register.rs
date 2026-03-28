use leptos::*;
use crate::api::{api_post_auth, set_token, set_selected_company_id};

#[component]
pub fn Register() -> impl IntoView {
    // Step: 1=user details, 2=company details
    let step = create_rw_signal(1);

    // User fields
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let confirm_password = create_rw_signal(String::new());
    let first_name = create_rw_signal(String::new());
    let last_name = create_rw_signal(String::new());

    // Company fields
    let company_name = create_rw_signal(String::new());
    let company_eik = create_rw_signal(String::new());
    let company_vat = create_rw_signal(String::new());
    let company_address = create_rw_signal(String::new());
    let company_city = create_rw_signal(String::new());

    let error = create_rw_signal(String::new());
    let success = create_rw_signal(false);
    let loading = create_rw_signal(false);

    let go_back = move |_| {
        let current = step.get();
        if current > 1 {
            step.set(current - 1);
        }
    };

    let validate_user = move |_| {
        error.set(String::new());

        if first_name.get().is_empty() || last_name.get().is_empty() {
            error.set("Моля въведете име и фамилия".to_string());
            return;
        }
        if email.get().is_empty() || !email.get().contains('@') {
            error.set("Моля въведете валиден email".to_string());
            return;
        }
        if password.get().len() < 8 {
            error.set("Паролата трябва да е поне 8 символа".to_string());
            return;
        }
        if password.get() != confirm_password.get() {
            error.set("Паролите не съвпадат".to_string());
            return;
        }

        step.set(2);
    };

    let submit_with_company = move |_| {
        error.set(String::new());

        if company_name.get().is_empty() {
            error.set("Моля въведете име на фирмата".to_string());
            return;
        }

        register_with_company(
            email.get(), password.get(), first_name.get(), last_name.get(),
            company_name.get(), company_eik.get(), company_vat.get(),
            company_address.get(), company_city.get(),
            loading, error, success
        );
    };

    // SVG: Hooded vagabond logo
    let vagabond_svg = r##"<svg viewBox="0 0 80 90" xmlns="http://www.w3.org/2000/svg" style="filter:drop-shadow(0 0 12px rgba(255,170,0,0.25))">
<path d="M18,42 Q16,18 40,8 Q64,18 62,42 L68,84 Q40,90 12,84 Z" fill="#1a1510"/>
<path d="M24,40 Q23,20 40,13 Q57,20 56,40 L54,48 Q40,52 26,48 Z" fill="#0a0806"/>
<path d="M22,48 L17,82" stroke="rgba(50,40,25,0.4)" stroke-width="0.7" fill="none"/>
<path d="M58,48 L63,82" stroke="rgba(50,40,25,0.4)" stroke-width="0.7" fill="none"/>
<path d="M33,52 L31,80" stroke="rgba(40,32,20,0.25)" stroke-width="0.4" fill="none"/>
<path d="M47,52 L49,80" stroke="rgba(40,32,20,0.25)" stroke-width="0.4" fill="none"/>
<path d="M26,56 Q40,60 54,56" stroke="rgba(80,65,40,0.6)" stroke-width="1.5" fill="none"/>
<circle cx="40" cy="58" r="2" fill="rgba(120,100,60,0.5)"/>
<ellipse cx="34" cy="33" rx="4.5" ry="3" fill="#ffaa00" class="eye-glow"/>
<ellipse cx="46" cy="33" rx="4.5" ry="3" fill="#ffaa00" class="eye-glow"/>
<ellipse cx="34" cy="33" rx="2.5" ry="1.5" fill="#ffcc44"/>
<ellipse cx="46" cy="33" rx="2.5" ry="1.5" fill="#ffcc44"/>
<ellipse cx="34" cy="33" rx="8" ry="5.5" fill="rgba(255,170,0,0.2)"/>
<ellipse cx="46" cy="33" rx="8" ry="5.5" fill="rgba(255,170,0,0.2)"/>
</svg>"##;

    // SVG: Star Destroyer
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

    // SVG: Space Station
    let station_svg = r##"<svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
<circle cx="100" cy="100" r="92" fill="rgba(55,58,68,0.07)" stroke="rgba(100,105,115,0.1)" stroke-width="0.8"/>
<ellipse cx="100" cy="102" rx="91" ry="8" fill="none" stroke="rgba(100,105,115,0.09)" stroke-width="1.5"/>
<circle cx="60" cy="60" r="28" fill="rgba(50,55,65,0.07)" stroke="rgba(100,105,115,0.08)" stroke-width="0.5"/>
<circle cx="60" cy="60" r="14" fill="rgba(80,200,80,0.03)"/>
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

            // Space Station
            <div style="position:absolute; top:3%; left:3%; width:180px; height:180px; opacity:0.5; animation:stationRotate 400s linear infinite; pointer-events:none;" inner_html=station_svg></div>

            // Star Destroyer
            <div style="position:absolute; top:8%; width:320px; animation:driftLeft 110s linear infinite; pointer-events:none; opacity:0.8;" inner_html=star_destroyer_svg></div>

            // TIE Fighters
            {vec![
                ("top:20%; width:30px;", "driftLeft 50s linear 5s infinite"),
                ("top:28%; width:22px;", "driftLeft 55s linear 18s infinite"),
            ].into_iter().map(|(pos, anim)| {
                let tie = tie_fighter_svg;
                view! { <div style=format!("position:absolute; {} animation:{}; pointer-events:none;", pos, anim) inner_html=tie></div> }
            }).collect_view()}

            // X-Wings
            {vec![
                ("bottom:30%; width:35px;", "driftRight 55s linear 10s infinite"),
            ].into_iter().map(|(pos, anim)| {
                let xw = xwing_svg;
                view! { <div style=format!("position:absolute; {} animation:{}; pointer-events:none;", pos, anim) inner_html=xw></div> }
            }).collect_view()}

            // Laser bolts
            {vec![
                ("top:22%; height:2px; width:20px; background:linear-gradient(90deg, transparent, #ff3333, transparent);", "laserFlyLeft 4s linear 3s infinite"),
                ("bottom:32%; height:2px; width:18px; background:linear-gradient(90deg, transparent, #44ff44, transparent);", "laserFlyRight 3.5s linear 7s infinite"),
            ].into_iter().map(|(style, anim)| {
                view! { <div style=format!("position:absolute; {} border-radius:1px; animation:{}; pointer-events:none;", style, anim)></div> }
            }).collect_view()}

            <div style="position: relative; z-index: 10; width: 100%; max-width: 480px; margin: 20px;">
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
                        <div style="width:70px; height:80px; margin:0 auto 12px;" inner_html=vagabond_svg></div>
                        <h1 style="font-size: 24px; font-weight: 700; color: white; margin: 0 0 5px 0;">"Регистрация"</h1>
                        <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0;">
                            {move || match step.get() {
                                1 => "Стъпка 1: Лични данни",
                                2 => "Стъпка 2: Данни на фирмата",
                                _ => ""
                            }}
                        </p>
                    </div>

                    // Progress indicator
                    {move || {
                        let current_step = step.get();
                        if !success.get() {
                            view! {
                                <div style="display: flex; justify-content: center; gap: 8px; margin-bottom: 20px;">
                                    {(1..=2).map(|i| {
                                        let is_active = i <= current_step;
                                        view! {
                                            <div style=format!(
                                                "width: 40px; height: 4px; border-radius: 2px; transition: all 0.3s; {}",
                                                if is_active { "background: #d97706;" } else { "background: rgba(255,255,255,0.1);" }
                                            )></div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }
                    }}

                    // Success message
                    {move || {
                        if success.get() {
                            view! {
                                <div style="text-align: center; padding: 20px 0;">
                                    <div style="
                                        width: 60px; height: 60px; margin: 0 auto 20px;
                                        background: rgba(59, 130, 246, 0.15);
                                        border-radius: 50%; display: flex; align-items: center; justify-content: center;
                                        font-size: 30px;
                                    ">"📧"</div>
                                    <h3 style="color: #60a5fa; margin: 0 0 10px 0;">"Проверете вашия email!"</h3>
                                    <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0 0 20px 0;">
                                        "Изпратихме ви email за потвърждение. Моля, кликнете на линка в него, за да активирате акаунта си."
                                    </p>
                                    <div style="
                                        background: rgba(59, 130, 246, 0.08);
                                        border: 1px solid rgba(59, 130, 246, 0.2);
                                        border-radius: 10px; padding: 15px; margin-bottom: 20px;
                                        font-size: 13px; color: rgba(255, 255, 255, 0.6);
                                    ">
                                        "💡 Не получихте email? Проверете папка \"Spam\" или "
                                        <a href="/login" style="color: #60a5fa; text-decoration: underline;">"поискайте нов"</a>
                                    </div>
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

                    // Error message
                    {move || {
                        let err = error.get();
                        if !err.is_empty() && !success.get() {
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

                    // Step 1: User details
                    {move || {
                        if success.get() || step.get() != 1 {
                            return view! { <span></span> }.into_view();
                        }

                        view! {
                            <div>
                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 15px;">
                                    <div>
                                        <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Име"</label>
                                        <input
                                            type="text"
                                            placeholder="Иван"
                                            class="cosmic-input"
                                            style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                            prop:value=move || first_name.get()
                                            on:input=move |e| first_name.set(event_target_value(&e))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Фамилия"</label>
                                        <input
                                            type="text"
                                            placeholder="Петров"
                                            class="cosmic-input"
                                            style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                            prop:value=move || last_name.get()
                                            on:input=move |e| last_name.set(event_target_value(&e))
                                        />
                                    </div>
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Email"</label>
                                    <input
                                        type="email"
                                        placeholder="you@example.com"
                                        class="cosmic-input"
                                        style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                        prop:value=move || email.get()
                                        on:input=move |e| email.set(event_target_value(&e))
                                    />
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Парола"</label>
                                    <input
                                        type="password"
                                        placeholder="Минимум 8 символа"
                                        class="cosmic-input"
                                        style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
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
                                        style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                        prop:value=move || confirm_password.get()
                                        on:input=move |e| confirm_password.set(event_target_value(&e))
                                    />
                                </div>

                                <button
                                    style="
                                        width: 100%; padding: 14px;
                                        background: linear-gradient(135deg, #b45309 0%, #d97706 50%, #f59e0b 100%);
                                        border: none; border-radius: 12px; color: white; font-size: 15px;
                                        font-weight: 600; cursor: pointer; text-transform: uppercase; letter-spacing: 1px;
                                        box-shadow: 0 4px 15px rgba(217, 119, 6, 0.35);
                                    "
                                    on:click=validate_user
                                    disabled=move || loading.get()
                                >
                                    "Напред →"
                                </button>

                                <div style="text-align: center; margin-top: 25px;">
                                    <p style="color: rgba(255, 255, 255, 0.5); font-size: 14px; margin: 0;">
                                        "Вече имате акаунт? "
                                        <a href="/login" style="color: #fbbf24; text-decoration: none; font-weight: 600;">"Вход"</a>
                                    </p>
                                </div>
                            </div>
                        }.into_view()
                    }}

                    // Step 2: Company details
                    {move || {
                        if success.get() || step.get() != 2 {
                            return view! { <span></span> }.into_view();
                        }

                        view! {
                            <div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Име на фирмата *"</label>
                                    <input
                                        type="text"
                                        placeholder="ООД Пример"
                                        class="cosmic-input"
                                        style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                        prop:value=move || company_name.get()
                                        on:input=move |e| company_name.set(event_target_value(&e))
                                    />
                                </div>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 15px;">
                                    <div>
                                        <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"ЕИК"</label>
                                        <input
                                            type="text"
                                            placeholder="123456789"
                                            class="cosmic-input"
                                            style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                            prop:value=move || company_eik.get()
                                            on:input=move |e| company_eik.set(event_target_value(&e))
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"ДДС номер"</label>
                                        <input
                                            type="text"
                                            placeholder="BG000000000"
                                            class="cosmic-input"
                                            style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                            prop:value=move || company_vat.get()
                                            on:input=move |e| company_vat.set(event_target_value(&e))
                                        />
                                    </div>
                                </div>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Адрес"</label>
                                    <input
                                        type="text"
                                        placeholder="ул. Примерна 1"
                                        class="cosmic-input"
                                        style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                        prop:value=move || company_address.get()
                                        on:input=move |e| company_address.set(event_target_value(&e))
                                    />
                                </div>

                                <div style="margin-bottom: 25px;">
                                    <label style="display: block; color: rgba(255,255,255,0.6); font-size: 12px; font-weight: 500; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">"Град"</label>
                                    <input
                                        type="text"
                                        placeholder="София"
                                        class="cosmic-input"
                                        style="width:100%; padding:12px; background:rgba(255,255,255,0.05); border:1px solid rgba(255,255,255,0.1); border-radius:10px; color:white; font-size:14px; box-sizing:border-box; outline:none;"
                                        prop:value=move || company_city.get()
                                        on:input=move |e| company_city.set(event_target_value(&e))
                                    />
                                </div>

                                <div style="
                                    background: rgba(217, 119, 6, 0.08);
                                    border: 1px solid rgba(217, 119, 6, 0.2);
                                    border-radius: 10px; padding: 12px; margin-bottom: 20px;
                                    font-size: 13px; color: rgba(255, 255, 255, 0.6);
                                ">
                                    "ℹ️ Ще станете администратор на фирмата. След това ще можете да добавяте други потребители."
                                </div>

                                <div style="display: flex; gap: 10px;">
                                    <button
                                        style="
                                            padding: 14px 20px;
                                            background: rgba(255, 255, 255, 0.06);
                                            border: 1px solid rgba(255, 255, 255, 0.12);
                                            border-radius: 12px; color: white; font-size: 14px;
                                            cursor: pointer;
                                        "
                                        on:click=go_back
                                    >
                                        "← Назад"
                                    </button>
                                    <button
                                        style="
                                            flex: 1; padding: 14px;
                                            background: linear-gradient(135deg, #10b981 0%, #059669 100%);
                                            border: none; border-radius: 12px; color: white; font-size: 15px;
                                            font-weight: 600; cursor: pointer; text-transform: uppercase; letter-spacing: 1px;
                                            box-shadow: 0 4px 15px rgba(16, 185, 129, 0.3);
                                        "
                                        on:click=submit_with_company
                                        disabled=move || loading.get()
                                    >
                                        {move || if loading.get() { "Създаване..." } else { "Създай фирма" }}
                                    </button>
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

fn register_with_company(
    email: String, password: String, first_name: String, last_name: String,
    company_name: String, company_eik: String, company_vat: String,
    company_address: String, company_city: String,
    loading: RwSignal<bool>, error: RwSignal<String>, success: RwSignal<bool>,
) {
    loading.set(true);
    error.set(String::new());

    wasm_bindgen_futures::spawn_local(async move {
        let body = serde_json::json!({
            "user:email": email,
            "user:password": password,
            "user:password_confirmation": password,
            "user:first_name": first_name,
            "user:last_name": last_name,
            "company:name": company_name,
            "company:eik": company_eik,
            "company:vat_number": company_vat,
            "company:address": company_address,
            "company:city": company_city,
        });

        match api_post_auth("/api/auth/register_with_company", &body).await {
            Ok(data) => {
                if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if let Some(token) = data.get("token").and_then(|v| v.as_str()) {
                        set_token(token);
                    }
                    if let Some(company) = data.get("company") {
                        if let Some(company_id) = company.get("id").and_then(|v| v.as_i64()) {
                            set_selected_company_id(company_id);
                        }
                    }
                    success.set(true);
                } else {
                    let msg = data.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Грешка при регистрация");
                    error.set(msg.to_string());
                }
            }
            Err(e) => {
                error.set(format!("Грешка: {}", e));
            }
        }
        loading.set(false);
    });
}
