use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::context::use_company_context;
use crate::api::{api_get, api_get_text, api_post, api_put, api_delete, company_api_url, download_text_file};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvoiceRow {
    pub id: i64,
    pub invoice_number: Option<String>,
    pub document_type: Option<String>,
    #[serde(default)]
    pub nap_document_type: Option<String>,
    pub direction: Option<String>,
    pub issue_date: Option<String>,
    pub counterpart_id: Option<i64>,
    pub subtotal: Option<f64>,
    pub vat_amount: Option<f64>,
    pub total_amount: Option<f64>,
    pub currency: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub counterpart_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationOpt {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub holds_stock: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CounterpartOpt {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub eik: Option<String>,
    #[serde(default)]
    pub vat_number: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductOpt {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub price: Option<f64>,
    pub measure_unit: Option<String>,
    pub uom_standard: Option<String>,
    #[serde(default)]
    pub goods_services_id: Option<String>,
    #[serde(default)]
    pub product_type: Option<String>,
    #[serde(default)]
    pub inventory_account_id: Option<i64>,
    #[serde(default)]
    pub revenue_account_id: Option<i64>,
    #[serde(default)]
    pub expense_account_id: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AccountOpt {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurrencyOpt {
    pub id: i64,
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub name_bg: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VatRateOpt {
    pub id: i64,
    pub name: String,
    pub percentage: f64,
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VatExemptionOpt {
    pub id: i64,
    pub reason_code: String,
    pub reason_name: String,
    #[serde(default)]
    pub legal_basis: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BankAccountOpt {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub bank_name: Option<String>,
    #[serde(default)]
    pub iban: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnitOpt {
    pub code: String,
    #[serde(default)]
    pub name_bg: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
}

#[derive(Debug, Clone)]
struct LineDraft {
    id: u64,
    description: String,
    quantity: String,
    unit_price: String,
    vat_rate: String,
    product_id: Option<i64>,
    unit_code: String,
    vat_exemption_reason_id: Option<i64>,
    account_id: Option<i64>,
    product_query: String,
}

fn today_iso() -> String {
    let d = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", d.get_full_year() as i32, d.get_month() as i32 + 1, d.get_date() as i32)
}

fn add_days_iso(iso: &str, days: i32) -> String {
    let v = wasm_bindgen::JsValue::from_str(iso);
    let d = js_sys::Date::new(&v);
    let day = (d.get_date() as i32) + days;
    d.set_date(day as u32);
    format!("{:04}-{:02}-{:02}", d.get_full_year() as i32, d.get_month() as i32 + 1, d.get_date() as i32)
}

fn parse_f(s: &str) -> f64 {
    s.replace(',', ".").replace(' ', "").parse().unwrap_or(0.0)
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn line_net(l: &LineDraft) -> f64 {
    round2(parse_f(&l.quantity) * parse_f(&l.unit_price))
}

fn line_vat(l: &LineDraft) -> f64 {
    round2(line_net(l) * parse_f(&l.vat_rate) / 100.0)
}

fn unique_vat_percents(rates: &[VatRateOpt]) -> Vec<f64> {
    let mut v: Vec<f64> = rates.iter().map(|r| r.percentage).collect();
    if v.is_empty() {
        v = vec![20.0, 9.0, 0.0];
    }
    v.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    v.dedup_by(|a, b| (*a - *b).abs() < 0.001);
    v
}

fn unique_banks(accounts: &[BankAccountOpt]) -> Vec<String> {
    let mut names: Vec<String> = accounts
        .iter()
        .map(|a| {
            let n = a.bank_name.clone().unwrap_or_default();
            if n.trim().is_empty() { "Други сметки".into() } else { n }
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

fn bank_key(a: &BankAccountOpt) -> String {
    let n = a.bank_name.clone().unwrap_or_default();
    if n.trim().is_empty() { "Други сметки".into() } else { n }
}

fn counterpart_label(c: &CounterpartOpt) -> String {
    let id = c.eik.clone().filter(|s| !s.trim().is_empty())
        .or_else(|| c.vat_number.clone().filter(|s| !s.trim().is_empty()));
    match id {
        Some(e) => format!("{} — {}", e, c.name),
        None => c.name.clone(),
    }
}

fn counterpart_matches(c: &CounterpartOpt, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let q = q.to_lowercase();
    c.name.to_lowercase().contains(&q)
        || c.eik.clone().unwrap_or_default().to_lowercase().contains(&q)
        || c.vat_number.clone().unwrap_or_default().to_lowercase().contains(&q)
}

fn stored_eik(eik: &str, vat: &str) -> String {
    let from_eik = crate::constants::strip_eu_vat_prefix(eik);
    if from_eik.is_empty() {
        crate::constants::strip_eu_vat_prefix(vat)
    } else {
        from_eik
    }
}

fn find_counterpart_by_eik(list: &[CounterpartOpt], eik: &str) -> Option<CounterpartOpt> {
    if eik.is_empty() || crate::constants::is_generic_eik(eik) {
        return None;
    }
    list.iter().find(|c| {
        c.eik.as_ref()
            .map(|stored| crate::constants::strip_eu_vat_prefix(stored) == eik)
            .unwrap_or(false)
    }).cloned()
}

fn product_matches(p: &ProductOpt, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let q = q.to_lowercase();
    p.code.to_lowercase().contains(&q) || p.name.to_lowercase().contains(&q)
}

fn api_error_text(response: &serde_json::Value) -> String {
    let from_fields = response.get("errors")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.get("messages").and_then(|m| m.as_array()))
                .flatten()
                .filter_map(|m| m.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    if !from_fields.is_empty() {
        return from_fields;
    }
    response.get("error").and_then(|e| e.as_str()).unwrap_or("Неизвестна грешка").to_string()
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ViesHit {
    valid: bool,
    #[serde(default)]
    name: String,
    #[serde(default)]
    address: String,
    #[serde(default)]
    country_code: String,
    #[serde(default)]
    vat_number: String,
    #[serde(default)]
    eik: String,
}

fn country_choices() -> Vec<(&'static str, &'static str)> {
    vec![
        ("BG", "България"),
        ("DE", "Германия"),
        ("AT", "Австрия"),
        ("BE", "Белгия"),
        ("FR", "Франция"),
        ("IT", "Италия"),
        ("ES", "Испания"),
        ("NL", "Нидерландия"),
        ("PL", "Полша"),
        ("RO", "Румъния"),
        ("GR", "Гърция"),
        ("CZ", "Чехия"),
        ("HU", "Унгария"),
        ("PT", "Португалия"),
        ("SE", "Швеция"),
        ("HR", "Хърватия"),
        ("SK", "Словакия"),
        ("SI", "Словения"),
        ("XX", "Извън ЕС"),
    ]
}

fn nomenclature_account(p: &ProductOpt, direction: &str) -> Option<i64> {
    if direction == "purchase" {
        if product_holds_stock(p) {
            p.inventory_account_id.or(p.expense_account_id)
        } else {
            p.expense_account_id
        }
    } else {
        p.revenue_account_id
    }
}

fn account_choices(accounts: &[AccountOpt], direction: &str, selected: Option<i64>) -> Vec<AccountOpt> {
    accounts.iter().filter(|a| {
        if selected == Some(a.id) {
            return true;
        }
        let code = a.code.as_str();
        if direction == "purchase" {
            code.starts_with('6') || code.starts_with('3')
        } else {
            code.starts_with('7')
        }
    }).cloned().collect()
}

fn nap_code_of(document_type: &str, nap: Option<&str>) -> String {
    if document_type == "PROFORMA" {
        return "PROFORMA".into();
    }
    if let Some(code) = nap.map(str::trim).filter(|s| !s.is_empty()) {
        return code.to_string();
    }
    match document_type {
        "DEBIT_NOTE" => "02",
        "CREDIT_NOTE" => "03",
        "PROTOCOL" => "09",
        "SALES_REPORT" => "81",
        _ => "01",
    }.into()
}

fn days_between(from: &str, to: &str) -> Option<i32> {
    let a = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(from));
    let b = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(to));
    if a.get_time().is_nan() || b.get_time().is_nan() {
        return None;
    }
    Some(((b.get_time() - a.get_time()) / 86_400_000.0).round() as i32)
}

fn product_holds_stock(p: &ProductOpt) -> bool {
    if p.goods_services_id.as_deref() == Some("02") {
        return false;
    }
    let t = p.product_type.as_deref().unwrap_or("");
    if !["10", "20", "30", "40"].contains(&t) {
        return false;
    }
    p.inventory_account_id.is_some()
}

fn sm_reason_id(reasons: &[VatExemptionOpt]) -> Option<i64> {
    reasons.iter().find(|r| r.reason_code == "VATEX-EU-SM").map(|r| r.id)
}

fn input_cls() -> &'static str {
    "w-full rounded-lg border border-gray-200 bg-white px-3 py-2 text-sm text-gray-800 shadow-sm placeholder-gray-400 focus:border-slate-400 focus:ring-1 focus:ring-slate-400"
}

fn chip_on() -> &'static str {
    "px-3 py-1.5 rounded-full text-sm border border-slate-600 bg-slate-700 text-white"
}

fn chip_off() -> &'static str {
    "px-3 py-1.5 rounded-full text-sm border border-gray-200 bg-white text-gray-600 hover:bg-gray-50"
}

fn invoice_selection_valid(direction: &str, code: &str) -> bool {
    if code == "PROFORMA" {
        return true;
    }
    let kind = if direction == "purchase" { "PURCHASE" } else { "SALE" };
    crate::constants::doc_types_for(kind).iter().any(|entry| entry.code == code)
}

fn invoice_type_label(inv: &InvoiceRow) -> String {
    if inv.document_type.as_deref() == Some("PROFORMA") {
        return "Проформа".into();
    }
    let journal = if inv.direction.as_deref() == Some("purchase") { "purchase" } else { "sales" };
    if let Some(code) = inv.nap_document_type.as_ref().filter(|s| !s.is_empty()) {
        if let Some(name) = crate::constants::get_doc_type_name(code, journal) {
            return format!("{} — {}", code, name);
        }
        return code.clone();
    }
    match inv.document_type.as_deref() {
        Some("DEBIT_NOTE") => "02 — Дебитно известие".into(),
        Some("CREDIT_NOTE") => "03 — Кредитно известие".into(),
        Some("PROTOCOL") => "09 — Протокол или друг документ".into(),
        Some("SALES_REPORT") => "81 — Отчет за извършените продажби".into(),
        Some("INVOICE") | None => "01 — Фактура".into(),
        Some(other) => other.into(),
    }
}

#[component]
pub fn Invoices() -> impl IntoView {
    let ctx = use_company_context();
    let invoices = RwSignal::new(Vec::<InvoiceRow>::new());
    let counterparts = RwSignal::new(Vec::<CounterpartOpt>::new());
    let products = RwSignal::new(Vec::<ProductOpt>::new());
    let locations = RwSignal::new(Vec::<LocationOpt>::new());
    let currencies = RwSignal::new(Vec::<CurrencyOpt>::new());
    let vat_rates = RwSignal::new(Vec::<VatRateOpt>::new());
    let exemptions = RwSignal::new(Vec::<VatExemptionOpt>::new());
    let bank_accounts = RwSignal::new(Vec::<BankAccountOpt>::new());
    let units = RwSignal::new(Vec::<UnitOpt>::new());
    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let direction = RwSignal::new(String::from("sale"));
    let show_form = RwSignal::new(false);
    let saving = RwSignal::new(false);
    let posting_id = RwSignal::new(Option::<i64>::None);
    let editing_id = RwSignal::new(Option::<i64>::None);
    let form_existing_number = RwSignal::new(String::new());
    let gl_accounts = RwSignal::new(Vec::<AccountOpt>::new());
    let form_error = RwSignal::new(String::new());

    let company_currency = RwSignal::new(String::from("EUR"));
    let vat_registered = RwSignal::new(true);
    let counterpart_filter = RwSignal::new(String::new());
    let show_cp_results = RwSignal::new(false);
    let show_new_counterpart = RwSignal::new(false);
    let cp_saving = RwSignal::new(false);
    let cp_form_error = RwSignal::new(String::new());
    let cp_name = RwSignal::new(String::new());
    let cp_eik = RwSignal::new(String::new());
    let cp_vat = RwSignal::new(String::new());
    let cp_address = RwSignal::new(String::new());
    let cp_city = RwSignal::new(String::new());
    let cp_country = RwSignal::new(String::from("BG"));
    let cp_type = RwSignal::new(String::from("both"));
    let cp_vies_loading = RwSignal::new(false);
    let cp_vies_msg = RwSignal::new(String::new());
    let cp_vies_ok = RwSignal::new(false);
    let product_picker = RwSignal::new(Option::<u64>::None);

    let form_counterpart = RwSignal::new(String::new());
    let form_date = RwSignal::new(String::new());
    let form_tax_date = RwSignal::new(String::new());
    let form_due_date = RwSignal::new(String::new());
    let form_doc_type = RwSignal::new(String::from("01"));
    let form_purchase_number = RwSignal::new(String::new());
    let form_notes = RwSignal::new(String::new());
    let form_location = RwSignal::new(String::new());
    let form_payment = RwSignal::new(String::from("BANK"));
    let form_bank_name = RwSignal::new(String::new());
    let form_bank_account = RwSignal::new(String::new());
    let form_currency = RwSignal::new(String::from("EUR"));
    let form_exchange_rate = RwSignal::new(1.0f64);
    let form_ecb_rate = RwSignal::new(1.0f64);
    let form_rate_date = RwSignal::new(String::new());
    let form_rate_source = RwSignal::new(String::from("identity"));
    let form_rate_found = RwSignal::new(true);
    let form_payment_terms = RwSignal::new(15i32);
    let form_original = RwSignal::new(String::new());
    let next_line_id = RwSignal::new(1u64);
    let preview = RwSignal::new(String::new());
    let lines = RwSignal::new(vec![LineDraft {
        id: 0,
        description: String::new(),
        quantity: "1".into(),
        unit_price: "0".into(),
        vat_rate: "20".into(),
        product_id: None,
        unit_code: "C62".into(),
        vat_exemption_reason_id: None,
        account_id: None,
        product_query: String::new(),
    }]);

    let reload = move || {
        loading.set(true);
        let dir = direction.get();
        spawn_local(async move {
            match api_get(&format!("{}/invoices?direction={}", company_api_url(), dir)).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let rows: Vec<InvoiceRow> = arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect();
                        invoices.set(rows);
                    }
                    loading.set(false);
                }
                Err(e) => { error_msg.set(e); loading.set(false); }
            }
        });
    };

    let load_refs = move || {
        spawn_local(async move {
            if let Ok(data) = api_get(&format!("{}/counterparts", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    counterparts.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get(&format!("{}/products", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    products.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get(&format!("{}/accounts?per_page=1000", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    let accs = arr.iter().filter_map(|v| {
                        let id = v.get("id")?.as_i64()?;
                        let code = v.get("code")?.as_str()?.to_string();
                        let name = v.get("name")?.as_str()?.to_string();
                        Some(AccountOpt { id, code, name })
                    }).collect();
                    gl_accounts.set(accs);
                }
            }
            if let Ok(data) = api_get(&format!("{}/locations", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    locations.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get(&format!("{}/bank_accounts", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    bank_accounts.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get("/api/currencies").await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    currencies.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get("/api/vat_rates").await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    vat_rates.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get("/api/nomenclatures/vat_exemptions").await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    exemptions.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get("/api/nomenclatures/units_of_measure?common=true").await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    units.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            let cid = crate::api::get_selected_company_id().unwrap_or(0);
            if cid > 0 {
                if let Ok(data) = api_get(&format!("/api/companies/{}", cid)).await {
                    if let Some(d) = data.get("data") {
                        let mut cur = d.get("currency").and_then(|v| v.as_str()).unwrap_or("EUR").trim().to_string();
                        if cur.is_empty() || cur.eq_ignore_ascii_case("BGN") {
                            cur = "EUR".to_string();
                        }
                        company_currency.set(cur.clone());
                        if !show_form.get_untracked() {
                            form_currency.set(cur);
                        }
                        vat_registered.set(d.get("is_vat_registered").and_then(|v| v.as_bool()).unwrap_or(true));
                    }
                }
            }
        });
    };

    Effect::new(move |_| {
        let _ = ctx.selected_company_id.get();
        reload();
        load_refs();
    });

    Effect::new(move |_| {
        let dir = direction.get();
        let cur = form_doc_type.get_untracked();
        if !invoice_selection_valid(&dir, &cur) {
            form_doc_type.set("01".into());
        }
    });

    Effect::new(move |_| {
        let dt = form_doc_type.get();
        if direction.get() != "sale" {
            return;
        }
        let kind = crate::constants::invoice_kind_for_nap(&dt);
        spawn_local(async move {
            if let Ok(data) = api_get(&format!("{}/sequences/preview?document_type={}", company_api_url(), kind)).await {
                if let Some(p) = data.pointer("/data/preview").and_then(|v| v.as_str()) {
                    preview.set(p.to_string());
                }
            }
        });
    });

    Effect::new(move |_| {
        if !show_form.get() {
            return;
        }
        let code = form_currency.get();
        let date = form_date.get();
        let func = company_currency.get();
        if code.is_empty() || date.is_empty() {
            return;
        }
        spawn_local(async move {
            match api_get(&format!(
                "/api/exchange_rates/for_currency?code={}&date={}&company_currency={}",
                code, date, func
            )).await {
                Ok(data) => {
                    if let Some(d) = data.get("data") {
                        form_exchange_rate.set(d.get("exchange_rate").and_then(|v| v.as_f64()).unwrap_or(1.0));
                        form_ecb_rate.set(d.get("ecb_rate").and_then(|v| v.as_f64()).unwrap_or(1.0));
                        form_rate_date.set(d.get("rate_date").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        form_rate_source.set(d.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string());
                        form_rate_found.set(d.get("found").and_then(|v| v.as_bool()).unwrap_or(false));
                    }
                }
                Err(_) => form_rate_found.set(false),
            }
        });
    });

    Effect::new(move |_| {
        let d = form_date.get();
        let terms = form_payment_terms.get();
        if d.len() >= 10 {
            form_due_date.set(add_days_iso(&d, terms));
        }
    });

    let open_form = move |_| {
        let today = today_iso();
        form_date.set(today.clone());
        form_tax_date.set(today.clone());
        form_due_date.set(add_days_iso(&today, 15));
        form_counterpart.set(String::new());
        form_doc_type.set("01".into());
        form_purchase_number.set(String::new());
        form_notes.set(String::new());
        form_location.set(String::new());
        form_payment.set("BANK".into());
        form_bank_name.set(String::new());
        form_bank_account.set(String::new());
        form_currency.set(company_currency.get());
        form_payment_terms.set(15);
        form_original.set(String::new());
        counterpart_filter.set(String::new());
        show_cp_results.set(false);
        show_new_counterpart.set(false);
        cp_form_error.set(String::new());
        cp_name.set(String::new());
        cp_eik.set(String::new());
        cp_vat.set(String::new());
        cp_address.set(String::new());
        cp_city.set(String::new());
        cp_country.set("BG".into());
        cp_type.set("both".into());
        cp_vies_msg.set(String::new());
        cp_vies_ok.set(false);
        product_picker.set(None);
        form_error.set(String::new());
        editing_id.set(None);
        form_existing_number.set(String::new());
        let vat = if vat_registered.get() { "20" } else { "0" };
        let reason = if vat_registered.get() { None } else { sm_reason_id(&exemptions.get()) };
        next_line_id.set(1);
        lines.set(vec![LineDraft {
            id: 0,
            description: String::new(),
            quantity: "1".into(),
            unit_price: "0".into(),
            vat_rate: vat.into(),
            product_id: None,
            unit_code: "C62".into(),
            vat_exemption_reason_id: reason,
            account_id: None,
            product_query: String::new(),
        }]);
        show_form.set(true);
    };

    let open_edit = move |id: i64| {
        form_error.set(String::new());
        error_msg.set(String::new());
        spawn_local(async move {
            match api_get(&format!("{}/invoices/{}", company_api_url(), id)).await {
                Ok(data) => {
                    let Some(d) = data.get("data") else {
                        error_msg.set("Фактурата не се зареди".into());
                        return;
                    };
                    let dir = d.get("direction").and_then(|v| v.as_str()).unwrap_or("sale").to_string();
                    direction.set(dir.clone());
                    editing_id.set(Some(id));
                    let number = d.get("invoice_number").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    form_existing_number.set(number.clone());
                    if dir == "purchase" {
                        form_purchase_number.set(number);
                    }
                    let issue = d.get("issue_date").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let due = d.get("due_date").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let tax = d.get("tax_event_date").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if let Some(days) = days_between(&issue, &due) {
                        if days >= 0 {
                            form_payment_terms.set(days);
                        }
                    }
                    form_date.set(issue);
                    form_tax_date.set(if tax.is_empty() { form_date.get_untracked() } else { tax });
                    form_due_date.set(due);
                    let doc_type = d.get("document_type").and_then(|v| v.as_str()).unwrap_or("INVOICE");
                    let nap = d.get("nap_document_type").and_then(|v| v.as_str());
                    form_doc_type.set(nap_code_of(doc_type, nap));
                    let cp_id = d.get("counterpart_id").and_then(|v| v.as_i64());
                    form_counterpart.set(cp_id.map(|n| n.to_string()).unwrap_or_default());
                    if let Some(cid) = cp_id {
                        if let Some(cp) = counterparts.get_untracked().iter().find(|c| c.id == cid) {
                            counterpart_filter.set(counterpart_label(cp));
                        } else if let Some(name) = d.get("counterpart_name").and_then(|v| v.as_str()) {
                            counterpart_filter.set(name.to_string());
                        }
                    } else {
                        counterpart_filter.set(String::new());
                    }
                    form_notes.set(d.get("notes").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    form_location.set(d.get("location_id").and_then(|v| v.as_i64()).map(|n| n.to_string()).unwrap_or_default());
                    let pay = d.get("payment_method").and_then(|v| v.as_str()).unwrap_or("BANK").to_string();
                    form_payment.set(if pay.is_empty() { "BANK".into() } else { pay });
                    let bank_id = d.get("bank_account_id").and_then(|v| v.as_i64());
                    form_bank_account.set(bank_id.map(|n| n.to_string()).unwrap_or_default());
                    if let Some(bid) = bank_id {
                        if let Some(bank) = bank_accounts.get_untracked().iter().find(|b| b.id == bid) {
                            form_bank_name.set(bank_key(bank));
                        }
                    } else {
                        form_bank_name.set(String::new());
                    }
                    let cur = d.get("currency").and_then(|v| v.as_str()).unwrap_or("EUR").to_string();
                    form_currency.set(if cur.is_empty() { company_currency.get_untracked() } else { cur });
                    form_exchange_rate.set(d.get("exchange_rate").and_then(|v| v.as_f64()).unwrap_or(1.0));
                    form_original.set(d.get("original_invoice_id").and_then(|v| v.as_i64()).map(|n| n.to_string()).unwrap_or_default());
                    show_cp_results.set(false);
                    show_new_counterpart.set(false);
                    product_picker.set(None);
                    let parsed: Vec<LineDraft> = d.get("lines").and_then(|v| v.as_array()).map(|arr| {
                        arr.iter().enumerate().filter_map(|(i, ln)| {
                            let description = ln.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            if description.trim().is_empty() {
                                return None;
                            }
                            let product_id = ln.get("product_id").and_then(|v| v.as_i64());
                            let product = product_id.and_then(|pid| products.get_untracked().iter().find(|p| p.id == pid).cloned());
                            let product_query = product.as_ref().map(|p| format!("{} {}", p.code, p.name)).unwrap_or_default();
                            let account_id = ln.get("account_id").and_then(|v| v.as_i64())
                                .or_else(|| product.as_ref().and_then(|p| nomenclature_account(p, &dir)));
                            let qty = ln.get("quantity").and_then(|v| v.as_f64()).unwrap_or(1.0);
                            let price = ln.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let vat = ln.get("vat_rate").and_then(|v| v.as_f64()).unwrap_or(20.0);
                            Some(LineDraft {
                                id: i as u64,
                                description,
                                quantity: format!("{}", qty),
                                unit_price: format!("{:.2}", price),
                                vat_rate: format!("{}", vat),
                                product_id,
                                unit_code: ln.get("unit_code").and_then(|v| v.as_str()).unwrap_or("C62").to_string(),
                                vat_exemption_reason_id: ln.get("vat_exemption_reason_id").and_then(|v| v.as_i64()),
                                account_id,
                                product_query,
                            })
                        }).collect()
                    }).unwrap_or_default();
                    if parsed.is_empty() {
                        let vat = if vat_registered.get_untracked() { "20" } else { "0" };
                        lines.set(vec![LineDraft {
                            id: 0,
                            description: String::new(),
                            quantity: "1".into(),
                            unit_price: "0".into(),
                            vat_rate: vat.into(),
                            product_id: None,
                            unit_code: "C62".into(),
                            vat_exemption_reason_id: if vat_registered.get_untracked() { None } else { sm_reason_id(&exemptions.get_untracked()) },
                            account_id: None,
                            product_query: String::new(),
                        }]);
                        next_line_id.set(1);
                    } else {
                        next_line_id.set(parsed.len() as u64);
                        lines.set(parsed);
                    }
                    show_form.set(true);
                }
                Err(e) => error_msg.set(e),
            }
        });
    };

    let do_cp_vies = move |_| {
        let vat = cp_vat.get().trim().to_string();
        if vat.len() < 3 {
            cp_vies_ok.set(false);
            cp_vies_msg.set("ДДС номерът трябва да е поне 3 символа".into());
            return;
        }
        cp_vies_loading.set(true);
        cp_vies_msg.set(String::new());
        cp_vies_ok.set(false);
        spawn_local(async move {
            let payload = serde_json::json!({ "vat_number": vat });
            match api_post("/api/vies/check", &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Ok(result) = serde_json::from_value::<ViesHit>(response.get("data").cloned().unwrap_or(serde_json::Value::Null)) {
                            if result.valid {
                                let eik = if !result.eik.is_empty() {
                                    crate::constants::strip_eu_vat_prefix(&result.eik)
                                } else {
                                    crate::constants::strip_eu_vat_prefix(&result.vat_number)
                                };
                                cp_name.set(result.name.clone());
                                cp_address.set(result.address.clone());
                                if !result.country_code.is_empty() {
                                    cp_country.set(result.country_code.clone());
                                }
                                cp_eik.set(eik.clone());
                                if !result.country_code.is_empty() && !result.vat_number.is_empty() {
                                    cp_vat.set(format!("{}{}", result.country_code, result.vat_number));
                                }
                                if let Some(existing) = find_counterpart_by_eik(&counterparts.get(), &eik) {
                                    form_counterpart.set(existing.id.to_string());
                                    counterpart_filter.set(counterpart_label(&existing));
                                    show_cp_results.set(false);
                                    show_new_counterpart.set(false);
                                    cp_vies_ok.set(true);
                                    cp_vies_msg.set(format!("ЕИК {} вече е въведен като {}. Избран е.", eik, existing.name));
                                } else {
                                    cp_vies_ok.set(true);
                                    cp_vies_msg.set("Валиден ДДС номер. Полетата са попълнени — запишете контрагента.".into());
                                }
                            } else {
                                cp_vies_ok.set(false);
                                cp_vies_msg.set("ДДС номерът не е валиден".into());
                            }
                        }
                    } else {
                        cp_vies_ok.set(false);
                        cp_vies_msg.set(api_error_text(&response));
                    }
                }
                Err(e) => {
                    cp_vies_ok.set(false);
                    cp_vies_msg.set(format!("Грешка: {}", e));
                }
            }
            cp_vies_loading.set(false);
        });
    };

    let save_new_counterpart = move |_| {
        let name = cp_name.get().trim().to_string();
        if name.is_empty() {
            cp_form_error.set("Името е задължително".into());
            return;
        }
        let eik = stored_eik(&cp_eik.get(), &cp_vat.get());
        cp_eik.set(eik.clone());
        if let Some(existing) = find_counterpart_by_eik(&counterparts.get(), &eik) {
            form_counterpart.set(existing.id.to_string());
            counterpart_filter.set(counterpart_label(&existing));
            show_cp_results.set(false);
            show_new_counterpart.set(false);
            cp_form_error.set(String::new());
            return;
        }
        cp_saving.set(true);
        cp_form_error.set(String::new());
        let vat = cp_vat.get().trim().to_string();
        let address = cp_address.get().trim().to_string();
        let city = cp_city.get().trim().to_string();
        let country = cp_country.get();
        let kind = cp_type.get();
        let payload = serde_json::json!({
            "counterpart": {
                "name": name.clone(),
                "eik": eik.clone(),
                "vat_number": vat.clone(),
                "address": address.clone(),
                "city": city.clone(),
                "country_code": country,
                "counterpart_type": kind,
                "is_active": true
            }
        });
        spawn_local(async move {
            match api_post(&format!("{}/counterparts", company_api_url()), &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Some(id) = response.pointer("/data/id").and_then(|v| v.as_i64()) {
                            let created = CounterpartOpt {
                                id,
                                name: name.clone(),
                                eik: Some(eik.clone()),
                                vat_number: Some(vat.clone()),
                                address: Some(address.clone()),
                                city: Some(city.clone()),
                            };
                            counterparts.update(|list| {
                                if !list.iter().any(|c| c.id == id) {
                                    list.push(created.clone());
                                }
                            });
                            form_counterpart.set(id.to_string());
                            counterpart_filter.set(counterpart_label(&created));
                            show_cp_results.set(false);
                            show_new_counterpart.set(false);
                            cp_form_error.set(String::new());
                        }
                    } else {
                        cp_form_error.set(api_error_text(&response));
                    }
                }
                Err(e) => cp_form_error.set(e),
            }
            cp_saving.set(false);
        });
    };

    view! {
        <Layout>
            <div class="flex flex-wrap justify-between items-start gap-4">
                <div>
                    <h1 class="text-2xl font-semibold text-gray-800">"Фактури"</h1>
                    <p class="text-sm text-gray-500 mt-1">
                        "Продажби и покупки по стойност. Склад се ползва само ако има стоки — услугите не изискват наличност."
                    </p>
                </div>
                <div class="flex flex-wrap gap-2">
                    <button class=move || if direction.get()=="sale" { chip_on() } else { chip_off() }
                        on:click=move |_| { direction.set("sale".into()); reload(); }>"Продажби"</button>
                    <button class=move || if direction.get()=="purchase" { chip_on() } else { chip_off() }
                        on:click=move |_| { direction.set("purchase".into()); reload(); }>"Покупки"</button>
                    <button class="px-4 py-2 rounded-lg bg-slate-700 text-white text-sm hover:bg-slate-800" on:click=open_form>"+ Нова"</button>
                </div>
            </div>
            <Show when=move || !error_msg.get().is_empty()>
                <p class="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">{move || error_msg.get()}</p>
            </Show>
            <Show when=move || !success_msg.get().is_empty()>
                <p class="mt-3 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-800">{move || success_msg.get()}</p>
            </Show>

            <div class="bg-white rounded-xl mt-4 overflow-hidden border border-gray-200 shadow-sm">
                <table class="w-full text-sm">
                    <thead class="bg-gray-100 text-gray-600">
                        <tr>
                            <th class="p-3 text-left font-medium">"Номер"</th>
                            <th class="p-3 text-left font-medium">"Дата"</th>
                            <th class="p-3 text-left font-medium">"Контрагент"</th>
                            <th class="p-3 text-left font-medium">"Тип"</th>
                            <th class="p-3 text-right font-medium">"Сума"</th>
                            <th class="p-3 text-left font-medium">"Статус"</th>
                            <th class="p-3 text-center font-medium">"Действия"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = invoices.get();
                            if rows.is_empty() {
                                view! { <tr><td colspan="7" class="p-8 text-center text-gray-500">"Няма документи"</td></tr> }.into_any()
                            } else {
                                rows.into_iter().map(|inv| {
                                    let id = inv.id;
                                    let status = inv.status.clone().unwrap_or_default();
                                    let can_post = (status == "draft" || status == "issued")
                                        && inv.document_type.as_deref() != Some("PROFORMA");
                                    let can_edit = status == "draft";
                                    let can_draft = status == "issued" || status == "posted";
                                    let can_delete = status == "draft";
                                    let can_ubl = (status == "issued" || status == "posted") && inv.direction.as_deref() != Some("purchase");
                                    let status_label = match status.as_str() {
                                        "draft" => "чернова",
                                        "issued" => "издадена",
                                        "posted" => "осчетоводена",
                                        "cancelled" => "анулирана",
                                        other => other,
                                    }.to_string();
                                    let cur = inv.currency.clone().unwrap_or_default();
                                    view! {
                                        <tr class="border-b border-gray-100 hover:bg-gray-50">
                                            <td class="p-3 text-gray-800">{inv.invoice_number.clone().unwrap_or_else(|| "чернова".into())}</td>
                                            <td class="p-3 text-gray-600">{inv.issue_date.clone().unwrap_or_default()}</td>
                                            <td class="p-3 text-gray-800">{inv.counterpart_name.clone().unwrap_or_default()}</td>
                                            <td class="p-3 text-gray-600">{invoice_type_label(&inv)}</td>
                                            <td class="p-3 text-right tabular-nums text-gray-800">{format!("{:.2} {}", inv.total_amount.unwrap_or(0.0), cur)}</td>
                                            <td class="p-3"><span class="inline-flex rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600">{status_label}</span></td>
                                            <td class="p-3 text-center flex gap-2 justify-center flex-wrap">
                                                <Show when=move || can_edit>
                                                    <button class="px-2.5 py-1 bg-gray-100 text-gray-700 rounded-md text-sm hover:bg-gray-200"
                                                        on:click=move |_| open_edit(id)
                                                    >"Редактирай"</button>
                                                </Show>
                                                <Show when=move || can_draft>
                                                    <button class="px-2.5 py-1 bg-amber-50 text-amber-900 rounded-md text-sm hover:bg-amber-100"
                                                        on:click=move |_| {
                                                            let ok = web_sys::window().and_then(|w| {
                                                                w.confirm_with_message(
                                                                    "Фактурата ще се върне в чернова и ще може да се редактира. Счетоводна статия в чернова се изтрива. Осчетоводена статия се сторнира и остава в журнала. Продължавате?"
                                                                ).ok()
                                                            }).unwrap_or(false);
                                                            if !ok { return; }
                                                            spawn_local(async move {
                                                                match api_post(&format!("{}/invoices/{}/return_to_draft", company_api_url(), id), &serde_json::json!({})).await {
                                                                    Ok(resp) => {
                                                                        let msg = resp.get("message").and_then(|v| v.as_str())
                                                                            .unwrap_or("Фактурата е върната в чернова");
                                                                        success_msg.set(msg.to_string());
                                                                        reload();
                                                                    }
                                                                    Err(e) => error_msg.set(e),
                                                                }
                                                            });
                                                        }
                                                    >"Върни в чернова"</button>
                                                </Show>
                                                <Show when=move || can_post>
                                                    <button class="px-2.5 py-1 bg-slate-700 text-white rounded-md text-sm hover:bg-slate-800"
                                                        on:click=move |_| {
                                                            posting_id.set(Some(id));
                                                            spawn_local(async move {
                                                                match api_post(&format!("{}/invoices/{}/issue_and_post", company_api_url(), id), &serde_json::json!({})).await {
                                                                    Ok(_) => { success_msg.set("Осчетоводена".into()); reload(); }
                                                                    Err(e) => error_msg.set(e),
                                                                }
                                                                posting_id.set(None);
                                                            });
                                                        }
                                                    >"Издай и осчетоводи"</button>
                                                </Show>
                                                <a class="px-2.5 py-1 bg-gray-100 text-gray-700 rounded-md text-sm hover:bg-gray-200" href=format!("/invoices/{}/print", id)>"Печат"</a>
                                                <Show when=move || can_ubl>
                                                    <button class="px-2.5 py-1 bg-gray-100 text-gray-700 rounded-md text-sm hover:bg-gray-200"
                                                        on:click=move |_| {
                                                            spawn_local(async move {
                                                                match api_get_text(&format!("{}/invoices/{}/ubl", company_api_url(), id)).await {
                                                                    Ok(xml) => download_text_file(&format!("invoice-{}.xml", id), &xml, "application/xml"),
                                                                    Err(e) => error_msg.set(e),
                                                                }
                                                            });
                                                        }
                                                    >"UBL"</button>
                                                </Show>
                                                <Show when=move || can_delete>
                                                    <button class="px-2.5 py-1 bg-gray-100 text-gray-600 rounded-md text-sm hover:bg-red-50 hover:text-red-700"
                                                        on:click=move |_| {
                                                            spawn_local(async move {
                                                                match api_delete(&format!("{}/invoices/{}", company_api_url(), id)).await {
                                                                    Ok(_) => { success_msg.set("Черновата е изтрита".into()); reload(); }
                                                                    Err(e) => error_msg.set(e),
                                                                }
                                                            });
                                                        }
                                                    >"Изтрий"</button>
                                                </Show>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </tbody>
                </table>
            </div>

            <Show when=move || show_form.get()>
                <div class="fixed inset-0 bg-slate-900/40 flex items-start justify-center overflow-auto z-50 text-left">
                    <div class="bg-slate-50 rounded-2xl mt-6 mb-10 w-full max-w-6xl border border-gray-200 shadow-xl">
                        <div class="sticky top-0 z-10 flex justify-between items-start gap-4 px-6 py-4 border-b border-gray-200 bg-slate-50/95 rounded-t-2xl">
                            <div>
                                <h2 class="text-xl font-semibold text-gray-800">{move || {
                                    if editing_id.get().is_some() {
                                        "Редакция на фактура"
                                    } else if direction.get()=="sale" {
                                        "Нова продажба / доставка"
                                    } else {
                                        "Нова покупка / доставка"
                                    }
                                }}</h2>
                                <p class="text-sm text-gray-500 mt-1">
                                    "По стойност — описание, к-во, цена, ДДС. Приходната и разходната сметка се взимат от артикула в номенклатурата."
                                </p>
                            </div>
                            <button class="h-8 w-8 rounded-full bg-gray-200 text-gray-600 hover:bg-gray-300" on:click=move |_| show_form.set(false)>"×"</button>
                        </div>
                        <div class="px-6 py-5 space-y-4">
                        <Show when=move || !form_error.get().is_empty()>
                            <p class="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">{move || form_error.get()}</p>
                        </Show>

                        <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
                        <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-3">"Заглавна информация"</h3>
                        <label class="text-sm block mb-1">"Вид документ"
                            <select class=input_cls()
                                prop:value=move || form_doc_type.get()
                                on:change=move |ev| {
                                    form_doc_type.set(event_target_value(&ev));
                                    form_original.set(String::new());
                                }>
                                <option value="PROFORMA">"Проформа — не е данъчен документ, не се осчетоводява"</option>
                                {move || {
                                    let dir = direction.get();
                                    let selected = form_doc_type.get();
                                    let kind = if dir == "purchase" { "PURCHASE" } else { "SALE" };
                                    crate::constants::doc_types_for(kind).iter().map(|entry| {
                                        let code = entry.code.to_string();
                                        let label = format!("{} — {}", entry.code, entry.name);
                                        let is_selected = selected == entry.code;
                                        view! { <option value=code selected=is_selected>{label}</option> }
                                    }).collect_view()
                                }}
                            </select>
                        </label>
                        <p class="text-xs text-gray-500 mb-4">
                            "Списъкът е по ППЗДДС. Проформата не е данъчен документ, не се осчетоводява и има отделен номер. Фактура, дебитно и кредитно известие ползват един 10-цифрен интервал, протоколите — друг. Интервалите са в "
                            <a href="/settings" class="text-blue-600">"Настройки → Номерация"</a>
                            "."
                        </p>
                        <div class="grid grid-cols-1 md:grid-cols-4 gap-3 mb-4">
                            <Show when=move || direction.get()=="sale" && editing_id.get().is_some() && !form_existing_number.get().is_empty()>
                                <label class="text-sm">"Номер"
                                    <input class=format!("{} bg-gray-100", input_cls()) prop:value=move || form_existing_number.get() disabled=true />
                                </label>
                            </Show>
                            <Show when=move || direction.get()=="sale" && (editing_id.get().is_none() || form_existing_number.get().is_empty())>
                                <label class="text-sm">"Номер (преглед)"
                                    <input class=format!("{} bg-gray-100", input_cls()) prop:value=move || preview.get() disabled=true />
                                </label>
                            </Show>
                            <Show when=move || direction.get()=="purchase">
                                <label class="text-sm">"Номер на доставчика"
                                    <input class=input_cls() prop:value=move || form_purchase_number.get()
                                        on:input=move |ev| form_purchase_number.set(event_target_value(&ev)) />
                                </label>
                            </Show>
                            <label class="text-sm">"Дата на издаване"
                                <input class=input_cls() type="date" prop:value=move || form_date.get()
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        form_date.set(v.clone());
                                        if form_tax_date.get().is_empty() { form_tax_date.set(v); }
                                    } />
                            </label>
                            <label class="text-sm">"Дата на данъчно събитие"
                                <input class=input_cls() type="date" prop:value=move || form_tax_date.get()
                                    on:input=move |ev| form_tax_date.set(event_target_value(&ev)) />
                            </label>
                            <label class="text-sm">"Падеж"
                                <input class=input_cls() type="date" prop:value=move || form_due_date.get()
                                    on:input=move |ev| form_due_date.set(event_target_value(&ev)) />
                            </label>
                        </div>

                        <Show when=move || crate::constants::is_corrective_nap(&form_doc_type.get())>
                            <label class="text-sm block mb-4">"Към документ"
                                <select class=input_cls() prop:value=move || form_original.get()
                                    on:change=move |ev| form_original.set(event_target_value(&ev))>
                                    <option value="">"— изберете оригинал —"</option>
                                    {move || invoices.get().into_iter().filter(|i| {
                                        let s = i.status.clone().unwrap_or_default();
                                        s == "issued" || s == "posted"
                                    }).map(|i| {
                                        let label = format!("{} {} {}", i.invoice_number.clone().unwrap_or_default(), i.issue_date.clone().unwrap_or_default(), i.counterpart_name.clone().unwrap_or_default());
                                        view! { <option value=i.id.to_string()>{label}</option> }
                                    }).collect_view()}
                                </select>
                            </label>
                        </Show>

                        </div>

                        <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
                        <div class="flex flex-wrap items-center justify-between gap-2 mb-3">
                            <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500">{move || if direction.get()=="sale" { "Клиент" } else { "Доставчик" }}</h3>
                            <button class="text-sm text-slate-700 hover:text-slate-900" type="button"
                                on:click=move |_| {
                                    let opening = !show_new_counterpart.get();
                                    show_new_counterpart.set(opening);
                                    show_cp_results.set(false);
                                    if opening {
                                        cp_form_error.set(String::new());
                                        cp_vies_msg.set(String::new());
                                        cp_vies_ok.set(false);
                                        cp_type.set(if direction.get() == "purchase" { "supplier" } else { "customer" }.into());
                                        let q = counterpart_filter.get().trim().to_string();
                                        if cp_name.get().is_empty() && cp_vat.get().is_empty() && !q.is_empty() {
                                            let compact: String = q.chars().filter(|c| !c.is_whitespace()).collect();
                                            if compact.len() >= 4 && compact.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                                                cp_vat.set(compact.to_uppercase());
                                            } else {
                                                cp_name.set(q);
                                            }
                                        }
                                    }
                                }
                            >{move || if show_new_counterpart.get() { "Затвори добавянето" } else { "+ Нов контрагент" }}</button>
                        </div>
                        <label class="text-sm block mb-2">"Търсене по име, ЕИК или ДДС номер"
                            <input class=input_cls() placeholder="Започнете да пишете…"
                                prop:value=move || counterpart_filter.get()
                                on:focus=move |_| show_cp_results.set(true)
                                on:input=move |ev| {
                                    counterpart_filter.set(event_target_value(&ev));
                                    form_counterpart.set(String::new());
                                    show_cp_results.set(true);
                                } />
                        </label>
                        {move || {
                            if !show_cp_results.get() {
                                return view! { <div></div> }.into_any();
                            }
                            let q = counterpart_filter.get().trim().to_lowercase();
                            let rows: Vec<CounterpartOpt> = counterparts.get().into_iter()
                                .filter(|c| counterpart_matches(c, &q))
                                .take(20)
                                .collect();
                            if rows.is_empty() {
                                view! {
                                    <div class="mb-2 rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-600">
                                        "Няма такъв контрагент. Добавете го ръчно или чрез VIES."
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="mb-2 max-h-52 overflow-auto rounded-lg border border-gray-200 bg-white">
                                        {rows.into_iter().map(|c| {
                                            let label = counterpart_label(&c);
                                            let id = c.id;
                                            let picked = c.clone();
                                            view! {
                                                <button class="block w-full text-left px-3 py-2 text-sm border-b border-gray-100 hover:bg-gray-50" type="button"
                                                    on:click=move |_| {
                                                        form_counterpart.set(id.to_string());
                                                        counterpart_filter.set(counterpart_label(&picked));
                                                        show_cp_results.set(false);
                                                        show_new_counterpart.set(false);
                                                    }
                                                >{label}</button>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }}
                        {move || {
                            let id = form_counterpart.get();
                            let cp = counterparts.get().into_iter().find(|c| c.id.to_string() == id);
                            if let Some(c) = cp {
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-2 text-sm bg-gray-50 border border-gray-100 p-3 rounded-lg">
                                        <div><span class="text-gray-500">"Име: "</span>{c.name}</div>
                                        <div><span class="text-gray-500">"ЕИК: "</span>{c.eik.unwrap_or_default()}</div>
                                        <div><span class="text-gray-500">"ДДС №: "</span>{c.vat_number.unwrap_or_default()}</div>
                                        <div class="md:col-span-3"><span class="text-gray-500">"Адрес: "</span>{c.address.unwrap_or_default()}{" "}{c.city.unwrap_or_default()}</div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}
                        <Show when=move || show_new_counterpart.get()>
                            <div class="mt-2 rounded-lg border border-gray-200 bg-gray-50 p-3">
                                <div class="mb-3">
                                    <label class="text-sm font-medium text-gray-700">"Търсене по ДДС номер (VIES)"</label>
                                    <p class="text-xs text-gray-500 mb-2">"Буквите BG, IT, ES и др. се махат. Номерът се записва в ЕИК."</p>
                                    <div class="flex gap-2">
                                        <input class=input_cls() placeholder="напр. BG123456789"
                                            prop:value=move || cp_vat.get()
                                            on:input=move |ev| cp_vat.set(event_target_value(&ev)) />
                                        <button class="px-3 py-2 rounded-lg bg-slate-700 text-white text-sm hover:bg-slate-800 disabled:opacity-60 whitespace-nowrap" type="button"
                                            disabled=move || cp_vies_loading.get()
                                            on:click=do_cp_vies
                                        >{move || if cp_vies_loading.get() { "Търсене..." } else { "Провери" }}</button>
                                    </div>
                                    {move || {
                                        let msg = cp_vies_msg.get();
                                        if msg.is_empty() {
                                            view! { <div></div> }.into_any()
                                        } else if cp_vies_ok.get() {
                                            view! { <p class="mt-2 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-800">{msg}</p> }.into_any()
                                        } else {
                                            view! { <p class="mt-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">{msg}</p> }.into_any()
                                        }
                                    }}
                                </div>
                                <p class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-2">"Или въведете ръчно"</p>
                                {move || {
                                    let err = cp_form_error.get();
                                    if err.is_empty() {
                                        view! { <div></div> }.into_any()
                                    } else {
                                        view! { <p class="mb-2 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">{err}</p> }.into_any()
                                    }
                                }}
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                                    <label class="text-sm md:col-span-2">"Име *"
                                        <input class=input_cls() prop:value=move || cp_name.get()
                                            on:input=move |ev| cp_name.set(event_target_value(&ev)) />
                                    </label>
                                    <label class="text-sm">"ЕИК"
                                        <input class=input_cls() placeholder="123456789" prop:value=move || cp_eik.get()
                                            on:input=move |ev| cp_eik.set(event_target_value(&ev)) />
                                    </label>
                                    <label class="text-sm">"Държава"
                                        <select class=input_cls() prop:value=move || cp_country.get()
                                            on:change=move |ev| cp_country.set(event_target_value(&ev))>
                                            {move || {
                                                let current = cp_country.get();
                                                country_choices().into_iter().map(|(code, name)| {
                                                    let selected = code == current;
                                                    view! { <option value=code selected=selected>{name}</option> }
                                                }).collect_view()
                                            }}
                                        </select>
                                    </label>
                                    <label class="text-sm">"Град"
                                        <input class=input_cls() prop:value=move || cp_city.get()
                                            on:input=move |ev| cp_city.set(event_target_value(&ev)) />
                                    </label>
                                    <label class="text-sm">"Тип"
                                        <select class=input_cls() prop:value=move || cp_type.get()
                                            on:change=move |ev| cp_type.set(event_target_value(&ev))>
                                            {move || {
                                                let current = cp_type.get();
                                                [("both", "Клиент и доставчик"), ("customer", "Само клиент"), ("supplier", "Само доставчик")].into_iter().map(|(code, name)| {
                                                    let selected = code == current;
                                                    view! { <option value=code selected=selected>{name}</option> }
                                                }).collect_view()
                                            }}
                                        </select>
                                    </label>
                                    <label class="text-sm md:col-span-2">"Адрес"
                                        <input class=input_cls() prop:value=move || cp_address.get()
                                            on:input=move |ev| cp_address.set(event_target_value(&ev)) />
                                    </label>
                                </div>
                                <div class="mt-3 flex justify-end">
                                    <button class="px-4 py-2 rounded-lg bg-slate-700 text-white text-sm hover:bg-slate-800 disabled:opacity-60" type="button"
                                        disabled=move || cp_saving.get()
                                        on:click=save_new_counterpart
                                    >{move || if cp_saving.get() { "Запис..." } else { "Запиши и избери" }}</button>
                                </div>
                            </div>
                        </Show>
                        </div>

                        <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
                        <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-3">"Валута и курс"</h3>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-4">
                            <label class="text-sm">"Валута"
                                <select class=input_cls() prop:value=move || form_currency.get()
                                    on:change=move |ev| form_currency.set(event_target_value(&ev))>
                                    {move || {
                                        let current = form_currency.get();
                                        let mut list = currencies.get();
                                        if list.is_empty() {
                                            ["EUR", "USD", "GBP"].into_iter().map(|c| {
                                                let selected = c == current;
                                                view! { <option value=c selected=selected>{c}</option> }
                                            }).collect_view().into_any()
                                        } else {
                                            list.sort_by(|a, b| {
                                                let rank = |code: &str| if code == "EUR" { 0 } else { 1 };
                                                rank(&a.code).cmp(&rank(&b.code)).then(a.code.cmp(&b.code))
                                            });
                                            list.into_iter().map(|c| {
                                                let label = format!("{} — {}", c.code, c.name_bg.clone().unwrap_or(c.name.clone()));
                                                let code = c.code.clone();
                                                let selected = code == current;
                                                view! { <option value=code selected=selected>{label}</option> }
                                            }).collect_view().into_any()
                                        }
                                    }}
                                </select>
                            </label>
                            <div class="text-sm md:col-span-2 flex items-end">
                                {move || {
                                    let src = form_rate_source.get();
                                    let found = form_rate_found.get();
                                    let code = form_currency.get();
                                    let func = company_currency.get();
                                    if code == func {
                                        view! { <p class="text-gray-600">{format!("Курс 1 — {} е валутата на фирмата", func)}</p> }.into_any()
                                    } else if !found {
                                        view! { <p class="text-amber-700">"Няма курс за тази дата. Заредете курсове от Валути или сменете датата."</p> }.into_any()
                                    } else if src == "fixed" {
                                        view! { <p class="text-gray-600">{format!("Фиксиран курс 1 EUR = 1.95583 BGN · 1 {} = {:.6} {}", code, form_exchange_rate.get(), func)}</p> }.into_any()
                                    } else {
                                        view! { <p class="text-gray-600">{format!("ЕЦБ 1 EUR = {:.5} {} от {} · към {} × {:.6}", form_ecb_rate.get(), code, form_rate_date.get(), func, form_exchange_rate.get())}</p> }.into_any()
                                    }
                                }}
                            </div>
                        </div>

                        </div>

                        <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
                        <div class="flex items-center justify-between mb-3">
                            <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500">"Артикули / услуги"</h3>
                            <button class="text-sm text-slate-700 hover:text-slate-900" type="button" on:click=move |_| {
                                let vat = if vat_registered.get() { "20" } else { "0" };
                                let reason = if vat_registered.get() { None } else { sm_reason_id(&exemptions.get()) };
                                let id = next_line_id.get();
                                next_line_id.set(id + 1);
                                lines.update(|ls| ls.push(LineDraft {
                                    id,
                                    description: String::new(),
                                    quantity: "1".into(),
                                    unit_price: "0".into(),
                                    vat_rate: vat.into(),
                                    product_id: None,
                                    unit_code: "C62".into(),
                                    vat_exemption_reason_id: reason,
                                    account_id: None,
                                    product_query: String::new(),
                                }));
                            }>"+ Ред"</button>
                        </div>
                        <div class="overflow-x-auto mb-2">
                            <table class="w-full text-sm">
                                <thead>
                                    <tr class="bg-gray-50 text-gray-500">
                                        <th class="p-2 text-left font-medium">"Артикул / описание"</th>
                                        <th class="p-2 w-24 font-medium">"К-во"</th>
                                        <th class="p-2 w-28 font-medium">"Мярка"</th>
                                        <th class="p-2 w-28 font-medium">"Ед. цена"</th>
                                        <th class="p-2 w-28 font-medium">"ДДС %"</th>
                                        <th class="p-2 w-28 text-right font-medium">"Общо"</th>
                                        <th class="p-2 w-10"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || lines.get().into_iter().map(|line| {
                                        let lid = line.id;
                                        let desc = line.description.clone();
                                        let qty = line.quantity.clone();
                                        let price = line.unit_price.clone();
                                        let vat = line.vat_rate.clone();
                                        let unit = line.unit_code.clone();
                                        let product_query = line.product_query.clone();
                                        let ex_id = line.vat_exemption_reason_id;
                                        let account_id = line.account_id;
                                        let net = line_net(&line);
                                        let show_ex = parse_f(&vat).abs() < 0.0001;
                                        view! {
                                            <tr class="border-b border-gray-100 align-top">
                                                <td class="p-2">
                                                    <input class=format!("{} mb-1", input_cls()) placeholder="Търсене на артикул — код или име"
                                                        prop:value=product_query
                                                        on:focus=move |_| product_picker.set(Some(lid))
                                                        on:input=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            product_picker.set(Some(lid));
                                                            lines.update(|ls| {
                                                                if let Some(l) = ls.iter_mut().find(|x| x.id == lid) {
                                                                    l.product_query = val;
                                                                    l.product_id = None;
                                                                }
                                                            });
                                                        } />
                                                    {move || {
                                                        if product_picker.get() != Some(lid) {
                                                            return view! { <div></div> }.into_any();
                                                        }
                                                        let q = lines.get().into_iter().find(|l| l.id == lid)
                                                            .map(|l| l.product_query.clone())
                                                            .unwrap_or_default();
                                                        let matches: Vec<ProductOpt> = products.get().into_iter()
                                                            .filter(|p| product_matches(p, q.trim()))
                                                            .take(12)
                                                            .collect();
                                                        if matches.is_empty() {
                                                            view! { <p class="mb-1 text-xs text-gray-500">"Няма артикул. Попълнете описанието по-долу — услугата може и без номенклатура."</p> }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="mb-1 max-h-40 overflow-auto rounded-lg border border-gray-200 bg-white">
                                                                    {matches.into_iter().map(|p| {
                                                                        let id = p.id;
                                                                        let name = p.name.clone();
                                                                        let price = p.price;
                                                                        let unit_code = p.uom_standard.clone().or(p.measure_unit.clone());
                                                                        let account = nomenclature_account(&p, &direction.get());
                                                                        let label = format!("{} {}", p.code, p.name);
                                                                        let shown = label.clone();
                                                                        view! {
                                                                            <button class="block w-full text-left px-2 py-1.5 text-sm border-b border-gray-100 hover:bg-gray-50" type="button"
                                                                                on:click=move |_| {
                                                                                    let name = name.clone();
                                                                                    let label = label.clone();
                                                                                    let unit_code = unit_code.clone();
                                                                                    lines.update(|ls| {
                                                                                        if let Some(l) = ls.iter_mut().find(|x| x.id == lid) {
                                                                                            l.product_id = Some(id);
                                                                                            l.product_query = label;
                                                                                            l.account_id = account;
                                                                                            if l.description.trim().is_empty() { l.description = name; }
                                                                                            if let Some(pr) = price { l.unit_price = format!("{:.2}", pr); }
                                                                                            if let Some(u) = unit_code.clone() { l.unit_code = u; }
                                                                                        }
                                                                                    });
                                                                                    product_picker.set(None);
                                                                                }
                                                                            >{shown}</button>
                                                                        }
                                                                    }).collect_view()}
                                                                </div>
                                                            }.into_any()
                                                        }
                                                    }}
                                                    <input class=input_cls() placeholder="Описание" prop:value=desc
                                                        on:input=move |ev| lines.update(|ls| { if let Some(l)=ls.iter_mut().find(|x| x.id == lid) { l.description = event_target_value(&ev); }}) />
                                                    <select class=format!("{} mt-1", input_cls())
                                                        on:change=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            lines.update(|ls| {
                                                                if let Some(l) = ls.iter_mut().find(|x| x.id == lid) {
                                                                    l.account_id = val.parse::<i64>().ok();
                                                                }
                                                            });
                                                        }>
                                                        <option value="">"— сметка от номенклатурата —"</option>
                                                        {move || {
                                                            let dir = direction.get();
                                                            account_choices(&gl_accounts.get(), &dir, account_id).into_iter().map(|a| {
                                                                let selected = account_id == Some(a.id);
                                                                let label = format!("{} — {}", a.code, a.name);
                                                                view! { <option value=a.id.to_string() selected=selected>{label}</option> }
                                                            }).collect_view()
                                                        }}
                                                    </select>
                                                    <Show when=move || show_ex>
                                                        <select class=format!("{} mt-1", input_cls())
                                                            on:change=move |ev| {
                                                                let val = event_target_value(&ev);
                                                                lines.update(|ls| {
                                                                    if let Some(l) = ls.iter_mut().find(|x| x.id == lid) {
                                                                        l.vat_exemption_reason_id = val.parse::<i64>().ok();
                                                                    }
                                                                });
                                                            }>
                                                            <option value="">"— основание за неначисляване —"</option>
                                                            {move || exemptions.get().into_iter().map(|r| {
                                                                let selected = ex_id == Some(r.id);
                                                                let label = match r.legal_basis.clone() {
                                                                    Some(b) if !b.is_empty() => format!("{} ({})", r.reason_name, b),
                                                                    _ => r.reason_name.clone(),
                                                                };
                                                                view! { <option value=r.id.to_string() selected=selected>{label}</option> }
                                                            }).collect_view()}
                                                        </select>
                                                    </Show>
                                                </td>
                                                <td class="p-2">
                                                    <input class=input_cls() prop:value=qty
                                                        on:input=move |ev| lines.update(|ls| { if let Some(l)=ls.iter_mut().find(|x| x.id == lid) { l.quantity = event_target_value(&ev); }}) />
                                                </td>
                                                <td class="p-2">
                                                    <select class=input_cls()
                                                        on:change=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            lines.update(|ls| { if let Some(l)=ls.iter_mut().find(|x| x.id == lid) { l.unit_code = val; }});
                                                        }>
                                                        {move || {
                                                            let list = units.get();
                                                            let current = unit.clone();
                                                            if list.is_empty() {
                                                                view! { <option value="C62" selected=true>"брой (C62)"</option> }.into_any()
                                                            } else {
                                                                list.into_iter().map(|u| {
                                                                    let selected = u.code == current;
                                                                    let label = format!("{} ({})", u.name_bg.unwrap_or_else(|| u.code.clone()), u.symbol.unwrap_or(u.code.clone()));
                                                                    view! { <option value=u.code.clone() selected=selected>{label}</option> }
                                                                }).collect_view().into_any()
                                                            }
                                                        }}
                                                    </select>
                                                </td>
                                                <td class="p-2">
                                                    <input class=input_cls() prop:value=price
                                                        on:input=move |ev| lines.update(|ls| { if let Some(l)=ls.iter_mut().find(|x| x.id == lid) { l.unit_price = event_target_value(&ev); }}) />
                                                </td>
                                                <td class="p-2">
                                                    <select class=input_cls()
                                                        on:change=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            lines.update(|ls| {
                                                                if let Some(l) = ls.iter_mut().find(|x| x.id == lid) {
                                                                    l.vat_rate = val.clone();
                                                                    if parse_f(&val).abs() > 0.0001 {
                                                                        l.vat_exemption_reason_id = None;
                                                                    } else if l.vat_exemption_reason_id.is_none() && !vat_registered.get() {
                                                                        l.vat_exemption_reason_id = sm_reason_id(&exemptions.get());
                                                                    }
                                                                }
                                                            });
                                                        }>
                                                        {move || unique_vat_percents(&vat_rates.get()).into_iter().map(|p| {
                                                            let selected = (parse_f(&vat) - p).abs() < 0.001;
                                                            view! { <option value=format!("{}", p) selected=selected>{format!("{}%", p)}</option> }
                                                        }).collect_view()}
                                                    </select>
                                                </td>
                                                <td class="p-2 text-right font-medium whitespace-nowrap">{format!("{:.2}", net)}</td>
                                                <td class="p-2">
                                                    <button class="h-8 w-8 rounded-full text-gray-400 hover:bg-gray-100 hover:text-gray-700" type="button"
                                                        on:click=move |_| lines.update(|ls| { if ls.len() > 1 { ls.retain(|x| x.id != lid); } })
                                                    >"×"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>

                        {move || {
                            let needs_loc = lines.get().iter().any(|l| {
                                l.product_id.and_then(|id| products.get().into_iter().find(|p| p.id == id)).map(|p| product_holds_stock(&p)).unwrap_or(false)
                            });
                            if needs_loc {
                                view! {
                                    <label class="text-sm block mb-4">"Обект (склад) — задължителен само за стоки с наличност"
                                        <select class=input_cls() prop:value=move || form_location.get()
                                            on:change=move |ev| form_location.set(event_target_value(&ev))>
                                            <option value="">"—"</option>
                                            {move || locations.get().into_iter().filter(|l| l.holds_stock.unwrap_or(true)).map(|l| {
                                                view! { <option value=l.id.to_string()>{l.name}</option> }
                                            }).collect_view()}
                                        </select>
                                    </label>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}

                        </div>

                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
                                <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-3">"Плащане"</h3>
                                <label class="text-sm block mb-2">"Начин на плащане"
                                    <select class=input_cls() prop:value=move || form_payment.get()
                                        on:change=move |ev| {
                                            let v = event_target_value(&ev);
                                            form_payment.set(v.clone());
                                            if v != "BANK" {
                                                form_bank_name.set(String::new());
                                                form_bank_account.set(String::new());
                                            }
                                        }>
                                        <option value="BANK">"По банкова сметка"</option>
                                        <option value="CASH">"В брой"</option>
                                        <option value="CARD">"Карта"</option>
                                    </select>
                                </label>
                                <Show when=move || form_payment.get()=="BANK">
                                    <div class="grid grid-cols-1 gap-2">
                                        <label class="text-sm">"Банка"
                                            <select class=input_cls() prop:value=move || form_bank_name.get()
                                                on:change=move |ev| {
                                                    form_bank_name.set(event_target_value(&ev));
                                                    form_bank_account.set(String::new());
                                                }>
                                                <option value="">"— изберете банка —"</option>
                                                {move || unique_banks(&bank_accounts.get()).into_iter().map(|n| {
                                                    let label = n.clone();
                                                    view! { <option value=n>{label}</option> }
                                                }).collect_view()}
                                            </select>
                                        </label>
                                        <Show when=move || !form_bank_name.get().is_empty()>
                                            <label class="text-sm">"Банкова сметка"
                                                <select class=input_cls() prop:value=move || form_bank_account.get()
                                                    on:change=move |ev| form_bank_account.set(event_target_value(&ev))>
                                                    <option value="">"— изберете сметка —"</option>
                                                    {move || {
                                                        let bank = form_bank_name.get();
                                                        let cur = form_currency.get();
                                                        let mut accs: Vec<BankAccountOpt> = bank_accounts.get().into_iter().filter(|a| bank_key(a) == bank).collect();
                                                        accs.sort_by(|a, b| {
                                                            let ac = a.currency.clone().unwrap_or_default();
                                                            let bc = b.currency.clone().unwrap_or_default();
                                                            (bc == cur).cmp(&(ac == cur)).then(ac.cmp(&bc))
                                                        });
                                                        accs.into_iter().map(|a| {
                                                            let iban = a.iban.clone().unwrap_or_default();
                                                            let ccy = a.currency.clone().unwrap_or_default();
                                                            let label = format!("{} — {} ({})", a.name, iban, ccy);
                                                            view! { <option value=a.id.to_string()>{label}</option> }
                                                        }).collect_view()
                                                    }}
                                                </select>
                                            </label>
                                        </Show>
                                    </div>
                                </Show>
                                <label class="text-sm block mt-2">"Срок за плащане (дни)"
                                    <input class=input_cls() type="number" prop:value=move || form_payment_terms.get().to_string()
                                        on:input=move |ev| form_payment_terms.set(event_target_value(&ev).parse().unwrap_or(15)) />
                                </label>
                            </div>
                            <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
                                <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-3">"Изчисления"</h3>
                                <div class="rounded-lg bg-gray-50 border border-gray-100 p-3 text-sm space-y-1 text-gray-700">
                                    {move || {
                                        let ls = lines.get();
                                        let mut by_rate: Vec<(f64, f64, f64)> = Vec::new();
                                        let mut subtotal = 0.0;
                                        for l in &ls {
                                            let net = line_net(l);
                                            let vat = line_vat(l);
                                            let rate = parse_f(&l.vat_rate);
                                            subtotal += net;
                                            if let Some(row) = by_rate.iter_mut().find(|r| (r.0 - rate).abs() < 0.001) {
                                                row.1 += net;
                                                row.2 += vat;
                                            } else {
                                                by_rate.push((rate, net, vat));
                                            }
                                        }
                                        by_rate.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                                        let vat_total: f64 = by_rate.iter().map(|r| r.2).sum();
                                        let total = round2(subtotal + vat_total);
                                        let code = form_currency.get();
                                        let func = company_currency.get();
                                        let rate = form_exchange_rate.get();
                                        let func_total = round2(total * rate);
                                        let show_func = code != func;
                                        let func_label = func.clone();
                                        view! {
                                            <div class="flex justify-between"><span>"Данъчна основа"</span><span>{format!("{:.2} {}", round2(subtotal), code.clone())}</span></div>
                                            {by_rate.into_iter().map(|(r, _b, a)| {
                                                let c = code.clone();
                                                view! { <div class="flex justify-between"><span>{format!("ДДС {}%", r)}</span><span>{format!("{:.2} {}", round2(a), c)}</span></div> }
                                            }).collect_view()}
                                            <div class="flex justify-between font-semibold text-base pt-2 border-t border-gray-200 text-gray-800">
                                                <span>"Обща сума"</span>
                                                <span>{format!("{:.2} {}", total, code)}</span>
                                            </div>
                                            <Show when=move || show_func>
                                                <div class="flex justify-between text-gray-500">
                                                    <span>{format!("Във валута на фирмата ({})", func_label.clone())}</span>
                                                    <span>{format!("{:.2} {}", func_total, func_label.clone())}</span>
                                                </div>
                                            </Show>
                                        }.into_any()
                                    }}
                                </div>
                                <label class="text-sm block mt-3">"Бележки"
                                    <textarea class=input_cls() rows="3"
                                        prop:value=move || form_notes.get()
                                        on:input=move |ev| form_notes.set(event_target_value(&ev))></textarea>
                                </label>
                            </div>
                        </div>

                        </div>
                        <div class="sticky bottom-0 flex justify-end gap-2 px-6 py-3 border-t border-gray-200 bg-white/95 rounded-b-2xl">
                            <button class="px-4 py-2 rounded-lg text-gray-600 hover:bg-gray-100" type="button" on:click=move |_| show_form.set(false)>"Отказ"</button>
                            <button class="px-4 py-2 rounded-lg bg-slate-700 text-white hover:bg-slate-800 disabled:opacity-60" disabled=move || saving.get() type="button"
                                on:click=move |_| {
                                    form_error.set(String::new());
                                    if form_counterpart.get().is_empty() {
                                        form_error.set("Изберете контрагент".into());
                                        return;
                                    }
                                    if form_date.get().is_empty() {
                                        form_error.set("Датата на издаване е задължителна".into());
                                        return;
                                    }
                                    let dt = form_doc_type.get();
                                    if crate::constants::is_corrective_nap(&dt) && form_original.get().is_empty() {
                                        form_error.set("Кредитно или дебитно известие изисква оригинален документ".into());
                                        return;
                                    }
                                    if form_payment.get() == "BANK" {
                                        if form_bank_name.get().is_empty() {
                                            form_error.set("Изберете банка, след това банкова сметка".into());
                                            return;
                                        }
                                        if form_bank_account.get().is_empty() {
                                            form_error.set("Изберете банкова сметка".into());
                                            return;
                                        }
                                    }
                                    let ls = lines.get();
                                    let filled: Vec<&LineDraft> = ls.iter().filter(|l| !l.description.trim().is_empty()).collect();
                                    if filled.is_empty() {
                                        form_error.set("Добавете поне един ред с описание".into());
                                        return;
                                    }
                                    for (i, l) in filled.iter().enumerate() {
                                        if parse_f(&l.vat_rate).abs() < 0.0001 && l.vat_exemption_reason_id.is_none() {
                                            form_error.set(format!("Ред {}: основание за неначисляване на ДДС е задължително при 0%", i + 1));
                                            return;
                                        }
                                    }
                                    let needs_loc = filled.iter().any(|l| {
                                        l.product_id.and_then(|id| products.get().into_iter().find(|p| p.id == id)).map(|p| product_holds_stock(&p)).unwrap_or(false)
                                    });
                                    if needs_loc && form_location.get().is_empty() {
                                        form_error.set("За стоки с наличност изберете обект/склад".into());
                                        return;
                                    }
                                    saving.set(true);
                                    let dir = direction.get();
                                    let edit_id = editing_id.get();
                                    let mut payload = serde_json::json!({
                                        "counterpart_id": form_counterpart.get().parse::<i64>().ok(),
                                        "issue_date": form_date.get(),
                                        "tax_event_date": if form_tax_date.get().is_empty() { form_date.get() } else { form_tax_date.get() },
                                        "due_date": form_due_date.get(),
                                        "document_type": crate::constants::invoice_kind_for_nap(&form_doc_type.get()),
                                        "nap_document_type": if form_doc_type.get() == "PROFORMA" { serde_json::Value::Null } else { serde_json::Value::String(form_doc_type.get()) },
                                        "direction": dir,
                                        "notes": form_notes.get(),
                                        "location_id": form_location.get().parse::<i64>().ok(),
                                        "payment_method": form_payment.get(),
                                        "bank_account_id": if form_payment.get()=="BANK" { serde_json::json!(form_bank_account.get().parse::<i64>().ok()) } else { serde_json::Value::Null },
                                        "currency": form_currency.get(),
                                        "exchange_rate": form_exchange_rate.get(),
                                        "original_invoice_id": form_original.get().parse::<i64>().ok(),
                                        "lines": filled.iter().map(|l| serde_json::json!({
                                            "description": l.description,
                                            "quantity": parse_f(&l.quantity),
                                            "unit_price": parse_f(&l.unit_price),
                                            "vat_rate": parse_f(&l.vat_rate),
                                            "unit_code": l.unit_code,
                                            "product_id": l.product_id,
                                            "account_id": l.account_id,
                                            "vat_exemption_reason_id": l.vat_exemption_reason_id,
                                        })).collect::<Vec<_>>(),
                                    });
                                    if dir == "purchase" {
                                        payload["invoice_number"] = serde_json::Value::String(form_purchase_number.get());
                                    } else if edit_id.is_none() {
                                        payload["invoice_number"] = serde_json::Value::Null;
                                    }
                                    spawn_local(async move {
                                        let result = if let Some(invoice_id) = edit_id {
                                            api_put(&format!("{}/invoices/{}", company_api_url(), invoice_id), &payload).await
                                        } else {
                                            api_post(&format!("{}/invoices", company_api_url()), &payload).await
                                        };
                                        match result {
                                            Ok(_) => {
                                                success_msg.set(if edit_id.is_some() { "Фактурата е записана".into() } else { "Записана чернова".into() });
                                                show_form.set(false);
                                                editing_id.set(None);
                                                reload();
                                            }
                                            Err(e) => form_error.set(e),
                                        }
                                        saving.set(false);
                                    });
                                }
                            >{move || if editing_id.get().is_some() { "Запиши" } else { "Запиши чернова" }}</button>
                        </div>
                    </div>
                </div>
            </Show>
        </Layout>
    }
}
