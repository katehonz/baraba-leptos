use leptos::*;
use leptos_router::use_navigate;
use crate::api::{get_token, clear_token};
use crate::context::{use_company_context, use_user_context};

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let is_logged_in = get_token().is_some();

    // Redirect to login if not authenticated
    if !is_logged_in {
        let navigate = use_navigate();
        navigate("/login", Default::default());
    }

    let handle_logout = move |_| {
        clear_token();
        let navigate = use_navigate();
        navigate("/login", Default::default());
    };

    let ctx = use_company_context();
    let user_ctx = use_user_context();

    // Load companies and user info once when Layout mounts (user is authenticated)
    // Use create_effect with previous value to ensure it only runs once
    create_effect(move |prev_ran: Option<bool>| {
        if prev_ran.is_none() {
            // First run - load companies and user
            ctx.load_companies();
            user_ctx.load_user();
        }
        true // Return value becomes prev_ran for next run
    });

    view! {
        <div style="display: flex; min-height: 100vh;">
            // Sidebar
            <nav style="width: 220px; background: #2c3e50; color: white; padding: 20px; overflow-y: auto;">
                <h2 style="margin-bottom: 15px; font-size: 1.5em;">"Baraba"</h2>

                // Company selector - uses global context
                <CompanySelector />

                <ul style="list-style: none; padding: 0; margin: 0;">
                    <li style="margin: 10px 0;">
                        <a href="/dashboard" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📊 Табло"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/journal-entries" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📝 Счетоводни записи"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/invoices" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📄 Фактури"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/accounts" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📒 Сметкоплан"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/counterparts" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"👥 Контрагенти"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/products" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📦 Номенклатура"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/vat-returns" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📊 ДДС Дневници"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/dividends" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"💰 Дивиденти"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/reports" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📈 Отчети"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/scanned-invoices" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"🤖 AI Сканиране"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/bank-transactions" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"🏦 Банки"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/saft-export" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📋 SAF-T Експорт"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/saft-movement-mappings" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📦 SAF-T Движения"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/opening-balances" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📊 Начални салда"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/accounting-periods" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"🔒 Периоди"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/fixed-assets" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"🏭 ДМА"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/currencies" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"💱 Валути"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/controlisy-import" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📥 Импорт Controlisy"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/settings" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"🏢 Настройки фирма"</a>
                    </li>
                    <li style="margin: 10px 0;">
                        <a href="/profile" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"👤 Моят профил"</a>
                    </li>
                </ul>

                // Administration section - only visible for super_admin
                {move || {
                    if user_ctx.is_super_admin() {
                        view! {
                            <div style="margin-top: 20px; padding-top: 15px; border-top: 1px solid #34495e;">
                                <div style="font-size: 0.75em; color: #7f8c8d; margin-bottom: 10px; padding: 0 8px;">"АДМИНИСТРАЦИЯ"</div>
                                <ul style="list-style: none; padding: 0; margin: 0;">
                                    <li style="margin: 8px 0;">
                                        <a href="/admin" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📊 Табло"</a>
                                    </li>
                                    <li style="margin: 8px 0;">
                                        <a href="/users" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"👤 Потребители"</a>
                                    </li>
                                    <li style="margin: 8px 0;">
                                        <a href="/companies" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"🏢 Фирми"</a>
                                    </li>
                                    <li style="margin: 8px 0;">
                                        <a href="/roles" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"🔐 Роли"</a>
                                    </li>
                                    <li style="margin: 8px 0;">
                                        <a href="/system-settings" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"⚙️ Система"</a>
                                    </li>
                                    <li style="margin: 8px 0;">
                                        <a href="/audit-logs" style="color: #ecf0f1; text-decoration: none; display: block; padding: 8px; border-radius: 4px;">"📋 Одит лог"</a>
                                    </li>
                                </ul>
                            </div>
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    }
                }}
            </nav>

            // Main content area with header
            <div style="flex: 1; display: flex; flex-direction: column;">
                // Top header bar
                <header style="height: 50px; background: #34495e; display: flex; align-items: center; justify-content: space-between; padding: 0 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                    // Company name
                    <div style="display: flex; align-items: center; color: white;">
                        {move || {
                            let companies = ctx.companies.get();
                            let selected_id = ctx.selected_company_id.get();
                            if let Some(company) = companies.iter().find(|c| Some(c.id) == selected_id) {
                                view! {
                                    <span style="font-weight: 500; font-size: 15px;">
                                        {company.name.clone()}
                                        {company.eik.as_ref().map(|eik| format!(" ({})", eik)).unwrap_or_default()}
                                    </span>
                                }.into_view()
                            } else {
                                view! {
                                    <span style="color: #bdc3c7;">"Няма избрана фирма"</span>
                                }.into_view()
                            }
                        }}
                    </div>

                    // Logout button
                    <button
                        on:click=handle_logout
                        style="background: #e74c3c; color: white; border: none; padding: 8px 16px; cursor: pointer; border-radius: 4px; font-size: 14px;"
                    >
                        "Изход"
                    </button>
                </header>

                // Main content
                <main style="flex: 1; padding: 20px; background: #ecf0f1; overflow-y: auto;">
                    {children()}
                </main>
            </div>
        </div>
    }
}

#[component]
fn CompanySelector() -> impl IntoView {
    let ctx = use_company_context();

    view! {
        <div style="margin-bottom: 20px; padding: 10px; background: #34495e; border-radius: 6px;">
            <div style="font-size: 0.8em; color: #bdc3c7; margin-bottom: 5px;">"Активна фирма:"</div>
            {move || {
                if ctx.loading.get() {
                    view! {
                        <div style="color: #bdc3c7; padding: 8px;">"Зареждане..."</div>
                    }.into_view()
                } else {
                    let companies = ctx.companies.get();
                    let selected_id = ctx.selected_company_id.get();

                    if companies.is_empty() {
                        view! {
                            <div style="color: #e74c3c; padding: 8px;">"Няма фирми"</div>
                        }.into_view()
                    } else {
                        view! {
                            <select
                                style="width: 100%; padding: 8px; border: none; border-radius: 4px; background: #2c3e50; color: white; cursor: pointer;"
                                on:change=move |e| {
                                    if let Ok(id) = event_target_value(&e).parse::<i64>() {
                                        ctx.select_company(id);
                                    }
                                }
                            >
                                {companies.iter().map(|c| {
                                    let is_selected = Some(c.id) == selected_id;
                                    let display = if let Some(eik) = &c.eik {
                                        format!("{} ({})", c.name, eik)
                                    } else {
                                        c.name.clone()
                                    };
                                    view! {
                                        <option value={c.id.to_string()} selected=is_selected>
                                            {display}
                                        </option>
                                    }
                                }).collect_view()}
                            </select>
                        }.into_view()
                    }
                }
            }}
        </div>
    }
}
