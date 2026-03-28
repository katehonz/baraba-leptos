use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use crate::api::{api_get, api_post};
use crate::components::Layout;
use crate::context::use_company_context;
use js_sys::Date;

fn get_current_year() -> i32 {
    Date::new_0().get_full_year() as i32
}

fn generate_year_range() -> Vec<i32> {
    let current = get_current_year();
    // 5 years back and 5 years forward, sorted descending (most recent first)
    let mut years: Vec<i32> = ((current - 5)..=(current + 5)).collect();
    years.reverse();
    years
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct MonthData {
    year: i32,
    month: i32,
    month_name: String,
    status: String,
    is_closed: bool,
    closed_at: Option<String>,
    closed_by: Option<String>,
    notes: Option<String>,
    has_record: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CalendarResponse {
    success: bool,
    year: i32,
    months: Vec<MonthData>,
    closed_count: i32,
    open_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YearsResponse {
    success: bool,
    data: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloseResponse {
    success: bool,
    message: String,
}

fn generate_default_months(year: i32) -> Vec<MonthData> {
    let month_names = [
        "Януари", "Февруари", "Март", "Април", "Май", "Юни",
        "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември"
    ];
    (1..=12).map(|m| MonthData {
        year,
        month: m,
        month_name: month_names[(m - 1) as usize].to_string(),
        status: "open".to_string(),
        is_closed: false,
        closed_at: None,
        closed_by: None,
        notes: None,
        has_record: false,
    }).collect()
}

#[component]
pub fn AccountingPeriods() -> impl IntoView {
    let ctx = use_company_context();
    let current_year = get_current_year();
    let selected_year = create_rw_signal(current_year);
    let available_years = create_rw_signal(generate_year_range());
    let months_data = create_rw_signal(generate_default_months(current_year));
    let loading = create_rw_signal(false);
    let error = create_rw_signal(Option::<String>::None);
    let success_msg = create_rw_signal(Option::<String>::None);
    let closed_count = create_rw_signal(0i32);
    let open_count = create_rw_signal(12i32);

    // Modal state
    let show_close_modal = create_rw_signal(false);
    let closing_month = create_rw_signal(Option::<MonthData>::None);
    let close_notes = create_rw_signal(String::new());

    // Load available years - merges API data with the default range
    let load_years = move || {
        let cid = ctx.selected_company_id.get().unwrap_or(1);
        spawn_local(async move {
            match api_get(&format!("/api/companies/{}/accounting_periods/years", cid)).await {
                Ok(resp) => {
                    if let Ok(years_resp) = serde_json::from_value::<YearsResponse>(resp) {
                        if years_resp.success && !years_resp.data.is_empty() {
                            // Merge API years with default range
                            let mut all_years: Vec<i32> = generate_year_range();
                            for year in years_resp.data {
                                if !all_years.contains(&year) {
                                    all_years.push(year);
                                }
                            }
                            all_years.sort();
                            all_years.reverse(); // Most recent first
                            available_years.set(all_years);
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("Error loading years: {}", e).into());
                    // Keep default range on error
                }
            }
        });
    };

    // Load calendar data
    let load_calendar = move || {
        let cid = ctx.selected_company_id.get().unwrap_or(1);
        let year = selected_year.get();
        loading.set(true);
        error.set(None);

        spawn_local(async move {
            match api_get(&format!("/api/companies/{}/accounting_periods/calendar/{}", cid, year)).await {
                Ok(resp) => {
                    if let Ok(cal_resp) = serde_json::from_value::<CalendarResponse>(resp) {
                        if cal_resp.success {
                            months_data.set(cal_resp.months);
                            closed_count.set(cal_resp.closed_count);
                            open_count.set(cal_resp.open_count);
                        }
                    }
                    loading.set(false);
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("Error loading calendar: {}", e).into());
                    months_data.set(generate_default_months(year));
                    closed_count.set(0);
                    open_count.set(12);
                    loading.set(false);
                }
            }
        });
    };

    // Close period
    let close_period = move |month_data: MonthData| {
        closing_month.set(Some(month_data));
        close_notes.set(String::new());
        show_close_modal.set(true);
    };

    let confirm_close = move |_| {
        if let Some(month) = closing_month.get() {
            let cid = ctx.selected_company_id.get().unwrap_or(1);
            let year = month.year;
            let m = month.month;
            let month_name = month.month_name.clone();
            let notes = close_notes.get();
            loading.set(true);
            show_close_modal.set(false);

            spawn_local(async move {
                let body = serde_json::json!({ "notes": notes });
                match api_post(&format!("/api/companies/{}/accounting_periods/{}/{}/close", cid, year, m), &body).await {
                    Ok(resp) => {
                        if let Ok(close_resp) = serde_json::from_value::<CloseResponse>(resp) {
                            if close_resp.success {
                                success_msg.set(Some(format!("{} {} е заключен", month_name, year)));
                                load_calendar();
                            }
                        }
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Грешка при затваряне: {}", e)));
                        loading.set(false);
                    }
                }
            });
        }
    };

    // Reopen period
    let reopen_period = move |month_data: MonthData| {
        let cid = ctx.selected_company_id.get().unwrap_or(1);
        let year = month_data.year;
        let m = month_data.month;
        let month_name = month_data.month_name.clone();
        loading.set(true);

        spawn_local(async move {
            let body = serde_json::json!({});
            match api_post(&format!("/api/companies/{}/accounting_periods/{}/{}/reopen", cid, year, m), &body).await {
                Ok(resp) => {
                    if let Ok(close_resp) = serde_json::from_value::<CloseResponse>(resp) {
                        if close_resp.success {
                            success_msg.set(Some(format!("{} {} е отключен", month_name, year)));
                            load_calendar();
                        }
                    }
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Грешка при отваряне: {}", e)));
                    loading.set(false);
                }
            }
        });
    };

    // Load data on mount
    create_effect(move |_| {
        load_years();
        load_calendar();
    });

    // Reload when year changes
    create_effect(move |prev| {
        let year = selected_year.get();
        if prev.is_some() {
            load_calendar();
        }
        year
    });

    // Auto-hide success message
    create_effect(move |_| {
        if success_msg.get().is_some() {
            set_timeout(move || success_msg.set(None), std::time::Duration::from_secs(3));
        }
    });

    view! {
        <Layout>
            <div style="max-width: 1200px; margin: 0 auto;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                    <h1 style="margin: 0;">"Счетоводни периоди"</h1>

                    <div style="display: flex; align-items: center; gap: 15px;">
                        <label style="font-weight: bold;">"Година:"</label>
                        <select
                            style="padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; font-size: 16px; min-width: 100px;"
                            on:change=move |e| {
                                if let Ok(year) = event_target_value(&e).parse::<i32>() {
                                    selected_year.set(year);
                                }
                            }
                        >
                            <For
                                each=move || available_years.get()
                                key=|year| *year
                                children=move |year| {
                                    view! {
                                        <option value={year.to_string()} selected=move || selected_year.get() == year>
                                            {year}
                                        </option>
                                    }
                                }
                            />
                        </select>
                    </div>
                </div>

                // Summary cards
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 30px;">
                    <div style="background: white; padding: 20px; border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.08);">
                        <div style="font-size: 14px; color: #7f8c8d; margin-bottom: 5px;">"Общо периоди"</div>
                        <div style="font-size: 32px; font-weight: bold; color: #2c3e50;">"12"</div>
                    </div>
                    <div style="background: white; padding: 20px; border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.08);">
                        <div style="font-size: 14px; color: #7f8c8d; margin-bottom: 5px;">"Заключени"</div>
                        <div style="font-size: 32px; font-weight: bold; color: #e67e22;">
                            {move || closed_count.get()}
                        </div>
                    </div>
                    <div style="background: white; padding: 20px; border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.08);">
                        <div style="font-size: 14px; color: #7f8c8d; margin-bottom: 5px;">"Отворени"</div>
                        <div style="font-size: 32px; font-weight: bold; color: #27ae60;">
                            {move || open_count.get()}
                        </div>
                    </div>
                </div>

                // Success message
                {move || success_msg.get().map(|msg| view! {
                    <div style="background: #d4edda; color: #155724; padding: 15px 20px; border-radius: 8px; margin-bottom: 20px; display: flex; align-items: center; gap: 10px;">
                        <span style="font-size: 18px;">"✓"</span>
                        {msg}
                    </div>
                })}

                // Error message
                {move || error.get().map(|e| view! {
                    <div style="background: #f8d7da; color: #721c24; padding: 15px 20px; border-radius: 8px; margin-bottom: 20px;">
                        {e}
                    </div>
                })}

                // Loading indicator
                {move || loading.get().then(|| view! {
                    <div style="text-align: center; padding: 20px; color: #7f8c8d;">
                        "Зареждане..."
                    </div>
                })}

                // Calendar grid
                <div style="background: white; border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.08); overflow: hidden;">
                    <div style="padding: 15px 20px; background: #f8f9fa; border-bottom: 1px solid #eee;">
                        <h3 style="margin: 0; color: #2c3e50;">"Календар " {move || selected_year.get()}</h3>
                    </div>

                    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px; padding: 20px;">
                        {move || months_data.get().into_iter().map(|month| {
                            let month_num = month.month;
                            let month_for_close = month.clone();
                            let month_for_reopen = month.clone();
                            let is_closed = month.is_closed;
                            let month_name = month.month_name.clone();
                            let closed_at = month.closed_at.clone();
                            let closed_by = month.closed_by.clone();
                            let notes = month.notes.clone();

                            view! {
                                <div style=format!(
                                    "border-radius: 10px; padding: 16px; min-height: 160px; display: flex; flex-direction: column; transition: all 0.2s; {}",
                                    if is_closed {
                                        "background: linear-gradient(135deg, #fff5f5 0%, #fed7d7 100%); border: 2px solid #fc8181;"
                                    } else {
                                        "background: linear-gradient(135deg, #f0fff4 0%, #c6f6d5 100%); border: 2px solid #68d391;"
                                    }
                                )>
                                    // Header with month name and status
                                    <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
                                        <div>
                                            <div style="font-weight: 600; font-size: 16px; color: #2d3748; margin-bottom: 4px;">
                                                {format!("{:02} - {}", month_num, month_name)}
                                            </div>
                                            <div style=format!(
                                                "display: inline-flex; align-items: center; gap: 5px; padding: 3px 10px; border-radius: 20px; font-size: 12px; font-weight: 500; {}",
                                                if is_closed {
                                                    "background: #e53e3e; color: white;"
                                                } else {
                                                    "background: #38a169; color: white;"
                                                }
                                            )>
                                                {if is_closed { "🔒 Заключен" } else { "🔓 Отворен" }}
                                            </div>
                                        </div>
                                    </div>

                                    // Closed info
                                    {if is_closed {
                                        let at = closed_at.clone().unwrap_or_default();
                                        let by = closed_by.clone().unwrap_or_else(|| "—".to_string());
                                        view! {
                                            <div style="flex: 1; font-size: 11px; color: #718096; line-height: 1.5;">
                                                <div>"Заключен: " <strong>{at}</strong></div>
                                                <div>"От: " <strong>{by}</strong></div>
                                                {notes.clone().map(|n| view! {
                                                    <div style="margin-top: 6px; padding: 6px 8px; background: rgba(0,0,0,0.05); border-radius: 6px; font-style: italic; color: #4a5568;">
                                                        {n}
                                                    </div>
                                                })}
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <div style="flex: 1;"></div> }.into_view()
                                    }}

                                    // Both action buttons
                                    <div style="margin-top: 10px; display: flex; gap: 8px;">
                                        <button
                                            style=format!(
                                                "flex: 1; padding: 8px; border: none; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 500; transition: all 0.2s; {}",
                                                if is_closed {
                                                    "background: #e2e8f0; color: #a0aec0; cursor: not-allowed;"
                                                } else {
                                                    "background: #e53e3e; color: white;"
                                                }
                                            )
                                            on:click={
                                                let m = month_for_close.clone();
                                                move |_| close_period(m.clone())
                                            }
                                            disabled=is_closed
                                        >
                                            "🔒 Заключи"
                                        </button>
                                        <button
                                            style=format!(
                                                "flex: 1; padding: 8px; border: none; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 500; transition: all 0.2s; {}",
                                                if is_closed {
                                                    "background: #48bb78; color: white;"
                                                } else {
                                                    "background: #e2e8f0; color: #a0aec0; cursor: not-allowed;"
                                                }
                                            )
                                            on:click={
                                                let m = month_for_reopen.clone();
                                                move |_| reopen_period(m.clone())
                                            }
                                            disabled=!is_closed
                                        >
                                            "🔓 Отключи"
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Info box
                <div style="margin-top: 20px; padding: 15px 20px; background: #ebf8ff; border-radius: 8px; color: #2b6cb0; font-size: 14px;">
                    <strong>"💡 Информация: "</strong>
                    "Заключването на счетоводен период предотвратява добавянето или редактирането на записи за този период."
                </div>
            </div>

            // Close modal
            {move || show_close_modal.get().then(|| {
                let month = closing_month.get();
                view! {
                    <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.4); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                        <div style="background: white; padding: 30px; border-radius: 16px; width: 420px; max-width: 90%; box-shadow: 0 20px 40px rgba(0,0,0,0.15);">
                            <h2 style="margin-top: 0; margin-bottom: 20px; color: #2d3748; font-size: 20px;">
                                "🔒 Заключване на период"
                            </h2>

                            {month.map(|m| view! {
                                <p style="color: #718096; margin-bottom: 20px; line-height: 1.6;">
                                    "Сигурни ли сте, че искате да заключите "
                                    <strong style="color: #2d3748;">{format!("{:02} - {} {}", m.month, m.month_name, m.year)}</strong>
                                    "?"
                                </p>
                            })}

                            <div style="margin-bottom: 20px;">
                                <label style="display: block; margin-bottom: 8px; font-weight: 500; color: #4a5568; font-size: 14px;">
                                    "Бележки (незадължително)"
                                </label>
                                <textarea
                                    style="width: 100%; padding: 12px; border: 1px solid #e2e8f0; border-radius: 8px; min-height: 80px; font-family: inherit; box-sizing: border-box; font-size: 14px; resize: vertical;"
                                    placeholder="Добавете бележки..."
                                    on:input=move |e| close_notes.set(event_target_value(&e))
                                    prop:value=move || close_notes.get()
                                />
                            </div>

                            <div style="display: flex; gap: 12px; justify-content: flex-end;">
                                <button
                                    style="padding: 10px 20px; background: #edf2f7; color: #4a5568; border: none; border-radius: 8px; cursor: pointer; font-weight: 500;"
                                    on:click=move |_| show_close_modal.set(false)
                                >
                                    "Отказ"
                                </button>
                                <button
                                    style="padding: 10px 20px; background: #e53e3e; color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 500;"
                                    on:click=confirm_close
                                >
                                    "Заключи"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </Layout>
    }
}
