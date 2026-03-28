use leptos::*;
use crate::components::Layout;
use crate::context::company_context::use_company_context;
use crate::api::api_get;

#[component]
pub fn Dashboard() -> impl IntoView {
    let ctx = use_company_context();

    let journal_entries = create_rw_signal(0i64);
    let counterparts = create_rw_signal(0i64);
    let revenue = create_rw_signal(0.0f64);
    let expenses = create_rw_signal(0.0f64);
    let profit = create_rw_signal(0.0f64);
    let loading = create_rw_signal(true);

    // Load dashboard data
    create_effect(move |_| {
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
            // Company info header
            {move || {
                if let Some(company) = ctx.selected_company() {
                    view! {
                        <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px 25px; border-radius: 12px; margin-bottom: 25px; display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <h1 style="margin: 0 0 5px 0; font-size: 24px;">{company.name}</h1>
                                <div style="display: flex; gap: 20px; font-size: 14px; opacity: 0.9;">
                                    <span style="display: flex; align-items: center; gap: 5px;">
                                        <span style="background: rgba(255,255,255,0.2); padding: 2px 8px; border-radius: 4px; font-family: monospace;">"ID: " {company.id}</span>
                                    </span>
                                    {company.eik.map(|eik| view! {
                                        <span>"EИК: " {eik}</span>
                                    })}
                                </div>
                            </div>
                            <div style="text-align: right;">
                                <div style="font-size: 12px; opacity: 0.8;">"Фирма"</div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <h1>"Табло"</h1>
                    }.into_view()
                }
            }}

            // Metrics cards
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-top: 20px;">
                <div style="background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                    <h3 style="margin: 0 0 10px 0; font-size: 14px; color: #7f8c8d; font-weight: 500;">"Счетоводни записи"</h3>
                    <p style="font-size: 2em; color: #3498db; margin: 0; font-weight: 600;">
                        {move || if loading.get() { "...".to_string() } else { journal_entries.get().to_string() }}
                    </p>
                </div>
                <div style="background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                    <h3 style="margin: 0 0 10px 0; font-size: 14px; color: #7f8c8d; font-weight: 500;">"Контрагенти"</h3>
                    <p style="font-size: 2em; color: #2ecc71; margin: 0; font-weight: 600;">
                        {move || if loading.get() { "...".to_string() } else { counterparts.get().to_string() }}
                    </p>
                </div>
                <div style="background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                    <h3 style="margin: 0 0 10px 0; font-size: 14px; color: #7f8c8d; font-weight: 500;">"Приходи"</h3>
                    <p style="font-size: 2em; color: #27ae60; margin: 0; font-weight: 600;">
                        {move || if loading.get() { "...".to_string() } else { format!("{:.2} EUR", revenue.get()) }}
                    </p>
                </div>
                <div style="background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                    <h3 style="margin: 0 0 10px 0; font-size: 14px; color: #7f8c8d; font-weight: 500;">"Разходи"</h3>
                    <p style="font-size: 2em; color: #e74c3c; margin: 0; font-weight: 600;">
                        {move || if loading.get() { "...".to_string() } else { format!("{:.2} EUR", expenses.get()) }}
                    </p>
                </div>
            </div>

            // Profit card
            <div style="margin-top: 20px;">
                <div style=move || format!("padding: 25px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); text-align: center; {}",
                    if profit.get() >= 0.0 { "background: #e8f5e9;" } else { "background: #ffebee;" }
                )>
                    <h3 style="margin: 0 0 10px 0; font-size: 14px; color: #7f8c8d; font-weight: 500;">
                        {move || if profit.get() >= 0.0 { "Печалба" } else { "Загуба" }}
                    </h3>
                    <p style=move || format!("font-size: 2.5em; margin: 0; font-weight: 700; {}",
                        if profit.get() >= 0.0 { "color: #27ae60;" } else { "color: #e74c3c;" }
                    )>
                        {move || if loading.get() { "...".to_string() } else { format!("{:.2} EUR", profit.get().abs()) }}
                    </p>
                </div>
            </div>
        </Layout>
    }
}
