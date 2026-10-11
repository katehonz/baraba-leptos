use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::api::{get_token, clear_token};
use crate::context::{use_company_context, use_user_context};

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let is_logged_in = get_token().is_some();

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

    Effect::new(move |prev_ran: Option<bool>| {
        if prev_ran.is_none() {
            ctx.load_companies();
            user_ctx.load_user();
        }
        true
    });

    view! {
        <div class="flex h-screen bg-gray-50 overflow-hidden">
            <aside class="w-64 bg-sidebar-bg flex flex-col h-full shadow-xl">
                <div class="flex items-center gap-3 px-6 py-5 border-b border-gray-700">
                    <div class="w-10 h-10 bg-primary-600 rounded-xl flex items-center justify-center shadow-lg">
                        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>
                        </svg>
                    </div>
                    <div>
                        <h1 class="text-xl font-bold text-white tracking-tight">"Baraba"</h1>
                        <p class="text-xs text-sidebar-muted">"Счетоводство"</p>
                    </div>
                </div>

                <div class="px-4 py-4 border-b border-gray-700">
                    <CompanySelector />
                </div>

                <nav class="flex-1 overflow-y-auto py-4 px-3 space-y-1">
                    <NavLink href="/dashboard" icon="chart">"Табло"</NavLink>
                    <NavLink href="/journal-entries" icon="book">"Счетоводни записи"</NavLink>
                    <NavLink href="/invoices" icon="document">"Фактури"</NavLink>
                    <NavLink href="/warehouse" icon="cube">"Склад"</NavLink>
                    <NavLink href="/warehouse/scan" icon="scan">"AI склад"</NavLink>
                    <NavLink href="/accounts" icon="collection">"Сметкоплан"</NavLink>
                    <NavLink href="/counterparts" icon="users">"Контрагенти"</NavLink>
                    <NavLink href="/products" icon="cube">"Номенклатура"</NavLink>
                    <NavLink href="/vat-returns" icon="calculator">"ДДС Дневници"</NavLink>
                    <NavLink href="/dividends" icon="currency">"Дивиденти"</NavLink>
                    <NavLink href="/reports" icon="chart-bar">"Отчети"</NavLink>
                    <NavLink href="/scanned-invoices" icon="scan">"AI Сканиране"</NavLink>
                    <NavLink href="/bank-transactions" icon="bank">"Банки"</NavLink>
                    <NavLink href="/bank-matching-rules" icon="map">"Банкови правила"</NavLink>
                    
                    <div class="pt-4 pb-2">
                        <p class="px-3 text-xs font-semibold text-sidebar-muted uppercase tracking-wider">"Експорт"</p>
                    </div>
                    <NavLink href="/saft-export" icon="export">"SAF-T Експорт"</NavLink>
                    <NavLink href="/saft-movement-mappings" icon="map">"SAF-T Движения"</NavLink>
                    <NavLink href="/controlisy-import" icon="import">"Импорт Controlisy"</NavLink>
                    
                    <div class="pt-4 pb-2">
                        <p class="px-3 text-xs font-semibold text-sidebar-muted uppercase tracking-wider">"Настройки"</p>
                    </div>
                    <NavLink href="/opening-balances" icon="scale">"Начални салда"</NavLink>
                    <NavLink href="/accounting-periods" icon="lock">"Периоди"</NavLink>
                    <NavLink href="/fixed-assets" icon="office">"ДМА"</NavLink>
                    <NavLink href="/currencies" icon="cash">"Валути"</NavLink>
                    <NavLink href="/settings" icon="cog">"Настройки фирма"</NavLink>
                    <NavLink href="/companies" icon="building">"Фирми"</NavLink>
                    <NavLink href="/profile" icon="user">"Моят профил"</NavLink>

                    {move || {
                        if user_ctx.is_super_admin() {
                            view! {
                                <div class="pt-4 pb-2">
                                    <p class="px-3 text-xs font-semibold text-amber-400 uppercase tracking-wider">"Администрация"</p>
                                </div>
                                <NavLink href="/admin" icon="shield">"Админ Табло"</NavLink>
                                <NavLink href="/users" icon="user-group">"Потребители"</NavLink>
                                <NavLink href="/roles" icon="key">"Роли"</NavLink>
                                <NavLink href="/system-settings" icon="adjustments">"Система"</NavLink>
                                <NavLink href="/audit-logs" icon="clipboard">"Одит лог"</NavLink>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}
                </nav>

                <div class="border-t border-gray-700 p-4">
                    {move || {
                        let user = user_ctx.user.get();
                        view! {
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-full bg-gradient-to-br from-primary-500 to-purple-600 flex items-center justify-center text-white font-semibold">
                                    {user.as_ref().map(|u| u.email.chars().next().unwrap_or('U').to_uppercase().to_string()).unwrap_or_else(|| "U".to_string())}
                                </div>
                                <div class="flex-1 min-w-0">
                                    <p class="text-sm font-medium text-white truncate">
                                        {user.as_ref().map(|u| u.email.clone()).unwrap_or_else(|| "Потребител".to_string())}
                                    </p>
                                    <p class="text-xs text-sidebar-muted truncate">
                                        {if user_ctx.is_super_admin() { "Администратор" } else { "Потребител" }}
                                    </p>
                                </div>
                            </div>
                        }
                    }}
                </div>
            </aside>

            <div class="flex-1 flex flex-col h-full overflow-hidden">
                <header class="min-h-16 bg-white border-b border-gray-200 flex items-center justify-between px-8 py-2 shadow-sm">
                    <div class="flex items-center gap-4 flex-wrap">
                        {move || {
                            let companies = ctx.companies.get();
                            let selected_id = ctx.selected_company_id.get();
                            if let Some(company) = companies.iter().find(|c| Some(c.id) == selected_id) {
                                view! {
                                    <div class="flex items-center gap-3">
                                        <div class="w-8 h-8 rounded-lg bg-primary-100 flex items-center justify-center">
                                            <svg class="w-5 h-5 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>
                                            </svg>
                                        </div>
                                        <div>
                                            <h2 class="text-lg font-semibold text-gray-900">{company.name.clone()}</h2>
                                            {company.eik.as_ref().map(|eik| view! {
                                                <p class="text-sm text-gray-500">{format!("ЕИК: {}", eik)}</p>
                                            })}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <span class="text-gray-500">"Няма избрана фирма"</span>
                                }.into_any()
                            }
                        }}
                        <a
                            href="/companies"
                            class="inline-flex items-center gap-2 px-4 py-2 bg-primary-50 text-primary-700 rounded-lg hover:bg-primary-100 transition-colors font-medium text-sm border border-primary-200"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
                            </svg>
                            "Смени фирма"
                        </a>
                    </div>

                    <div class="flex items-center gap-4">
                        <button
                            on:click=handle_logout
                            class="inline-flex items-center gap-2 px-4 py-2 bg-red-50 text-red-700 rounded-lg hover:bg-red-100 transition-colors font-medium text-sm"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/>
                            </svg>
                            "Изход"
                        </button>
                    </div>
                </header>

                <main class="flex-1 overflow-y-auto p-8 bg-gray-50">
                    {children()}
                </main>
            </div>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, icon: &'static str, children: Children) -> impl IntoView {
    use leptos_router::hooks::use_location;
    
    let location = use_location();
    let is_active = move || location.pathname.get() == href;
    
    let icon_svg = match icon {
        "chart" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 3.055A9.001 9.001 0 1020.945 13H11V3.055z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.488 9H15V3.512A9.025 9.025 0 0120.488 9z"/>"#,
        "book" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>"#,
        "document" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>"#,
        "collection" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/>"#,
        "users" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z"/>"#,
        "cube" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>"#,
        "calculator" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>"#,
        "currency" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>"#,
        "chart-bar" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>"#,
        "scan" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z"/>"#,
        "bank" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 14v3m4-3v3m4-3v3M3 21h18M3 10h18M3 7l9-4 9 4M4 10h16v11H4V10z"/>"#,
        "export" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>"#,
        "map" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0121 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"/>"#,
        "import" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/>"#,
        "scale" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6l3 1m0 0l-3 9a5.002 5.002 0 006.001 0M6 7l3 9M6 7l6-2m6 2l3-1m-3 1l-3 9a5.002 5.002 0 006.001 0M18 7l3 9m-3-9l-6-2m0-2v2m0 16V5m0 16H9m3 0h3"/>"#,
        "lock" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>"#,
        "office" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>"#,
        "cash" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>"#,
        "cog" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>"#,
        "user" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>"#,
        "shield" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>"#,
        "user-group" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>"#,
        "building" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>"#,
        "key" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/>"#,
        "adjustments" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"/>"#,
        "clipboard" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>"#,
        _ => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>"#,
    };
    
    view! {
        <a
            href=href
            class=move || {
                if is_active() {
                    "flex items-center gap-3 px-3 py-2.5 text-sm font-medium rounded-lg bg-primary-600 text-white transition-colors duration-150"
                } else {
                    "flex items-center gap-3 px-3 py-2.5 text-sm font-medium rounded-lg text-sidebar-text hover:bg-sidebar-hover transition-colors duration-150"
                }
            }
        >
            <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" inner_html=icon_svg></svg>
            {children()}
        </a>
    }
}

#[component]
fn CompanySelector() -> impl IntoView {
    let ctx = use_company_context();

    view! {
        <div class="space-y-2">
            <label class="block text-xs font-medium text-sidebar-muted uppercase tracking-wider">
                "Активна фирма"
            </label>
            {move || {
                if ctx.loading.get() {
                    view! {
                        <div class="flex items-center gap-2 text-sidebar-muted">
                            <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            <span class="text-sm">"Зареждане..."</span>
                        </div>
                    }.into_any()
                } else {
                    let companies = ctx.companies.get();
                    let selected_id = ctx.selected_company_id.get();
                    let active = companies.iter().find(|c| Some(c.id) == selected_id).cloned();

                    view! {
                        <div class="space-y-2">
                            {match active {
                                Some(company) => view! {
                                    <div class="rounded-lg bg-gray-800 px-3 py-2.5">
                                        <p class="text-sm font-medium text-white truncate" title=company.name.clone()>
                                            {company.name.clone()}
                                        </p>
                                        {company.eik.as_ref().map(|eik| view! {
                                            <p class="text-xs text-sidebar-muted mt-0.5">{format!("ЕИК: {}", eik)}</p>
                                        })}
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="rounded-lg bg-gray-800 px-3 py-2.5 text-sm text-sidebar-muted">
                                        {if companies.is_empty() { "Няма фирми" } else { "Няма избрана фирма" }}
                                    </div>
                                }.into_any(),
                            }}
                            <a
                                href="/companies"
                                class="flex items-center justify-center gap-2 w-full rounded-lg bg-primary-600 hover:bg-primary-500 text-white py-2 px-3 text-sm font-medium transition-colors"
                            >
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
                                </svg>
                                "Смени фирма"
                            </a>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
