use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete, company_api_url};

// CITA Categories for Bulgarian tax depreciation
const CITA_CATEGORIES: &[(&str, &str, f64)] = &[
    ("I", "I - Масивни сгради (4%)", 4.0),
    ("II", "II - Машини и оборудване (30%)", 30.0),
    ("III", "III - Транспортни средства (10%)", 10.0),
    ("IV", "IV - Компютри и софтуер (50%)", 50.0),
    ("V", "V - Телефони и оборудване (15%)", 15.0),
    ("VI", "VI - Дълготрайни биологични активи (15%)", 15.0),
    ("VII", "VII - Други ДМА (15%)", 15.0),
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixedAssetCategory {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub cita_category: Option<String>,
    pub min_depreciation_rate: f64,
    pub max_depreciation_rate: f64,
    pub default_method: String,
    pub asset_account_id: Option<i64>,
    pub depreciation_account_id: Option<i64>,
    pub expense_account_id: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixedAsset {
    pub id: i64,
    pub inventory_number: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub acquisition_date: String,
    pub put_into_service_date: Option<String>,
    pub acquisition_cost: f64,
    pub residual_value: Option<f64>,
    pub accounting_book_value: Option<f64>,
    pub depreciation_method: String,
    pub accounting_depreciation_rate: f64,
    pub tax_depreciation_rate: f64,
    pub status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
}

fn get_status_label(status: &str) -> &'static str {
    match status {
        "active" => "Активен",
        "disposed" => "Изведен",
        "fully_depreciated" => "Напълно амортизиран",
        _ => "Неизвестен",
    }
}

fn get_status_color(status: &str) -> &'static str {
    match status {
        "active" => "#27ae60",
        "disposed" => "#e74c3c",
        "fully_depreciated" => "#f39c12",
        _ => "#7f8c8d",
    }
}

fn get_method_label(method: &str) -> &'static str {
    match method {
        "straight_line" => "Линеен",
        "declining_balance" => "Намаляващ остатък",
        _ => "Друг",
    }
}

/// Extract "YYYY-MM-DD" from a timestamp like "2023-12-01T00:00:00Z" or pass through if already short
fn date_to_input(s: &str) -> String {
    if s.len() >= 10 {
        s[..10].to_string()
    } else {
        s.to_string()
    }
}

fn parse_decimal(val: &str) -> f64 {
    val.replace(',', ".").parse().unwrap_or(0.0)
}

fn get_report_period() -> (i32, i32) {
    let year = js_sys::eval("document.getElementById('report-year')?.value || '2026'")
        .ok()
        .and_then(|v| v.as_string())
        .and_then(|s| s.parse().ok())
        .unwrap_or(2026);
    let month = js_sys::eval("document.getElementById('report-month')?.value || '2'")
        .ok()
        .and_then(|v| v.as_string())
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    (year, month)
}

fn month_name(m: i32) -> &'static str {
    match m {
        1 => "Януари", 2 => "Февруари", 3 => "Март", 4 => "Април",
        5 => "Май", 6 => "Юни", 7 => "Юли", 8 => "Август",
        9 => "Септември", 10 => "Октомври", 11 => "Ноември", 12 => "Декември",
        _ => "",
    }
}

fn calc_useful_life_months(rate: f64) -> i32 {
    if rate <= 0.0 { return 0; }
    ((12.0 / (rate / 100.0)).ceil()) as i32
}

fn calc_depr_end_date(service_date: &str, useful_months: i32) -> String {
    let d = date_to_input(service_date);
    if d.len() < 10 { return String::new(); }
    let year: i32 = d[..4].parse().unwrap_or(0);
    let month: i32 = d[5..7].parse().unwrap_or(0);
    // depreciation starts month after service date
    let start_m = month + 1;
    let start_y = year + if start_m > 12 { 1 } else { 0 };
    let start_m = if start_m > 12 { start_m - 12 } else { start_m };
    // end date = start + useful_months - 1
    let total = (start_y * 12 + start_m - 1) + useful_months - 1;
    let end_y = total / 12;
    let end_m = total % 12 + 1;
    format!("{:02}.{:04}", end_m, end_y)
}

fn useful_life_str(months: i32) -> String {
    let y = months / 12;
    let m = months % 12;
    if m == 0 { format!("{}г.", y) }
    else if y == 0 { format!("{}м.", m) }
    else { format!("{}г. {}м.", y, m) }
}

fn calc_monthly_depr(cost: f64, residual: f64, rate: f64) -> f64 {
    let depreciable = cost - residual;
    if depreciable <= 0.0 || rate <= 0.0 { return 0.0; }
    ((cost * rate / 100.0) / 12.0 * 100.0).round() / 100.0
}

fn calc_accumulated(cost: f64, residual: f64, rate: f64, service_date: &str, target_year: i32, target_month: i32) -> f64 {
    let d = date_to_input(service_date);
    if d.len() < 10 { return 0.0; }
    let sy: i32 = d[..4].parse().unwrap_or(0);
    let sm: i32 = d[5..7].parse().unwrap_or(0);
    let start_m = sm + 1;
    let start_y = sy + if start_m > 12 { 1 } else { 0 };
    let start_m = if start_m > 12 { start_m - 12 } else { start_m };
    let months = (target_year - start_y) * 12 + (target_month - start_m) + 1;
    if months <= 0 { return 0.0; }
    let monthly = calc_monthly_depr(cost, residual, rate);
    let depreciable = cost - residual;
    let acc = (monthly * months as f64 * 100.0).round() / 100.0;
    if acc > depreciable { depreciable } else { acc }
}

fn print_depreciation_schedule(assets: Vec<FixedAsset>, categories: Vec<FixedAssetCategory>, accounts: Vec<Account>) {
    let (year, month) = get_report_period();
    let period_str = format!("{:02}-{}", month, year);

    // Build account code lookup
    let acc_map: std::collections::HashMap<i64, String> = accounts.iter().map(|a| (a.id, a.code.clone())).collect();

    let mut rows = String::new();
    let mut total_depreciable = 0.0f64;
    let mut total_accumulated = 0.0f64;
    let mut total_residual = 0.0f64;
    let mut total_year_depr = 0.0f64;

    for a in &assets {
        if a.status != "active" { continue; }
        let residual = a.residual_value.unwrap_or(0.0);
        let depreciable = a.acquisition_cost - residual;
        let cat = a.category_id
            .and_then(|cid| categories.iter().find(|c| c.id == cid));
        let cat_name = cat.map(|c| c.name.as_str()).unwrap_or("-");
        let account_code = cat
            .and_then(|c| c.asset_account_id)
            .and_then(|aid| acc_map.get(&aid))
            .map(|s| s.as_str())
            .unwrap_or("");
        let service = a.put_into_service_date.as_deref().unwrap_or("");
        let service_fmt = if service.len() >= 10 {
            format!("{}.{}.{}", &service[8..10], &service[5..7], &service[..4])
        } else { String::new() };
        let useful = calc_useful_life_months(a.accounting_depreciation_rate);
        let end_date = calc_depr_end_date(service, useful);
        let life_str = useful_life_str(useful);
        let accumulated = calc_accumulated(a.acquisition_cost, residual, a.accounting_depreciation_rate, service, year, month);
        let book_value = depreciable - accumulated;
        // Depreciation for current year (Jan to target month)
        let acc_prev_year = calc_accumulated(a.acquisition_cost, residual, a.accounting_depreciation_rate, service, year - 1, 12);
        let year_depr = accumulated - acc_prev_year;
        let year_depr = if year_depr < 0.0 { 0.0 } else { year_depr };

        total_depreciable += depreciable;
        total_accumulated += accumulated;
        total_residual += book_value;
        total_year_depr += year_depr;

        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td style='text-align:right'>{:.2}</td><td style='text-align:right'>{:.2}</td><td style='text-align:right'>{:.2}</td><td>{:.2}%</td><td style='text-align:right'>{:.2}</td><td></td><td></td><td></td></tr>",
            a.inventory_number, a.name, account_code, cat_name, service_fmt, life_str,
            depreciable, accumulated, book_value, a.accounting_depreciation_rate, year_depr
        ));
    }

    let html = format!(r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>Амортизационен план</title>
<style>
body {{ font-family: Arial, sans-serif; padding: 15px; margin: 0; font-size: 11px; }}
h2, h3 {{ text-align: center; margin: 5px 0; }}
table {{ width: 100%; border-collapse: collapse; margin-top: 10px; }}
th, td {{ border: 1px solid #333; padding: 4px 6px; }}
th {{ background: #4fc3f7; font-size: 10px; }}
tfoot td {{ font-weight: bold; background: #ecf0f1; }}
@media print {{ @page {{ size: landscape; margin: 0.8cm; }} }}
</style></head><body>
<h2>Счетоводен амортизационен план</h2>
<h3>Към месец: {}</h3>
<table>
<thead><tr>
<th>Инв.номер</th><th>Наименование</th><th>Сметка</th><th>Категории</th>
<th>Дата на въвеждане</th><th>Години Месеци</th>
<th>Амортизуема стойност</th><th>Начислени амортизации</th><th>Остатъчна стойност</th>
<th>Год. аморт. норма</th><th>Начислени за годината</th>
<th>Месец на преуст.</th><th>Месец на възст.</th><th>Месец на отписване</th>
</tr></thead>
<tbody>{}</tbody>
<tfoot><tr>
<td colspan="6" style="text-align:right">Общо:</td>
<td style="text-align:right">{:.2}</td><td style="text-align:right">{:.2}</td>
<td style="text-align:right">{:.2}</td><td></td>
<td style="text-align:right">{:.2}</td><td></td><td></td><td></td>
</tr></tfoot>
</table>
<script>window.onload=function(){{window.print();}}</script>
</body></html>"#,
        period_str, rows, total_depreciable, total_accumulated, total_residual, total_year_depr
    );

    let _ = js_sys::eval(&format!(
        "var w=window.open('','_blank');w.document.write(`{}`);w.document.close();",
        html.replace('\\', "\\\\").replace('`', "\\`")
    ));
}

fn print_depreciation_quotas(assets: Vec<FixedAsset>, categories: Vec<FixedAssetCategory>, _accounts: Vec<Account>) {
    let (report_year, _report_month) = get_report_period();

    let mut rows = String::new();
    let mut grand_total = 0.0f64;

    for a in &assets {
        if a.status != "active" { continue; }
        let service = a.put_into_service_date.as_deref().unwrap_or("");
        let service_fmt = if service.len() >= 10 {
            format!("{}.{}.{}", &service[8..10], &service[5..7], &service[..4])
        } else { String::new() };
        let residual = a.residual_value.unwrap_or(0.0);
        let depreciable = a.acquisition_cost - residual;
        let monthly = calc_monthly_depr(a.acquisition_cost, residual, a.accounting_depreciation_rate);
        let useful = calc_useful_life_months(a.accounting_depreciation_rate);

        // Determine start/end year
        let d = date_to_input(service);
        if d.len() < 10 { continue; }
        let sy: i32 = d[..4].parse().unwrap_or(0);
        let sm: i32 = d[5..7].parse().unwrap_or(0);
        let start_m = sm + 1;
        let start_y = sy + if start_m > 12 { 1 } else { 0 };
        let start_m_adj = if start_m > 12 { start_m - 12 } else { start_m };
        let end_total = (start_y * 12 + start_m_adj - 1) + useful - 1;
        let end_y = end_total / 12;

        for yr in start_y..=end_y.min(report_year) {
            let mut year_total = 0.0f64;
            let mut cells = String::new();
            for m in 1..=12 {
                let acc = calc_accumulated(a.acquisition_cost, residual, a.accounting_depreciation_rate, service, yr, m);
                let acc_prev = if m == 1 {
                    calc_accumulated(a.acquisition_cost, residual, a.accounting_depreciation_rate, service, yr - 1, 12)
                } else {
                    calc_accumulated(a.acquisition_cost, residual, a.accounting_depreciation_rate, service, yr, m - 1)
                };
                let month_val = acc - acc_prev;
                let month_val = if month_val < 0.001 { 0.0 } else { ((month_val * 100.0).round()) / 100.0 };
                year_total += month_val;
                cells.push_str(&format!("<td style='text-align:right'>{}</td>",
                    if month_val > 0.0 { format!("{:.2}", month_val) } else { "0.00".to_string() }
                ));
            }
            grand_total += year_total;
            rows.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td>{}<td style='text-align:right;font-weight:bold'>{:.2}</td></tr>",
                a.inventory_number, a.name, service_fmt, yr, cells, year_total
            ));
        }
    }

    let html = format!(r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>Амортизационни квоти</title>
<style>
body {{ font-family: Arial, sans-serif; padding: 15px; margin: 0; font-size: 10px; }}
h2 {{ text-align: center; margin: 5px 0; }}
table {{ width: 100%; border-collapse: collapse; margin-top: 10px; }}
th, td {{ border: 1px solid #333; padding: 3px 5px; }}
th {{ background: #4fc3f7; font-size: 9px; }}
@media print {{ @page {{ size: landscape; margin: 0.5cm; }} }}
</style></head><body>
<h2>Счетоводни амортизационни квоти</h2>
<table>
<thead><tr>
<th>Инв.номер</th><th>Име</th><th>Дата на въвеждане</th><th>Година</th>
<th>Яну</th><th>Фев</th><th>Мар</th><th>Апр</th><th>Май</th><th>Юни</th>
<th>Юли</th><th>Авг</th><th>Сеп</th><th>Окт</th><th>Ное</th><th>Дек</th><th>Общо</th>
</tr></thead>
<tbody>{}</tbody>
</table>
<p style="text-align:right;font-weight:bold;margin-top:10px">Общо за всички активи: {:.2}</p>
<script>window.onload=function(){{window.print();}}</script>
</body></html>"#,
        rows, grand_total
    );

    let _ = js_sys::eval(&format!(
        "var w=window.open('','_blank');w.document.write(`{}`);w.document.close();",
        html.replace('\\', "\\\\").replace('`', "\\`")
    ));
}

fn print_inventory_list(assets: Vec<FixedAsset>, categories: Vec<FixedAssetCategory>, accounts: Vec<Account>) {
    let (year, month) = get_report_period();
    let today = format!("{:02}/{:02}/{}", 17, month, year); // approximate

    // Build account code lookup
    let acc_map: std::collections::HashMap<i64, String> = accounts.iter().map(|a| (a.id, a.code.clone())).collect();

    let mut rows = String::new();
    let mut total = 0.0f64;

    for a in &assets {
        if a.status != "active" { continue; }
        let residual = a.residual_value.unwrap_or(0.0);
        let service = a.put_into_service_date.as_deref().unwrap_or("");
        let accumulated = calc_accumulated(a.acquisition_cost, residual, a.accounting_depreciation_rate, service, year, month);
        let book_value = a.acquisition_cost - accumulated;
        let book_value = if book_value < 0.0 { 0.0 } else { book_value };
        let service_fmt = if service.len() >= 10 {
            format!("{}/{}/{}", &service[8..10], &service[5..7], &service[..4])
        } else { String::new() };
        let account_code = a.category_id
            .and_then(|cid| categories.iter().find(|c| c.id == cid))
            .and_then(|c| c.asset_account_id)
            .and_then(|aid| acc_map.get(&aid))
            .map(|s| s.as_str())
            .unwrap_or("");

        total += book_value;
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td style='text-align:right'>{:.2}</td><td style='text-align:right'>{:.2}</td><td style='text-align:right'>{:.2}</td><td>{}</td><td></td><td></td><td></td><td></td></tr>",
            a.inventory_number, a.name, account_code, a.acquisition_cost, accumulated, book_value, service_fmt
        ));
    }

    let html = format!(r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>Инвентаризационен опис</title>
<style>
body {{ font-family: Arial, sans-serif; padding: 15px; margin: 0; font-size: 11px; }}
h2, h3, h4 {{ text-align: center; margin: 5px 0; }}
table {{ width: 100%; border-collapse: collapse; margin-top: 10px; }}
th, td {{ border: 1px solid #333; padding: 4px 6px; }}
th {{ background: #eee; font-size: 10px; }}
tfoot td {{ font-weight: bold; }}
.sign {{ margin-top: 40px; display: flex; justify-content: space-around; }}
.sign div {{ text-align: center; border-top: 1px solid #333; padding-top: 5px; min-width: 200px; }}
@media print {{ @page {{ size: landscape; margin: 1cm; }} }}
</style></head><body>
<h2>Инвентаризационен опис на ДМА</h2>
<h4>Към дата: {}</h4>
<h4>за всички дълготрайни активи</h4>
<table>
<thead><tr>
<th>Инв.номер</th><th>Наименование</th><th>Сметка</th>
<th>Отчетна стойност</th><th>Начислена амортизация</th><th>Балансова стойност</th>
<th>Дата на въвеждане</th><th>Налично при инвентаризация</th>
<th colspan="2">Констатирано</th><th>Забележка</th>
</tr>
<tr><th></th><th></th><th></th><th></th><th></th><th></th><th></th><th></th><th>Липса</th><th>Излишък</th><th></th></tr>
</thead>
<tbody>{}</tbody>
<tfoot><tr><td colspan="5" style="text-align:right">Общо:</td><td style="text-align:right">{:.2}</td><td colspan="5"></td></tr></tfoot>
</table>
<div class="sign">
<div>Име, презиме, фамилия</div><div>Длъжност:</div><div>Подпис:</div>
</div>
<script>window.onload=function(){{window.print();}}</script>
</body></html>"#,
        today, rows, total
    );

    let _ = js_sys::eval(&format!(
        "var w=window.open('','_blank');w.document.write(`{}`);w.document.close();",
        html.replace('\\', "\\\\").replace('`', "\\`")
    ));
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepreciationResult {
    pub asset_id: i64,
    pub inventory_number: String,
    pub name: String,
    pub acquisition_cost: f64,
    pub residual_value: f64,
    pub rate: f64,
    pub monthly_depreciation: f64,
    pub accumulated_depreciation: f64,
    pub already_posted: f64,
    pub category_name: String,
}

#[component]
pub fn FixedAssets() -> impl IntoView {
    let active_tab = create_rw_signal(0); // 0=Assets, 1=Categories, 2=Depreciation
    let assets = create_rw_signal(Vec::<FixedAsset>::new());
    let categories = create_rw_signal(Vec::<FixedAssetCategory>::new());
    let accounts = create_rw_signal(Vec::<Account>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());

    // Filters
    let search_query = create_rw_signal(String::new());
    let status_filter = create_rw_signal(String::new());
    let category_filter = create_rw_signal(0i64);

    // Modal states
    let show_asset_modal = create_rw_signal(false);
    let show_category_modal = create_rw_signal(false);
    let editing_asset = create_rw_signal(Option::<FixedAsset>::None);
    let editing_category = create_rw_signal(Option::<FixedAssetCategory>::None);

    // Asset form fields
    let asset_inventory_number = create_rw_signal(String::new());
    let asset_name = create_rw_signal(String::new());
    let asset_description = create_rw_signal(String::new());
    let asset_category_id = create_rw_signal(0i64);
    let asset_acquisition_date = create_rw_signal(String::new());
    let asset_put_into_service = create_rw_signal(String::new());
    let asset_acquisition_cost = create_rw_signal(0.0f64);
    let asset_residual_value = create_rw_signal(0.0f64);
    let asset_depreciation_method = create_rw_signal("straight_line".to_string());
    let asset_accounting_rate = create_rw_signal(0.0f64);
    let asset_tax_rate = create_rw_signal(0.0f64);

    // Category form fields
    let cat_name = create_rw_signal(String::new());
    let cat_description = create_rw_signal(String::new());
    let cat_cita = create_rw_signal(String::new());
    let cat_min_rate = create_rw_signal(0.0f64);
    let cat_max_rate = create_rw_signal(100.0f64);
    let cat_default_method = create_rw_signal("straight_line".to_string());
    let cat_asset_account = create_rw_signal(0i64);
    let cat_depreciation_account = create_rw_signal(0i64);
    let cat_expense_account = create_rw_signal(0i64);

    // Depreciation calculation
    let depreciation_year = create_rw_signal(2025i32);
    let depreciation_month = create_rw_signal(1i32);
    let depreciation_results = create_rw_signal(Vec::<DepreciationResult>::new());
    let depreciation_loading = create_rw_signal(false);
    let depreciation_success = create_rw_signal(String::new());

    // Fetch data on mount
    create_effect(move |_| {
        spawn_local(async move {
            let mut has_error = false;

            // Fetch assets
            match api_get("/api/companies/1/fixed_assets").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<FixedAsset> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        assets.set(items);
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане на активи: {}", e));
                    has_error = true;
                }
            }

            // Fetch categories
            match api_get("/api/companies/1/fixed_asset_categories").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<FixedAssetCategory> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        categories.set(items);
                    }
                }
                Err(e) if !has_error => {
                    error_msg.set(format!("Грешка при зареждане на категории: {}", e));
                }
                _ => {}
            }

            // Fetch accounts for dropdowns
            match api_get(&format!("{}/accounts?per_page=1000", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Account> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        accounts.set(items);
                    }
                }
                _ => {}
            }

            loading.set(false);
        });
    });

    // Filtered assets
    let filtered_assets = move || {
        let query = search_query.get().to_lowercase();
        let status = status_filter.get();
        let cat_id = category_filter.get();

        assets.get()
            .into_iter()
            .filter(|a| {
                if !status.is_empty() && a.status != status {
                    return false;
                }
                if cat_id > 0 && a.category_id != Some(cat_id) {
                    return false;
                }
                if !query.is_empty() {
                    let matches = a.inventory_number.to_lowercase().contains(&query)
                        || a.name.to_lowercase().contains(&query);
                    if !matches {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<_>>()
    };

    // Calculate total values
    let totals = move || {
        let accs = filtered_assets();
        let total_cost: f64 = accs.iter().map(|a| a.acquisition_cost).sum();
        let total_book: f64 = accs.iter().map(|a| a.accounting_book_value.unwrap_or(a.acquisition_cost)).sum();
        (accs.len(), total_cost, total_book)
    };

    // Get category name by id
    let get_category_name = move |cat_id: Option<i64>| {
        cat_id
            .and_then(|id| categories.get().iter().find(|c| c.id == id).map(|c| c.name.clone()))
            .unwrap_or_else(|| "-".to_string())
    };

    // Open new asset modal
    let open_new_asset = move |_| {
        editing_asset.set(None);
        asset_inventory_number.set(String::new());
        asset_name.set(String::new());
        asset_description.set(String::new());
        asset_category_id.set(0);
        asset_acquisition_date.set(String::new());
        asset_put_into_service.set(String::new());
        asset_acquisition_cost.set(0.0);
        asset_residual_value.set(0.0);
        asset_depreciation_method.set("straight_line".to_string());
        asset_accounting_rate.set(0.0);
        asset_tax_rate.set(0.0);
        show_asset_modal.set(true);
    };

    // Open edit asset modal
    let open_edit_asset = move |asset: FixedAsset| {
        editing_asset.set(Some(asset.clone()));
        asset_inventory_number.set(asset.inventory_number);
        asset_name.set(asset.name);
        asset_description.set(asset.description.unwrap_or_default());
        asset_category_id.set(asset.category_id.unwrap_or(0));
        asset_acquisition_date.set(date_to_input(&asset.acquisition_date));
        asset_put_into_service.set(date_to_input(&asset.put_into_service_date.unwrap_or_default()));
        asset_acquisition_cost.set(asset.acquisition_cost);
        asset_residual_value.set(asset.residual_value.unwrap_or(0.0));
        asset_depreciation_method.set(asset.depreciation_method);
        asset_accounting_rate.set(asset.accounting_depreciation_rate);
        asset_tax_rate.set(asset.tax_depreciation_rate);
        show_asset_modal.set(true);
    };

    // Open new category modal
    let open_new_category = move |_| {
        editing_category.set(None);
        cat_name.set(String::new());
        cat_description.set(String::new());
        cat_cita.set(String::new());
        cat_min_rate.set(0.0);
        cat_max_rate.set(100.0);
        cat_default_method.set("straight_line".to_string());
        cat_asset_account.set(0);
        cat_depreciation_account.set(0);
        cat_expense_account.set(0);
        show_category_modal.set(true);
    };

    // Open edit category modal
    let open_edit_category = move |cat: FixedAssetCategory| {
        editing_category.set(Some(cat.clone()));
        cat_name.set(cat.name);
        cat_description.set(cat.description.unwrap_or_default());
        cat_cita.set(cat.cita_category.unwrap_or_default());
        cat_min_rate.set(cat.min_depreciation_rate);
        cat_max_rate.set(cat.max_depreciation_rate);
        cat_default_method.set(cat.default_method);
        cat_asset_account.set(cat.asset_account_id.unwrap_or(0));
        cat_depreciation_account.set(cat.depreciation_account_id.unwrap_or(0));
        cat_expense_account.set(cat.expense_account_id.unwrap_or(0));
        show_category_modal.set(true);
    };

    // Save asset
    let save_asset = move |_| {
        let is_edit = editing_asset.get().is_some();
        let inv_num = asset_inventory_number.get();
        let name = asset_name.get();

        if inv_num.is_empty() || name.is_empty() {
            error_msg.set("Инвентарен номер и наименование са задължителни".to_string());
            return;
        }

        spawn_local(async move {
            let body = serde_json::json!({
                "fixed_asset": {
                    "inventory_number": asset_inventory_number.get(),
                    "name": asset_name.get(),
                    "description": if asset_description.get().is_empty() { None } else { Some(asset_description.get()) },
                    "category_id": if asset_category_id.get() == 0 { None } else { Some(asset_category_id.get()) },
                    "acquisition_date": asset_acquisition_date.get(),
                    "put_into_service_date": if asset_put_into_service.get().is_empty() { None } else { Some(asset_put_into_service.get()) },
                    "acquisition_cost": asset_acquisition_cost.get(),
                    "residual_value": asset_residual_value.get(),
                    "depreciation_method": asset_depreciation_method.get(),
                    "accounting_depreciation_rate": asset_accounting_rate.get(),
                    "tax_depreciation_rate": asset_tax_rate.get(),
                }
            });

            let result = if is_edit {
                let id = editing_asset.get().unwrap().id;
                api_put(&format!("/api/companies/1/fixed_assets/{}", id), &body).await
            } else {
                api_post("/api/companies/1/fixed_assets", &body).await
            };

            match result {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        show_asset_modal.set(false);
                        // Reload assets
                        if let Ok(data) = api_get("/api/companies/1/fixed_assets").await {
                            if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                                let items: Vec<FixedAsset> = arr.iter()
                                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                    .collect();
                                assets.set(items);
                            }
                        }
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Неизвестна грешка");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Save category
    let save_category = move |_| {
        let name = cat_name.get();

        if name.is_empty() {
            error_msg.set("Наименованието е задължително".to_string());
            return;
        }

        let is_edit = editing_category.get().is_some();

        spawn_local(async move {
            let body = serde_json::json!({
                "fixed_asset_category": {
                    "name": cat_name.get(),
                    "description": if cat_description.get().is_empty() { None } else { Some(cat_description.get()) },
                    "cita_category": if cat_cita.get().is_empty() { None } else { Some(cat_cita.get()) },
                    "min_depreciation_rate": cat_min_rate.get(),
                    "max_depreciation_rate": cat_max_rate.get(),
                    "default_method": cat_default_method.get(),
                    "asset_account_id": if cat_asset_account.get() == 0 { None } else { Some(cat_asset_account.get()) },
                    "depreciation_account_id": if cat_depreciation_account.get() == 0 { None } else { Some(cat_depreciation_account.get()) },
                    "expense_account_id": if cat_expense_account.get() == 0 { None } else { Some(cat_expense_account.get()) },
                }
            });

            let result = if is_edit {
                let id = editing_category.get().unwrap().id;
                api_put(&format!("/api/companies/1/fixed_asset_categories/{}", id), &body).await
            } else {
                api_post("/api/companies/1/fixed_asset_categories", &body).await
            };

            match result {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        show_category_modal.set(false);
                        // Reload categories
                        if let Ok(data) = api_get("/api/companies/1/fixed_asset_categories").await {
                            if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                                let items: Vec<FixedAssetCategory> = arr.iter()
                                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                    .collect();
                                categories.set(items);
                            }
                        }
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Неизвестна грешка");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Delete asset
    let delete_asset = move |id: i64| {
        spawn_local(async move {
            match api_delete(&format!("/api/companies/1/fixed_assets/{}", id)).await {
                Ok(_) => {
                    assets.update(|list| list.retain(|a| a.id != id));
                }
                Err(e) => error_msg.set(format!("Грешка при изтриване: {}", e)),
            }
        });
    };

    // Delete category
    let delete_category = move |id: i64| {
        spawn_local(async move {
            match api_delete(&format!("/api/companies/1/fixed_asset_categories/{}", id)).await {
                Ok(_) => {
                    categories.update(|list| list.retain(|c| c.id != id));
                }
                Err(e) => error_msg.set(format!("Грешка при изтриване: {}", e)),
            }
        });
    };

    // Calculate depreciation
    let calculate_depreciation = move |_| {
        depreciation_loading.set(true);
        depreciation_success.set(String::new());
        error_msg.set(String::new());
        spawn_local(async move {
            let body = serde_json::json!({
                "year": depreciation_year.get_untracked(),
                "month": depreciation_month.get_untracked(),
            });
            match api_post("/api/companies/1/fixed_assets/calculate_depreciation", &body).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<DepreciationResult> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        depreciation_results.set(items);
                    }
                }
                Err(e) => error_msg.set(format!("Грешка при изчисление: {}", e)),
            }
            depreciation_loading.set(false);
        });
    };

    let post_depreciation = move |_| {
        depreciation_loading.set(true);
        depreciation_success.set(String::new());
        error_msg.set(String::new());
        spawn_local(async move {
            let body = serde_json::json!({
                "year": depreciation_year.get_untracked(),
                "month": depreciation_month.get_untracked(),
            });
            match api_post("/api/companies/1/fixed_assets/post_depreciation", &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Осчетоводено успешно");
                        depreciation_success.set(msg.to_string());
                        depreciation_results.set(Vec::new());
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Грешка при осчетоводяване");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            depreciation_loading.set(false);
        });
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Дълготрайни материални активи"</h1>
            </div>

            // Tabs
            <div style="display: flex; gap: 0; margin-top: 20px; border-bottom: 2px solid #3498db;">
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == 0 {
                            "background: #3498db; color: white; border-radius: 4px 4px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set(0)
                >
                    "Активи"
                </button>
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == 1 {
                            "background: #3498db; color: white; border-radius: 4px 4px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set(1)
                >
                    "Категории"
                </button>
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == 2 {
                            "background: #3498db; color: white; border-radius: 4px 4px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set(2)
                >
                    "Амортизация"
                </button>
                <button
                    style=move || format!(
                        "padding: 12px 24px; border: none; cursor: pointer; font-size: 15px; font-weight: 500; transition: all 0.2s; {}",
                        if active_tab.get() == 3 {
                            "background: #3498db; color: white; border-radius: 4px 4px 0 0;"
                        } else {
                            "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                        }
                    )
                    on:click=move |_| active_tab.set(3)
                >
                    "Справки"
                </button>
            </div>

            // Error message
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 20px; display: flex; justify-content: space-between; align-items: center;">
                            <span>{err}</span>
                            <button
                                style="background: transparent; border: none; color: white; cursor: pointer; font-size: 18px;"
                                on:click=move |_| error_msg.set(String::new())
                            >"×"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Loading state
            {move || {
                if loading.get() {
                    view! {
                        <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                            <p style="color: #7f8c8d;">"Зареждане..."</p>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Tab content
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                match active_tab.get() {
                    0 => {
                        // Assets tab
                        let (count, total_cost, total_book) = totals();
                        view! {
                            <div>
                                // Filters and actions
                                <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                                    <div style="display: flex; gap: 15px; flex-wrap: wrap; align-items: center;">
                                        <div style="flex: 1; min-width: 200px;">
                                            <input
                                                type="text"
                                                placeholder="Търсене по инв. номер или наименование..."
                                                style="width: 100%; padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px;"
                                                on:input=move |ev| search_query.set(event_target_value(&ev))
                                                prop:value=move || search_query.get()
                                            />
                                        </div>
                                        <select
                                            style="padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; min-width: 150px;"
                                            on:change=move |ev| status_filter.set(event_target_value(&ev))
                                        >
                                            <option value="">"Всички статуси"</option>
                                            <option value="active">"Активни"</option>
                                            <option value="disposed">"Изведени"</option>
                                            <option value="fully_depreciated">"Амортизирани"</option>
                                        </select>
                                        <select
                                            style="padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; min-width: 150px;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                category_filter.set(val.parse().unwrap_or(0));
                                            }
                                        >
                                            <option value="0">"Всички категории"</option>
                                            {move || categories.get().into_iter().map(|c| {
                                                let id = c.id;
                                                view! {
                                                    <option value=id.to_string()>{c.name}</option>
                                                }
                                            }).collect_view()}
                                        </select>
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=open_new_asset
                                        >
                                            "+ Нов актив"
                                        </button>
                                    </div>
                                </div>

                                // Summary cards
                                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-top: 20px;">
                                    <div style="background: white; border-radius: 8px; padding: 20px; border-left: 4px solid #3498db;">
                                        <div style="color: #7f8c8d; font-size: 13px;">"Брой активи"</div>
                                        <div style="font-size: 24px; font-weight: bold; color: #2c3e50;">{count}</div>
                                    </div>
                                    <div style="background: white; border-radius: 8px; padding: 20px; border-left: 4px solid #27ae60;">
                                        <div style="color: #7f8c8d; font-size: 13px;">"Обща стойност"</div>
                                        <div style="font-size: 24px; font-weight: bold; color: #2c3e50;">
                                            {format!("{:.2} EUR", total_cost)}
                                        </div>
                                    </div>
                                    <div style="background: white; border-radius: 8px; padding: 20px; border-left: 4px solid #f39c12;">
                                        <div style="color: #7f8c8d; font-size: 13px;">"Балансова стойност"</div>
                                        <div style="font-size: 24px; font-weight: bold; color: #2c3e50;">
                                            {format!("{:.2} EUR", total_book)}
                                        </div>
                                    </div>
                                </div>

                                // Assets table
                                <div style="background: white; border-radius: 8px; margin-top: 20px; overflow: hidden;">
                                    <table style="width: 100%; border-collapse: collapse;">
                                        <thead style="background: #34495e; color: white;">
                                            <tr>
                                                <th style="padding: 12px; text-align: left; width: 100px;">"Инв. №"</th>
                                                <th style="padding: 12px; text-align: left;">"Наименование"</th>
                                                <th style="padding: 12px; text-align: left; width: 150px;">"Категория"</th>
                                                <th style="padding: 12px; text-align: right; width: 120px;">"Стойност"</th>
                                                <th style="padding: 12px; text-align: right; width: 120px;">"Баланс. ст-ст"</th>
                                                <th style="padding: 12px; text-align: center; width: 100px;">"Статус"</th>
                                                <th style="padding: 12px; text-align: center; width: 100px;">"Действия"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {move || {
                                                let accs = filtered_assets();
                                                if accs.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="7" style="padding: 40px; text-align: center; color: #7f8c8d;">
                                                                "Няма намерени активи"
                                                            </td>
                                                        </tr>
                                                    }.into_view()
                                                } else {
                                                    accs.into_iter().map(|asset| {
                                                        let asset_for_edit = asset.clone();
                                                        let asset_id = asset.id;
                                                        let cat_name = get_category_name(asset.category_id);
                                                        let status_color = get_status_color(&asset.status);
                                                        let status_label = get_status_label(&asset.status);
                                                        let book_val = asset.accounting_book_value.unwrap_or(asset.acquisition_cost);

                                                        view! {
                                                            <tr style="border-bottom: 1px solid #ecf0f1;">
                                                                <td style="padding: 12px; font-family: monospace; font-weight: bold;">
                                                                    {asset.inventory_number}
                                                                </td>
                                                                <td style="padding: 12px;">
                                                                    <div style="font-weight: 500;">{asset.name}</div>
                                                                    {asset.description.map(|d| view! {
                                                                        <div style="font-size: 12px; color: #7f8c8d;">{d}</div>
                                                                    })}
                                                                </td>
                                                                <td style="padding: 12px; color: #7f8c8d;">{cat_name}</td>
                                                                <td style="padding: 12px; text-align: right; font-family: monospace;">
                                                                    {format!("{:.2}", asset.acquisition_cost)}
                                                                </td>
                                                                <td style="padding: 12px; text-align: right; font-family: monospace;">
                                                                    {format!("{:.2}", book_val)}
                                                                </td>
                                                                <td style="padding: 12px; text-align: center;">
                                                                    <span style=format!(
                                                                        "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                                        status_color
                                                                    )>
                                                                        {status_label}
                                                                    </span>
                                                                </td>
                                                                <td style="padding: 12px; text-align: center;">
                                                                    <button
                                                                        style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-right: 5px; font-size: 12px;"
                                                                        title="Редактиране"
                                                                        on:click=move |_| open_edit_asset(asset_for_edit.clone())
                                                                    >"✏"</button>
                                                                    <button
                                                                        style="background: #e74c3c; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                                        title="Изтриване"
                                                                        on:click=move |_| delete_asset(asset_id)
                                                                    >"🗑"</button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }
                                            }}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        }.into_view()
                    }
                    1 => {
                        // Categories tab
                        view! {
                            <div>
                                <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px; display: flex; justify-content: space-between; align-items: center;">
                                    <div>
                                        <h3 style="margin: 0; color: #2c3e50;">"Категории ДМА"</h3>
                                        <p style="margin: 5px 0 0 0; color: #7f8c8d; font-size: 14px;">
                                            "Управление на категориите дълготрайни активи по ЗКПО"
                                        </p>
                                    </div>
                                    <button
                                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=open_new_category
                                    >
                                        "+ Нова категория"
                                    </button>
                                </div>

                                <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 20px; margin-top: 20px;">
                                    {move || categories.get().into_iter().map(|cat| {
                                        let cat_for_edit = cat.clone();
                                        let cat_id = cat.id;
                                        let asset_count = assets.get().iter().filter(|a| a.category_id == Some(cat.id)).count();

                                        view! {
                                            <div style="background: white; border-radius: 8px; padding: 20px; border-left: 4px solid #3498db;">
                                                <div style="display: flex; justify-content: space-between; align-items: start;">
                                                    <div>
                                                        <h4 style="margin: 0 0 5px 0; color: #2c3e50;">{cat.name.clone()}</h4>
                                                        {cat.description.clone().map(|d| view! {
                                                            <p style="margin: 0 0 10px 0; color: #7f8c8d; font-size: 13px;">{d}</p>
                                                        })}
                                                    </div>
                                                    <div style="display: flex; gap: 5px;">
                                                        <button
                                                            style="background: #3498db; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                            on:click=move |_| open_edit_category(cat_for_edit.clone())
                                                        >"✏"</button>
                                                        <button
                                                            style="background: #e74c3c; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                            on:click=move |_| delete_category(cat_id)
                                                        >"🗑"</button>
                                                    </div>
                                                </div>

                                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 15px; padding-top: 15px; border-top: 1px solid #ecf0f1;">
                                                    <div>
                                                        <div style="font-size: 12px; color: #7f8c8d;">"ЗКПО категория"</div>
                                                        <div style="font-weight: 500;">
                                                            {cat.cita_category.clone().unwrap_or_else(|| "-".to_string())}
                                                        </div>
                                                    </div>
                                                    <div>
                                                        <div style="font-size: 12px; color: #7f8c8d;">"Амортизация"</div>
                                                        <div style="font-weight: 500;">
                                                            {format!("{:.0}% - {:.0}%", cat.min_depreciation_rate, cat.max_depreciation_rate)}
                                                        </div>
                                                    </div>
                                                    <div>
                                                        <div style="font-size: 12px; color: #7f8c8d;">"Метод"</div>
                                                        <div style="font-weight: 500;">{get_method_label(&cat.default_method)}</div>
                                                    </div>
                                                    <div>
                                                        <div style="font-size: 12px; color: #7f8c8d;">"Активи"</div>
                                                        <div style="font-weight: 500;">{asset_count}</div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>

                                // CITA reference table
                                <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                                    <h4 style="margin: 0 0 15px 0; color: #2c3e50;">"Справка: Категории по ЗКПО"</h4>
                                    <table style="width: 100%; border-collapse: collapse;">
                                        <thead style="background: #ecf0f1;">
                                            <tr>
                                                <th style="padding: 10px; text-align: left;">"Категория"</th>
                                                <th style="padding: 10px; text-align: left;">"Описание"</th>
                                                <th style="padding: 10px; text-align: right;">"Макс. норма"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {CITA_CATEGORIES.iter().map(|(code, desc, rate)| {
                                                view! {
                                                    <tr style="border-bottom: 1px solid #ecf0f1;">
                                                        <td style="padding: 10px; font-weight: bold;">{*code}</td>
                                                        <td style="padding: 10px;">{*desc}</td>
                                                        <td style="padding: 10px; text-align: right;">{format!("{:.0}%", rate)}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        }.into_view()
                    }
                    2 => {
                        // Depreciation tab
                        view! {
                            <div>
                                <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                                    <h3 style="margin: 0 0 20px 0; color: #2c3e50;">"Изчисляване на амортизации"</h3>

                                    <div style="display: flex; gap: 15px; flex-wrap: wrap; align-items: end;">
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-size: 14px; color: #7f8c8d;">"Година"</label>
                                            <select
                                                style="padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; min-width: 100px;"
                                                on:change=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    depreciation_year.set(val.parse().unwrap_or(2025));
                                                }
                                            >
                                                <option value="2024">"2024"</option>
                                                <option value="2025" selected>"2025"</option>
                                                <option value="2026">"2026"</option>
                                            </select>
                                        </div>
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-size: 14px; color: #7f8c8d;">"Месец"</label>
                                            <select
                                                style="padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; min-width: 120px;"
                                                on:change=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    depreciation_month.set(val.parse().unwrap_or(1));
                                                }
                                            >
                                                <option value="1">"Януари"</option>
                                                <option value="2">"Февруари"</option>
                                                <option value="3">"Март"</option>
                                                <option value="4">"Април"</option>
                                                <option value="5">"Май"</option>
                                                <option value="6">"Юни"</option>
                                                <option value="7">"Юли"</option>
                                                <option value="8">"Август"</option>
                                                <option value="9">"Септември"</option>
                                                <option value="10">"Октомври"</option>
                                                <option value="11">"Ноември"</option>
                                                <option value="12">"Декември"</option>
                                            </select>
                                        </div>
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            on:click=calculate_depreciation
                                        >
                                            "Изчисли"
                                        </button>
                                        <button
                                            style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                            disabled=move || depreciation_results.get().is_empty() || depreciation_loading.get()
                                            on:click=post_depreciation
                                        >
                                            "Осчетоводи"
                                        </button>
                                    </div>
                                </div>

                                // Success message
                                {move || {
                                    let msg = depreciation_success.get();
                                    if msg.is_empty() {
                                        view! { <span></span> }.into_view()
                                    } else {
                                        view! {
                                            <div style="background: #d4edda; color: #155724; border: 1px solid #c3e6cb; border-radius: 4px; padding: 12px 20px; margin-top: 15px;">
                                                {msg}
                                            </div>
                                        }.into_view()
                                    }
                                }}

                                // Results table
                                {move || {
                                    let results = depreciation_results.get();
                                    if results.is_empty() {
                                        view! {
                                            <div style="background: white; border-radius: 8px; margin-top: 20px; padding: 40px; text-align: center;">
                                                <p style="color: #7f8c8d;">"Изберете период и натиснете \"Изчисли\" за да видите амортизациите"</p>
                                            </div>
                                        }.into_view()
                                    } else {
                                        let total_monthly: f64 = results.iter().map(|r| r.monthly_depreciation).sum();
                                        let total_accumulated: f64 = results.iter().map(|r| r.accumulated_depreciation).sum();
                                        let total_posted: f64 = results.iter().map(|r| r.already_posted).sum();

                                        view! {
                                            <div style="background: white; border-radius: 8px; margin-top: 20px; overflow: hidden;">
                                                <div style="padding: 15px 20px; background: #ecf0f1; border-bottom: 1px solid #ddd;">
                                                    <strong>"Резултати за "{move || depreciation_month.get()}"/" {move || depreciation_year.get()}</strong>
                                                </div>
                                                <table style="width: 100%; border-collapse: collapse;">
                                                    <thead style="background: #34495e; color: white;">
                                                        <tr>
                                                            <th style="padding: 12px; text-align: left;">"Инв. №"</th>
                                                            <th style="padding: 12px; text-align: left;">"Наименование"</th>
                                                            <th style="padding: 12px; text-align: right;">"Стойност"</th>
                                                            <th style="padding: 12px; text-align: right;">"Норма %"</th>
                                                            <th style="padding: 12px; text-align: right;">"Месечна амортизация"</th>
                                                            <th style="padding: 12px; text-align: right;">"Натрупана"</th>
                                                            <th style="padding: 12px; text-align: right;">"Осчетоводена"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {results.into_iter().map(|r| {
                                                            view! {
                                                                <tr style="border-bottom: 1px solid #ecf0f1;">
                                                                    <td style="padding: 12px; font-family: monospace;">{r.inventory_number}</td>
                                                                    <td style="padding: 12px;">{r.name}</td>
                                                                    <td style="padding: 12px; text-align: right; font-family: monospace;">
                                                                        {format!("{:.2}", r.acquisition_cost)}
                                                                    </td>
                                                                    <td style="padding: 12px; text-align: right;">
                                                                        {format!("{:.1}%", r.rate)}
                                                                    </td>
                                                                    <td style="padding: 12px; text-align: right; font-family: monospace; font-weight: bold; color: #e74c3c;">
                                                                        {format!("{:.2}", r.monthly_depreciation)}
                                                                    </td>
                                                                    <td style="padding: 12px; text-align: right; font-family: monospace;">
                                                                        {format!("{:.2}", r.accumulated_depreciation)}
                                                                    </td>
                                                                    <td style="padding: 12px; text-align: right; font-family: monospace; color: #27ae60;">
                                                                        {format!("{:.2}", r.already_posted)}
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                    <tfoot style="background: #ecf0f1; font-weight: bold;">
                                                        <tr>
                                                            <td colspan="4" style="padding: 12px; text-align: right;">"ОБЩО:"</td>
                                                            <td style="padding: 12px; text-align: right; font-family: monospace; color: #e74c3c;">
                                                                {format!("{:.2}", total_monthly)}
                                                            </td>
                                                            <td style="padding: 12px; text-align: right; font-family: monospace;">
                                                                {format!("{:.2}", total_accumulated)}
                                                            </td>
                                                            <td style="padding: 12px; text-align: right; font-family: monospace; color: #27ae60;">
                                                                {format!("{:.2}", total_posted)}
                                                            </td>
                                                        </tr>
                                                    </tfoot>
                                                </table>
                                            </div>
                                        }.into_view()
                                    }
                                }}
                            </div>
                        }.into_view()
                    }
                    3 => {
                        // Reports tab
                        view! {
                            <div>
                                <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                                    <h3 style="margin: 0 0 20px 0; color: #2c3e50;">"Справки за ДМА"</h3>
                                    <div style="display: flex; gap: 15px; flex-wrap: wrap; align-items: end; margin-bottom: 20px;">
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-size: 14px; color: #7f8c8d;">"Към месец"</label>
                                            <select
                                                id="report-month"
                                                style="padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; min-width: 120px;"
                                            >
                                                <option value="1">"Януари"</option>
                                                <option value="2">"Февруари"</option>
                                                <option value="3">"Март"</option>
                                                <option value="4">"Април"</option>
                                                <option value="5">"Май"</option>
                                                <option value="6">"Юни"</option>
                                                <option value="7">"Юли"</option>
                                                <option value="8">"Август"</option>
                                                <option value="9">"Септември"</option>
                                                <option value="10">"Октомври"</option>
                                                <option value="11">"Ноември"</option>
                                                <option value="12">"Декември"</option>
                                            </select>
                                        </div>
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-size: 14px; color: #7f8c8d;">"Година"</label>
                                            <select
                                                id="report-year"
                                                style="padding: 10px 15px; border: 1px solid #ddd; border-radius: 4px; min-width: 100px;"
                                            >
                                                <option value="2024">"2024"</option>
                                                <option value="2025">"2025"</option>
                                                <option value="2026" selected>"2026"</option>
                                            </select>
                                        </div>
                                    </div>
                                    <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 15px;">
                                        <div
                                            style="background: #f8f9fa; border: 1px solid #ddd; border-radius: 8px; padding: 20px; cursor: pointer; transition: all 0.2s;"
                                            on:click=move |_| {
                                                print_depreciation_schedule(assets.get_untracked(), categories.get_untracked(), accounts.get_untracked());
                                            }
                                        >
                                            <h4 style="margin: 0 0 10px 0; color: #2c3e50;">"Амортизационен план"</h4>
                                            <p style="margin: 0; color: #7f8c8d; font-size: 13px;">"Счетоводен амортизационен план с натрупани амортизации и остатъчна стойност"</p>
                                        </div>
                                        <div
                                            style="background: #f8f9fa; border: 1px solid #ddd; border-radius: 8px; padding: 20px; cursor: pointer; transition: all 0.2s;"
                                            on:click=move |_| {
                                                print_depreciation_quotas(assets.get_untracked(), categories.get_untracked(), accounts.get_untracked());
                                            }
                                        >
                                            <h4 style="margin: 0 0 10px 0; color: #2c3e50;">"Амортизационни квоти"</h4>
                                            <p style="margin: 0; color: #7f8c8d; font-size: 13px;">"Месечни амортизационни квоти по години за всеки актив"</p>
                                        </div>
                                        <div
                                            style="background: #f8f9fa; border: 1px solid #ddd; border-radius: 8px; padding: 20px; cursor: pointer; transition: all 0.2s;"
                                            on:click=move |_| {
                                                print_inventory_list(assets.get_untracked(), categories.get_untracked(), accounts.get_untracked());
                                            }
                                        >
                                            <h4 style="margin: 0 0 10px 0; color: #2c3e50;">"Инвентаризационен опис"</h4>
                                            <p style="margin: 0; color: #7f8c8d; font-size: 13px;">"Опис на ДМА с отчетна стойност, амортизации и балансова стойност"</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_view()
                    }
                    _ => view! { <span></span> }.into_view()
                }
            }}

            // Asset Modal
            {move || {
                if show_asset_modal.get() {
                    let is_edit = editing_asset.get().is_some();
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 600px; max-height: 90vh; overflow-y: auto;">
                                <h2 style="margin: 0 0 20px 0;">
                                    {if is_edit { "Редактиране на актив" } else { "Нов актив" }}
                                </h2>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Инвентарен номер *"</label>
                                        <input
                                            type="text"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:input=move |ev| asset_inventory_number.set(event_target_value(&ev))
                                            prop:value=move || asset_inventory_number.get()
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Категория"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                asset_category_id.set(val.parse().unwrap_or(0));
                                            }
                                        >
                                            <option value="0">"-- Избери --"</option>
                                            {move || categories.get().into_iter().map(|c| {
                                                let id = c.id;
                                                let selected = asset_category_id.get() == id;
                                                view! {
                                                    <option value=id.to_string() selected=selected>{c.name}</option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Наименование *"</label>
                                        <input
                                            type="text"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:input=move |ev| asset_name.set(event_target_value(&ev))
                                            prop:value=move || asset_name.get()
                                        />
                                    </div>
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Описание"</label>
                                        <textarea
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; min-height: 60px; box-sizing: border-box;"
                                            on:input=move |ev| asset_description.set(event_target_value(&ev))
                                            prop:value=move || asset_description.get()
                                        ></textarea>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Дата на придобиване"</label>
                                        <input
                                            type="date"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:input=move |ev| asset_acquisition_date.set(event_target_value(&ev))
                                            prop:value=move || asset_acquisition_date.get()
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Въведен в експлоатация"</label>
                                        <input
                                            type="date"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:input=move |ev| asset_put_into_service.set(event_target_value(&ev))
                                            prop:value=move || asset_put_into_service.get()
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Стойност на придобиване"</label>
                                        <input
                                            type="text"
                                            inputmode="decimal"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                asset_acquisition_cost.set(parse_decimal(&val));
                                            }
                                            prop:value=move || {
                                                let v = asset_acquisition_cost.get();
                                                if v == 0.0 { String::new() } else { format!("{:.2}", v) }
                                            }
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Остатъчна стойност"</label>
                                        <input
                                            type="text"
                                            inputmode="decimal"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                asset_residual_value.set(parse_decimal(&val));
                                            }
                                            prop:value=move || {
                                                let v = asset_residual_value.get();
                                                if v == 0.0 { String::new() } else { format!("{:.2}", v) }
                                            }
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Метод на амортизация"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| asset_depreciation_method.set(event_target_value(&ev))
                                        >
                                            <option value="straight_line" selected=move || asset_depreciation_method.get() == "straight_line">"Линеен"</option>
                                            <option value="declining_balance" selected=move || asset_depreciation_method.get() == "declining_balance">"Намаляващ остатък"</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Счетоводна норма %"</label>
                                        <input
                                            type="text"
                                            inputmode="decimal"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                asset_accounting_rate.set(parse_decimal(&val));
                                            }
                                            prop:value=move || {
                                                let v = asset_accounting_rate.get();
                                                if v == 0.0 { String::new() } else { v.to_string() }
                                            }
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Данъчна норма %"</label>
                                        <input
                                            type="text"
                                            inputmode="decimal"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                asset_tax_rate.set(parse_decimal(&val));
                                            }
                                            prop:value=move || {
                                                let v = asset_tax_rate.get();
                                                if v == 0.0 { String::new() } else { v.to_string() }
                                            }
                                        />
                                    </div>
                                </div>

                                <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 25px; padding-top: 20px; border-top: 1px solid #ecf0f1;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_asset_modal.set(false)
                                    >"Отказ"</button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=save_asset
                                    >"Запази"</button>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Category Modal
            {move || {
                if show_category_modal.get() {
                    let is_edit = editing_category.get().is_some();
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 550px; max-height: 90vh; overflow-y: auto;">
                                <h2 style="margin: 0 0 20px 0;">
                                    {if is_edit { "Редактиране на категория" } else { "Нова категория" }}
                                </h2>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Наименование *"</label>
                                        <input
                                            type="text"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:input=move |ev| cat_name.set(event_target_value(&ev))
                                            prop:value=move || cat_name.get()
                                        />
                                    </div>
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Описание"</label>
                                        <textarea
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; min-height: 60px; box-sizing: border-box;"
                                            on:input=move |ev| cat_description.set(event_target_value(&ev))
                                            prop:value=move || cat_description.get()
                                        ></textarea>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"ЗКПО категория"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| cat_cita.set(event_target_value(&ev))
                                        >
                                            <option value="">"-- Избери --"</option>
                                            {CITA_CATEGORIES.iter().map(|(code, desc, _)| {
                                                let code_str = code.to_string();
                                                let code_val = code.to_string();
                                                view! {
                                                    <option value=code_val selected=move || cat_cita.get() == code_str>{*desc}</option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Метод по подразбиране"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| cat_default_method.set(event_target_value(&ev))
                                        >
                                            <option value="straight_line" selected=move || cat_default_method.get() == "straight_line">"Линеен"</option>
                                            <option value="declining_balance" selected=move || cat_default_method.get() == "declining_balance">"Намаляващ остатък"</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Мин. норма %"</label>
                                        <input
                                            type="text"
                                            inputmode="decimal"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                cat_min_rate.set(parse_decimal(&val));
                                            }
                                            prop:value=move || {
                                                let v = cat_min_rate.get();
                                                if v == 0.0 { String::new() } else { v.to_string() }
                                            }
                                        />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Макс. норма %"</label>
                                        <input
                                            type="text"
                                            inputmode="decimal"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                cat_max_rate.set(parse_decimal(&val));
                                            }
                                            prop:value=move || {
                                                let v = cat_max_rate.get();
                                                if v == 0.0 { String::new() } else { v.to_string() }
                                            }
                                        />
                                    </div>

                                    <div style="grid-column: span 2; margin-top: 10px;">
                                        <h4 style="margin: 0 0 10px 0; color: #7f8c8d;">"Свързани сметки"</h4>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Сметка актив"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                cat_asset_account.set(val.parse().unwrap_or(0));
                                            }
                                        >
                                            <option value="0">"-- Избери --"</option>
                                            {move || accounts.get().into_iter().map(|acc| {
                                                let id = acc.id;
                                                view! {
                                                    <option value=id.to_string() selected=move || cat_asset_account.get() == id>
                                                        {format!("{} - {}", acc.code, acc.name)}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Сметка амортизации"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                cat_depreciation_account.set(val.parse().unwrap_or(0));
                                            }
                                        >
                                            <option value="0">"-- Избери --"</option>
                                            {move || accounts.get().into_iter().map(|acc| {
                                                let id = acc.id;
                                                view! {
                                                    <option value=id.to_string() selected=move || cat_depreciation_account.get() == id>
                                                        {format!("{} - {}", acc.code, acc.name)}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                    <div style="grid-column: span 2;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Сметка разходи"</label>
                                        <select
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                cat_expense_account.set(val.parse().unwrap_or(0));
                                            }
                                        >
                                            <option value="0">"-- Избери --"</option>
                                            {move || accounts.get().into_iter().map(|acc| {
                                                let id = acc.id;
                                                view! {
                                                    <option value=id.to_string() selected=move || cat_expense_account.get() == id>
                                                        {format!("{} - {}", acc.code, acc.name)}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </div>
                                </div>

                                <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 25px; padding-top: 20px; border-top: 1px solid #ecf0f1;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_category_modal.set(false)
                                    >"Отказ"</button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=save_category
                                    >"Запази"</button>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
        </Layout>
    }
}
