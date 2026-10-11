use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, company_api_url};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StockRow {
    pub id: i64,
    pub product_id: i64,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub warehouse_name: Option<String>,
    pub qty: f64,
    pub avg_cost: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WhDoc {
    pub id: i64,
    pub document_type: String,
    pub document_number: Option<String>,
    pub issue_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationOpt {
    pub id: i64,
    pub name: String,
    pub holds_stock: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductOpt {
    pub id: i64,
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub measure_unit: Option<String>,
    #[serde(default)]
    pub uom_standard: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct UnitOpt {
    pub code: String,
    #[serde(default)]
    pub name_bg: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
}

#[derive(Debug, Clone)]
struct WhLine {
    id: u64,
    product_id: Option<i64>,
    product_query: String,
    quantity: String,
    unit_code: String,
    unit_price: String,
}

fn today_iso() -> String {
    let d = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", d.get_full_year() as i32, d.get_month() as i32 + 1, d.get_date() as i32)
}

fn parse_f(s: &str) -> f64 {
    s.replace(',', ".").replace(' ', "").parse().unwrap_or(0.0)
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn blank_line(id: u64) -> WhLine {
    WhLine {
        id,
        product_id: None,
        product_query: String::new(),
        quantity: "1".into(),
        unit_code: "C62".into(),
        unit_price: "0".into(),
    }
}

fn product_matches(p: &ProductOpt, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let q = q.to_lowercase();
    p.code.to_lowercase().contains(&q) || p.name.to_lowercase().contains(&q)
}

fn input_cls() -> &'static str {
    "w-full rounded-lg border border-gray-200 bg-white px-3 py-2 text-sm text-gray-800 shadow-sm placeholder-gray-400 focus:border-slate-400 focus:ring-1 focus:ring-slate-400"
}

fn doc_type_label(code: &str) -> String {
    match code {
        "TRANSFER" => "Вътрешен трансфер".into(),
        "RECEIPT" => "Приход / складова разписка".into(),
        "ISSUE" => "Вътрешен разход".into(),
        "SALES_ISSUE" => "Продажба".into(),
        "SALES_RETURN" => "Връщане от клиент".into(),
        "PURCHASE_RETURN" => "Връщане към доставчик".into(),
        other => other.to_string(),
    }
}

#[component]
pub fn Warehouse() -> impl IntoView {
    let balances = RwSignal::new(Vec::<StockRow>::new());
    let docs = RwSignal::new(Vec::<WhDoc>::new());
    let locations = RwSignal::new(Vec::<LocationOpt>::new());
    let products = RwSignal::new(Vec::<ProductOpt>::new());
    let units = RwSignal::new(Vec::<UnitOpt>::new());
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let form_error = RwSignal::new(String::new());
    let show_form = RwSignal::new(false);
    let from_loc = RwSignal::new(String::new());
    let to_loc = RwSignal::new(String::new());
    let doc_type = RwSignal::new(String::from("TRANSFER"));
    let issue_date = RwSignal::new(today_iso());
    let lines = RwSignal::new(vec![blank_line(1)]);
    let next_line_id = RwSignal::new(2u64);
    let product_picker = RwSignal::new(None::<u64>);

    let reload = move || {
        spawn_local(async move {
            if let Ok(data) = api_get(&format!("{}/stock_balances", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    balances.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get(&format!("{}/warehouse_documents", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    docs.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get(&format!("{}/locations", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    locations.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get(&format!("{}/products", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    products.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
            if let Ok(data) = api_get("/api/nomenclatures/units_of_measure?common=true").await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    units.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
        });
    };
    Effect::new(move |_| { reload(); });

    let open_form = move |_| {
        form_error.set(String::new());
        error_msg.set(String::new());
        doc_type.set("TRANSFER".into());
        issue_date.set(today_iso());
        from_loc.set(String::new());
        to_loc.set(String::new());
        lines.set(vec![blank_line(1)]);
        next_line_id.set(2);
        product_picker.set(None);
        show_form.set(true);
    };

    view! {
        <Layout>
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold">"Склад"</h1>
                    <p class="text-sm text-gray-500 mt-1">
                        "За фирми със стоки и наличности. Услуги и малки фирми фактурират само по стойност от Фактури — склад не е задължителен."
                    </p>
                </div>
                <div class="flex gap-2">
                    <a class="px-3 py-2 bg-blue-600 text-white rounded" href="/warehouse">"Наличности"</a>
                    <a class="px-3 py-2 bg-gray-200 rounded" href="/warehouse/scan">"AI сканиране"</a>
                    <button class="px-4 py-2 bg-blue-600 text-white rounded" on:click=open_form>"Нов документ"</button>
                </div>
            </div>
            <p class="text-red-600">{move || error_msg.get()}</p>
            <p class="text-green-600">{move || success_msg.get()}</p>
            <h2 class="text-lg font-semibold mt-4">"Наличности"</h2>
            <table class="w-full bg-white rounded shadow mt-2">
                <thead class="bg-slate-700 text-white"><tr>
                    <th class="p-2 text-left">"Код"</th><th class="p-2 text-left">"Артикул"</th>
                    <th class="p-2 text-left">"Склад"</th><th class="p-2 text-right">"К-во"</th>
                    <th class="p-2 text-right">"Ср. цена"</th>
                </tr></thead>
                <tbody>
                    {move || balances.get().into_iter().map(|b| view! {
                        <tr class="border-b">
                            <td class="p-2">{b.product_code.unwrap_or_default()}</td>
                            <td class="p-2">{b.product_name.unwrap_or_default()}</td>
                            <td class="p-2">{b.warehouse_name.unwrap_or_default()}</td>
                            <td class="p-2 text-right">{format!("{:.3}", b.qty)}</td>
                            <td class="p-2 text-right">{format!("{:.2}", b.avg_cost)}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
            <h2 class="text-lg font-semibold mt-6">"Складови документи"</h2>
            <table class="w-full bg-white rounded shadow mt-2">
                <thead class="bg-slate-700 text-white"><tr>
                    <th class="p-2 text-left">"Номер"</th><th class="p-2 text-left">"Тип"</th>
                    <th class="p-2 text-left">"Дата"</th><th class="p-2 text-left">"Статус"</th>
                </tr></thead>
                <tbody>
                    {move || docs.get().into_iter().map(|d| view! {
                        <tr class="border-b">
                            <td class="p-2">{d.document_number.unwrap_or_else(|| "чернова".into())}</td>
                            <td class="p-2">{doc_type_label(&d.document_type)}</td>
                            <td class="p-2">{d.issue_date}</td>
                            <td class="p-2">{d.status}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
            <Show when=move || show_form.get()>
                <div class="fixed inset-0 bg-slate-900/40 flex items-start justify-center overflow-auto z-50 text-left">
                    <div class="bg-slate-50 rounded-2xl mt-6 mb-10 w-full max-w-6xl border border-gray-200 shadow-xl">
                        <div class="sticky top-0 z-10 flex justify-between items-start gap-4 px-6 py-4 border-b border-gray-200 bg-slate-50/95 rounded-t-2xl">
                            <div>
                                <h2 class="text-xl font-semibold text-gray-800">"Складов документ"</h2>
                                <p class="text-sm text-gray-500 mt-1">"Няколко реда, както при фактурата: артикул, количество, мярка и единична цена."</p>
                            </div>
                            <button class="h-8 w-8 rounded-full bg-gray-200 text-gray-600 hover:bg-gray-300" on:click=move |_| show_form.set(false)>"×"</button>
                        </div>
                        <div class="px-6 py-5 space-y-4">
                            <Show when=move || !form_error.get().is_empty()>
                                <p class="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">{move || form_error.get()}</p>
                            </Show>
                            <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
                                <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-3">"Заглавна информация"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    <label class="text-sm block">"Тип"
                                        <select class=input_cls() prop:value=move || doc_type.get()
                                            on:change=move |ev| doc_type.set(event_target_value(&ev))>
                                            <option value="TRANSFER">"Вътрешен трансфер"</option>
                                            <option value="RECEIPT">"Приход / складова разписка"</option>
                                            <option value="ISSUE">"Вътрешен разход"</option>
                                        </select>
                                    </label>
                                    <label class="text-sm block">"Дата"
                                        <input class=input_cls() type="date" prop:value=move || issue_date.get()
                                            on:input=move |ev| issue_date.set(event_target_value(&ev)) />
                                    </label>
                                    <Show when=move || doc_type.get() != "RECEIPT">
                                        <label class="text-sm block">"От обект"
                                            <select class=input_cls() prop:value=move || from_loc.get()
                                                on:change=move |ev| from_loc.set(event_target_value(&ev))>
                                                <option value="">"—"</option>
                                                {move || locations.get().into_iter().filter(|l| l.holds_stock.unwrap_or(true)).map(|l| view! { <option value=l.id.to_string()>{l.name}</option> }).collect_view()}
                                            </select>
                                        </label>
                                    </Show>
                                    <Show when=move || doc_type.get() != "ISSUE">
                                        <label class="text-sm block">"Към обект"
                                            <select class=input_cls() prop:value=move || to_loc.get()
                                                on:change=move |ev| to_loc.set(event_target_value(&ev))>
                                                <option value="">"—"</option>
                                                {move || locations.get().into_iter().filter(|l| l.holds_stock.unwrap_or(true)).map(|l| view! { <option value=l.id.to_string()>{l.name}</option> }).collect_view()}
                                            </select>
                                        </label>
                                    </Show>
                                </div>
                            </div>

                            <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm">
                                <div class="flex items-center justify-between mb-3">
                                    <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500">"Артикули"</h3>
                                    <button class="text-sm text-slate-700 hover:text-slate-900" type="button" on:click=move |_| {
                                        let id = next_line_id.get();
                                        next_line_id.set(id + 1);
                                        lines.update(|ls| ls.push(blank_line(id)));
                                    }>"+ Ред"</button>
                                </div>
                                <div class="overflow-x-auto mb-2">
                                    <table class="w-full text-sm">
                                        <thead>
                                            <tr class="bg-gray-50 text-gray-500">
                                                <th class="p-2 text-left font-medium">"Артикул"</th>
                                                <th class="p-2 w-24 font-medium">"К-во"</th>
                                                <th class="p-2 w-36 font-medium">"Мярка"</th>
                                                <th class="p-2 w-28 font-medium">"Ед. цена"</th>
                                                <th class="p-2 w-28 text-right font-medium">"Общо"</th>
                                                <th class="p-2 w-10"></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {move || lines.get().into_iter().map(|line| {
                                                let lid = line.id;
                                                let qty = line.quantity.clone();
                                                let price = line.unit_price.clone();
                                                let unit = line.unit_code.clone();
                                                let product_query = line.product_query.clone();
                                                let net = round2(parse_f(&line.quantity) * parse_f(&line.unit_price));
                                                view! {
                                                    <tr class="border-b border-gray-100 align-top">
                                                        <td class="p-2">
                                                            <input class=input_cls() placeholder="Търсене — код или име"
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
                                                                    view! { <p class="mb-1 text-xs text-gray-500">"Няма артикул с този код или име."</p> }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <div class="mb-1 max-h-40 overflow-auto rounded-lg border border-gray-200 bg-white">
                                                                            {matches.into_iter().map(|p| {
                                                                                let id = p.id;
                                                                                let price = p.price;
                                                                                let unit_code = p.uom_standard.clone().filter(|s| !s.is_empty())
                                                                                    .or(p.measure_unit.clone().filter(|s| !s.is_empty()));
                                                                                let label = format!("{} {}", p.code, p.name);
                                                                                let shown = label.clone();
                                                                                view! {
                                                                                    <button class="block w-full text-left px-2 py-1.5 text-sm border-b border-gray-100 hover:bg-gray-50" type="button"
                                                                                        on:click=move |_| {
                                                                                            let label = label.clone();
                                                                                            let unit_code = unit_code.clone();
                                                                                            lines.update(|ls| {
                                                                                                if let Some(l) = ls.iter_mut().find(|x| x.id == lid) {
                                                                                                    l.product_id = Some(id);
                                                                                                    l.product_query = label;
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
                                                            <Show when=move || {
                                                                lines.get().into_iter().find(|l| l.id == lid).and_then(|l| l.product_id).is_none()
                                                                    && !lines.get().into_iter().find(|l| l.id == lid).map(|l| l.product_query.trim().is_empty()).unwrap_or(true)
                                                            }>
                                                                <p class="mt-1 text-xs text-amber-700">"Изберете артикул от списъка."</p>
                                                            </Show>
                                                        </td>
                                                        <td class="p-2">
                                                            <input class=input_cls() prop:value=qty
                                                                on:input=move |ev| lines.update(|ls| { if let Some(l) = ls.iter_mut().find(|x| x.id == lid) { l.quantity = event_target_value(&ev); }}) />
                                                        </td>
                                                        <td class="p-2">
                                                            <select class=input_cls()
                                                                on:change=move |ev| {
                                                                    let val = event_target_value(&ev);
                                                                    lines.update(|ls| { if let Some(l) = ls.iter_mut().find(|x| x.id == lid) { l.unit_code = val; }});
                                                                }>
                                                                {move || {
                                                                    let mut list = units.get();
                                                                    let current = unit.clone();
                                                                    if !current.is_empty() && !list.iter().any(|u| u.code == current) {
                                                                        list.insert(0, UnitOpt { code: current.clone(), name_bg: Some(current.clone()), symbol: None });
                                                                    }
                                                                    if list.is_empty() {
                                                                        view! { <option value="C62" selected=true>"брой (C62)"</option> }.into_any()
                                                                    } else {
                                                                        list.into_iter().map(|u| {
                                                                            let selected = u.code == current;
                                                                            let label = format!("{} ({})", u.name_bg.clone().unwrap_or_else(|| u.code.clone()), u.symbol.clone().unwrap_or_else(|| u.code.clone()));
                                                                            view! { <option value=u.code.clone() selected=selected>{label}</option> }
                                                                        }).collect_view().into_any()
                                                                    }
                                                                }}
                                                            </select>
                                                        </td>
                                                        <td class="p-2">
                                                            <input class=input_cls() prop:value=price
                                                                on:input=move |ev| lines.update(|ls| { if let Some(l) = ls.iter_mut().find(|x| x.id == lid) { l.unit_price = event_target_value(&ev); }}) />
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
                                <p class="text-xs text-gray-500">"При приход единичната цена влиза в средната. При разход и трансфер се ползва средната цена на склада."</p>
                            </div>

                            <div class="flex justify-end gap-2">
                                <button class="px-4 py-2 rounded border border-gray-300" type="button" on:click=move |_| show_form.set(false)>"Отказ"</button>
                                <button class="px-4 py-2 bg-blue-600 text-white rounded" type="button" on:click=move |_| {
                                    form_error.set(String::new());
                                    let dtype = doc_type.get();
                                    let from = from_loc.get();
                                    let to = to_loc.get();
                                    if dtype == "TRANSFER" && (from.is_empty() || to.is_empty()) {
                                        form_error.set("За трансфер изберете обект от и към.".into());
                                        return;
                                    }
                                    if dtype == "TRANSFER" && from == to {
                                        form_error.set("Обектите за трансфер трябва да са различни.".into());
                                        return;
                                    }
                                    if dtype == "RECEIPT" && to.is_empty() {
                                        form_error.set("За приход изберете обект.".into());
                                        return;
                                    }
                                    if dtype == "ISSUE" && from.is_empty() {
                                        form_error.set("За разход изберете обект.".into());
                                        return;
                                    }
                                    let drafts = lines.get();
                                    if drafts.iter().any(|l| l.product_id.is_none() && !l.product_query.trim().is_empty()) {
                                        form_error.set("Изберете артикул от списъка за всеки попълнен ред.".into());
                                        return;
                                    }
                                    let line_json: Vec<serde_json::Value> = drafts.iter().filter_map(|l| {
                                        let pid = l.product_id?;
                                        let qty = parse_f(&l.quantity);
                                        if qty == 0.0 { return None; }
                                        Some(serde_json::json!({
                                            "product_id": pid,
                                            "quantity": qty,
                                            "unit_code": if l.unit_code.trim().is_empty() { "C62".to_string() } else { l.unit_code.clone() },
                                            "unit_price": parse_f(&l.unit_price),
                                        }))
                                    }).collect();
                                    if line_json.is_empty() {
                                        form_error.set("Добавете поне един ред с артикул и количество.".into());
                                        return;
                                    }
                                    let from_id = if dtype == "RECEIPT" { None } else { from.parse::<i64>().ok() };
                                    let to_id = if dtype == "ISSUE" { None } else { to.parse::<i64>().ok() };
                                    let payload = serde_json::json!({
                                        "document_type": dtype,
                                        "issue_date": issue_date.get(),
                                        "from_location_id": from_id,
                                        "to_location_id": to_id,
                                        "lines": line_json,
                                    });
                                    spawn_local(async move {
                                        match api_post(&format!("{}/warehouse_documents", company_api_url()), &payload).await {
                                            Ok(data) => {
                                                if let Some(id) = data.pointer("/data/id").and_then(|v| v.as_i64()) {
                                                    match api_post(&format!("{}/warehouse_documents/{}/post", company_api_url(), id), &serde_json::json!({})).await {
                                                        Ok(_) => {
                                                            success_msg.set("Документът е осчетоводен".into());
                                                            show_form.set(false);
                                                            reload();
                                                        }
                                                        Err(e) => form_error.set(e),
                                                    }
                                                } else {
                                                    form_error.set("Документът не се записа.".into());
                                                }
                                            }
                                            Err(e) => form_error.set(e),
                                        }
                                    });
                                }>"Осчетоводи"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </Layout>
    }
}
