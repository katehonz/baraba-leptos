use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use wasm_bindgen_futures::spawn_local;
use serde::Deserialize;
use crate::api::{api_get, company_api_url};
use crate::bg_amount_words;

#[derive(Debug, Clone, Default, Deserialize)]
struct PrintLine {
    #[serde(default)]
    description: String,
    #[serde(default)]
    quantity: f64,
    #[serde(default)]
    unit_price: f64,
    #[serde(default)]
    unit_code: Option<String>,
    #[serde(default)]
    unit_symbol: Option<String>,
    #[serde(default)]
    vat_rate: f64,
    #[serde(default)]
    vat_amount: f64,
    #[serde(default)]
    line_total: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct InvoiceShow {
    #[serde(default)]
    invoice_number: Option<String>,
    #[serde(default)]
    document_type: Option<String>,
    #[serde(default)]
    nap_document_type: Option<String>,
    #[serde(default)]
    direction: Option<String>,
    #[serde(default)]
    issue_date: Option<String>,
    #[serde(default)]
    tax_event_date: Option<String>,
    #[serde(default)]
    due_date: Option<String>,
    #[serde(default)]
    total_amount: Option<f64>,
    #[serde(default)]
    vat_amount: Option<f64>,
    #[serde(default)]
    subtotal: Option<f64>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    payment_method: Option<String>,
    #[serde(default)]
    vat_exemption_reason: Option<String>,
    #[serde(default)]
    counterpart_name: Option<String>,
    #[serde(default)]
    counterpart_eik: Option<String>,
    #[serde(default)]
    counterpart_vat: Option<String>,
    #[serde(default)]
    counterpart_address: Option<String>,
    #[serde(default)]
    counterpart_city: Option<String>,
    #[serde(default)]
    counterpart_contact: Option<String>,
    #[serde(default)]
    company_name: Option<String>,
    #[serde(default)]
    company_eik: Option<String>,
    #[serde(default)]
    company_vat: Option<String>,
    #[serde(default)]
    company_address: Option<String>,
    #[serde(default)]
    company_city: Option<String>,
    #[serde(default)]
    company_post_code: Option<String>,
    #[serde(default)]
    company_mol: Option<String>,
    #[serde(default)]
    location_name: Option<String>,
    #[serde(default)]
    bank_iban: Option<String>,
    #[serde(default)]
    bank_name: Option<String>,
    #[serde(default)]
    bank_bic: Option<String>,
    #[serde(default)]
    original_invoice_number: Option<String>,
    #[serde(default)]
    original_invoice_date: Option<String>,
    #[serde(default)]
    amount_in_words: Option<String>,
    #[serde(default)]
    lines: Vec<PrintLine>,
}

fn doc_title(doc_type: &str, nap: Option<&str>, direction: &str) -> String {
    if doc_type != "PROFORMA" {
        if let Some(code) = nap.filter(|s| !s.is_empty()) {
            let journal = if direction == "purchase" { "purchase" } else { "sales" };
            if let Some(name) = crate::constants::get_doc_type_name(code, journal) {
                return name.to_uppercase();
            }
        }
    }
    match doc_type {
        "CREDIT_NOTE" => "КРЕДИТНО ИЗВЕСТИЕ",
        "DEBIT_NOTE" => "ДЕБИТНО ИЗВЕСТИЕ",
        "PROFORMA" => "ПРОФОРМА ФАКТУРА",
        "PROTOCOL" => "ПРОТОКОЛ",
        "SALES_REPORT" => "ОТЧЕТ ЗА ИЗВЪРШЕНИТЕ ПРОДАЖБИ",
        _ => "ФАКТУРА",
    }.to_string()
}

fn number_prefix(doc_type: &str) -> &'static str {
    match doc_type {
        "CREDIT_NOTE" => "КИ-",
        "DEBIT_NOTE" => "ДИ-",
        "PROFORMA" => "ПФ-",
        _ => "",
    }
}

fn unit_label(line: &PrintLine) -> String {
    if let Some(s) = line.unit_symbol.as_ref().filter(|s| !s.is_empty()) {
        return s.clone();
    }
    match line.unit_code.as_deref().unwrap_or("C62") {
        "C62" => "бр.".into(),
        "KGM" => "кг".into(),
        "LTR" => "л".into(),
        "MTR" => "м".into(),
        "MTK" => "м²".into(),
        "MTQ" => "м³".into(),
        other => other.to_string(),
    }
}

#[component]
pub fn InvoicePrint() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let data = RwSignal::new(Option::<InvoiceShow>::None);
    let load_error = RwSignal::new(String::new());

    Effect::new(move |_| {
        let id = params.with(|p| p.get("id").unwrap_or_default());
        spawn_local(async move {
            match api_get(&format!("{}/invoices/{}", company_api_url(), id)).await {
                Ok(body) => {
                    if let Some(d) = body.get("data") {
                        data.set(serde_json::from_value(d.clone()).ok());
                    }
                }
                Err(e) => load_error.set(e),
            }
        });
    });

    view! {
        <div class="print-invoice-page p-8 max-w-4xl mx-auto bg-white text-black" style="font-family: 'Courier New', Courier, monospace; font-size: 13px; position: relative;">
            <div class="no-print mb-4 flex gap-2 items-center">
                <a class="px-3 py-1 bg-gray-200 rounded" href="/invoices">"Назад"</a>
                <button class="px-4 py-2 bg-blue-600 text-white rounded" on:click=move |_| { let _ = window().print(); }>"Печат"</button>
                {move || {
                    let id = params.with(|p| p.get("id").unwrap_or_default());
                    view! {
                        <a class="px-3 py-1 bg-gray-100 rounded border" href=format!("/invoices/{}/print", id)>"Оригинал"</a>
                        <a class="px-3 py-1 bg-gray-100 rounded border" href=format!("/invoices/{}/print?copy=1", id)>"Копие"</a>
                    }
                }}
            </div>
            <p class="text-red-600">{move || load_error.get()}</p>
            {move || {
                let is_copy = query.with(|q| {
                    q.get("copy").map(|v| v == "1" || v == "true").unwrap_or(false)
                });
                data.get().map(|inv| {
                    let doc_type = inv.document_type.clone().unwrap_or_else(|| "INVOICE".into());
                    let status = inv.status.clone().unwrap_or_default();
                    let purchase = inv.direction.as_deref() == Some("purchase");
                    let number = format!(
                        "{}{}",
                        number_prefix(&doc_type),
                        inv.invoice_number.clone().unwrap_or_else(|| "чернова".into())
                    );
                    let currency = inv.currency.clone().unwrap_or_else(|| "BGN".into());
                    let subtotal = inv.subtotal.unwrap_or(0.0);
                    let vat = inv.vat_amount.unwrap_or(0.0);
                    let total = inv.total_amount.unwrap_or(0.0);
                    let words = inv.amount_in_words.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| {
                        bg_amount_words::convert(total, &currency)
                    });
                    let vat_rate_label = inv.lines.first().map(|l| l.vat_rate).unwrap_or(20.0);
                    let supplier_name = if purchase { inv.counterpart_name.clone() } else { inv.company_name.clone() };
                    let supplier_addr = if purchase { inv.counterpart_address.clone() } else { inv.company_address.clone() };
                    let supplier_city = if purchase { inv.counterpart_city.clone() } else { inv.company_city.clone() };
                    let supplier_eik = if purchase { inv.counterpart_eik.clone() } else { inv.company_eik.clone() };
                    let supplier_vat = if purchase { inv.counterpart_vat.clone() } else { inv.company_vat.clone() };
                    let buyer_name = if purchase { inv.company_name.clone() } else { inv.counterpart_name.clone() };
                    let buyer_addr = if purchase { inv.company_address.clone() } else { inv.counterpart_address.clone() };
                    let buyer_city = if purchase { inv.company_city.clone() } else { inv.counterpart_city.clone() };
                    let buyer_eik = if purchase { inv.company_eik.clone() } else { inv.counterpart_eik.clone() };
                    let buyer_vat = if purchase { inv.company_vat.clone() } else { inv.counterpart_vat.clone() };
                    let watermark = if status == "cancelled" {
                        Some("АНУЛИРАНА")
                    } else if doc_type == "PROFORMA" {
                        Some("ПРОФОРМА")
                    } else {
                        None
                    };
                    let lines = inv.lines.clone();
                    view! {
                        <div style="position: relative;">
                            {watermark.map(|w| view! {
                                <div style="position:absolute;top:40%;left:50%;transform:translate(-50%,-50%) rotate(-35deg);font-size:72px;font-weight:bold;color:rgba(200,0,0,0.18);border:6px dashed rgba(200,0,0,0.18);padding:8px 32px;pointer-events:none;z-index:1;white-space:nowrap;">
                                    {w}
                                </div>
                            })}
                            <div style="display:flex;justify-content:space-between;align-items:flex-start;border-bottom:2px solid #000;padding-bottom:8px;margin-bottom:16px;">
                                <div>
                                    <div style="font-size:16px;font-weight:bold;text-transform:uppercase;">{doc_title(&doc_type, inv.nap_document_type.as_deref(), inv.direction.as_deref().unwrap_or("sale"))}</div>
                                    <div style="font-size:11px;">{if is_copy { "( КОПИЕ )" } else { "( ОРИГИНАЛ )" }}</div>
                                </div>
                                <div style="font-size:16px;font-weight:bold;">"№ " {number}</div>
                                <div style="text-align:right;">
                                    <div>"Дата: " {inv.issue_date.clone().unwrap_or_default()}</div>
                                    {inv.tax_event_date.clone().map(|d| view! { <div>"Дан. събитие: " {d}</div> })}
                                </div>
                            </div>

                            <div style="display:flex;gap:16px;margin-bottom:16px;">
                                <div style="flex:1;border:1px solid #000;padding:10px;">
                                    <div style="font-weight:bold;border-bottom:1px solid #000;display:inline-block;margin-bottom:6px;">"ПОЛУЧАТЕЛ"</div>
                                    <div>{buyer_name.unwrap_or_default()}</div>
                                    <div>{buyer_addr.unwrap_or_default()}</div>
                                    <div>{buyer_city.unwrap_or_default()}</div>
                                    <div>"ЕИК: " {buyer_eik.unwrap_or_else(|| "—".into())}</div>
                                    <div>"ДДС №: " {buyer_vat.unwrap_or_else(|| "—".into())}</div>
                                </div>
                                <div style="flex:1;border:1px solid #000;padding:10px;">
                                    <div style="font-weight:bold;border-bottom:1px solid #000;display:inline-block;margin-bottom:6px;">"ДОСТАВЧИК"</div>
                                    <div>{supplier_name.unwrap_or_default()}</div>
                                    <div>{supplier_addr.unwrap_or_default()}</div>
                                    <div>{supplier_city.clone().unwrap_or_default()} " " {inv.company_post_code.clone().filter(|_| !purchase).unwrap_or_default()}</div>
                                    <div>"ЕИК: " {supplier_eik.unwrap_or_else(|| "—".into())}</div>
                                    <div>"ДДС №: " {supplier_vat.unwrap_or_else(|| "—".into())}</div>
                                    {inv.company_mol.clone().filter(|s| !s.is_empty() && !purchase).map(|mol| view! { <div>"МОЛ: " {mol}</div> })}
                                    {inv.location_name.clone().filter(|s| !s.is_empty()).map(|n| view! { <div>"Обект: " {n}</div> })}
                                </div>
                            </div>

                            {inv.original_invoice_number.clone().filter(|s| !s.is_empty()).map(|n| {
                                let d = inv.original_invoice_date.clone().unwrap_or_default();
                                view! {
                                    <p style="margin-bottom:10px;">"Към документ № " {n} " / " {d}</p>
                                }
                            })}

                            <table style="width:100%;border-collapse:collapse;margin-bottom:12px;">
                                <thead>
                                    <tr>
                                        <th style="border-bottom:1px solid #000;text-align:left;padding:4px;width:5%;">"№"</th>
                                        <th style="border-bottom:1px solid #000;text-align:left;padding:4px;width:40%;">"Стока / услуга"</th>
                                        <th style="border-bottom:1px solid #000;text-align:right;padding:4px;">"К-во"</th>
                                        <th style="border-bottom:1px solid #000;text-align:right;padding:4px;">"Ед. цена"</th>
                                        <th style="border-bottom:1px solid #000;text-align:right;padding:4px;">"ДДС %"</th>
                                        <th style="border-bottom:1px solid #000;text-align:right;padding:4px;">"Сума"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {lines.into_iter().enumerate().map(|(i, line)| {
                                        let unit = unit_label(&line);
                                        view! {
                                            <tr>
                                                <td style="padding:3px 4px;vertical-align:top;">{(i + 1).to_string()}</td>
                                                <td style="padding:3px 4px;">{line.description.clone()}</td>
                                                <td style="padding:3px 4px;text-align:right;white-space:nowrap;">{format!("{:.3} {}", line.quantity, unit)}</td>
                                                <td style="padding:3px 4px;text-align:right;">{format!("{:.2}", line.unit_price)}</td>
                                                <td style="padding:3px 4px;text-align:right;">{format!("{:.0}", line.vat_rate)}</td>
                                                <td style="padding:3px 4px;text-align:right;">{format!("{:.2}", line.line_total)}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>

                            <div style="display:flex;border-top:2px solid #000;padding-top:10px;gap:16px;">
                                <div style="flex:1;">
                                    {inv.bank_iban.clone().filter(|s| !s.is_empty()).map(|iban| {
                                        let bank = inv.bank_name.clone().unwrap_or_default();
                                        let bic = inv.bank_bic.clone().unwrap_or_default();
                                        view! {
                                            <div>
                                                <div style="font-weight:bold;">"БАНКОВА СМЕТКА"</div>
                                                <div>{bank}</div>
                                                <div>"IBAN: " {iban}</div>
                                                <div>"BIC: " {bic}</div>
                                            </div>
                                        }
                                    })}
                                    {inv.due_date.clone().map(|d| view! { <div style="margin-top:8px;">"Срок за плащане: " {d}</div> })}
                                    {inv.payment_method.clone().map(|m| view! { <div>"Начин на плащане: " {m}</div> })}
                                    {inv.vat_exemption_reason.clone().filter(|s| !s.is_empty()).map(|r| view! {
                                        <div style="margin-top:8px;"><b>"Основание за 0% ДДС: "</b>{r}</div>
                                    })}
                                    {inv.notes.clone().filter(|s| !s.is_empty()).map(|n| view! {
                                        <div style="margin-top:8px;font-style:italic;">"Забележка: " {n}</div>
                                    })}
                                </div>
                                <div style="flex:1;">
                                    <div style="display:flex;justify-content:space-between;"><span>"Данъчна основа:"</span><span>{format!("{:.2} {}", subtotal, currency)}</span></div>
                                    <div style="display:flex;justify-content:space-between;"><span>{format!("ДДС {:.0}%:", vat_rate_label)}</span><span>{format!("{:.2} {}", vat, currency)}</span></div>
                                    <div style="display:flex;justify-content:space-between;font-weight:bold;font-size:15px;border-top:1px dashed #000;margin-top:6px;padding-top:6px;">
                                        <span>"ОБЩО ЗА ПЛАЩАНЕ:"</span>
                                        <span>{format!("{:.2} {}", total, currency)}</span>
                                    </div>
                                    <div style="margin-top:10px;font-size:11px;">"Словом: " {words}</div>
                                </div>
                            </div>

                            <div style="display:flex;justify-content:space-between;margin-top:48px;">
                                <div>
                                    <div>"Получил: " {inv.counterpart_contact.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| "....................".into())}</div>
                                    <div style="font-size:10px;margin-left:70px;">"(подпис)"</div>
                                </div>
                                <div>
                                    <div>"Съставил: ...................."</div>
                                    <div style="font-size:10px;margin-left:80px;">"(подпис)"</div>
                                </div>
                            </div>
                        </div>
                    }
                })
            }}
        </div>
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}
