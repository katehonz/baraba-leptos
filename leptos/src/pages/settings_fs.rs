use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use crate::api::{api_get, api_put, api_post, company_api_url};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FsLine {
    pub code: String,
    #[serde(default)]
    pub parent: String,
    pub kind: String,
    pub side: String,
    pub name: String,
    #[serde(default)]
    pub formula: String,
    #[serde(default)]
    pub current: f64,
    #[serde(default)]
    pub prior: f64,
}

const STATEMENTS: [(&str, &str); 4] = [
    ("balance", "Баланс"),
    ("pl", "ОПР"),
    ("cashflow", "Паричен поток"),
    ("equity", "Собствен капитал"),
];

fn indent_for(code: &str, parent_of: &[(String, String)]) -> String {
    let mut d = 0i32;
    let mut cur = code.to_string();
    for _ in 0..12 {
        if let Some((_, p)) = parent_of.iter().find(|(c, _)| c == &cur) {
            if p.is_empty() {
                break;
            }
            d += 1;
            cur = p.clone();
        } else {
            break;
        }
    }
    "\u{00a0}\u{00a0}".repeat(d as usize)
}

#[component]
pub fn FsFormulasTab(can_write: bool) -> impl IntoView {
    let statement = RwSignal::new(String::from("balance"));
    let lines = RwSignal::new(Vec::<FsLine>::new());
    let selected = RwSignal::new(String::new());
    let formula_text = RwSignal::new(String::new());
    let date_from = RwSignal::new(String::from("2025-01-01"));
    let date_to = RwSignal::new(String::from("2025-12-31"));
    let loading = RwSignal::new(false);
    let saving = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());
    let success_msg = RwSignal::new(String::new());
    let preview_note = RwSignal::new(String::new());

    let load_all = move || {
        let st = statement.get();
        loading.set(true);
        error_msg.set(String::new());
        spawn_local(async move {
            let base = company_api_url();
            let lines_url = format!("{}/fs/lines?statement={}", base, st);
            let formulas_url = format!("{}/fs/formulas?statement={}", base, st);
            let mut merged: Vec<FsLine> = Vec::new();
            match api_get(&lines_url).await {
                Ok(data) => {
                    if let Some(arr) = data.pointer("/data/items").and_then(|v| v.as_array()) {
                        merged = arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect();
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                    loading.set(false);
                    return;
                }
            }
            if let Ok(data) = api_get(&formulas_url).await {
                if let Some(arr) = data.pointer("/data/items").and_then(|v| v.as_array()) {
                    for item in arr {
                        let code = item.get("line_code").and_then(|v| v.as_str()).unwrap_or("");
                        let formula = item.get("formula").and_then(|v| v.as_str()).unwrap_or("");
                        if let Some(row) = merged.iter_mut().find(|r| r.code == code) {
                            row.formula = formula.to_string();
                        }
                    }
                }
            }
            lines.set(merged);
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        statement.get();
        load_all();
        selected.set(String::new());
        formula_text.set(String::new());
        preview_note.set(String::new());
    });

    let pick = move |code: String| {
        let list = lines.get();
        if let Some(row) = list.iter().find(|r| r.code == code) {
            selected.set(code);
            formula_text.set(row.formula.clone());
        }
    };

    let save_formula = move |_| {
        if !can_write {
            error_msg.set("Нямате права за запис".to_string());
            return;
        }
        let st = statement.get();
        let code = selected.get();
        if code.is_empty() {
            error_msg.set("Изберете статия".to_string());
            return;
        }
        saving.set(true);
        error_msg.set(String::new());
        let formula = formula_text.get();
        spawn_local(async move {
            let url = format!("{}/fs/formulas", company_api_url());
            let body = serde_json::json!({
                "statement": st,
                "line_code": code,
                "formula": formula,
            });
            match api_put(&url, &body).await {
                Ok(_) => {
                    success_msg.set("Формулата е записана".to_string());
                    load_all();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    let load_defaults = move |_| {
        if !can_write { return; }
        let st = statement.get();
        saving.set(true);
        spawn_local(async move {
            let url = format!("{}/fs/formulas/defaults", company_api_url());
            let mut body = serde_json::json!({"statement": st});
            match api_post(&url, &body).await {
                Ok(_) => {
                    success_msg.set("Заредени са стандартните формули".to_string());
                    load_all();
                }
                Err(_) => {
                    body["mode"] = serde_json::json!("fill_empty");
                    match api_post(&url, &body).await {
                        Ok(_) => {
                            success_msg.set("Допълнени са липсващите стандартни формули".to_string());
                            load_all();
                        }
                        Err(e) => error_msg.set(format!("Грешка: {}", e)),
                    }
                }
            }
            saving.set(false);
        });
    };

    let clear_all = move |_| {
        if !can_write { return; }
        let ok = web_sys::window()
            .unwrap()
            .confirm_with_message("Изтриване на всички формули за тази бланка?")
            .unwrap_or(false);
        if !ok { return; }
        let st = statement.get();
        saving.set(true);
        spawn_local(async move {
            let url = format!("{}/fs/formulas/clear", company_api_url());
            match api_post(&url, &serde_json::json!({"statement": st})).await {
                Ok(_) => {
                    success_msg.set("Формулите са изчистени".to_string());
                    load_all();
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    let preview = move |_| {
        let st = statement.get();
        let kind = match st.as_str() {
            "balance" => "balance_sheet",
            "pl" => "income_statement",
            "cashflow" => "cash_flow",
            "equity" => "equity",
            _ => "balance_sheet",
        };
        let from = date_from.get();
        let to = date_to.get();
        if from.is_empty() || to.is_empty() {
            error_msg.set("Посочете период за преглед".to_string());
            return;
        }
        loading.set(true);
        spawn_local(async move {
            let url = format!(
                "{}/reports/financial_statements?kind={}&date_from={}&date_to={}",
                company_api_url(), kind, from, to
            );
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.pointer("/data/rows").and_then(|v| v.as_array()) {
                        let parsed: Vec<FsLine> = arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect();
                        lines.set(parsed);
                    }
                    let mut note = String::new();
                    if let Some(d) = data.pointer("/data/diagnostics") {
                        if let (Some(a), Some(l)) = (d.get("asset_total").and_then(|v| v.as_f64()), d.get("liability_total").and_then(|v| v.as_f64())) {
                            note = format!("Актив {:.2} / Пасив {:.2} / Δ {:.2}", a, l, a - l);
                        }
                        if let Some(p) = d.get("pl_profit").and_then(|v| v.as_f64()) {
                            note = format!("Печалба {:.2}", p);
                        }
                    }
                    preview_note.set(note);
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            loading.set(false);
        });
    };

    view! {
        <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px;">
            <h3 style="margin: 0 0 8px 0; color: #2c3e50;">"Финансови отчети"</h3>
            <p style="color: #7f8c8d; margin: 0 0 16px 0; font-size: 14px; line-height: 1.5;">
                "Бланките са по СС 1. Формулата казва коя сметка влиза в коя статия, напр. "
                <code>"+304Д"</code>
                " (крайно дебитно салдо на 304) или "
                <code>"+50*Д/411К"</code>
                " (постъпления от клиенти). МЗ се пишат като стойност във формулата — без метод на изписване."
            </p>
            <p style="color: #8e6b00; background: #fff8e1; padding: 10px 12px; border-radius: 4px; font-size: 13px; margin: 0 0 16px 0;">
                "Отчетът е след приключване към с/ка 123. Скелетът не форсира Актив = Пасив."
            </p>

            <div style="display: flex; gap: 8px; margin-bottom: 16px; flex-wrap: wrap;">
                {STATEMENTS.into_iter().map(|(id, label)| {
                    let id_style = id.to_string();
                    let id_click = id.to_string();
                    view! {
                        <button
                            style=move || format!(
                                "padding: 8px 14px; border: none; border-radius: 4px; cursor: pointer; {}",
                                if statement.get() == id_style { "background:#3498db;color:white;" } else { "background:#ecf0f1;color:#34495e;" }
                            )
                            on:click=move |_| statement.set(id_click.clone())
                        >{label}</button>
                    }
                }).collect_view()}
            </div>

            <div style="display: flex; gap: 10px; flex-wrap: wrap; align-items: flex-end; margin-bottom: 16px;">
                <button
                    style="background:#8e44ad;color:white;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;"
                    on:click=load_defaults
                    disabled=move || saving.get() || !can_write
                >"Зареди стандартни"</button>
                <button
                    style="background:#c0392b;color:white;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;"
                    on:click=clear_all
                    disabled=move || saving.get() || !can_write
                >"Изчисти"</button>
                <div>
                    <label style="display:block;font-size:12px;color:#7f8c8d;">"От"</label>
                    <input type="date" style="padding:8px;border:1px solid #ddd;border-radius:4px;"
                        prop:value=move || date_from.get()
                        on:input=move |ev| date_from.set(event_target_value(&ev)) />
                </div>
                <div>
                    <label style="display:block;font-size:12px;color:#7f8c8d;">"До"</label>
                    <input type="date" style="padding:8px;border:1px solid #ddd;border-radius:4px;"
                        prop:value=move || date_to.get()
                        on:input=move |ev| date_to.set(event_target_value(&ev)) />
                </div>
                <button
                    style="background:#27ae60;color:white;border:none;padding:8px 14px;border-radius:4px;cursor:pointer;"
                    on:click=preview
                    disabled=move || loading.get()
                >"Преглед"</button>
            </div>

            {move || {
                let e = error_msg.get();
                if e.is_empty() { view!{<span></span>}.into_any() } else {
                    view!{<div style="background:#e74c3c;color:white;padding:10px;border-radius:4px;margin-bottom:12px;">{e}</div>}.into_any()
                }
            }}
            {move || {
                let m = success_msg.get();
                if m.is_empty() { view!{<span></span>}.into_any() } else {
                    view!{<div style="background:#27ae60;color:white;padding:10px;border-radius:4px;margin-bottom:12px;">{m}</div>}.into_any()
                }
            }}
            {move || {
                let n = preview_note.get();
                if n.is_empty() { view!{<span></span>}.into_any() } else {
                    view!{<div style="background:#eef6ff;padding:10px;border-radius:4px;margin-bottom:12px;">{n}</div>}.into_any()
                }
            }}

            <div style="display:grid;grid-template-columns: 1.2fr 0.8fr; gap: 16px;">
                <div style="border:1px solid #eee;border-radius:6px;max-height:560px;overflow:auto;">
                    {move || {
                        if loading.get() {
                            view!{<div style="padding:20px;color:#7f8c8d;">"Зареждане..."</div>}.into_any()
                        } else {
                            let list = lines.get();
                            let parents: Vec<(String,String)> = list.iter().map(|r| (r.code.clone(), r.parent.clone())).collect();
                            view! {
                                <table style="width:100%;border-collapse:collapse;font-size:13px;">
                                    <thead>
                                        <tr style="background:#f8f9fa;">
                                            <th style="text-align:left;padding:8px;">"Статия"</th>
                                            <th style="text-align:right;padding:8px;">"Текуща"</th>
                                            <th style="text-align:left;padding:8px;">"Формула"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {list.into_iter().map(|row| {
                                            let code = row.code.clone();
                                            let code2 = row.code.clone();
                                            let pad = indent_for(&row.code, &parents);
                                            let bold = row.kind == "total" || row.kind == "header";
                                            view! {
                                                <tr
                                                    style=move || format!(
                                                        "cursor:pointer; border-bottom:1px solid #f0f0f0; {}",
                                                        if selected.get() == code2 { "background:#eaf4fb;" } else { "" }
                                                    )
                                                    on:click=move |_| pick(code.clone())
                                                >
                                                    <td style=if bold { "padding:6px 8px;font-weight:600;" } else { "padding:6px 8px;" }>
                                                        {pad}{row.name}
                                                    </td>
                                                    <td style="padding:6px 8px;text-align:right;font-variant-numeric:tabular-nums;">
                                                        {format!("{:.2}", row.current)}
                                                    </td>
                                                    <td style="padding:6px 8px;font-family:monospace;font-size:12px;color:#555;">
                                                        {row.formula}
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }
                    }}
                </div>
                <div>
                    <div style="font-size:13px;color:#7f8c8d;margin-bottom:6px;">
                        {move || {
                            let c = selected.get();
                            if c.is_empty() { "Изберете статия отляво".to_string() } else { format!("Статия {}", c) }
                        }}
                    </div>
                    <textarea
                        style="width:100%;min-height:140px;padding:10px;border:1px solid #ddd;border-radius:4px;box-sizing:border-box;font-family:monospace;"
                        prop:value=move || formula_text.get()
                        on:input=move |ev| formula_text.set(event_target_value(&ev))
                        placeholder="+304Д-241К  или  +50*Д/411К"
                    />
                    <button
                        style="margin-top:10px;background:#27ae60;color:white;border:none;padding:10px 18px;border-radius:4px;cursor:pointer;"
                        on:click=save_formula
                        disabled=move || saving.get() || !can_write
                    >"Запази формулата"</button>
                    <p style="font-size:12px;color:#7f8c8d;margin-top:12px;line-height:1.45;">
                        "Д = крайно дебит, К = крайно кредит, ОД/ОК = оборот, НД/НК = начално, / = кореспонденция, @код = друга статия."
                    </p>
                </div>
            </div>
        </div>
    }
}
