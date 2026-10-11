use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete, company_api_url};
use crate::context::use_company_context;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeneficialOwner {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub first_name_bg: String,
    #[serde(default)]
    pub last_name_bg: String,
    pub egn: Option<String>,
    pub ownership_percentage: Option<f64>,
}

impl BeneficialOwner {
    fn display_name(&self) -> String {
        format!("{} {}", self.first_name_bg, self.last_name_bg)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendDistribution {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub year: i32,
    #[serde(default)]
    pub total_amount: f64,
    #[serde(default)]
    pub decision_date: String,
    pub decision_number: Option<String>,
    #[serde(default)]
    pub status: String,
    pub status_label: Option<String>,
    pub notes: Option<String>,
    pub dividends_count: Option<i64>,
    pub total_net: Option<f64>,
    pub total_paid: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dividend {
    #[serde(default)]
    pub id: i64,
    pub beneficial_owner_id: Option<i64>,
    pub owner_name: Option<String>,
    pub owner_egn: Option<String>,
    pub ownership_percentage: Option<f64>,
    #[serde(default)]
    pub gross_amount: f64,
    #[serde(default)]
    pub tax_rate: f64,
    #[serde(default)]
    pub tax_amount: f64,
    #[serde(default)]
    pub net_amount: f64,
    #[serde(default)]
    pub is_paid: bool,
    pub payment_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendDistributionDetail {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub year: i32,
    #[serde(default)]
    pub total_amount: f64,
    #[serde(default)]
    pub decision_date: String,
    pub decision_number: Option<String>,
    #[serde(default)]
    pub status: String,
    pub status_label: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub dividends: Vec<Dividend>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendReportRow {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub distribution_id: Option<i64>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub decision_date: Option<String>,
    #[serde(default)]
    pub decision_number: Option<String>,
    #[serde(default)]
    pub payment_date: Option<String>,
    #[serde(default)]
    pub owner_name: Option<String>,
    #[serde(default)]
    pub owner_egn: Option<String>,
    #[serde(default)]
    pub ownership_percentage: Option<f64>,
    #[serde(default)]
    pub gross_amount: f64,
    #[serde(default)]
    pub tax_rate: f64,
    #[serde(default)]
    pub tax_amount: f64,
    #[serde(default)]
    pub net_amount: f64,
    #[serde(default)]
    pub is_paid: bool,
    pub status: Option<String>,
    pub status_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendReportTotals {
    #[serde(default)]
    pub gross_amount: f64,
    #[serde(default)]
    pub tax_amount: f64,
    #[serde(default)]
    pub net_amount: f64,
    #[serde(default)]
    pub paid_net: f64,
    #[serde(default)]
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendReportOwnerTotal {
    #[serde(default)]
    pub owner_name: String,
    pub owner_egn: Option<String>,
    #[serde(default)]
    pub gross_amount: f64,
    #[serde(default)]
    pub tax_amount: f64,
    #[serde(default)]
    pub net_amount: f64,
    #[serde(default)]
    pub paid_net: f64,
    #[serde(default)]
    pub count: i64,
}

#[derive(Debug, Clone)]
struct CompanyBrief {
    name: String,
    eik: String,
    address: String,
    city: String,
    manager_name: String,
    currency: String,
    tax_rate: f64,
}

impl Default for CompanyBrief {
    fn default() -> Self {
        Self {
            name: String::new(),
            eik: String::new(),
            address: String::new(),
            city: String::new(),
            manager_name: String::new(),
            currency: "EUR".to_string(),
            tax_rate: 5.0,
        }
    }
}

#[component]
pub fn Dividends() -> impl IntoView {
    let company_ctx = use_company_context();

    let distributions = RwSignal::new(Vec::<DividendDistribution>::new());
    let beneficial_owners = RwSignal::new(Vec::<BeneficialOwner>::new());
    let company = RwSignal::new(CompanyBrief::default());
    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());

    let show_new_modal = RwSignal::new(false);
    let editing_id = RwSignal::new(Option::<i64>::None);
    let new_year = RwSignal::new(get_current_year().to_string());
    let new_total_amount = RwSignal::new(String::new());
    let new_tax_rate = RwSignal::new("5".to_string());
    let new_decision_date = RwSignal::new(get_today_date());
    let new_decision_number = RwSignal::new(String::new());
    let new_notes = RwSignal::new(String::new());
    let creating = RwSignal::new(false);

    let selected_distribution = RwSignal::new(Option::<DividendDistributionDetail>::None);
    let detail_loading = RwSignal::new(false);

    let active_tab = RwSignal::new("list".to_string());
    let report_date_from = RwSignal::new(year_start_date());
    let report_date_to = RwSignal::new(get_today_date());
    let report_date_field = RwSignal::new("decision_date".to_string());
    let report_include_drafts = RwSignal::new(false);
    let report_rows = RwSignal::new(Vec::<DividendReportRow>::new());
    let report_totals = RwSignal::new(DividendReportTotals::default());
    let report_by_owner = RwSignal::new(Vec::<DividendReportOwnerTotal>::new());
    let report_loaded = RwSignal::new(false);
    let report_loading = RwSignal::new(false);

    let load_distributions = move || {
        loading.set(true);
        spawn_local(async move {
            match api_get(&format!("{}/dividend_distributions", company_api_url())).await {
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

    let load_owners = move || {
        spawn_local(async move {
            match api_get(&format!("{}/beneficial_owners", company_api_url())).await {
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

    let load_company = move || {
        spawn_local(async move {
            match api_get(&company_api_url()).await {
                Ok(data) => {
                    if let Some(c) = data.get("data") {
                        let tax_rate = c.get("settings")
                            .and_then(|s| s.get("dividends"))
                            .and_then(|d| d.get("tax_rate"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(5.0);
                        company.set(CompanyBrief {
                            name: c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            eik: c.get("eik").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            address: c.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            city: c.get("city").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            manager_name: c.get("manager_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            currency: c.get("currency").and_then(|v| v.as_str()).unwrap_or("EUR").to_string(),
                            tax_rate,
                        });
                        new_tax_rate.set(format_tax_rate_value(tax_rate));
                    }
                }
                Err(_) => {}
            }
        });
    };

    let load_detail = move |id: i64| {
        detail_loading.set(true);
        spawn_local(async move {
            let url = format!("{}/dividend_distributions/{}", company_api_url(), id);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(detail_data) = data.get("data") {
                        match serde_json::from_value::<DividendDistributionDetail>(detail_data.clone()) {
                            Ok(detail) => selected_distribution.set(Some(detail)),
                            Err(e) => error_msg.set(format!("Грешка при четене: {}", e)),
                        }
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            detail_loading.set(false);
        });
    };

    let reset_form = move || {
        editing_id.set(None);
        new_year.set(get_current_year().to_string());
        new_total_amount.set(String::new());
        new_tax_rate.set(format_tax_rate_value(company.get_untracked().tax_rate));
        new_decision_date.set(get_today_date());
        new_decision_number.set(String::new());
        new_notes.set(String::new());
    };

    let open_edit = move |dist: DividendDistributionDetail| {
        error_msg.set(String::new());
        editing_id.set(Some(dist.id));
        new_year.set(dist.year.to_string());
        new_total_amount.set(format_amount_value(dist.total_amount));
        let rate = dist.dividends.first().map(|d| d.tax_rate).unwrap_or(company.get_untracked().tax_rate);
        new_tax_rate.set(format_tax_rate_value(rate));
        new_decision_date.set(date_input_value(&dist.decision_date));
        new_decision_number.set(dist.decision_number.clone().unwrap_or_default());
        new_notes.set(dist.notes.clone().unwrap_or_default());
        show_new_modal.set(true);
    };

    let create_distribution = move |_| {
        let year: i32 = match new_year.get().trim().parse() {
            Ok(v) => v,
            Err(_) => {
                error_msg.set("Невалидна финансова година".to_string());
                return;
            }
        };
        let total_amount = parse_decimal(&new_total_amount.get());
        if total_amount <= 0.0 {
            error_msg.set("Въведете сума за разпределяне по-голяма от 0".to_string());
            return;
        }
        let tax_rate = parse_decimal(&new_tax_rate.get());
        if tax_rate < 0.0 || tax_rate > 100.0 {
            error_msg.set("Данъчната ставка трябва да е между 0 и 100".to_string());
            return;
        }
        let owners_ok = beneficial_owners.get().iter().any(|o| o.ownership_percentage.unwrap_or(0.0) > 0.0);
        if !owners_ok {
            error_msg.set("Няма действителни собственици с процент. Добавете ги в Настройки → SAFt.".to_string());
            return;
        }

        creating.set(true);
        error_msg.set(String::new());

        let decision_date = new_decision_date.get();
        let decision_number = new_decision_number.get();
        let notes = new_notes.get();
        let edit_id = editing_id.get();

        spawn_local(async move {
            let payload = serde_json::json!({
                "dividend_distribution": {
                    "year": year,
                    "total_amount": total_amount,
                    "tax_rate": tax_rate,
                    "decision_date": decision_date,
                    "decision_number": if decision_number.trim().is_empty() { serde_json::Value::Null } else { serde_json::json!(decision_number.trim()) },
                    "notes": if notes.trim().is_empty() { serde_json::Value::Null } else { serde_json::json!(notes.trim()) }
                }
            });

            let result = if let Some(id) = edit_id {
                api_put(&format!("{}/dividend_distributions/{}", company_api_url(), id), &payload).await
            } else {
                api_post(&format!("{}/dividend_distributions", company_api_url()), &payload).await
            };

            match result {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set(if edit_id.is_some() {
                            "Разпределението е обновено".to_string()
                        } else {
                            "Разпределението е създадено".to_string()
                        });
                        show_new_modal.set(false);
                        reset_form();
                        load_distributions();
                        let id = edit_id.or_else(|| {
                            response.get("data").and_then(|d| d.get("id")).and_then(|v| v.as_i64())
                        });
                        if let Some(id) = id {
                            load_detail(id);
                        }
                    } else {
                        error_msg.set(format!("Грешка: {}", extract_response_error(&response)));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            creating.set(false);
        });
    };

    let mark_paid = move |dividend_id: i64, is_paid: bool| {
        spawn_local(async move {
            let url = format!("{}/dividends/{}", company_api_url(), dividend_id);
            let payload = serde_json::json!({
                "dividend": {
                    "is_paid": is_paid,
                    "payment_date": if is_paid { Some(get_today_date()) } else { None::<String> }
                }
            });

            match api_put(&url, &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set(if is_paid { "Маркирано като платено" } else { "Маркирано като неплатено" }.to_string());
                        if let Some(dist) = selected_distribution.get_untracked() {
                            load_detail(dist.id);
                        }
                        load_distributions();
                    } else {
                        error_msg.set(format!("Грешка: {}", extract_response_error(&response)));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    let delete_distribution = move |id: i64| {
        spawn_local(async move {
            let url = format!("{}/dividend_distributions/{}", company_api_url(), id);
            match api_delete(&url).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Разпределението е изтрито".to_string());
                        selected_distribution.set(None);
                        load_distributions();
                    } else {
                        error_msg.set(extract_response_error(&response));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    let approve_distribution = move |id: i64| {
        spawn_local(async move {
            let url = format!("{}/dividend_distributions/{}", company_api_url(), id);
            let payload = serde_json::json!({
                "dividend_distribution": { "status": "approved" }
            });

            match api_put(&url, &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Разпределението е одобрено".to_string());
                        load_detail(id);
                        load_distributions();
                    } else {
                        error_msg.set(format!("Грешка: {}", extract_response_error(&response)));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    let revert_to_draft = move |id: i64| {
        spawn_local(async move {
            let url = format!("{}/dividend_distributions/{}", company_api_url(), id);
            let payload = serde_json::json!({
                "dividend_distribution": { "status": "draft" }
            });

            match api_put(&url, &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Разпределението е върнато в чернова. Сега може да се редактира или изтрие.".to_string());
                        load_detail(id);
                        load_distributions();
                    } else {
                        error_msg.set(format!("Грешка: {}", extract_response_error(&response)));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    let load_report = move || {
        let from = report_date_from.get();
        let to = report_date_to.get();
        if !from.is_empty() && !to.is_empty() && from > to {
            error_msg.set("Началната дата трябва да е преди крайната".to_string());
            return;
        }

        report_loading.set(true);
        error_msg.set(String::new());
        let field = report_date_field.get();
        let drafts = report_include_drafts.get();

        spawn_local(async move {
            let mut url = format!("{}/reports/dividends?", company_api_url());
            if !from.is_empty() {
                url.push_str(&format!("date_from={}&", from));
            }
            if !to.is_empty() {
                url.push_str(&format!("date_to={}&", to));
            }
            url.push_str(&format!(
                "date_field={}&include_drafts={}",
                field,
                if drafts { "true" } else { "false" }
            ));

            match api_get(&url).await {
                Ok(data) => {
                    if let Some(payload) = data.get("data") {
                        let rows: Vec<DividendReportRow> = payload
                            .get("rows")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let totals: DividendReportTotals = payload
                            .get("totals")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let by_owner: Vec<DividendReportOwnerTotal> = payload
                            .get("by_owner")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        report_rows.set(rows);
                        report_totals.set(totals);
                        report_by_owner.set(by_owner);
                        report_loaded.set(true);
                    } else {
                        error_msg.set("Невалиден отговор от сървъра".to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            report_loading.set(false);
        });
    };

    Effect::new(move |_| {
        let _ = company_ctx.selected_company_id.get();
        selected_distribution.set(None);
        report_rows.set(Vec::new());
        report_totals.set(DividendReportTotals::default());
        report_by_owner.set(Vec::new());
        report_loaded.set(false);
        report_date_from.set(year_start_date());
        report_date_to.set(get_today_date());
        load_distributions();
        load_owners();
        load_company();
    });

    view! {
        <Layout>
            {move || if !error_msg.get().is_empty() {
                view! {
                    <div style="background: #f8d7da; color: #721c24; padding: 10px; border-radius: 4px; margin-bottom: 15px;">
                        {error_msg.get()}
                        <button style="float: right; border: none; background: none; cursor: pointer;" on:click=move |_| error_msg.set(String::new())>"×"</button>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}

            {move || if !success_msg.get().is_empty() {
                view! {
                    <div style="background: #d4edda; color: #155724; padding: 10px; border-radius: 4px; margin-bottom: 15px;">
                        {success_msg.get()}
                        <button style="float: right; border: none; background: none; cursor: pointer;" on:click=move |_| success_msg.set(String::new())>"×"</button>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}

            {move || {
                if let Some(dist) = selected_distribution.get() {
                    let dist_for_approve = dist.clone();
                    let dist_for_delete = dist.clone();
                    let dist_for_revert = dist.clone();
                    let dist_for_print = dist.clone();
                    let dist_for_word = dist.clone();
                    let dist_for_edit = dist.clone();
                    let any_paid = dist.dividends.iter().any(|d| d.is_paid);
                    let total_net: f64 = dist.dividends.iter().map(|d| d.net_amount).sum();
                    let total_paid: f64 = dist.dividends.iter().filter(|d| d.is_paid).map(|d| d.net_amount).sum();
                    let total_tax: f64 = dist.dividends.iter().map(|d| d.tax_amount).sum();
                    let currency = if company.get().currency.is_empty() { "EUR".to_string() } else { company.get().currency.clone() };

                    view! {
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; flex-wrap: wrap; gap: 10px;">
                                <div style="display: flex; align-items: center; gap: 15px; flex-wrap: wrap;">
                                    <button
                                        style="background: #ecf0f1; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| selected_distribution.set(None)
                                    >
                                        "← Назад"
                                    </button>
                                    <h2 style="margin: 0;">{format!("Дивиденти за {} г.", dist.year)}</h2>
                                    <span style=status_style(&dist.status)>
                                        {dist.status_label.clone().unwrap_or(dist.status.clone())}
                                    </span>
                                </div>
                                <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                    <button
                                        style="background: #e74c3c; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                        title="HTML преглед и печат / запис като PDF"
                                        on:click=move |_| {
                                            open_protocol_html(company.get_untracked(), dist_for_print.clone(), true);
                                        }
                                    >
                                        "Печат/PDF"
                                    </button>
                                    <button
                                        style="background: #2b579a; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                        title="Свали като Word документ"
                                        on:click=move |_| {
                                            download_protocol_word(company.get_untracked(), dist_for_word.clone());
                                        }
                                    >
                                        "Word"
                                    </button>
                                    {if !any_paid {
                                        view! {
                                            <button
                                                style="background: #f39c12; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                                title="Поправи брутна сума, данък, дата и бележки"
                                                on:click=move |_| open_edit(dist_for_edit.clone())
                                            >
                                                "Редактирай"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    {if dist.status == "draft" {
                                        let id = dist_for_approve.id;
                                        view! {
                                            <button
                                                style="background: #27ae60; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                                on:click=move |_| approve_distribution(id)
                                            >
                                                "Одобри"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    {if dist.status != "draft" {
                                        let id = dist_for_revert.id;
                                        let paid = any_paid;
                                        view! {
                                            <button
                                                style="background: #8e44ad; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                                title="Връща разпределението в чернова, за да може да се редактира или изтрие"
                                                on:click=move |_| {
                                                    let msg = if paid {
                                                        "Ще се отменят всички отбелязани плащания и разпределението ще се върне в чернова. След това можете да го изтриете. Продължавате?"
                                                    } else {
                                                        "Разпределението ще се върне в чернова. След това можете да го редактирате или изтриете. Продължавате?"
                                                    };
                                                    if web_sys::window()
                                                        .and_then(|w| w.confirm_with_message(msg).ok())
                                                        .unwrap_or(false)
                                                    {
                                                        revert_to_draft(id);
                                                    }
                                                }
                                            >
                                                "Върни в чернова"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
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
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                            </div>

                            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 20px; margin-bottom: 20px;">
                                <div style="padding: 20px; background: #e3f2fd; border-radius: 8px; text-align: center;">
                                    <div style="font-size: 0.85em; color: #666;">"Обща сума"</div>
                                    <div style="font-size: 1.5em; font-weight: bold; color: #1976d2;">{format!("{:.2} {}", dist.total_amount, currency)}</div>
                                </div>
                                <div style="padding: 20px; background: #fff3e0; border-radius: 8px; text-align: center;">
                                    <div style="font-size: 0.85em; color: #666;">"Данък"</div>
                                    <div style="font-size: 1.5em; font-weight: bold; color: #f57c00;">{format!("{:.2} {}", total_tax, currency)}</div>
                                </div>
                                <div style="padding: 20px; background: #e8f5e9; border-radius: 8px; text-align: center;">
                                    <div style="font-size: 0.85em; color: #666;">"Нетни дивиденти"</div>
                                    <div style="font-size: 1.5em; font-weight: bold; color: #388e3c;">{format!("{:.2} {}", total_net, currency)}</div>
                                </div>
                                <div style="padding: 20px; background: #fce4ec; border-radius: 8px; text-align: center;">
                                    <div style="font-size: 0.85em; color: #666;">"Изплатено"</div>
                                    <div style="font-size: 1.5em; font-weight: bold; color: #c2185b;">{format!("{:.2} {}", total_paid, currency)}</div>
                                </div>
                            </div>

                            <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px; display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 15px;">
                                <div>
                                    <div style="color: #666; font-size: 0.85em;">"Дата на решение"</div>
                                    <div style="font-weight: 500;">{format_bg_date(&dist.decision_date)}</div>
                                </div>
                                <div>
                                    <div style="color: #666; font-size: 0.85em;">"Номер на протокол"</div>
                                    <div style="font-weight: 500;">{dist.decision_number.clone().unwrap_or_else(|| "-".to_string())}</div>
                                </div>
                                <div>
                                    <div style="color: #666; font-size: 0.85em;">"Бележки"</div>
                                    <div style="font-weight: 500;">{dist.notes.clone().unwrap_or_else(|| "-".to_string())}</div>
                                </div>
                            </div>

                            <div style="background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: auto;">
                                <table style="width: 100%; border-collapse: collapse;">
                                    <thead>
                                        <tr style="background: #f8f9fa;">
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"Собственик"</th>
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"ЕГН / идентификатор"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Дял %"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Брутна сума"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Данък"</th>
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
                                                    <td style="padding: 15px; font-weight: 500;">{d.owner_name.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                    <td style="padding: 15px;">{d.owner_egn.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                    <td style="padding: 15px; text-align: right;">{format!("{:.2}%", d.ownership_percentage.unwrap_or(0.0))}</td>
                                                    <td style="padding: 15px; text-align: right;">{format!("{:.2}", d.gross_amount)}</td>
                                                    <td style="padding: 15px; text-align: right; color: #e67e22;">{format!("{:.2}", d.tax_amount)}</td>
                                                    <td style="padding: 15px; text-align: right; font-weight: bold; color: #27ae60;">{format!("{:.2}", d.net_amount)}</td>
                                                    <td style="padding: 15px; text-align: center;">
                                                        {if d.is_paid {
                                                            view! {
                                                                <span style="background: #27ae60; color: white; padding: 4px 10px; border-radius: 4px; font-size: 0.85em;">
                                                                    {if let Some(pd) = &d.payment_date {
                                                                        format!("Платено {}", format_bg_date(pd))
                                                                    } else {
                                                                        "Платено".to_string()
                                                                    }}
                                                                </span>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <span style="background: #bdc3c7; padding: 4px 10px; border-radius: 4px; font-size: 0.85em;">
                                                                    "Неплатено"
                                                                </span>
                                                            }.into_any()
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
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <button
                                                                        style="background: #27ae60; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 0.85em;"
                                                                        on:click=move |_| mark_paid(d_id, true)
                                                                    >
                                                                        "Плати"
                                                                    </button>
                                                                }.into_any()
                                                            }
                                                        } else {
                                                            view! {
                                                                <span style="color: #999; font-size: 0.85em;">"Одобри първо"</span>
                                                            }.into_any()
                                                        }}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                        {if dist.dividends.is_empty() {
                                            view! {
                                                <tr>
                                                    <td colspan="8" style="padding: 30px; text-align: center; color: #7f8c8d;">
                                                        "Няма собственици с определен процент. Добавете ги в "
                                                        <a href="/settings" style="color: #3498db;">"Настройки → SAFt → Действителни собственици"</a>
                                                        "."
                                                    </td>
                                                </tr>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <tr style="background: #f8f9fa; font-weight: bold;">
                                                    <td colspan="3" style="padding: 15px; text-align: right;">"ОБЩО:"</td>
                                                    <td style="padding: 15px; text-align: right;">{format!("{:.2}", dist.total_amount)}</td>
                                                    <td style="padding: 15px; text-align: right; color: #e67e22;">{format!("{:.2}", total_tax)}</td>
                                                    <td style="padding: 15px; text-align: right; color: #27ae60;">{format!("{:.2}", total_net)}</td>
                                                    <td colspan="2"></td>
                                                </tr>
                                            }.into_any()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    let owners = beneficial_owners.get();
                    let total_ownership: f64 = owners.iter()
                        .filter_map(|o| o.ownership_percentage)
                        .sum();
                    let currency = if company.get().currency.is_empty() { "EUR".to_string() } else { company.get().currency.clone() };
                    let tab = active_tab.get();
                    let dist_list = distributions.get();
                    let is_loading = loading.get();
                    let _ = report_rows.get();
                    let _ = report_totals.get();
                    let _ = report_by_owner.get();
                    let _ = report_loading.get();
                    let _ = report_loaded.get();

                    view! {
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; flex-wrap: wrap; gap: 10px;">
                                <h1 style="margin: 0;">"Дивиденти"</h1>
                                {if tab == "list" {
                                    view! {
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                                            on:click=move |_| {
                                                error_msg.set(String::new());
                                                reset_form();
                                                load_owners();
                                                show_new_modal.set(true);
                                            }
                                        >
                                            "+ Ново разпределение"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            style="background: #e74c3c; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                                            title="HTML преглед и печат / запис като PDF"
                                            on:click=move |_| {
                                                if !report_loaded.get_untracked() {
                                                    error_msg.set("Първо генерирайте справката с бутона „Покажи“.".to_string());
                                                    return;
                                                }
                                                open_dividend_report_html(
                                                    company.get_untracked(),
                                                    report_date_from.get_untracked(),
                                                    report_date_to.get_untracked(),
                                                    report_date_field.get_untracked(),
                                                    report_rows.get_untracked(),
                                                    report_totals.get_untracked(),
                                                    report_by_owner.get_untracked(),
                                                );
                                            }
                                        >
                                            "Печат/PDF"
                                        </button>
                                    }.into_any()
                                }}
                            </div>

                            <div style="display: flex; gap: 0; margin-bottom: 0; border-bottom: 2px solid #ddd;">
                                <button
                                    style=move || tab_style(active_tab.get() == "list")
                                    on:click=move |_| active_tab.set("list".to_string())
                                >
                                    "Разпределения"
                                </button>
                                <button
                                    style=move || tab_style(active_tab.get() == "report")
                                    on:click=move |_| {
                                        active_tab.set("report".to_string());
                                        if !report_loaded.get_untracked() && !report_loading.get_untracked() {
                                            load_report();
                                        }
                                    }
                                >
                                    "Справка"
                                </button>
                            </div>

                            {if tab == "list" {
                                view! {
                                    <div>
                            <div style="background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; flex-wrap: wrap; gap: 8px;">
                                    <h3 style="margin: 0;">"Действителни собственици"</h3>
                                    <a href="/settings" style="color: #3498db; font-size: 0.9em;">"Настройки → SAFt"</a>
                                </div>
                                {if owners.is_empty() {
                                    view! {
                                        <div style="color: #7f8c8d; padding: 20px; text-align: center;">
                                            <p>"Няма добавени собственици за тази фирма."</p>
                                            <a href="/settings" style="color: #3498db;">"Добавете ги в Настройки → SAFt → Действителни собственици"</a>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="display: flex; flex-wrap: wrap; gap: 15px;">
                                            {owners.iter().map(|o| {
                                                view! {
                                                    <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; min-width: 200px;">
                                                        <div style="font-weight: 500;">{o.display_name()}</div>
                                                        <div style="color: #666; font-size: 0.9em;">
                                                            {format!("{:.2}%", o.ownership_percentage.unwrap_or(0.0))}
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }}
                                {if total_ownership > 0.0 && (total_ownership - 100.0).abs() > 0.01 {
                                    view! {
                                        <div style="margin-top: 15px; padding: 10px; background: #fff3cd; border-radius: 4px; color: #856404;">
                                            {format!("Внимание: Общият процент на собственост е {:.2}% (трябва да е 100%)", total_ownership)}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </div>

                            <div style="background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: auto;">
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
                                        {dist_list.iter().map(|d| {
                                            let d_id = d.id;
                                            view! {
                                                <tr style="border-bottom: 1px solid #dee2e6;">
                                                    <td style="padding: 15px; font-weight: 500;">{d.year}</td>
                                                    <td style="padding: 15px;">{format_bg_date(&d.decision_date)}</td>
                                                    <td style="padding: 15px; text-align: right;">{format!("{:.2} {}", d.total_amount, currency)}</td>
                                                    <td style="padding: 15px; text-align: right;">
                                                        {format!("{:.2} / {:.2} {}", d.total_paid.unwrap_or(0.0), d.total_net.unwrap_or(0.0), currency)}
                                                    </td>
                                                    <td style="padding: 15px;">
                                                        <span style=status_style(&d.status)>
                                                            {d.status_label.clone().unwrap_or(d.status.clone())}
                                                        </span>
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
                                        {if dist_list.is_empty() && !is_loading {
                                            view! {
                                                <tr>
                                                    <td colspan="6" style="padding: 30px; text-align: center; color: #7f8c8d;">
                                                        "Няма разпределения на дивиденти. Натиснете \"+ Ново разпределение\" за да създадете."
                                                    </td>
                                                </tr>
                                            }.into_any()
                                        } else if is_loading {
                                            view! {
                                                <tr>
                                                    <td colspan="6" style="padding: 30px; text-align: center; color: #7f8c8d;">
                                                        "Зареждане..."
                                                    </td>
                                                </tr>
                                            }.into_any()
                                        } else {
                                            view! { <tr></tr> }.into_any()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                                    </div>
                                }.into_any()
                            } else {
                                let totals = report_totals.get();
                                let rows = report_rows.get();
                                let by_owner = report_by_owner.get();
                                let currency_r = currency.clone();
                                view! {
                                    <div>
                                        <div style="background: white; padding: 20px; border-radius: 0 8px 8px 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                            <div style="display: flex; gap: 16px; flex-wrap: wrap; align-items: flex-end;">
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">"От дата"</label>
                                                    <input
                                                        type="date"
                                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                        prop:value=move || report_date_from.get()
                                                        on:input=move |ev| report_date_from.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">"До дата"</label>
                                                    <input
                                                        type="date"
                                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                                        prop:value=move || report_date_to.get()
                                                        on:input=move |ev| report_date_to.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #34495e;">"По дата"</label>
                                                    <select
                                                        style="padding: 10px; border: 1px solid #ddd; border-radius: 4px; min-width: 180px;"
                                                        prop:value=move || report_date_field.get()
                                                        on:change=move |ev| report_date_field.set(event_target_value(&ev))
                                                    >
                                                        <option value="decision_date">"Дата на решение"</option>
                                                        <option value="payment_date">"Дата на плащане"</option>
                                                    </select>
                                                </div>
                                                <label style="display: flex; align-items: center; gap: 8px; padding-bottom: 10px; cursor: pointer;">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || report_include_drafts.get()
                                                        on:change=move |ev| report_include_drafts.set(event_target_checked(&ev))
                                                    />
                                                    <span>"Включи чернови"</span>
                                                </label>
                                                <button
                                                    style="padding: 10px 20px; background: #3498db; color: white; border: none; border-radius: 4px; cursor: pointer;"
                                                    on:click=move |_| load_report()
                                                    disabled=move || report_loading.get()
                                                >
                                                    {move || if report_loading.get() { "Зареждане..." } else { "Покажи" }}
                                                </button>
                                            </div>
                                            <div style="margin-top: 10px; color: #7f8c8d; font-size: 13px;">
                                                "По дата на решение — всички разпределения в периода. По дата на плащане — само изплатените дивиденти (за данък при източника)."
                                            </div>
                                        </div>

                                        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 16px; margin-bottom: 20px;">
                                            <div style="padding: 16px; background: #e3f2fd; border-radius: 8px; text-align: center;">
                                                <div style="font-size: 0.85em; color: #666;">"Брутна сума"</div>
                                                <div style="font-size: 1.35em; font-weight: bold; color: #1976d2;">{format!("{:.2} {}", totals.gross_amount, currency_r)}</div>
                                            </div>
                                            <div style="padding: 16px; background: #fff3e0; border-radius: 8px; text-align: center;">
                                                <div style="font-size: 0.85em; color: #666;">"Данък"</div>
                                                <div style="font-size: 1.35em; font-weight: bold; color: #f57c00;">{format!("{:.2} {}", totals.tax_amount, currency_r)}</div>
                                            </div>
                                            <div style="padding: 16px; background: #e8f5e9; border-radius: 8px; text-align: center;">
                                                <div style="font-size: 0.85em; color: #666;">"Нетна сума"</div>
                                                <div style="font-size: 1.35em; font-weight: bold; color: #388e3c;">{format!("{:.2} {}", totals.net_amount, currency_r)}</div>
                                            </div>
                                            <div style="padding: 16px; background: #fce4ec; border-radius: 8px; text-align: center;">
                                                <div style="font-size: 0.85em; color: #666;">"Изплатено"</div>
                                                <div style="font-size: 1.35em; font-weight: bold; color: #c2185b;">{format!("{:.2} {}", totals.paid_net, currency_r)}</div>
                                            </div>
                                        </div>

                                        <div style="background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: auto; margin-bottom: 20px;">
                                            <table style="width: 100%; border-collapse: collapse; font-size: 14px;">
                                                <thead>
                                                    <tr style="background: #f8f9fa;">
                                                        <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Дата решение"</th>
                                                        <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Протокол"</th>
                                                        <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Година"</th>
                                                        <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Собственик"</th>
                                                        <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"ЕГН"</th>
                                                        <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Брутна"</th>
                                                        <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Данък"</th>
                                                        <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Нетна"</th>
                                                        <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Плащане"</th>
                                                        <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Статус"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {if report_loading.get() {
                                                        view! {
                                                            <tr>
                                                                <td colspan="10" style="padding: 30px; text-align: center; color: #7f8c8d;">"Зареждане..."</td>
                                                            </tr>
                                                        }.into_any()
                                                    } else if rows.is_empty() {
                                                        view! {
                                                            <tr>
                                                                <td colspan="10" style="padding: 30px; text-align: center; color: #7f8c8d;">
                                                                    {if report_loaded.get() {
                                                                        "Няма дивиденти за избрания период.".to_string()
                                                                    } else {
                                                                        "Изберете период и натиснете „Покажи“.".to_string()
                                                                    }}
                                                                </td>
                                                            </tr>
                                                        }.into_any()
                                                    } else {
                                                        rows.iter().map(|r| {
                                                            view! {
                                                                <tr style="border-bottom: 1px solid #dee2e6;">
                                                                    <td style="padding: 10px 12px;">{format_bg_date(r.decision_date.as_deref().unwrap_or(""))}</td>
                                                                    <td style="padding: 10px 12px;">{r.decision_number.clone().unwrap_or_else(|| "—".to_string())}</td>
                                                                    <td style="padding: 10px 12px;">{r.year.map(|y| y.to_string()).unwrap_or_else(|| "—".to_string())}</td>
                                                                    <td style="padding: 10px 12px; font-weight: 500;">{r.owner_name.clone().unwrap_or_else(|| "—".to_string())}</td>
                                                                    <td style="padding: 10px 12px;">{r.owner_egn.clone().unwrap_or_else(|| "—".to_string())}</td>
                                                                    <td style="padding: 10px 12px; text-align: right;">{format!("{:.2}", r.gross_amount)}</td>
                                                                    <td style="padding: 10px 12px; text-align: right; color: #e67e22;">{format!("{:.2}", r.tax_amount)}</td>
                                                                    <td style="padding: 10px 12px; text-align: right; font-weight: 500; color: #27ae60;">{format!("{:.2}", r.net_amount)}</td>
                                                                    <td style="padding: 10px 12px;">
                                                                        {if r.is_paid {
                                                                            format_bg_date(r.payment_date.as_deref().unwrap_or(""))
                                                                        } else {
                                                                            "Неплатено".to_string()
                                                                        }}
                                                                    </td>
                                                                    <td style="padding: 10px 12px;">
                                                                        <span style=status_style(r.status.as_deref().unwrap_or(""))>
                                                                            {r.status_label.clone().unwrap_or_else(|| r.status.clone().unwrap_or_default())}
                                                                        </span>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view().into_any()
                                                    }}
                                                    {if !report_loading.get() && !rows.is_empty() {
                                                        view! {
                                                            <tr style="background: #f8f9fa; font-weight: bold;">
                                                                <td colspan="5" style="padding: 12px; text-align: right;">"ОБЩО:"</td>
                                                                <td style="padding: 12px; text-align: right;">{format!("{:.2}", totals.gross_amount)}</td>
                                                                <td style="padding: 12px; text-align: right; color: #e67e22;">{format!("{:.2}", totals.tax_amount)}</td>
                                                                <td style="padding: 12px; text-align: right; color: #27ae60;">{format!("{:.2}", totals.net_amount)}</td>
                                                                <td colspan="2" style="padding: 12px;">{format!("{} записа", totals.count)}</td>
                                                            </tr>
                                                        }.into_any()
                                                    } else {
                                                        view! { <tr></tr> }.into_any()
                                                    }}
                                                </tbody>
                                            </table>
                                        </div>

                                        {if !by_owner.is_empty() {
                                            view! {
                                                <div style="background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: auto;">
                                                    <h3 style="margin: 0; padding: 16px 16px 0 16px;">"Обобщение по собственик"</h3>
                                                    <table style="width: 100%; border-collapse: collapse; font-size: 14px;">
                                                        <thead>
                                                            <tr style="background: #f8f9fa;">
                                                                <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Собственик"</th>
                                                                <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"ЕГН"</th>
                                                                <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Брутна"</th>
                                                                <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Данък"</th>
                                                                <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Нетна"</th>
                                                                <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Изплатено"</th>
                                                                <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Брой"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            {by_owner.iter().map(|o| {
                                                                view! {
                                                                    <tr style="border-bottom: 1px solid #dee2e6;">
                                                                        <td style="padding: 10px 12px; font-weight: 500;">{o.owner_name.clone()}</td>
                                                                        <td style="padding: 10px 12px;">{o.owner_egn.clone().unwrap_or_else(|| "—".to_string())}</td>
                                                                        <td style="padding: 10px 12px; text-align: right;">{format!("{:.2}", o.gross_amount)}</td>
                                                                        <td style="padding: 10px 12px; text-align: right; color: #e67e22;">{format!("{:.2}", o.tax_amount)}</td>
                                                                        <td style="padding: 10px 12px; text-align: right; color: #27ae60;">{format!("{:.2}", o.net_amount)}</td>
                                                                        <td style="padding: 10px 12px; text-align: right;">{format!("{:.2}", o.paid_net)}</td>
                                                                        <td style="padding: 10px 12px; text-align: right;">{o.count}</td>
                                                                    </tr>
                                                                }
                                                            }).collect_view()}
                                                        </tbody>
                                                    </table>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                }
            }}

            {move || if show_new_modal.get() {
                let owners = beneficial_owners.get();
                let total_pct: f64 = owners.iter().filter_map(|o| o.ownership_percentage).sum();
                let has_owners = total_pct > 0.0;
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; padding: 20px;">
                        <div style="background: white; border-radius: 8px; padding: 30px; width: 100%; max-width: 520px; max-height: 90vh; overflow: auto; box-shadow: 0 4px 20px rgba(0,0,0,0.3);">
                            <h3 style="margin-top: 0;">
                                {if editing_id.get().is_some() {
                                    "Редакция на разпределение"
                                } else {
                                    "Ново разпределение на дивиденти"
                                }}
                            </h3>

                            {if !has_owners {
                                view! {
                                    <div style="background: #fff3cd; color: #856404; padding: 12px; border-radius: 4px; margin-bottom: 15px;">
                                        "Няма действителни собственици с процент за тази фирма. Добавете ги в "
                                        <a href="/settings" style="color: #856404; font-weight: 600;">"Настройки → SAFt"</a>
                                        " и отворете отново този прозорец."
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}

                            <div style="margin: 20px 0;">
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Финансова година"</label>
                                    <input
                                        type="text"
                                        inputmode="numeric"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_year.get()
                                        on:input=move |ev| new_year.set(event_target_value(&ev))
                                    />
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Обща сума за разпределяне"</label>
                                    <input
                                        type="text"
                                        inputmode="decimal"
                                        placeholder="0.00"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_total_amount.get()
                                        on:input=move |ev| new_total_amount.set(event_target_value(&ev))
                                    />
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Данък при източника (%)"</label>
                                    <input
                                        type="text"
                                        inputmode="decimal"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_tax_rate.get()
                                        on:input=move |ev| new_tax_rate.set(event_target_value(&ev))
                                    />
                                    <div style="font-size: 12px; color: #666; margin-top: 4px;">
                                        "По подразбиране от Настройки → Дивиденти. 5% за физически лица (ЗДДФЛ). 0% към българско ЮЛ."
                                    </div>
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Дата на решение (протокол)"</label>
                                    <input
                                        type="date"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_decision_date.get()
                                        on:input=move |ev| new_decision_date.set(event_target_value(&ev))
                                    />
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Номер на протокол (опционално)"</label>
                                    <input
                                        type="text"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_decision_number.get()
                                        on:input=move |ev| new_decision_number.set(event_target_value(&ev))
                                    />
                                </div>
                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Бележки (опционално)"</label>
                                    <textarea
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; min-height: 80px;"
                                        prop:value=move || new_notes.get()
                                        on:input=move |ev| new_notes.set(event_target_value(&ev))
                                    ></textarea>
                                </div>
                            </div>

                            {move || {
                                let amount = parse_decimal(&new_total_amount.get());
                                let rate = parse_decimal(&new_tax_rate.get());
                                let owners = beneficial_owners.get();
                                let total_pct: f64 = owners.iter().filter_map(|o| o.ownership_percentage).sum();
                                if amount > 0.0 && total_pct > 0.0 {
                                    view! {
                                        <div style="background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 15px;">
                                            <h5 style="margin-top: 0; margin-bottom: 10px;">"Предварителен преглед"</h5>
                                            {owners.iter().filter(|o| o.ownership_percentage.unwrap_or(0.0) > 0.0).map(|o| {
                                                let pct = o.ownership_percentage.unwrap_or(0.0);
                                                let gross = (amount * pct / total_pct * 100.0).round() / 100.0;
                                                let tax = (gross * rate / 100.0 * 100.0).round() / 100.0;
                                                let net = gross - tax;
                                                view! {
                                                    <div style="display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid #eee;">
                                                        <span>{format!("{} ({:.2}%)", o.display_name(), pct)}</span>
                                                        <span style="font-weight: 500;">{format!("{:.2} нето", net)}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }
                            }}

                            <div style="display: flex; justify-content: flex-end; gap: 10px;">
                                <button
                                    style="padding: 10px 20px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; background: white;"
                                    on:click=move |_| {
                                        show_new_modal.set(false);
                                        editing_id.set(None);
                                    }
                                >
                                    "Отказ"
                                </button>
                                <button
                                    style=move || {
                                        let disabled = creating.get() || parse_decimal(&new_total_amount.get()) <= 0.0 || !has_owners;
                                        if disabled {
                                            "padding: 10px 20px; border: none; border-radius: 4px; cursor: not-allowed; background: #95a5a6; color: white;".to_string()
                                        } else {
                                            "padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; background: #3498db; color: white;".to_string()
                                        }
                                    }
                                    on:click=create_distribution
                                    disabled=move || creating.get() || parse_decimal(&new_total_amount.get()) <= 0.0 || !has_owners
                                >
                                    {move || {
                                        if creating.get() {
                                            if editing_id.get().is_some() { "Запазване..." } else { "Създаване..." }
                                        } else if editing_id.get().is_some() {
                                            "Запази"
                                        } else {
                                            "Създай"
                                        }
                                    }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </Layout>
    }
}

fn status_style(status: &str) -> String {
    let colors = match status {
        "paid" => "background: #27ae60; color: white;",
        "partially_paid" => "background: #f39c12; color: white;",
        "approved" => "background: #3498db; color: white;",
        _ => "background: #bdc3c7;",
    };
    format!("padding: 4px 12px; border-radius: 4px; font-size: 0.9em; {}", colors)
}

fn tab_style(active: bool) -> String {
    if active {
        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; background: #3498db; color: white; border-radius: 8px 8px 0 0;".to_string()
    } else {
        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; background: #ecf0f1; color: #34495e; border-radius: 8px 8px 0 0;".to_string()
    }
}

fn parse_decimal(raw: &str) -> f64 {
    raw.trim().replace(',', ".").replace(' ', "").parse().unwrap_or(0.0)
}

fn format_tax_rate_value(rate: f64) -> String {
    if (rate - rate.round()).abs() < 0.001 {
        format!("{}", rate.round() as i32)
    } else {
        format!("{}", rate)
    }
}

fn format_amount_value(amount: f64) -> String {
    if (amount - amount.round()).abs() < 0.001 {
        format!("{}", amount.round() as i64)
    } else {
        format!("{:.2}", amount)
    }
}

fn date_input_value(s: &str) -> String {
    if s.len() >= 10 && s.as_bytes()[4] == b'-' {
        s[..10].to_string()
    } else {
        s.to_string()
    }
}

fn extract_response_error(response: &serde_json::Value) -> String {
    if let Some(msg) = response.get("message").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        return msg.to_string();
    }
    if let Some(arr) = response.get("errors").and_then(|e| e.as_array()) {
        let parts: Vec<String> = arr.iter().filter_map(|e| {
            e.get("messages")
                .and_then(|m| m.as_array())
                .and_then(|m| m.first())
                .and_then(|m| m.as_str())
                .map(|s| s.to_string())
        }).collect();
        if !parts.is_empty() {
            return parts.join(", ");
        }
    }
    "Неизвестна грешка".to_string()
}

fn get_current_year() -> i32 {
    let now = js_sys::Date::new_0();
    (now.get_full_year() - 1) as i32
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

fn year_start_date() -> String {
    let now = js_sys::Date::new_0();
    format!("{:04}-01-01", now.get_full_year())
}

fn format_bg_date(s: &str) -> String {
    if s.len() >= 10 {
        let y = &s[0..4];
        let m = &s[5..7];
        let d = &s[8..10];
        if y.chars().all(|c| c.is_ascii_digit()) && m.chars().all(|c| c.is_ascii_digit()) {
            return format!("{}.{}.{}", d, m, y);
        }
    }
    s.to_string()
}

fn month_name_bg(month: u32) -> &'static str {
    match month {
        1 => "януари",
        2 => "февруари",
        3 => "март",
        4 => "април",
        5 => "май",
        6 => "юни",
        7 => "юли",
        8 => "август",
        9 => "септември",
        10 => "октомври",
        11 => "ноември",
        12 => "декември",
        _ => "",
    }
}

fn long_bg_date(s: &str) -> String {
    if s.len() >= 10 {
        if let (Ok(y), Ok(m), Ok(d)) = (s[0..4].parse::<i32>(), s[5..7].parse::<u32>(), s[8..10].parse::<u32>()) {
            return format!("{} {} {} г.", d, month_name_bg(m), y);
        }
    }
    format_bg_date(s)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn protocol_filename(dist: &DividendDistributionDetail, ext: &str) -> String {
    let no = dist.decision_number.as_deref().unwrap_or("").trim();
    if no.is_empty() {
        format!("protokol-dividendi-{}.{}", dist.year, ext)
    } else {
        format!("protokol-dividendi-{}-{}.{}", dist.year, no.replace(' ', "-"), ext)
    }
}

fn protocol_doc_styles() -> &'static str {
    r#"
body { font-family: "Times New Roman", Times, serif; padding: 24px 36px; margin: 0; color: #111; font-size: 14px; line-height: 1.45; }
h1 { text-align: center; font-size: 18px; margin: 8px 0 4px; }
h2 { text-align: center; font-size: 16px; margin: 0 0 16px; font-weight: normal; }
.meta { text-align: center; margin-bottom: 18px; }
.draft { text-align: center; color: #c0392b; font-weight: bold; letter-spacing: 4px; border: 2px solid #c0392b; display: inline-block; padding: 2px 16px; margin: 8px auto; }
p { margin: 10px 0; text-align: justify; }
table { width: 100%; border-collapse: collapse; margin: 12px 0 16px; font-size: 13px; }
th, td { border: 1px solid #333; padding: 6px 8px; }
th { background: #eee; }
td.r, th.r { text-align: right; }
ol { margin: 8px 0 8px 22px; }
.sign { margin-top: 28px; }
.sign .name { margin: 6px 0; font-weight: bold; }
.sign .line { margin-top: 18px; }
.footer { margin-top: 36px; font-size: 12px; color: #444; }
"#
}

fn build_protocol_body(company: &CompanyBrief, dist: &DividendDistributionDetail) -> String {
    let currency = if company.currency.is_empty() { "EUR" } else { company.currency.as_str() };
    let sole_owner = dist.dividends.len() == 1;
    let title = if sole_owner {
        "РЕШЕНИЕ<br>на едноличния собственик на капитала"
    } else {
        "ПРОТОКОЛ<br>от общо събрание на съдружниците"
    };
    let actor = if sole_owner {
        "едноличният собственик на капитала"
    } else {
        "общото събрание на съдружниците"
    };
    let signer_label = if sole_owner {
        "Едноличен собственик на капитала"
    } else {
        "Съдружници"
    };

    let mut owner_lines = String::new();
    for d in &dist.dividends {
        let name = d.owner_name.as_deref().unwrap_or("—");
        let pct = d.ownership_percentage.unwrap_or(0.0);
        owner_lines.push_str(&format!(
            "<li><strong>{}</strong> — {:.2}% от капитала</li>",
            esc(name),
            pct
        ));
    }

    let mut rows = String::new();
    let mut total_tax = 0.0;
    let mut total_net = 0.0;
    for d in &dist.dividends {
        total_tax += d.tax_amount;
        total_net += d.net_amount;
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td class='r'>{:.2}%</td><td class='r'>{:.2}</td><td class='r'>{:.2}</td><td class='r'>{:.2}</td></tr>",
            esc(d.owner_name.as_deref().unwrap_or("—")),
            esc(d.owner_egn.as_deref().unwrap_or("—")),
            d.ownership_percentage.unwrap_or(0.0),
            d.gross_amount,
            d.tax_amount,
            d.net_amount
        ));
    }

    let tax_clause = if total_tax > 0.01 {
        format!(
            "От брутния дивидент се удържа данък при източника в размер на посочената ставка, общо {:.2} {}.",
            total_tax, currency
        )
    } else {
        "Данък при източника не се удържа (ставка 0%).".to_string()
    };

    let protocol_no = dist.decision_number.clone().unwrap_or_default();
    let protocol_no_line = if protocol_no.is_empty() {
        String::new()
    } else {
        format!("№ {}", esc(&protocol_no))
    };

    let city = if company.city.is_empty() { "........................".to_string() } else { esc(&company.city) };
    let address = if company.address.is_empty() { String::new() } else { format!("{}, ", esc(&company.address)) };
    let draft_banner = if dist.status == "draft" {
        "<div class='draft'>ЧЕРНОВА</div>"
    } else {
        ""
    };

    let signers = if sole_owner {
        let name = dist.dividends.first().and_then(|d| d.owner_name.clone()).unwrap_or_default();
        format!(
            "<div class='sign'><div>{}:</div><div class='name'>{}</div><div class='line'>Подпис: _______________________</div></div>",
            signer_label,
            esc(&name)
        )
    } else {
        let mut s = String::new();
        for d in &dist.dividends {
            s.push_str(&format!(
                "<div class='sign'><div class='name'>{}</div><div class='line'>Подпис: _______________________</div></div>",
                esc(d.owner_name.as_deref().unwrap_or("—"))
            ));
        }
        format!("<div>{}:</div>{}", signer_label, s)
    };

    let manager_line = if company.manager_name.is_empty() {
        String::new()
    } else {
        format!(
            "<div class='sign'><div>Управител:</div><div class='name'>{}</div><div class='line'>Подпис: _______________________</div></div>",
            esc(&company.manager_name)
        )
    };

    let notes = dist.notes.as_ref().filter(|n| !n.trim().is_empty()).map(|n| {
        format!("<p><strong>Бележки:</strong> {}</p>", esc(n))
    }).unwrap_or_default();

    format!(
        r#"{draft}
<h1>{title}</h1>
<h2>за разпределение на дивидент {proto}</h2>
<div class="meta">
  <div><strong>{cname}</strong></div>
  <div>ЕИК {eik}</div>
  <div>{address}{city}</div>
  <div>{city}, {longdate}</div>
</div>
<p>Днес, {longdate}, {actor} на <strong>{cname}</strong>, ЕИК {eik}, в състав:</p>
<ul>{owners}</ul>
<p>на основание чл. 137, ал. 1, т. 3 и чл. 147 от Търговския закон,</p>
<p style="text-align:center;font-weight:bold;">РЕШИ:</p>
<ol>
<li>Да се разпредели печалба за финансовата <strong>{year} година</strong> като дивидент в общ брутен размер на <strong>{total:.2} {cur}</strong>.</li>
<li>Разпределението е, както следва:</li>
</ol>
<table>
<thead><tr><th>Собственик</th><th>ЕГН / идентификатор</th><th class="r">Дял</th><th class="r">Брутна сума</th><th class="r">Данък</th><th class="r">Нетна сума</th></tr></thead>
<tbody>{rows}</tbody>
<tfoot><tr><th colspan="3" style="text-align:right">Общо</th><th class="r">{total:.2}</th><th class="r">{tax:.2}</th><th class="r">{net:.2}</th></tr></tfoot>
</table>
<ol start="3">
<li>{tax_clause}</li>
<li>Нетният дивидент в размер на <strong>{net:.2} {cur}</strong> да се изплати на собственика/собствениците.</li>
</ol>
{notes}
<p>Настоящият документ е съставен в два еднообразни екземпляра.</p>
{signers}
{manager}
<div class="footer">Документът е генериран от Baraba.</div>"#,
        draft = draft_banner,
        title = title,
        proto = protocol_no_line,
        cname = esc(&company.name),
        eik = esc(&if company.eik.is_empty() { "—".to_string() } else { company.eik.clone() }),
        address = address,
        city = city,
        longdate = long_bg_date(&dist.decision_date),
        actor = actor,
        owners = owner_lines,
        year = dist.year,
        total = dist.total_amount,
        cur = currency,
        rows = rows,
        tax = total_tax,
        net = total_net,
        tax_clause = tax_clause,
        notes = notes,
        signers = signers,
        manager = manager_line,
    )
}

fn js_escape_template(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("</script>", "<\\/script>")
}

fn js_escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn open_protocol_html(company: CompanyBrief, dist: DividendDistributionDetail, auto_print: bool) {
    let body = build_protocol_body(&company, &dist);
    let filename = protocol_filename(&dist, "doc");
    let word_html = word_document_html(&body);
    let print_onload = if auto_print {
        "window.onload=function(){setTimeout(function(){window.print();},250);};"
    } else {
        ""
    };

    let page = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Протокол за разпределение на дивиденти</title>
<style id="doc-style">{styles}
.toolbar {{ position: sticky; top: 0; background: #2c3e50; color: white; padding: 10px 16px; display: flex; gap: 10px; align-items: center; z-index: 10; }}
.toolbar button {{ border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; font-size: 14px; }}
.toolbar .pdf {{ background: #e74c3c; color: white; }}
.toolbar .word {{ background: #2b579a; color: white; }}
.toolbar .hint {{ font-size: 12px; color: #bdc3c7; margin-left: 8px; }}
@media print {{
  @page {{ size: A4; margin: 1.6cm; }}
  body {{ padding: 0; }}
  .toolbar {{ display: none !important; }}
  .draft {{ position: fixed; top: 40%; left: 20%; font-size: 48px; opacity: 0.12; border: none; transform: rotate(-20deg); }}
}}
</style>
</head>
<body>
<div class="toolbar no-print">
  <button class="pdf" onclick="window.print()">Печат / PDF</button>
  <button class="word" onclick="downloadWord()">Word</button>
  <span class="hint">За PDF изберете „Запазване като PDF“ в диалога за печат.</span>
</div>
<div id="doc">{body}</div>
<script>
function downloadWord(){{
  var html = "{word}";
  var blob = new Blob(['\ufeff', html], {{type:'application/msword'}});
  var a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = "{file}";
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(function(){{ URL.revokeObjectURL(a.href); }}, 500);
}}
{onload}
</script>
</body>
</html>"#,
        styles = protocol_doc_styles(),
        body = body,
        word = js_escape_string(&word_html),
        file = js_escape_string(&filename),
        onload = print_onload,
    );

    let _ = js_sys::eval(&format!(
        "var w=window.open('','_blank');if(w){{w.document.write(`{}`);w.document.close();}}else{{alert('Разрешете изскачащи прозорци, за да отворите протокола.');}}",
        js_escape_template(&page)
    ));
}

fn word_document_html(body: &str) -> String {
    format!(
        r#"<html xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:w="urn:schemas-microsoft-com:office:word" xmlns="http://www.w3.org/TR/REC-html40">
<head>
<meta charset="utf-8">
<!--[if gte mso 9]><xml><w:WordDocument><w:View>Print</w:View><w:Zoom>100</w:Zoom><w:DoNotOptimizeForBrowser/></w:WordDocument></xml><![endif]-->
<style>
@page WordSection1 {{ size: 21cm 29.7cm; margin: 2cm; }}
div.WordSection1 {{ page: WordSection1; }}
{styles}
</style>
</head>
<body>
<div class="WordSection1">
{body}
</div>
</body>
</html>"#,
        styles = protocol_doc_styles(),
        body = body
    )
}

fn date_field_label(field: &str) -> &'static str {
    if field == "payment_date" {
        "дата на плащане"
    } else {
        "дата на решение"
    }
}

fn open_dividend_report_html(
    company: CompanyBrief,
    date_from: String,
    date_to: String,
    date_field: String,
    rows: Vec<DividendReportRow>,
    totals: DividendReportTotals,
    by_owner: Vec<DividendReportOwnerTotal>,
) {
    let currency = if company.currency.is_empty() { "EUR" } else { company.currency.as_str() };
    let period = match (date_from.is_empty(), date_to.is_empty()) {
        (true, true) => "всички периоди".to_string(),
        (false, true) => format!("от {}", format_bg_date(&date_from)),
        (true, false) => format!("до {}", format_bg_date(&date_to)),
        (false, false) => format!("{} — {}", format_bg_date(&date_from), format_bg_date(&date_to)),
    };

    let mut body_rows = String::new();
    for r in &rows {
        body_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class='r'>{:.2}</td><td class='r'>{:.2}</td><td class='r'>{:.2}</td><td>{}</td></tr>",
            esc(&format_bg_date(r.decision_date.as_deref().unwrap_or(""))),
            esc(r.decision_number.as_deref().unwrap_or("—")),
            r.year.map(|y| y.to_string()).unwrap_or_else(|| "—".to_string()),
            esc(r.owner_name.as_deref().unwrap_or("—")),
            esc(r.owner_egn.as_deref().unwrap_or("—")),
            r.gross_amount,
            r.tax_amount,
            r.net_amount,
            if r.is_paid {
                esc(&format_bg_date(r.payment_date.as_deref().unwrap_or("")))
            } else {
                "Неплатено".to_string()
            }
        ));
    }

    let mut owner_rows = String::new();
    for o in &by_owner {
        owner_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td class='r'>{:.2}</td><td class='r'>{:.2}</td><td class='r'>{:.2}</td><td class='r'>{:.2}</td><td class='r'>{}</td></tr>",
            esc(&o.owner_name),
            esc(o.owner_egn.as_deref().unwrap_or("—")),
            o.gross_amount,
            o.tax_amount,
            o.net_amount,
            o.paid_net,
            o.count
        ));
    }

    let owner_table = if by_owner.is_empty() {
        String::new()
    } else {
        format!(
            r#"<h2>Обобщение по собственик</h2>
<table>
<thead><tr><th>Собственик</th><th>ЕГН</th><th class="r">Брутна</th><th class="r">Данък</th><th class="r">Нетна</th><th class="r">Изплатено</th><th class="r">Брой</th></tr></thead>
<tbody>{owner_rows}</tbody>
</table>"#
        )
    };

    let body = format!(
        r#"<h1>Справка за дивиденти</h1>
<div class="meta">
  <div><strong>{cname}</strong></div>
  <div>ЕИК {eik}</div>
  <div>Период ({field}): {period}</div>
</div>
<table>
<thead><tr><th>Дата решение</th><th>Протокол</th><th>Година</th><th>Собственик</th><th>ЕГН</th><th class="r">Брутна</th><th class="r">Данък</th><th class="r">Нетна</th><th>Плащане</th></tr></thead>
<tbody>{rows}</tbody>
<tfoot><tr><th colspan="5" style="text-align:right">Общо</th><th class="r">{gross:.2}</th><th class="r">{tax:.2}</th><th class="r">{net:.2}</th><th>{count} записа</th></tr></tfoot>
</table>
{owners}
<p style="margin-top:24px;font-size:12px;color:#555">Документът е генериран от Baraba. Суми в {cur}.</p>"#,
        cname = esc(&company.name),
        eik = esc(if company.eik.is_empty() { "—" } else { company.eik.as_str() }),
        field = date_field_label(&date_field),
        period = esc(&period),
        rows = body_rows,
        gross = totals.gross_amount,
        tax = totals.tax_amount,
        net = totals.net_amount,
        count = totals.count,
        owners = owner_table,
        cur = esc(currency),
    );

    let page = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Справка за дивиденти</title>
<style>
body {{ font-family: "Times New Roman", Times, serif; padding: 24px 36px; margin: 0; color: #111; font-size: 13px; line-height: 1.4; }}
h1 {{ text-align: center; font-size: 18px; margin: 8px 0 4px; }}
h2 {{ font-size: 15px; margin: 24px 0 8px; }}
.meta {{ text-align: center; margin-bottom: 18px; }}
table {{ width: 100%; border-collapse: collapse; margin: 12px 0 16px; font-size: 12px; }}
th, td {{ border: 1px solid #333; padding: 5px 6px; }}
th {{ background: #eee; }}
td.r, th.r {{ text-align: right; }}
.toolbar {{ position: sticky; top: 0; background: #2c3e50; color: white; padding: 10px 16px; display: flex; gap: 10px; align-items: center; z-index: 10; }}
.toolbar button {{ border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; font-size: 14px; background: #e74c3c; color: white; }}
@media print {{
  @page {{ size: A4 landscape; margin: 1.2cm; }}
  body {{ padding: 0; }}
  .toolbar {{ display: none !important; }}
}}
</style>
</head>
<body>
<div class="toolbar no-print">
  <button onclick="window.print()">Печат / PDF</button>
  <span style="font-size:12px;color:#bdc3c7">За PDF изберете „Запазване като PDF“ в диалога за печат.</span>
</div>
{body}
<script>window.onload=function(){{setTimeout(function(){{window.print();}},250);}};</script>
</body>
</html>"#,
        body = body
    );

    let _ = js_sys::eval(&format!(
        "var w=window.open('','_blank');if(w){{w.document.write(`{}`);w.document.close();}}else{{alert('Разрешете изскачащи прозорци, за да отворите справката.');}}",
        js_escape_template(&page)
    ));
}

fn download_protocol_word(company: CompanyBrief, dist: DividendDistributionDetail) {
    let body = build_protocol_body(&company, &dist);
    let html = word_document_html(&body);
    let filename = protocol_filename(&dist, "doc");
    let _ = js_sys::eval(&format!(
        r#"(function(){{
            var html = "{html}";
            var blob = new Blob(['\ufeff', html], {{type:'application/msword'}});
            var a = document.createElement('a');
            a.href = URL.createObjectURL(blob);
            a.download = "{file}";
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            setTimeout(function(){{ URL.revokeObjectURL(a.href); }}, 500);
        }})();"#,
        html = js_escape_string(&html),
        file = js_escape_string(&filename)
    ));
}
