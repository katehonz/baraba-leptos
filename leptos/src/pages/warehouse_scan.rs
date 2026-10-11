use leptos::prelude::*;
use leptos::html;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use web_sys::{DragEvent, FormData, HtmlInputElement};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_put, api_delete, api_post_formdata, company_api_url};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ScanLine {
    pub id: i64,
    #[serde(default)]
    pub line_number: i32,
    pub raw_name: Option<String>,
    pub raw_sku: Option<String>,
    pub raw_barcode: Option<String>,
    #[serde(default)]
    pub quantity: f64,
    pub raw_unit: Option<String>,
    #[serde(default)]
    pub unit_code: String,
    #[serde(default)]
    pub unit_price: f64,
    #[serde(default)]
    pub vat_rate: f64,
    #[serde(default)]
    pub mapping_status: String,
    pub product_id: Option<i64>,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ScanRow {
    pub id: i64,
    #[serde(default)]
    pub direction: String,
    #[serde(default)]
    pub status: String,
    pub original_file_name: Option<String>,
    pub vendor_name: Option<String>,
    pub vendor_vat: Option<String>,
    pub invoice_number: Option<String>,
    pub document_date: Option<String>,
    pub total_amount: Option<f64>,
    pub counterpart_id: Option<i64>,
    pub warehouse_document_id: Option<i64>,
    pub invoice_id: Option<i64>,
    #[serde(default)]
    pub unmapped_count: i64,
    #[serde(default)]
    pub lines: Vec<ScanLine>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MappingRow {
    pub id: i64,
    pub counterpart_id: Option<i64>,
    pub vendor_name: Option<String>,
    pub vendor_sku: Option<String>,
    pub vendor_description: Option<String>,
    pub product_id: i64,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    #[serde(default)]
    pub confirmed: Option<bool>,
    #[serde(default)]
    pub hit_count: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CounterpartOpt {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProductOpt {
    pub id: i64,
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LocationOpt {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub holds_stock: Option<bool>,
}

#[component]
pub fn WarehouseScan() -> impl IntoView {
    let tab = RwSignal::new(0); // 0 scan, 1 mappings
    let scans = RwSignal::new(Vec::<ScanRow>::new());
    let selected = RwSignal::new(Option::<ScanRow>::None);
    let mappings = RwSignal::new(Vec::<MappingRow>::new());
    let counterparts = RwSignal::new(Vec::<CounterpartOpt>::new());
    let products = RwSignal::new(Vec::<ProductOpt>::new());
    let locations = RwSignal::new(Vec::<LocationOpt>::new());
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let uploading = RwSignal::new(false);
    let applying = RwSignal::new(false);
    let direction = RwSignal::new(String::from("inbound"));
    let location_id = RwSignal::new(String::new());
    let file_input_ref = NodeRef::<html::Input>::new();
    let is_dragging = RwSignal::new(false);

    let map_cp = RwSignal::new(String::new());
    let map_sku = RwSignal::new(String::new());
    let map_desc = RwSignal::new(String::new());
    let map_product = RwSignal::new(String::new());
    let lookup_result = RwSignal::new(String::new());
    let product_q = RwSignal::new(String::new());

    let reload_scans = move || {
        spawn_local(async move {
            match api_get(&format!("{}/warehouse_scans", company_api_url())).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let rows: Vec<ScanRow> = arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect();
                        if let Some(cur) = selected.get_untracked() {
                            if let Some(updated) = rows.iter().find(|s| s.id == cur.id).cloned() {
                                selected.set(Some(updated));
                            }
                        }
                        scans.set(rows);
                    }
                }
                Err(e) => error_msg.set(e),
            }
        });
    };

    let reload_aux = move || {
        spawn_local(async move {
            if let Ok(data) = api_get(&format!("{}/vendor_product_mappings", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    mappings.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
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
            if let Ok(data) = api_get(&format!("{}/locations", company_api_url())).await {
                if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                    locations.set(arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect());
                }
            }
        });
    };

    Effect::new(move |_| {
        reload_scans();
        reload_aux();
    });

    let upload_file = move |file: web_sys::File| {
        uploading.set(true);
        error_msg.set(String::new());
        let dir = direction.get();
        spawn_local(async move {
            match FormData::new() {
                Ok(form) => {
                    let blob: web_sys::Blob = file.clone().unchecked_into();
                    let _ = form.append_with_blob_and_filename("file", &blob, &file.name());
                    let _ = form.append_with_str("direction", &dir);
                    match api_post_formdata(&format!("{}/warehouse_scans", company_api_url()), form).await {
                        Ok(body) => {
                            if let Some(d) = body.get("data") {
                                if let Ok(scan) = serde_json::from_value::<ScanRow>(d.clone()) {
                                    selected.set(Some(scan));
                                }
                            }
                            success_msg.set("Документът е разпознат".into());
                            reload_scans();
                        }
                        Err(e) => error_msg.set(e),
                    }
                }
                Err(_) => error_msg.set("Неуспешен FormData".into()),
            }
            uploading.set(false);
        });
    };

    view! {
        <Layout>
            <div class="flex flex-wrap justify-between items-start gap-4">
                <div>
                    <h1 class="text-2xl font-semibold text-gray-800">"AI склад"</h1>
                    <p class="text-sm text-gray-500 mt-1">"Съпоставяне на номенклатурата на доставчика с нашата."</p>
                </div>
                <div class="flex flex-wrap gap-2">
                    <a class="px-3 py-1.5 rounded-full text-sm border border-gray-200 bg-white text-gray-600 hover:bg-gray-50" href="/warehouse">"Наличности"</a>
                    <button class=move || if tab.get()==0 { "px-3 py-1.5 rounded-full text-sm border border-slate-600 bg-slate-700 text-white" } else { "px-3 py-1.5 rounded-full text-sm border border-gray-200 bg-white text-gray-600 hover:bg-gray-50" }
                        on:click=move |_| tab.set(0)>"Сканиране"</button>
                    <button class=move || if tab.get()==1 { "px-3 py-1.5 rounded-full text-sm border border-slate-600 bg-slate-700 text-white" } else { "px-3 py-1.5 rounded-full text-sm border border-gray-200 bg-white text-gray-600 hover:bg-gray-50" }
                        on:click=move |_| tab.set(1)>"Запомнени съответствия"</button>
                </div>
            </div>
            <div class="mt-4 text-sm text-gray-600 bg-white border border-gray-200 rounded-xl p-4 max-w-4xl shadow-sm">
                <p>"Доставчикът пише артикулите по своя номенклатура. При сканиране всеки ред се съпоставя с нашия артикул."</p>
                <p class="mt-1 text-gray-500">"Веднъж ръчно съпоставен (доставчик + SKU/име → наш код) се запомня. Следващият документ от същия доставчик се мапва автоматично."</p>
                <p class="mt-1 text-gray-500">"Отделно от фактурите: фирми от сектор услуги работят само по стойност и нямат материални запаси."</p>
            </div>
            <Show when=move || !error_msg.get().is_empty()>
                <p class="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">{move || error_msg.get()}</p>
            </Show>
            <Show when=move || !success_msg.get().is_empty()>
                <p class="mt-3 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-800">{move || success_msg.get()}</p>
            </Show>

            <Show when=move || tab.get()==0>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4">
                    <div>
                        <div class="bg-white rounded-xl border border-gray-200 shadow-sm p-4">
                            <label class="text-xs font-semibold uppercase tracking-wider text-gray-500 block mb-2">"Посока"
                                <select class="mt-1 w-full rounded-lg border-gray-200 text-sm text-gray-800" on:change=move |ev| direction.set(event_target_value(&ev))>
                                    <option value="inbound">"Входящ (покупка / приход)"</option>
                                    <option value="outbound">"Изходящ (разход)"</option>
                                </select>
                            </label>
                            <input class="hidden" type="file" accept=".pdf,.png,.jpg,.jpeg,.webp" node_ref=file_input_ref
                                on:change=move |ev| {
                                    let input: HtmlInputElement = event_target::<HtmlInputElement>(&ev);
                                    if let Some(files) = input.files() {
                                        if let Some(file) = files.get(0) {
                                            upload_file(file);
                                        }
                                    }
                                    input.set_value("");
                                }
                            />
                            <div
                                class=move || if is_dragging.get() { "border-2 border-dashed border-slate-400 bg-slate-100 p-8 text-center rounded-xl cursor-pointer text-slate-700" } else { "border-2 border-dashed border-gray-200 bg-gray-50 p-8 text-center rounded-xl cursor-pointer text-gray-500 hover:bg-gray-100" }
                                on:click=move |_| { if let Some(el) = file_input_ref.get() { el.click(); } }
                                on:dragover=move |ev: DragEvent| { ev.prevent_default(); is_dragging.set(true); }
                                on:dragleave=move |ev: DragEvent| { ev.prevent_default(); is_dragging.set(false); }
                                on:drop=move |ev: DragEvent| {
                                    ev.prevent_default();
                                    is_dragging.set(false);
                                    if let Some(dt) = ev.data_transfer() {
                                        if let Some(files) = dt.files() {
                                            if let Some(file) = files.get(0) {
                                                upload_file(file);
                                            }
                                        }
                                    }
                                }
                            >
                                {move || if uploading.get() { "Разпознаване…" } else { "Пуснете PDF/JPG тук или кликнете за избор" }}
                            </div>
                        </div>

                        <h2 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mt-4 mb-2">"Сканирания"</h2>
                        <table class="w-full bg-white rounded-xl border border-gray-200 shadow-sm text-sm overflow-hidden">
                            <thead class="bg-gray-100 text-gray-600"><tr>
                                <th class="p-2.5 text-left font-medium">"Файл"</th>
                                <th class="p-2.5 text-left font-medium">"Доставчик"</th>
                                <th class="p-2.5 text-left font-medium">"Статус"</th>
                            </tr></thead>
                            <tbody>
                                {move || scans.get().into_iter().map(|s| {
                                    let row = s.clone();
                                    let active = selected.get().map(|cur| cur.id == s.id).unwrap_or(false);
                                    view! {
                                        <tr class=move || if active { "border-b border-gray-100 cursor-pointer bg-slate-100" } else { "border-b border-gray-100 cursor-pointer hover:bg-gray-50" } on:click=move |_| selected.set(Some(row.clone()))>
                                            <td class="p-2.5 text-gray-800">{s.original_file_name.clone().unwrap_or_default()}</td>
                                            <td class="p-2.5 text-gray-600">{s.vendor_name.clone().unwrap_or_default()}</td>
                                            <td class="p-2.5 text-gray-500">{s.status.clone()} {if s.unmapped_count>0 { format!(" ({} несъпост.)", s.unmapped_count) } else { String::new() }}</td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>

                    <div>
                        {move || selected.get().map(|scan| {
                            let scan_id = scan.id;
                            let can_apply = scan.status != "applied" && scan.status != "rejected";
                            view! {
                                <div class="bg-white rounded-xl border border-gray-200 shadow-sm p-4">
                                    <h2 class="font-semibold text-gray-800 mb-1">{scan.original_file_name.clone().unwrap_or_else(|| format!("#{}", scan.id))}</h2>
                                    <p class="text-sm text-gray-500">"№ " {scan.invoice_number.clone().unwrap_or_default()} " · " {scan.document_date.clone().unwrap_or_default()}</p>
                                    <p class="text-sm text-gray-600 mt-1">"Доставчик: " {scan.vendor_name.clone().unwrap_or_default()} " " {scan.vendor_vat.clone().unwrap_or_default()}</p>
                                    <label class="text-xs font-semibold uppercase tracking-wider text-gray-500 block mt-3">"Контрагент"
                                        <select class="mt-1 w-full rounded-lg border-gray-200 text-sm" on:change=move |ev| {
                                            let val = event_target_value(&ev);
                                            spawn_local(async move {
                                                let _ = api_put(&format!("{}/warehouse_scans/{}", company_api_url(), scan_id), &serde_json::json!({"counterpart_id": val.parse::<i64>().ok()})).await;
                                                reload_scans();
                                            });
                                        }>
                                            <option value="">"— изберете —"</option>
                                            {move || counterparts.get().into_iter().map(|c| {
                                                let sel = scan.counterpart_id == Some(c.id);
                                                view! { <option value=c.id.to_string() selected=sel>{c.name}</option> }
                                            }).collect_view()}
                                        </select>
                                    </label>
                                    <label class="text-xs font-semibold uppercase tracking-wider text-gray-500 block mt-3">"Обект (склад)"
                                        <select class="mt-1 w-full rounded-lg border-gray-200 text-sm" on:change=move |ev| location_id.set(event_target_value(&ev))>
                                            <option value="">"—"</option>
                                            {move || locations.get().into_iter().filter(|l| l.holds_stock.unwrap_or(true)).map(|l| view! { <option value=l.id.to_string()>{l.name}</option> }).collect_view()}
                                        </select>
                                    </label>

                                    <div class="flex items-center justify-between mt-4 gap-2">
                                        <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500">"Редове — тяхна → наша"</h3>
                                        <input class="rounded-lg border-gray-200 text-sm w-48" placeholder="Търси наш артикул"
                                            prop:value=move || product_q.get()
                                            on:input=move |ev| product_q.set(event_target_value(&ev)) />
                                    </div>
                                    {scan.lines.clone().into_iter().map(|line| {
                                        let lid = line.id;
                                        let pid = line.product_id;
                                        let status = line.mapping_status.clone();
                                        let status_label = match status.as_str() {
                                            "auto" => "автоматично (запомнено)",
                                            "manual" => "ръчно — запомнено",
                                            _ => "несъпоставено",
                                        };
                                        let status_cls = match status.as_str() {
                                            "auto" => "inline-flex rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600",
                                            "manual" => "inline-flex rounded-full bg-slate-100 px-2 py-0.5 text-xs text-slate-700",
                                            _ => "inline-flex rounded-full bg-amber-50 px-2 py-0.5 text-xs text-amber-800",
                                        };
                                        let card_cls = match status.as_str() {
                                            "auto" => "rounded-lg border border-gray-200 bg-gray-50 p-3 mt-2 text-sm",
                                            "manual" => "rounded-lg border border-gray-200 bg-white p-3 mt-2 text-sm",
                                            _ => "rounded-lg border border-amber-100 bg-amber-50/40 p-3 mt-2 text-sm",
                                        };
                                        let vendor_sku = line.raw_sku.clone().unwrap_or_default();
                                        let vendor_name = line.raw_name.clone().unwrap_or_default();
                                        let unit_code = line.unit_code.clone();
                                        let unit_price = line.unit_price;
                                        let unmapped = status == "unmapped";
                                        view! {
                                            <div class=card_cls>
                                                <div class="flex justify-between gap-2">
                                                    <div class="font-medium text-gray-800">{vendor_name.clone()}</div>
                                                    <span class=status_cls>{status_label}</span>
                                                </div>
                                                <div class="text-gray-500 mt-0.5">{format!("техен SKU {} · {} {} · {:.2} · ДДС {:.0}%", vendor_sku.clone(), line.quantity, line.unit_code, line.unit_price, line.vat_rate)}</div>
                                                <Show when=move || pid.is_some()>
                                                    <div class="text-slate-700 mt-1">"наш: " {line.product_code.clone().unwrap_or_default()} " " {line.product_name.clone().unwrap_or_default()}</div>
                                                </Show>
                                                <select class="w-full rounded-lg border-gray-200 text-sm mt-2" on:change=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    spawn_local(async move {
                                                        let product_id = val.parse::<i64>().ok();
                                                        match api_put(&format!("{}/warehouse_scans/{}/lines/{}", company_api_url(), scan_id, lid), &serde_json::json!({"product_id": product_id})).await {
                                                            Ok(_) => {
                                                                success_msg.set("Съпоставено и запомнено за този доставчик".into());
                                                                reload_scans();
                                                                reload_aux();
                                                            }
                                                            Err(e) => error_msg.set(e),
                                                        }
                                                    });
                                                }>
                                                    <option value="">"— съпостави с наш артикул —"</option>
                                                    {move || {
                                                        let q = product_q.get().to_lowercase();
                                                        products.get().into_iter().filter(|p| {
                                                            q.is_empty()
                                                                || p.code.to_lowercase().contains(&q)
                                                                || p.name.to_lowercase().contains(&q)
                                                        }).map(|p| {
                                                            let selected = pid == Some(p.id);
                                                            view! { <option value=p.id.to_string() selected=selected>{format!("{} {}", p.code, p.name)}</option> }
                                                        }).collect_view()
                                                    }}
                                                </select>
                                                <Show when=move || unmapped>
                                                    <button class="mt-2 text-sm text-slate-700 hover:text-slate-900" on:click={
                                                        let vendor_sku = vendor_sku.clone();
                                                        let vendor_name = vendor_name.clone();
                                                        let unit_code = unit_code.clone();
                                                        move |_| {
                                                        let sku = vendor_sku.clone();
                                                        let vname = vendor_name.clone();
                                                        let uc = unit_code.clone();
                                                        let code = if sku.trim().is_empty() { format!("WS{}", lid) } else { sku };
                                                        let name = if vname.trim().is_empty() { code.clone() } else { vname };
                                                        spawn_local(async move {
                                                            match api_put(&format!("{}/warehouse_scans/{}/lines/{}", company_api_url(), scan_id, lid), &serde_json::json!({
                                                                "create_product": {
                                                                    "code": code,
                                                                    "name": name,
                                                                    "unit_code": uc,
                                                                    "price": unit_price
                                                                }
                                                            })).await {
                                                                Ok(_) => {
                                                                    success_msg.set("Създаден е наш артикул и съответствието е запомнено".into());
                                                                    reload_scans();
                                                                    reload_aux();
                                                                }
                                                                Err(e) => error_msg.set(e),
                                                            }
                                                        });
                                                        }
                                                    }>"Създай наш артикул от техния ред и запомни"</button>
                                                </Show>
                                            </div>
                                        }
                                    }).collect_view()}

                                    <Show when=move || can_apply>
                                        <div class="flex gap-2 mt-4">
                                            <button class="px-3 py-2 rounded-lg bg-slate-700 text-white text-sm hover:bg-slate-800 disabled:opacity-60" disabled=move || applying.get()
                                                on:click=move |_| {
                                                    applying.set(true);
                                                    let loc = location_id.get().parse::<i64>().ok();
                                                    spawn_local(async move {
                                                        match api_post(&format!("{}/warehouse_scans/{}/apply", company_api_url(), scan_id), &serde_json::json!({
                                                            "create_invoice": true,
                                                            "location_id": loc
                                                        })).await {
                                                            Ok(_) => { success_msg.set("Създадена е чернова на фактура".into()); reload_scans(); }
                                                            Err(e) => error_msg.set(e),
                                                        }
                                                        applying.set(false);
                                                    });
                                                }
                                            >"Създай фактура (чернова)"</button>
                                            <button class="px-3 py-2 rounded-lg bg-gray-100 text-gray-700 text-sm hover:bg-gray-200 disabled:opacity-60" disabled=move || applying.get()
                                                on:click=move |_| {
                                                    applying.set(true);
                                                    let loc = location_id.get().parse::<i64>().ok();
                                                    spawn_local(async move {
                                                        match api_post(&format!("{}/warehouse_scans/{}/apply", company_api_url(), scan_id), &serde_json::json!({
                                                            "create_invoice": false,
                                                            "create_orphan_receipt": true,
                                                            "location_id": loc
                                                        })).await {
                                                            Ok(_) => { success_msg.set("Създаден е складов документ".into()); reload_scans(); }
                                                            Err(e) => error_msg.set(e),
                                                        }
                                                        applying.set(false);
                                                    });
                                                }
                                            >"Само складова разписка"</button>
                                        </div>
                                    </Show>
                                    {scan.invoice_id.map(|id| view! { <p class="text-sm mt-2">"Фактура #" {id.to_string()}</p> })}
                                    {scan.warehouse_document_id.map(|id| view! { <p class="text-sm">"Складов документ #" {id.to_string()}</p> })}
                                </div>
                            }
                        })}
                    </div>
                </div>
            </Show>

            <Show when=move || tab.get()==1>
                <div class="bg-white rounded-xl border border-gray-200 shadow-sm p-4 mt-4 max-w-2xl">
                    <h2 class="font-semibold text-gray-800 mb-1">"Запомнени съответствия"</h2>
                    <p class="text-sm text-gray-500 mb-3">"Техен SKU/име при този доставчик → наш артикул. Ръчното съпоставяне при сканиране се записва тук и следващият път е автоматично."</p>
                    <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-2">"Ръчно запомни"</h3>
                    <label class="text-sm text-gray-600 block mb-2">"Доставчик"
                        <select class="mt-1 w-full rounded-lg border-gray-200 text-sm" on:change=move |ev| map_cp.set(event_target_value(&ev))>
                            <option value="">"— всички —"</option>
                            {move || counterparts.get().into_iter().map(|c| view! { <option value=c.id.to_string()>{c.name}</option> }).collect_view()}
                        </select>
                    </label>
                    <label class="text-sm text-gray-600 block mb-2">"SKU"
                        <input class="mt-1 w-full rounded-lg border-gray-200 text-sm" prop:value=move || map_sku.get() on:input=move |ev| map_sku.set(event_target_value(&ev)) />
                    </label>
                    <label class="text-sm text-gray-600 block mb-2">"Описание"
                        <input class="mt-1 w-full rounded-lg border-gray-200 text-sm" prop:value=move || map_desc.get() on:input=move |ev| map_desc.set(event_target_value(&ev)) />
                    </label>
                    <label class="text-sm text-gray-600 block mb-2">"Наш артикул"
                        <select class="mt-1 w-full rounded-lg border-gray-200 text-sm" on:change=move |ev| map_product.set(event_target_value(&ev))>
                            <option value="">"—"</option>
                            {move || products.get().into_iter().map(|p| view! { <option value=p.id.to_string()>{format!("{} {}", p.code, p.name)}</option> }).collect_view()}
                        </select>
                    </label>
                    <p class="text-sm">{move || lookup_result.get()}</p>
                    <div class="flex gap-2 mt-2">
                        <button class="px-4 py-2 rounded-lg bg-gray-100 text-gray-700 hover:bg-gray-200" on:click=move |_| {
                            let payload = serde_json::json!({
                                "vendor_sku": map_sku.get(),
                                "vendor_description": map_desc.get(),
                                "counterpart_id": map_cp.get().parse::<i64>().ok(),
                            });
                            spawn_local(async move {
                                match api_post(&format!("{}/warehouse_scans/lookup", company_api_url()), &payload).await {
                                    Ok(body) => {
                                        if let Some(pid) = body.pointer("/data/product_id").and_then(|v| v.as_i64()) {
                                            map_product.set(pid.to_string());
                                            lookup_result.set(format!("Намерен артикул #{}", pid));
                                        } else {
                                            lookup_result.set("Няма съответствие".into());
                                        }
                                    }
                                    Err(e) => error_msg.set(e),
                                }
                            });
                        }>"Търси"</button>
                        <button class="px-4 py-2 rounded-lg bg-slate-700 text-white hover:bg-slate-800" on:click=move |_| {
                            let Some(pid) = map_product.get().parse::<i64>().ok() else {
                                error_msg.set("Изберете артикул".into());
                                return;
                            };
                            let payload = serde_json::json!({
                                "product_id": pid,
                                "counterpart_id": map_cp.get().parse::<i64>().ok(),
                                "vendor_sku": map_sku.get(),
                                "vendor_description": map_desc.get(),
                            });
                            spawn_local(async move {
                                match api_post(&format!("{}/warehouse_scans/remember", company_api_url()), &payload).await {
                                    Ok(_) => { success_msg.set("Запомнено".into()); reload_aux(); }
                                    Err(e) => error_msg.set(e),
                                }
                            });
                        }>"Запомни"</button>
                    </div>
                </div>
                <table class="w-full bg-white rounded-xl border border-gray-200 shadow-sm mt-4 text-sm overflow-hidden">
                    <thead class="bg-gray-100 text-gray-600"><tr>
                        <th class="p-2.5 text-left font-medium">"Доставчик"</th>
                        <th class="p-2.5 text-left font-medium">"Техен SKU"</th>
                        <th class="p-2.5 text-left font-medium">"Тяхно име"</th>
                        <th class="p-2.5 text-left font-medium">"Наш артикул"</th>
                        <th class="p-2.5 text-left font-medium">"Пъти"</th>
                        <th class="p-2.5"></th>
                    </tr></thead>
                    <tbody>
                        {move || {
                            let cps = counterparts.get();
                            mappings.get().into_iter().map(|m| {
                            let mid = m.id;
                            let cp_name = cps.iter().find(|c| Some(c.id)==m.counterpart_id).map(|c| c.name.clone()).unwrap_or_default();
                            view! {
                                <tr class="border-b border-gray-100 hover:bg-gray-50">
                                    <td class="p-2.5 text-gray-800">{cp_name}</td>
                                    <td class="p-2.5 text-gray-600">{m.vendor_sku.unwrap_or_default()}</td>
                                    <td class="p-2.5 text-gray-600">{m.vendor_description.or(m.vendor_name).unwrap_or_default()}</td>
                                    <td class="p-2.5 text-gray-800">{format!("{} {}", m.product_code.unwrap_or_default(), m.product_name.unwrap_or_default())}</td>
                                    <td class="p-2.5 text-gray-500">{m.hit_count.unwrap_or(0).to_string()}</td>
                                    <td class="p-2.5">
                                        <button class="text-gray-500 hover:text-red-700 text-sm" on:click=move |_| {
                                            spawn_local(async move {
                                                let _ = api_delete(&format!("{}/vendor_product_mappings/{}", company_api_url(), mid)).await;
                                                reload_aux();
                                            });
                                        }>"Изтрий"</button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()
                        }}
                    </tbody>
                </table>
            </Show>
        </Layout>
    }
}
