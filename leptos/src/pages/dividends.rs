use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeneficialOwner {
    pub id: i64,
    pub first_name_bg: String,
    pub last_name_bg: String,
    pub egn: Option<String>,
    pub ownership_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendDistribution {
    pub id: i64,
    pub year: i32,
    pub total_amount: f64,
    pub decision_date: String,
    pub decision_number: Option<String>,
    pub status: String,
    pub status_label: Option<String>,
    pub notes: Option<String>,
    pub dividends_count: Option<i64>,
    pub total_net: Option<f64>,
    pub total_paid: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dividend {
    pub id: i64,
    pub beneficial_owner_id: Option<i64>,
    pub owner_name: Option<String>,
    pub owner_egn: Option<String>,
    pub ownership_percentage: Option<f64>,
    pub gross_amount: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
    pub net_amount: f64,
    pub is_paid: bool,
    pub payment_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendDistributionDetail {
    pub id: i64,
    pub year: i32,
    pub total_amount: f64,
    pub decision_date: String,
    pub decision_number: Option<String>,
    pub status: String,
    pub status_label: Option<String>,
    pub notes: Option<String>,
    pub dividends: Vec<Dividend>,
}

#[component]
pub fn Dividends() -> impl IntoView {
    // State
    let distributions = create_rw_signal(Vec::<DividendDistribution>::new());
    let beneficial_owners = create_rw_signal(Vec::<BeneficialOwner>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // New distribution modal
    let show_new_modal = create_rw_signal(false);
    let new_year = create_rw_signal(get_current_year());
    let new_total_amount = create_rw_signal(0.0f64);
    let new_decision_date = create_rw_signal(get_today_date());
    let new_decision_number = create_rw_signal(String::new());
    let new_notes = create_rw_signal(String::new());
    let creating = create_rw_signal(false);

    // Detail view
    let selected_distribution = create_rw_signal(Option::<DividendDistributionDetail>::None);
    let detail_loading = create_rw_signal(false);

    // Load distributions
    let load_distributions = move || {
        loading.set(true);
        spawn_local(async move {
            match api_get("/api/companies/1/dividend_distributions").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|d| d.as_array()) {
                        let items: Vec<DividendDistribution> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        distributions.set(items);
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            loading.set(false);
        });
    };

    // Load beneficial owners
    let load_owners = move || {
        spawn_local(async move {
            match api_get("/api/companies/1/beneficial_owners").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|d| d.as_array()) {
                        let items: Vec<BeneficialOwner> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        beneficial_owners.set(items);
                    }
                }
                Err(_) => {}
            }
        });
    };

    // Load distribution detail
    let load_detail = move |id: i64| {
        detail_loading.set(true);
        spawn_local(async move {
            let url = format!("/api/companies/1/dividend_distributions/{}", id);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(detail_data) = data.get("data") {
                        if let Ok(detail) = serde_json::from_value::<DividendDistributionDetail>(detail_data.clone()) {
                            selected_distribution.set(Some(detail));
                        }
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            detail_loading.set(false);
        });
    };

    // Create new distribution
    let create_distribution = move |_| {
        creating.set(true);
        error_msg.set(String::new());

        let year = new_year.get();
        let total_amount = new_total_amount.get();
        let decision_date = new_decision_date.get();
        let decision_number = new_decision_number.get();
        let notes = new_notes.get();

        spawn_local(async move {
            let payload = serde_json::json!({
                "year": year,
                "total_amount": total_amount,
                "decision_date": decision_date,
                "decision_number": if decision_number.is_empty() { None } else { Some(decision_number) },
                "notes": if notes.is_empty() { None } else { Some(notes) }
            });

            match api_post("/api/companies/1/dividend_distributions", &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Разпределението е създадено успешно".to_string());
                        show_new_modal.set(false);
                        // Reset form
                        new_total_amount.set(0.0);
                        new_decision_number.set(String::new());
                        new_notes.set(String::new());
                        load_distributions();
                    } else {
                        let err = response.get("errors")
                            .and_then(|e| e.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|e| e.get("messages").and_then(|m| m.as_array())?.first()?.as_str())
                                .collect::<Vec<_>>().join(", "))
                            .unwrap_or("Неизвестна грешка".to_string());
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            creating.set(false);
        });
    };

    // Mark dividend as paid
    let mark_paid = move |dividend_id: i64, is_paid: bool| {
        spawn_local(async move {
            let url = format!("/api/companies/1/dividends/{}", dividend_id);
            let payment_date = if is_paid { Some(get_today_date()) } else { None::<String> };
            let payload = serde_json::json!({
                "is_paid": is_paid,
                "payment_date": payment_date
            });

            match api_put(&url, &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set(if is_paid { "Маркирано като платено" } else { "Маркирано като неплатено" }.to_string());
                        // Reload detail
                        if let Some(dist) = selected_distribution.get() {
                            load_detail(dist.id);
                        }
                        load_distributions();
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Delete distribution
    let delete_distribution = move |id: i64| {
        spawn_local(async move {
            let url = format!("/api/companies/1/dividend_distributions/{}", id);
            match api_delete(&url).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Разпределението е изтрито".to_string());
                        selected_distribution.set(None);
                        load_distributions();
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Approve distribution
    let approve_distribution = move |id: i64| {
        spawn_local(async move {
            let url = format!("/api/companies/1/dividend_distributions/{}", id);
            let payload = serde_json::json!({
                "status": "approved"
            });

            match api_put(&url, &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Разпределението е одобрено".to_string());
                        load_detail(id);
                        load_distributions();
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Load on mount
    create_effect(move |_| {
        load_distributions();
        load_owners();
    });

    view! {
        <Layout>
            // Messages
            {move || if !error_msg.get().is_empty() {
                view! {
                    <div style="background: #f8d7da; color: #721c24; padding: 10px; border-radius: 4px; margin-bottom: 15px;">
                        {error_msg.get()}
                        <button style="float: right; border: none; background: none; cursor: pointer;" on:click=move |_| error_msg.set(String::new())>"×"</button>
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}

            {move || if !success_msg.get().is_empty() {
                view! {
                    <div style="background: #d4edda; color: #155724; padding: 10px; border-radius: 4px; margin-bottom: 15px;">
                        {success_msg.get()}
                        <button style="float: right; border: none; background: none; cursor: pointer;" on:click=move |_| success_msg.set(String::new())>"×"</button>
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}

            // Main content - either list or detail
            {move || {
                if let Some(dist) = selected_distribution.get() {
                    // Detail view
                    let dist_for_approve = dist.clone();
                    let dist_for_delete = dist.clone();
                    let total_net: f64 = dist.dividends.iter().map(|d| d.net_amount).sum();
                    let total_paid: f64 = dist.dividends.iter().filter(|d| d.is_paid).map(|d| d.net_amount).sum();
                    let total_tax: f64 = dist.dividends.iter().map(|d| d.tax_amount).sum();

                    view! {
                        <div>
                            // Header
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <div style="display: flex; align-items: center; gap: 15px;">
                                    <button
                                        style="background: #ecf0f1; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| selected_distribution.set(None)
                                    >
                                        "← Назад"
                                    </button>
                                    <h2 style="margin: 0;">{format!("Дивиденти за {} г.", dist.year)}</h2>
                                    <span style=format!("padding: 4px 12px; border-radius: 4px; font-size: 0.9em; {}",
                                        match dist.status.as_str() {
                                            "paid" => "background: #27ae60; color: white;",
                                            "partially_paid" => "background: #f39c12; color: white;",
                                            "approved" => "background: #3498db; color: white;",
                                            _ => "background: #bdc3c7;"
                                        }
                                    )>{dist.status_label.clone().unwrap_or(dist.status.clone())}</span>
                                </div>
                                <div style="display: flex; gap: 10px;">
                                    {if dist.status == "draft" {
                                        let id = dist_for_approve.id;
                                        view! {
                                            <button
                                                style="background: #27ae60; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                                on:click=move |_| approve_distribution(id)
                                            >
                                                "Одобри"
                                            </button>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    {if dist.status == "draft" {
                                        let id = dist_for_delete.id;
                                        view! {
                                            <button
                                                style="background: #e74c3c; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                                on:click=move |_| {
                                                    if web_sys::window()
                                                        .and_then(|w| w.confirm_with_message("Сигурни ли сте, че искате да изтриете това разпределение?").ok())
                                                        .unwrap_or(false)
                                                    {
                                                        delete_distribution(id);
                                                    }
                                                }
                                            >
                                                "Изтрий"
                                            </button>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                </div>
                            </div>

                            // Summary cards
                            <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; margin-bottom: 20px;">
                                <div style="padding: 20px; background: #e3f2fd; border-radius: 8px; text-align: center;">
                                    <div style="font-size: 0.85em; color: #666;">"Обща сума"</div>
                                    <div style="font-size: 1.5em; font-weight: bold; color: #1976d2;">{format!("{:.2} EUR", dist.total_amount)}</div>
                                </div>
                                <div style="padding: 20px; background: #fff3e0; border-radius: 8px; text-align: center;">
                                    <div style="font-size: 0.85em; color: #666;">"Данък (5%)"</div>
                                    <div style="font-size: 1.5em; font-weight: bold; color: #f57c00;">{format!("{:.2} EUR", total_tax)}</div>
                                </div>
                                <div style="padding: 20px; background: #e8f5e9; border-radius: 8px; text-align: center;">
                                    <div style="font-size: 0.85em; color: #666;">"Нетни дивиденти"</div>
                                    <div style="font-size: 1.5em; font-weight: bold; color: #388e3c;">{format!("{:.2} EUR", total_net)}</div>
                                </div>
                                <div style="padding: 20px; background: #fce4ec; border-radius: 8px; text-align: center;">
                                    <div style="font-size: 0.85em; color: #666;">"Изплатено"</div>
                                    <div style="font-size: 1.5em; font-weight: bold; color: #c2185b;">{format!("{:.2} EUR", total_paid)}</div>
                                </div>
                            </div>

                            // Info box
                            <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px; display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px;">
                                <div>
                                    <div style="color: #666; font-size: 0.85em;">"Дата на решение"</div>
                                    <div style="font-weight: 500;">{dist.decision_date.clone()}</div>
                                </div>
                                <div>
                                    <div style="color: #666; font-size: 0.85em;">"Номер на протокол"</div>
                                    <div style="font-weight: 500;">{dist.decision_number.clone().unwrap_or("-".to_string())}</div>
                                </div>
                                <div>
                                    <div style="color: #666; font-size: 0.85em;">"Бележки"</div>
                                    <div style="font-weight: 500;">{dist.notes.clone().unwrap_or("-".to_string())}</div>
                                </div>
                            </div>

                            // Dividends table
                            <div style="background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: hidden;">
                                <table style="width: 100%; border-collapse: collapse;">
                                    <thead>
                                        <tr style="background: #f8f9fa;">
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"Собственик"</th>
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"ЕГН"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Дял %"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Брутна сума"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Данък 5%"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Нетна сума"</th>
                                            <th style="padding: 15px; text-align: center; border-bottom: 2px solid #dee2e6;">"Статус"</th>
                                            <th style="padding: 15px; text-align: center; border-bottom: 2px solid #dee2e6;">"Действия"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {dist.dividends.iter().map(|d| {
                                            let d_id = d.id;
                                            let d_is_paid = d.is_paid;
                                            let can_mark_paid = dist.status != "draft";
                                            view! {
                                                <tr style="border-bottom: 1px solid #dee2e6;">
                                                    <td style="padding: 15px; font-weight: 500;">{d.owner_name.clone().unwrap_or("-".to_string())}</td>
                                                    <td style="padding: 15px;">{d.owner_egn.clone().unwrap_or("-".to_string())}</td>
                                                    <td style="padding: 15px; text-align: right;">{format!("{:.2}%", d.ownership_percentage.unwrap_or(0.0))}</td>
                                                    <td style="padding: 15px; text-align: right;">{format!("{:.2}", d.gross_amount)}</td>
                                                    <td style="padding: 15px; text-align: right; color: #e67e22;">{format!("{:.2}", d.tax_amount)}</td>
                                                    <td style="padding: 15px; text-align: right; font-weight: bold; color: #27ae60;">{format!("{:.2}", d.net_amount)}</td>
                                                    <td style="padding: 15px; text-align: center;">
                                                        {if d.is_paid {
                                                            view! {
                                                                <span style="background: #27ae60; color: white; padding: 4px 10px; border-radius: 4px; font-size: 0.85em;">
                                                                    "Платено"
                                                                </span>
                                                            }.into_view()
                                                        } else {
                                                            view! {
                                                                <span style="background: #bdc3c7; padding: 4px 10px; border-radius: 4px; font-size: 0.85em;">
                                                                    "Неплатено"
                                                                </span>
                                                            }.into_view()
                                                        }}
                                                    </td>
                                                    <td style="padding: 15px; text-align: center;">
                                                        {if can_mark_paid {
                                                            if d_is_paid {
                                                                view! {
                                                                    <button
                                                                        style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 0.85em;"
                                                                        on:click=move |_| mark_paid(d_id, false)
                                                                    >
                                                                        "Отмени"
                                                                    </button>
                                                                }.into_view()
                                                            } else {
                                                                view! {
                                                                    <button
                                                                        style="background: #27ae60; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 0.85em;"
                                                                        on:click=move |_| mark_paid(d_id, true)
                                                                    >
                                                                        "Плати"
                                                                    </button>
                                                                }.into_view()
                                                            }
                                                        } else {
                                                            view! {
                                                                <span style="color: #999; font-size: 0.85em;">"Одобри първо"</span>
                                                            }.into_view()
                                                        }}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                        {if dist.dividends.is_empty() {
                                            view! {
                                                <tr>
                                                    <td colspan="8" style="padding: 30px; text-align: center; color: #7f8c8d;">
                                                        "Няма собственици с определен процент на собственост. Добавете собственици в настройките на фирмата (SAFt раздел)."
                                                    </td>
                                                </tr>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <tr style="background: #f8f9fa; font-weight: bold;">
                                                    <td colspan="3" style="padding: 15px; text-align: right;">"ОБЩО:"</td>
                                                    <td style="padding: 15px; text-align: right;">{format!("{:.2}", dist.total_amount)}</td>
                                                    <td style="padding: 15px; text-align: right; color: #e67e22;">{format!("{:.2}", total_tax)}</td>
                                                    <td style="padding: 15px; text-align: right; color: #27ae60;">{format!("{:.2}", total_net)}</td>
                                                    <td colspan="2"></td>
                                                </tr>
                                            }.into_view()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    // List view
                    let owners = beneficial_owners.get();
                    let total_ownership: f64 = owners.iter()
                        .filter_map(|o| o.ownership_percentage)
                        .sum();

                    view! {
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <h1 style="margin: 0;">"Дивиденти"</h1>
                                <button
                                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                                    on:click=move |_| show_new_modal.set(true)
                                    disabled=move || total_ownership == 0.0
                                    title=move || if total_ownership == 0.0 { "Добавете собственици в настройките на фирмата" } else { "" }
                                >
                                    "+ Ново разпределение"
                                </button>
                            </div>

                            // Owners summary card
                            <div style="background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                <h3 style="margin-top: 0; margin-bottom: 15px;">"Действителни собственици"</h3>
                                {if owners.is_empty() {
                                    view! {
                                        <div style="color: #7f8c8d; padding: 20px; text-align: center;">
                                            <p>"Няма добавени собственици."</p>
                                            <a href="/settings" style="color: #3498db;">"Добавете собственици в настройките на фирмата (SAFt раздел)"</a>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <div style="display: flex; flex-wrap: wrap; gap: 15px;">
                                            {owners.iter().map(|o| {
                                                view! {
                                                    <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; min-width: 200px;">
                                                        <div style="font-weight: 500;">{format!("{} {}", o.first_name_bg, o.last_name_bg)}</div>
                                                        <div style="color: #666; font-size: 0.9em;">
                                                            {format!("{:.2}%", o.ownership_percentage.unwrap_or(0.0))}
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_view()
                                }}
                                {if total_ownership > 0.0 && (total_ownership - 100.0).abs() > 0.01 {
                                    view! {
                                        <div style="margin-top: 15px; padding: 10px; background: #fff3cd; border-radius: 4px; color: #856404;">
                                            {format!("Внимание: Общият процент на собственост е {:.2}% (трябва да е 100%)", total_ownership)}
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <span></span> }.into_view()
                                }}
                            </div>

                            // Distributions table
                            <div style="background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: hidden;">
                                <table style="width: 100%; border-collapse: collapse;">
                                    <thead>
                                        <tr style="background: #f8f9fa;">
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"Година"</th>
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"Дата на решение"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Обща сума"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Изплатено"</th>
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"Статус"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Действия"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || distributions.get().iter().map(|d| {
                                            let d_id = d.id;
                                            view! {
                                                <tr style="border-bottom: 1px solid #dee2e6;">
                                                    <td style="padding: 15px; font-weight: 500;">{d.year}</td>
                                                    <td style="padding: 15px;">{d.decision_date.clone()}</td>
                                                    <td style="padding: 15px; text-align: right;">{format!("{:.2} EUR", d.total_amount)}</td>
                                                    <td style="padding: 15px; text-align: right;">
                                                        {format!("{:.2} / {:.2} EUR", d.total_paid.unwrap_or(0.0), d.total_net.unwrap_or(0.0))}
                                                    </td>
                                                    <td style="padding: 15px;">
                                                        <span style=format!("padding: 4px 12px; border-radius: 4px; font-size: 0.85em; {}",
                                                            match d.status.as_str() {
                                                                "paid" => "background: #27ae60; color: white;",
                                                                "partially_paid" => "background: #f39c12; color: white;",
                                                                "approved" => "background: #3498db; color: white;",
                                                                _ => "background: #bdc3c7;"
                                                            }
                                                        )>{d.status_label.clone().unwrap_or(d.status.clone())}</span>
                                                    </td>
                                                    <td style="padding: 15px; text-align: right;">
                                                        <button
                                                            style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                            on:click=move |_| load_detail(d_id)
                                                        >
                                                            "Преглед"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                        {move || if distributions.get().is_empty() && !loading.get() {
                                            view! {
                                                <tr>
                                                    <td colspan="6" style="padding: 30px; text-align: center; color: #7f8c8d;">
                                                        "Няма разпределения на дивиденти. Натиснете \"+ Ново разпределение\" за да създадете."
                                                    </td>
                                                </tr>
                                            }.into_view()
                                        } else {
                                            view! { <tr></tr> }.into_view()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }.into_view()
                }
            }}

            // New distribution modal
            {move || if show_new_modal.get() {
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                        <div style="background: white; border-radius: 8px; padding: 30px; min-width: 450px; box-shadow: 0 4px 20px rgba(0,0,0,0.3);">
                            <h3 style="margin-top: 0;">"Ново разпределение на дивиденти"</h3>
                            <div style="margin: 20px 0;">
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Финансова година"</label>
                                    <input
                                        type="text"
                                        inputmode="numeric"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_year.get()
                                        on:input=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse() {
                                                new_year.set(v);
                                            }
                                        }
                                    />
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Обща сума за разпределяне (EUR)"</label>
                                    <input
                                        type="text"
                                        inputmode="decimal"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || format!("{:.2}", new_total_amount.get())
                                        on:input=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse() {
                                                new_total_amount.set(v);
                                            }
                                        }
                                    />
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Дата на решение (протокол ОС)"</label>
                                    <input
                                        type="date"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_decision_date.get()
                                        on:input=move |ev| {
                                            new_decision_date.set(event_target_value(&ev));
                                        }
                                    />
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Номер на протокол (опционално)"</label>
                                    <input
                                        type="text"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_decision_number.get()
                                        on:input=move |ev| {
                                            new_decision_number.set(event_target_value(&ev));
                                        }
                                    />
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Бележки (опционално)"</label>
                                    <textarea
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; min-height: 80px;"
                                        prop:value=move || new_notes.get()
                                        on:input=move |ev| {
                                            new_notes.set(event_target_value(&ev));
                                        }
                                    ></textarea>
                                </div>
                            </div>

                            // Preview of distribution
                            {move || {
                                let amount = new_total_amount.get();
                                let owners = beneficial_owners.get();
                                if amount > 0.0 && !owners.is_empty() {
                                    let total_pct: f64 = owners.iter().filter_map(|o| o.ownership_percentage).sum();
                                    if total_pct > 0.0 {
                                        view! {
                                            <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 15px;">
                                                <h5 style="margin-top: 0; margin-bottom: 10px;">"Предварителен преглед"</h5>
                                                {owners.iter().filter(|o| o.ownership_percentage.unwrap_or(0.0) > 0.0).map(|o| {
                                                    let pct = o.ownership_percentage.unwrap_or(0.0);
                                                    let gross = (amount * pct / total_pct * 100.0).round() / 100.0;
                                                    let tax = (gross * 5.0 / 100.0 * 100.0).round() / 100.0;
                                                    let net = gross - tax;
                                                    view! {
                                                        <div style="display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid #eee;">
                                                            <span>{format!("{} {} ({:.2}%)", o.first_name_bg, o.last_name_bg, pct)}</span>
                                                            <span style="font-weight: 500;">{format!("{:.2} EUR (нето)", net)}</span>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }
                                } else {
                                    view! { <span></span> }.into_view()
                                }
                            }}

                            <div style="display: flex; justify-content: flex-end; gap: 10px;">
                                <button
                                    style="padding: 10px 20px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; background: white;"
                                    on:click=move |_| show_new_modal.set(false)
                                >
                                    "Отказ"
                                </button>
                                <button
                                    style="padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; background: #3498db; color: white;"
                                    on:click=create_distribution
                                    disabled=move || creating.get() || new_total_amount.get() <= 0.0
                                >
                                    {move || if creating.get() { "Създаване..." } else { "Създай" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }}
        </Layout>
    }
}

fn get_current_year() -> i32 {
    let now = js_sys::Date::new_0();
    (now.get_full_year() - 1) as i32  // Usually dividends are for previous year
}

fn get_today_date() -> String {
    let now = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        now.get_full_year(),
        now.get_month() + 1,
        now.get_date()
    )
}
