use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::Layout;
use crate::context::company_context::use_company_context;
use crate::api::api_get;

#[component]
pub fn Dashboard() -> impl IntoView {
    let ctx = use_company_context();

    let journal_entries = RwSignal::new(0i64);
    let counterparts = RwSignal::new(0i64);
    let revenue = RwSignal::new(0.0f64);
    let expenses = RwSignal::new(0.0f64);
    let profit = RwSignal::new(0.0f64);
    let loading = RwSignal::new(true);

    // Load dashboard data
    Effect::new(move |_| {
        let cid = ctx.selected_company_id.get().unwrap_or(1);
        spawn_local(async move {
            let url = format!("/api/companies/{}/dashboard", cid);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(d) = data.get("data") {
                        journal_entries.set(d.get("journal_entries").and_then(|v| v.as_i64()).unwrap_or(0));
                        counterparts.set(d.get("counterparts").and_then(|v| v.as_i64()).unwrap_or(0));
                        revenue.set(d.get("revenue").and_then(|v| v.as_f64()).unwrap_or(0.0));
                        expenses.set(d.get("expenses").and_then(|v| v.as_f64()).unwrap_or(0.0));
                        profit.set(d.get("profit").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    }
                }
                Err(_) => {}
            }
            loading.set(false);
        });
    });

    view! {
        <Layout>
            // Welcome section
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 mb-2">
                    {move || {
                        if let Some(company) = ctx.selected_company() {
                            format!("Добре дошли в {}", company.name)
                        } else {
                            "Табло".to_string()
                        }
                    }}
                </h1>
                <p class="text-gray-600">
                    "Преглед на основните показатели на вашата фирма"
                </p>
            </div>

            // Stats Grid
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                // Journal Entries Card
                <StatCard
                    title="Счетоводни записи"
                    value=move || if loading.get() { "...".to_string() } else { journal_entries.get().to_string() }
                    icon="book"
                    color="blue"
                    trend=None
                />

                // Counterparts Card
                <StatCard
                    title="Контрагенти"
                    value=move || if loading.get() { "...".to_string() } else { counterparts.get().to_string() }
                    icon="users"
                    color="purple"
                    trend=None
                />

                // Revenue Card
                <StatCard
                    title="Приходи"
                    value=move || if loading.get() { "...".to_string() } else { format!("{:.2} €", revenue.get()) }
                    icon="trend-up"
                    color="emerald"
                    trend=Some((15.3, true))
                />

                // Expenses Card
                <StatCard
                    title="Разходи"
                    value=move || if loading.get() { "...".to_string() } else { format!("{:.2} €", expenses.get()) }
                    icon="trend-down"
                    color="rose"
                    trend=Some((8.2, false))
                />
            </div>

            // Profit Section
            <div class="mb-8">
                <ProfitCard profit=profit loading=loading />
            </div>

            // Quick Actions
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Recent Activity placeholder
                <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                    <h3 class="text-lg font-semibold text-gray-900 mb-4">
                        "Бързи действия"
                    </h3>
                    <div class="grid grid-cols-2 gap-4">
                        <QuickActionButton
                            href="/journal-entries"
                            icon="plus"
                            label="Нов запис"
                            description="Добави счетоводен запис"
                            color="blue"
                        />
                        <QuickActionButton
                            href="/invoices"
                            icon="document"
                            label="Нова фактура"
                            description="Създай нова фактура"
                            color="purple"
                        />
                        <QuickActionButton
                            href="/counterparts"
                            icon="user-plus"
                            label="Нов контрагент"
                            description="Добави контрагент"
                            color="emerald"
                        />
                        <QuickActionButton
                            href="/reports"
                            icon="chart"
                            label="Отчети"
                            description="Виж финансови отчети"
                            color="amber"
                        />
                    </div>
                </div>

                // Help & Info
                <div class="bg-gradient-to-br from-primary-600 to-primary-700 rounded-xl shadow-lg p-6 text-white">
                    <h3 class="text-lg font-semibold mb-2">
                        "Нуждаете се от помощ?"
                    </h3>
                    <p class="text-primary-100 mb-4">
                        "Разгледайте нашата документация или се свържете с поддръжката за съдействие."
                    </p>
                    <div class="flex gap-3">
                        <a
                            href="/help"
                            class="inline-flex items-center gap-2 px-4 py-2 bg-white/10 hover:bg-white/20 rounded-lg transition-colors text-sm font-medium"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
                            </svg>
                            "Документация"
                        </a>
                        <a
                            href="/support"
                            class="inline-flex items-center gap-2 px-4 py-2 bg-white/10 hover:bg-white/20 rounded-lg transition-colors text-sm font-medium"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192l-3.536 3.536M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-5 0a4 4 0 11-8 0 4 4 0 018 0z"/>
                            </svg>
                            "Поддръжка"
                        </a>
                    </div>
                </div>
            </div>
        </Layout>
    }
}

#[component]
fn StatCard(
    title: &'static str,
    value: impl Fn() -> String + Send + 'static,
    icon: &'static str,
    color: &'static str,
    trend: Option<(f64, bool)>,
) -> impl IntoView {
    let color_classes = match color {
        "blue" => "bg-blue-50 text-blue-600",
        "purple" => "bg-purple-50 text-purple-600",
        "emerald" => "bg-emerald-50 text-emerald-600",
        "rose" => "bg-rose-50 text-rose-600",
        "amber" => "bg-amber-50 text-amber-600",
        _ => "bg-gray-50 text-gray-600",
    };

    let icon_svg = match icon {
        "book" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>"#,
        "users" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>"#,
        "trend-up" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"/>"#,
        "trend-down" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 17h8m0 0V9m0 8l-8-8-4 4-6-6"/>"#,
        _ => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>"#,
    };

    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow">
            <div class="flex items-center justify-between mb-4">
                <div class={format!("w-12 h-12 rounded-xl flex items-center justify-center {}", color_classes)}>
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" inner_html=icon_svg></svg>
                </div>
                {trend.map(|(value, is_positive)| {
                    let trend_class = if is_positive { "text-emerald-600 bg-emerald-50" } else { "text-rose-600 bg-rose-50" };
                    let trend_icon = if is_positive { "↑" } else { "↓" };
                    view! {
                        <div class={format!("flex items-center gap-1 px-2 py-1 rounded-lg text-xs font-medium {}", trend_class)}>
                            <span>{trend_icon}</span>
                            <span>{format!("{:.1}%", value)}</span>
                        </div>
                    }
                })}
            </div>
            <h3 class="text-sm font-medium text-gray-600 mb-1">{title}</h3>
            <p class="text-2xl font-bold text-gray-900">{value}</p>
        </div>
    }
}

#[component]
fn ProfitCard(profit: RwSignal<f64>, loading: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class={format!("rounded-xl border-2 p-6 {}", if profit.get() >= 0.0 { "bg-emerald-50 border-emerald-200" } else { "bg-rose-50 border-rose-200" })}>
            <div class="flex items-center justify-between">
                <div>
                    <h3 class={format!("text-sm font-medium mb-1 {}", if profit.get() >= 0.0 { "text-emerald-700" } else { "text-rose-700" })}>
                        {move || if profit.get() >= 0.0 { "Печалба" } else { "Загуба" }}
                    </h3>
                    <p class={format!("text-4xl font-bold {}", if profit.get() >= 0.0 { "text-emerald-900" } else { "text-rose-900" })}>
                        {move || if loading.get() { "...".to_string() } else { format!("{:.2} €", profit.get().abs()) }}
                    </p>
                </div>
                <div class={format!("w-16 h-16 rounded-2xl flex items-center justify-center bg-white/80 {}", if profit.get() >= 0.0 { "text-emerald-600" } else { "text-rose-600" })}>
                    {move || {
                        let is_profit = profit.get() >= 0.0;
                        let icon = if is_profit {
                            r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>"#
                        } else {
                            r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>"#
                        };
                        view! {
                            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24" inner_html=icon></svg>
                        }
                    }}
                </div>
            </div>
            <p class={format!("mt-4 text-sm font-medium {}", if profit.get() >= 0.0 { "text-emerald-600" } else { "text-rose-600" })}>
                {move || {
                    if profit.get() >= 0.0 {
                        "✓ Фирмата е на печалба"
                    } else {
                        "⚠ Фирмата е на загуба"
                    }
                }}
            </p>
        </div>
    }
}

#[component]
fn QuickActionButton(
    href: &'static str,
    icon: &'static str,
    label: &'static str,
    description: &'static str,
    color: &'static str,
) -> impl IntoView {
    let color_classes = match color {
        "blue" => "bg-blue-50 text-blue-600 hover:bg-blue-100 border-blue-200",
        "purple" => "bg-purple-50 text-purple-600 hover:bg-purple-100 border-purple-200",
        "emerald" => "bg-emerald-50 text-emerald-600 hover:bg-emerald-100 border-emerald-200",
        "amber" => "bg-amber-50 text-amber-600 hover:bg-amber-100 border-amber-200",
        _ => "bg-gray-50 text-gray-600 hover:bg-gray-100 border-gray-200",
    };

    let icon_svg = match icon {
        "plus" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>"#,
        "document" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>"#,
        "user-plus" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z"/>"#,
        "chart" => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>"#,
        _ => r#"<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>"#,
    };

    view! {
        <a
            href=href
            class={format!("flex items-start gap-3 p-4 rounded-xl border transition-all hover:shadow-md {}", color_classes)}
        >
            <div class="w-10 h-10 rounded-lg bg-white/80 flex items-center justify-center flex-shrink-0">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" inner_html=icon_svg></svg>
            </div>
            <div>
                <h4 class="font-semibold text-sm">{label}</h4>
                <p class="text-xs opacity-75 mt-0.5">{description}</p>
            </div>
        </a>
    }
}
