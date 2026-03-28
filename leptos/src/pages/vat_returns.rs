use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_delete};
use crate::context::use_company_context;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VatReturn {
    pub id: i64,
    pub period_year: i32,
    pub period_month: i32,
    pub status: String,
    pub vat_due: Option<f64>,
    pub result_json: Option<String>,

    // Metadata
    pub representative_name: Option<String>,
    pub representative_egn: Option<String>,
    pub sales_count: Option<i32>,
    pub purchase_count: Option<i32>,

    // Section A - Output VAT (Sales)
    pub cell_01_01: Option<f64>,  // ДО облагаеми 20%
    pub cell_01_02: Option<f64>,  // ДДС 20%
    pub cell_01_03: Option<f64>,  // ДО ВОП
    pub cell_01_04: Option<f64>,  // ДДС ВОП
    pub cell_01_05: Option<f64>,  // ДО облагаеми 9%
    pub cell_01_06: Option<f64>,  // ДДС 9%
    pub cell_01_07: Option<f64>,  // ДО 0% глава 3
    pub cell_01_11: Option<f64>,  // ДО ВОД
    pub cell_01_12: Option<f64>,  // ДО чл.140,146,173
    pub cell_01_13: Option<f64>,  // ДО услуги чл.21
    pub cell_01_14: Option<f64>,  // ДО чл.69 ал.2
    pub cell_01_15: Option<f64>,  // ДО чл.69 ал.2 в друга ДЧ
    pub cell_01_16: Option<f64>,  // ДО освободени
    pub cell_01_17: Option<f64>,  // ДДС лични нужди
    pub cell_01_18: Option<f64>,  // ДО ВОП 9%

    // Section B - Input VAT (Purchases)
    pub cell_01_20: Option<f64>,  // ДО без право на ДК
    pub cell_01_21: Option<f64>,  // ДО с пълен ДК
    pub cell_01_22: Option<f64>,  // ДДС пълен ДК
    pub cell_01_23: Option<f64>,  // ДО с частичен ДК
    pub cell_01_24: Option<f64>,  // ДДС частичен ДК
    pub cell_01_25: Option<f64>,  // Год. корекция ДО
    pub cell_01_26: Option<f64>,  // Год. корекция ДДС
    pub cell_01_30: Option<f64>,  // ДО чл.163а
    pub cell_01_31: Option<f64>,  // ДДС чл.163а
    pub cell_01_32: Option<f64>,  // ДО ВОП
    pub cell_01_33: Option<f64>,  // ДДС ВОП

    // Section C - Result
    pub cell_01_40: Option<f64>,  // Общо начислен ДДС
    pub cell_01_41: Option<f64>,  // Данък за внасяне
    pub cell_01_42: Option<f64>,  // ДДС за възстановяване
    pub cell_01_43: Option<f64>,  // Ефективно внесен
    pub cell_01_50: Option<f64>,  // От предходен период
    pub cell_01_51: Option<f64>,  // За внасяне общо
    pub cell_01_52: Option<f64>,  // За възстановяване общо
    pub cell_01_60: Option<f64>,  // ДК за приспадане
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VatJournalEntry {
    pub id: i64,
    pub row_number: i32,
    pub document_type: String,
    pub document_number: String,
    pub document_date: String,
    pub counterpart_name: Option<String>,
    pub counterpart_vat: Option<String>,
    pub goods_description: Option<String>,
    pub delivery_code: Option<String>,
    pub base_amount: Option<f64>,
    pub vat_amount: Option<f64>,
    // Purchase-specific
    pub base_no_credit: Option<f64>,
    pub base_full_credit: Option<f64>,
    pub vat_full_credit: Option<f64>,
    pub base_partial_credit: Option<f64>,
    pub vat_partial_credit: Option<f64>,
    pub annual_adjustment_base: Option<f64>,
    pub annual_adjustment_vat: Option<f64>,
    pub vop_base: Option<f64>,
    pub vop_vat: Option<f64>,
    pub base_9: Option<f64>,
    pub vat_9: Option<f64>,
    // Sales-specific
    pub total_base: Option<f64>,
    pub total_vat: Option<f64>,
    pub sales_base_20: Option<f64>,
    pub sales_vat_20: Option<f64>,
    pub sales_base_vop: Option<f64>,
    pub sales_vat_vop: Option<f64>,
    pub sales_base_9: Option<f64>,
    pub sales_vat_9: Option<f64>,
    pub sales_base_0_chapter3: Option<f64>,
    pub sales_base_vod: Option<f64>,
    pub sales_base_0_articles: Option<f64>,
    pub sales_base_services_21: Option<f64>,
    pub sales_base_69_2: Option<f64>,
    pub sales_base_69_2_eu: Option<f64>,
    pub sales_base_exempt: Option<f64>,
    pub sales_vat_personal: Option<f64>,
    pub sales_base_vop_9: Option<f64>,
}

#[component]
pub fn VatReturns() -> impl IntoView {
    // State
    let returns = create_rw_signal(Vec::<VatReturn>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // New period modal
    let show_new_modal = create_rw_signal(false);
    let new_year = create_rw_signal(get_current_year());
    let new_month = create_rw_signal(get_current_month());
    let creating = create_rw_signal(false);

    // Detail view
    let selected_return = create_rw_signal(Option::<VatReturn>::None);
    let active_tab = create_rw_signal("summary");
    let purchases = create_rw_signal(Vec::<VatJournalEntry>::new());
    let sales = create_rw_signal(Vec::<VatJournalEntry>::new());
    let detail_loading = create_rw_signal(false);
    let company_name = create_rw_signal(String::new());
    let company_vat = create_rw_signal(String::new());

    // Import loading state
    let importing = create_rw_signal(false);

    // Bulk selection state
    let selected_purchase_ids = create_rw_signal(std::collections::HashSet::<i64>::new());
    let selected_sales_ids = create_rw_signal(std::collections::HashSet::<i64>::new());
    let bulk_deleting = create_rw_signal(false);

    let ctx = use_company_context();
    let company_url = move |path: &str| {
        let cid = ctx.selected_company_id.get().unwrap_or(1);
        format!("/api/companies/{}{}", cid, path)
    };

    // Load returns
    let load_returns = move || {
        loading.set(true);
        spawn_local(async move {
            match api_get(&company_url("/vat_returns")).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|d| d.as_array()) {
                        let items: Vec<VatReturn> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        returns.set(items);
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            loading.set(false);
        });
    };

    // Load entries for a return
    let load_entries = move |return_id: i64, entry_type: &'static str| {
        detail_loading.set(true);
        spawn_local(async move {
            let url = format!("{}/entries?type={}", company_url(&format!("/vat_returns/{}", return_id)), entry_type);
            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|d| d.as_array()) {
                        let items: Vec<VatJournalEntry> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        if entry_type == "purchase" {
                            purchases.set(items);
                        } else {
                            sales.set(items);
                        }
                    }
                }
                Err(_) => {}
            }
            detail_loading.set(false);
        });
    };

    // Create new return
    let create_return = move |_| {
        creating.set(true);
        error_msg.set(String::new());

        let year = new_year.get();
        let month = new_month.get();

        spawn_local(async move {
            let payload = serde_json::json!({
                "period_year": year,
                "period_month": month
            });

            match api_post(&company_url("/vat_returns"), &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Периодът е създаден успешно".to_string());
                        show_new_modal.set(false);
                        load_returns();
                    } else {
                        let err = response.get("error").and_then(|e| e.as_str()).unwrap_or("Неизвестна грешка");
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            creating.set(false);
        });
    };

    // Open detail view
    let open_detail = move |vr: VatReturn| {
        let id = vr.id;
        selected_return.set(Some(vr));
        active_tab.set("summary");
        load_entries(id, "purchase");
        load_entries(id, "sales");
    };

    // Export file (download)
    let export_file = move |return_id: i64, export_type: &'static str| {
        // Use full backend URL for direct download
        let cid = ctx.selected_company_id.get().unwrap_or(1);
        let url = format!("{}/api/companies/{}/vat_returns/{}/export/{}", crate::api::API_BASE, cid, return_id, export_type);
        // Open in new window to trigger download
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(&url, "_blank");
        }
    };

    // Calculate return
    let calculate_return = move |id: i64| {
        spawn_local(async move {
            let url = format!("{}/calculate", company_url(&format!("/vat_returns/{}", id)));
            match api_post(&url, &serde_json::json!({})).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Изчислението е готово".to_string());
                        // Reload returns and update selected_return with fresh data
                        match api_get(&company_url("/vat_returns")).await {
                            Ok(data) => {
                                if let Some(arr) = data.get("data").and_then(|d| d.as_array()) {
                                    let items: Vec<VatReturn> = arr.iter()
                                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                        .collect();
                                    if let Some(updated) = items.iter().find(|r| r.id == id) {
                                        selected_return.set(Some(updated.clone()));
                                    }
                                    returns.set(items);
                                }
                            }
                            Err(e) => error_msg.set(format!("Грешка: {}", e)),
                        }
                    } else {
                        let err = response.get("error").and_then(|v| v.as_str()).unwrap_or("Неизвестна грешка");
                        error_msg.set(format!("Грешка при изчисление: {}", err));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Import from journal entries
    let import_from_journal = move |id: i64| {
        importing.set(true);
        error_msg.set(String::new());
        spawn_local(async move {
            let url = format!("{}/import_from_journal", company_url(&format!("/vat_returns/{}", id)));
            match api_post(&url, &serde_json::json!({})).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let msg = response.get("data")
                            .and_then(|d| d.get("message"))
                            .and_then(|m| m.as_str())
                            .unwrap_or("Импортът е готов");
                        success_msg.set(msg.to_string());
                        selected_purchase_ids.set(std::collections::HashSet::new());
                        selected_sales_ids.set(std::collections::HashSet::new());
                        if let Some(vr) = selected_return.get() {
                            load_entries(vr.id, "purchase");
                            load_entries(vr.id, "sales");
                        }
                    } else {
                        let err = response.get("error").and_then(|e| e.as_str()).unwrap_or("Неизвестна грешка");
                        error_msg.set(format!("Грешка: {}", err));
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            importing.set(false);
        });
    };

    // Bulk delete selected entries
    let bulk_delete_entries = move |entry_type: &'static str| {
        let ids: Vec<i64> = if entry_type == "purchase" {
            selected_purchase_ids.get().iter().cloned().collect()
        } else {
            selected_sales_ids.get().iter().cloned().collect()
        };
        if ids.is_empty() { return; }
        bulk_deleting.set(true);
        error_msg.set(String::new());
        spawn_local(async move {
            if let Some(vr) = selected_return.get() {
                let cid = ctx.selected_company_id.get().unwrap_or(1);
                let url = format!("/api/companies/{}/vat_returns/{}/journal_entries/bulk_delete", cid, vr.id);
                let payload = serde_json::json!({ "ids": ids });
                match api_post(&url, &payload).await {
                    Ok(response) => {
                        if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            let msg = response.get("message").and_then(|m| m.as_str()).unwrap_or("Изтрити");
                            success_msg.set(msg.to_string());
                            if entry_type == "purchase" {
                                selected_purchase_ids.set(std::collections::HashSet::new());
                            } else {
                                selected_sales_ids.set(std::collections::HashSet::new());
                            }
                            load_entries(vr.id, entry_type);
                        } else {
                            let err = response.get("error").and_then(|e| e.as_str()).unwrap_or("Неизвестна грешка");
                            error_msg.set(format!("Грешка: {}", err));
                        }
                    }
                    Err(e) => error_msg.set(format!("Грешка: {}", e)),
                }
            }
            bulk_deleting.set(false);
        });
    };

    // Print/Excel for Purchase Journal
    let print_purchase_journal = move |_| {
        let items = purchases.get();
        if items.is_empty() { return; }
        if let Some(vr) = selected_return.get() {
            let period = format!("{:02}/{}", vr.period_month, vr.period_year);
            let content = build_purchase_journal_html(&items, &company_name.get(), &company_vat.get(), &period);
            open_journal_print(&content, "Дневник покупки");
        }
    };
    let excel_purchase_journal = move |_| {
        let items = purchases.get();
        if items.is_empty() { return; }
        if let Some(vr) = selected_return.get() {
            let period = format!("{:02}/{}", vr.period_month, vr.period_year);
            let content = build_purchase_journal_html(&items, &company_name.get(), &company_vat.get(), &period);
            export_journal_excel(&content, "Дневник покупки", &format!("pokupki_{:02}_{}.xlsx", vr.period_month, vr.period_year));
        }
    };

    // Print/Excel for Sales Journal
    let print_sales_journal = move |_| {
        let items = sales.get();
        if items.is_empty() { return; }
        if let Some(vr) = selected_return.get() {
            let period = format!("{:02}/{}", vr.period_month, vr.period_year);
            let content = build_sales_journal_html(&items, &company_name.get(), &company_vat.get(), &period);
            open_journal_print(&content, "Дневник продажби");
        }
    };
    let excel_sales_journal = move |_| {
        let items = sales.get();
        if items.is_empty() { return; }
        if let Some(vr) = selected_return.get() {
            let period = format!("{:02}/{}", vr.period_month, vr.period_year);
            let content = build_sales_journal_html(&items, &company_name.get(), &company_vat.get(), &period);
            export_journal_excel(&content, "Дневник продажби", &format!("prodajbi_{:02}_{}.xlsx", vr.period_month, vr.period_year));
        }
    };

    // Load on mount
    create_effect(move |_| {
        load_returns();
        // Load company info for reports
        spawn_local(async move {
            if let Ok(data) = api_get(&company_url("")).await {
                if let Some(c) = data.get("data") {
                    company_name.set(c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    let vat = c.get("vat_number").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let vat = if vat.starts_with("BG") { vat } else { format!("BG{}", vat) };
                    company_vat.set(vat);
                }
            }
        });
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
                if let Some(vr) = selected_return.get() {
                    // Detail view
                    let result: serde_json::Value = vr.result_json.as_ref()
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or(serde_json::json!({}));

                    let vr_for_import = vr.clone();
                    let vr_for_calc = vr.clone();

                    view! {
                        <div>
                            // Header
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <div style="display: flex; align-items: center; gap: 15px;">
                                    <button
                                        style="background: #ecf0f1; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| selected_return.set(None)
                                    >
                                        "← Назад"
                                    </button>
                                    <h2 style="margin: 0;">{format!("ДДС период: {:02}/{}", vr.period_month, vr.period_year)}</h2>
                                    <span style=format!("padding: 4px 12px; border-radius: 4px; font-size: 0.9em; {}",
                                        match vr.status.as_str() {
                                            "SUBMITTED" | "submitted" => "background: #27ae60; color: white;",
                                            "CALCULATED" | "calculated" => "background: #3498db; color: white;",
                                            _ => "background: #bdc3c7;"
                                        }
                                    )>{&vr.status}</span>
                                </div>
                                <div style="display: flex; gap: 10px;">
                                    <button
                                        style=move || format!("color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: {}; {}",
                                            if importing.get() { "wait" } else { "pointer" },
                                            if importing.get() { "background: #8e44ad; opacity: 0.7;" } else { "background: #9b59b6;" })
                                        on:click=move |_| {
                                            if !importing.get() {
                                                import_from_journal(vr_for_import.id);
                                            }
                                        }
                                        disabled=move || importing.get()
                                    >
                                        {move || if importing.get() { "Импортиране..." } else { "Импорт от дневник" }}
                                    </button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| calculate_return(vr_for_calc.id)
                                    >
                                        "Изчисли"
                                    </button>
                                </div>
                            </div>

                            // Tabs
                            <div style="display: flex; gap: 0; margin-bottom: 20px; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                <button
                                    style=move || format!("padding: 12px 24px; border: none; cursor: pointer; font-weight: 500; {}",
                                        if active_tab.get() == "summary" { "background: #3498db; color: white;" } else { "background: white;" })
                                    on:click=move |_| active_tab.set("summary")
                                >"Обобщение"</button>
                                <button
                                    style=move || format!("padding: 12px 24px; border: none; cursor: pointer; font-weight: 500; {}",
                                        if active_tab.get() == "purchases" { "background: #3498db; color: white;" } else { "background: white;" })
                                    on:click=move |_| active_tab.set("purchases")
                                >"Дневник покупки"</button>
                                <button
                                    style=move || format!("padding: 12px 24px; border: none; cursor: pointer; font-weight: 500; {}",
                                        if active_tab.get() == "sales" { "background: #3498db; color: white;" } else { "background: white;" })
                                    on:click=move |_| active_tab.set("sales")
                                >"Дневник продажби"</button>
                                <button
                                    style=move || format!("padding: 12px 24px; border: none; cursor: pointer; font-weight: 500; {}",
                                        if active_tab.get() == "declaration" { "background: #3498db; color: white;" } else { "background: white;" })
                                    on:click=move |_| active_tab.set("declaration")
                                >"Декларация"</button>
                                <button
                                    style=move || format!("padding: 12px 24px; border: none; cursor: pointer; font-weight: 500; {}",
                                        if active_tab.get() == "vies" { "background: #3498db; color: white;" } else { "background: white;" })
                                    on:click=move |_| active_tab.set("vies")
                                >"VIES"</button>
                            </div>

                            // Tab content
                            <div style="background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                {move || match active_tab.get() {
                                    "summary" => {
                                        let purchase_base = result.get("purchase_base_20").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let purchase_vat = result.get("purchase_vat_20").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let sales_base = result.get("sales_base_20").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let sales_vat = result.get("sales_vat_20").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let vat_due = result.get("vat_due").and_then(|v| v.as_f64()).unwrap_or(0.0);

                                        view! {
                                            <div>
                                                <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 20px;">
                                                    <div style="padding: 20px; background: #e8f5e9; border-radius: 8px;">
                                                        <h4 style="margin-top: 0;">"Покупки"</h4>
                                                        <p>"Данъчна основа 20%: " <strong>{format!("{:.2} EUR", purchase_base)}</strong></p>
                                                        <p>"ДДС 20%: " <strong>{format!("{:.2} EUR", purchase_vat)}</strong></p>
                                                    </div>
                                                    <div style="padding: 20px; background: #e3f2fd; border-radius: 8px;">
                                                        <h4 style="margin-top: 0;">"Продажби"</h4>
                                                        <p>"Данъчна основа 20%: " <strong>{format!("{:.2} EUR", sales_base)}</strong></p>
                                                        <p>"ДДС 20%: " <strong>{format!("{:.2} EUR", sales_vat)}</strong></p>
                                                    </div>
                                                </div>
                                                <div style=format!("margin-top: 20px; padding: 20px; border-radius: 8px; text-align: center; {}",
                                                    if vat_due >= 0.0 { "background: #ffebee;" } else { "background: #e8f5e9;" }
                                                )>
                                                    <h4 style="margin-top: 0;">{if vat_due >= 0.0 { "ДДС за внасяне" } else { "ДДС за възстановяване" }}</h4>
                                                    <p style="font-size: 2em; font-weight: bold; margin: 0;">
                                                        {format!("{:.2} EUR", vat_due.abs())}
                                                    </p>
                                                </div>
                                            </div>
                                        }.into_view()
                                    },
                                    "purchases" => {
                                        let items = purchases.get();
                                        let total_base: f64 = items.iter().map(|e| e.base_amount.unwrap_or(0.0)).sum();
                                        let total_vat: f64 = items.iter().map(|e| e.vat_amount.unwrap_or(0.0)).sum();
                                        let all_purchase_ids: Vec<i64> = items.iter().map(|e| e.id).collect();
                                        view! {
                                            <div>
                                                <div style="display: flex; gap: 10px; margin-bottom: 15px; align-items: center;">
                                                    <button
                                                        style="padding: 8px 16px; background: #e74c3c; color: white; border: none; border-radius: 4px; cursor: pointer; font-weight: 500;"
                                                        on:click=print_purchase_journal
                                                    >"Печат/PDF"</button>
                                                    <button
                                                        style="padding: 8px 16px; background: #27ae60; color: white; border: none; border-radius: 4px; cursor: pointer; font-weight: 500;"
                                                        on:click=excel_purchase_journal
                                                    >"Excel"</button>
                                                    {move || {
                                                        let sel_count = selected_purchase_ids.get().len();
                                                        if sel_count > 0 {
                                                            view! {
                                                                <button
                                                                    style=move || format!("padding: 8px 16px; background: #c0392b; color: white; border: none; border-radius: 4px; cursor: {}; font-weight: 500;",
                                                                        if bulk_deleting.get() { "wait" } else { "pointer" })
                                                                    on:click=move |_| bulk_delete_entries("purchase")
                                                                    disabled=move || bulk_deleting.get()
                                                                >
                                                                    {move || {
                                                                        let n = selected_purchase_ids.get().len();
                                                                        if bulk_deleting.get() {
                                                                            "Изтриване...".to_string()
                                                                        } else {
                                                                            format!("Изтрий избрани ({})", n)
                                                                        }
                                                                    }}
                                                                </button>
                                                            }.into_view()
                                                        } else {
                                                            view! { <span></span> }.into_view()
                                                        }
                                                    }}
                                                </div>
                                                <table style="width: 100%; border-collapse: collapse;">
                                                    <thead>
                                                        <tr style="background: #f8f9fa;">
                                                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6; width: 40px;">
                                                                <input type="checkbox"
                                                                    style="cursor: pointer;"
                                                                    prop:checked=move || {
                                                                        let sel = selected_purchase_ids.get();
                                                                        !purchases.get().is_empty() && sel.len() == purchases.get().len()
                                                                    }
                                                                    on:change=move |ev| {
                                                                        let checked = event_target_checked(&ev);
                                                                        if checked {
                                                                            let ids: std::collections::HashSet<i64> = all_purchase_ids.iter().cloned().collect();
                                                                            selected_purchase_ids.set(ids);
                                                                        } else {
                                                                            selected_purchase_ids.set(std::collections::HashSet::new());
                                                                        }
                                                                    }
                                                                />
                                                            </th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"№"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Вид"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Документ"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Дата"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Контрагент"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"ДДС №"</th>
                                                            <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Основа"</th>
                                                            <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"ДДС"</th>
                                                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6; width: 50px;"></th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {items.iter().map(|e| {
                                                            let entry_id = e.id;
                                                            let return_id = vr.id;
                                                            view! {
                                                                <tr style=move || format!("border-bottom: 1px solid #dee2e6; {}",
                                                                    if selected_purchase_ids.get().contains(&entry_id) { "background: #eaf2ff;" } else { "" })>
                                                                    <td style="padding: 10px; text-align: center;">
                                                                        <input type="checkbox"
                                                                            style="cursor: pointer;"
                                                                            prop:checked=move || selected_purchase_ids.get().contains(&entry_id)
                                                                            on:change=move |ev| {
                                                                                let checked = event_target_checked(&ev);
                                                                                let mut ids = selected_purchase_ids.get();
                                                                                if checked {
                                                                                    ids.insert(entry_id);
                                                                                } else {
                                                                                    ids.remove(&entry_id);
                                                                                }
                                                                                selected_purchase_ids.set(ids);
                                                                            }
                                                                        />
                                                                    </td>
                                                                    <td style="padding: 10px;">{e.row_number}</td>
                                                                    <td style="padding: 10px;">{&e.document_type}</td>
                                                                    <td style="padding: 10px;">{&e.document_number}</td>
                                                                    <td style="padding: 10px;">{&e.document_date}</td>
                                                                    <td style="padding: 10px;">{e.counterpart_name.clone().unwrap_or("-".to_string())}</td>
                                                                    <td style="padding: 10px;">{e.counterpart_vat.clone().unwrap_or("-".to_string())}</td>
                                                                    <td style="padding: 10px; text-align: right;">{format!("{:.2}", e.base_amount.unwrap_or(0.0))}</td>
                                                                    <td style="padding: 10px; text-align: right;">{format!("{:.2}", e.vat_amount.unwrap_or(0.0))}</td>
                                                                    <td style="padding: 10px; text-align: center;">
                                                                        <button
                                                                            style="background: #e74c3c; color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 11px;"
                                                                            title="Изтрий"
                                                                            on:click=move |_| {
                                                                                let cid = ctx.selected_company_id.get().unwrap_or(1);
                                                                                let url = format!("/api/companies/{}/vat_returns/{}/journal_entries/{}", cid, return_id, entry_id);
                                                                                spawn_local(async move {
                                                                                    match api_delete(&url).await {
                                                                                        Ok(_) => {
                                                                                            selected_purchase_ids.update(|ids| { ids.remove(&entry_id); });
                                                                                            load_entries(return_id, "purchase");
                                                                                        }
                                                                                        Err(e) => error_msg.set(format!("Грешка: {}", e)),
                                                                                    }
                                                                                });
                                                                            }
                                                                        >
                                                                            "X"
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                        {if items.is_empty() {
                                                            view! {
                                                                <tr><td colspan="10" style="padding: 20px; text-align: center; color: #7f8c8d;">"Няма записи. Натиснете \"Импорт от дневник\" за да импортирате записи."</td></tr>
                                                            }.into_view()
                                                        } else {
                                                            view! {
                                                                <tr style="background: #e8f5e9; font-weight: bold;">
                                                                    <td></td>
                                                                    <td colspan="6" style="padding: 10px; text-align: right;">"ОБЩО:"</td>
                                                                    <td style="padding: 10px; text-align: right;">{format!("{:.2}", total_base)}</td>
                                                                    <td style="padding: 10px; text-align: right;">{format!("{:.2}", total_vat)}</td>
                                                                    <td></td>
                                                                </tr>
                                                            }.into_view()
                                                        }}
                                                    </tbody>
                                                </table>
                                            </div>
                                        }.into_view()
                                    },
                                    "sales" => {
                                        let items = sales.get();
                                        let total_base: f64 = items.iter().map(|e| e.base_amount.unwrap_or(0.0)).sum();
                                        let total_vat: f64 = items.iter().map(|e| e.vat_amount.unwrap_or(0.0)).sum();
                                        let all_sales_ids: Vec<i64> = items.iter().map(|e| e.id).collect();
                                        view! {
                                            <div>
                                                <div style="display: flex; gap: 10px; margin-bottom: 15px; align-items: center;">
                                                    <button
                                                        style="padding: 8px 16px; background: #e74c3c; color: white; border: none; border-radius: 4px; cursor: pointer; font-weight: 500;"
                                                        on:click=print_sales_journal
                                                    >"Печат/PDF"</button>
                                                    <button
                                                        style="padding: 8px 16px; background: #27ae60; color: white; border: none; border-radius: 4px; cursor: pointer; font-weight: 500;"
                                                        on:click=excel_sales_journal
                                                    >"Excel"</button>
                                                    {move || {
                                                        let sel_count = selected_sales_ids.get().len();
                                                        if sel_count > 0 {
                                                            view! {
                                                                <button
                                                                    style=move || format!("padding: 8px 16px; background: #c0392b; color: white; border: none; border-radius: 4px; cursor: {}; font-weight: 500;",
                                                                        if bulk_deleting.get() { "wait" } else { "pointer" })
                                                                    on:click=move |_| bulk_delete_entries("sales")
                                                                    disabled=move || bulk_deleting.get()
                                                                >
                                                                    {move || {
                                                                        let n = selected_sales_ids.get().len();
                                                                        if bulk_deleting.get() {
                                                                            "Изтриване...".to_string()
                                                                        } else {
                                                                            format!("Изтрий избрани ({})", n)
                                                                        }
                                                                    }}
                                                                </button>
                                                            }.into_view()
                                                        } else {
                                                            view! { <span></span> }.into_view()
                                                        }
                                                    }}
                                                </div>
                                                <table style="width: 100%; border-collapse: collapse;">
                                                    <thead>
                                                        <tr style="background: #f8f9fa;">
                                                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6; width: 40px;">
                                                                <input type="checkbox"
                                                                    style="cursor: pointer;"
                                                                    prop:checked=move || {
                                                                        let sel = selected_sales_ids.get();
                                                                        !sales.get().is_empty() && sel.len() == sales.get().len()
                                                                    }
                                                                    on:change=move |ev| {
                                                                        let checked = event_target_checked(&ev);
                                                                        if checked {
                                                                            let ids: std::collections::HashSet<i64> = all_sales_ids.iter().cloned().collect();
                                                                            selected_sales_ids.set(ids);
                                                                        } else {
                                                                            selected_sales_ids.set(std::collections::HashSet::new());
                                                                        }
                                                                    }
                                                                />
                                                            </th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"№"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Вид"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Документ"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Дата"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"Контрагент"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 2px solid #dee2e6;">"ДДС №"</th>
                                                            <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"Основа"</th>
                                                            <th style="padding: 12px; text-align: right; border-bottom: 2px solid #dee2e6;">"ДДС"</th>
                                                            <th style="padding: 12px; text-align: center; border-bottom: 2px solid #dee2e6; width: 50px;"></th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {items.iter().map(|e| {
                                                            let entry_id = e.id;
                                                            let return_id = vr.id;
                                                            view! {
                                                                <tr style=move || format!("border-bottom: 1px solid #dee2e6; {}",
                                                                    if selected_sales_ids.get().contains(&entry_id) { "background: #eaf2ff;" } else { "" })>
                                                                    <td style="padding: 10px; text-align: center;">
                                                                        <input type="checkbox"
                                                                            style="cursor: pointer;"
                                                                            prop:checked=move || selected_sales_ids.get().contains(&entry_id)
                                                                            on:change=move |ev| {
                                                                                let checked = event_target_checked(&ev);
                                                                                let mut ids = selected_sales_ids.get();
                                                                                if checked {
                                                                                    ids.insert(entry_id);
                                                                                } else {
                                                                                    ids.remove(&entry_id);
                                                                                }
                                                                                selected_sales_ids.set(ids);
                                                                            }
                                                                        />
                                                                    </td>
                                                                    <td style="padding: 10px;">{e.row_number}</td>
                                                                    <td style="padding: 10px;">{&e.document_type}</td>
                                                                    <td style="padding: 10px;">{&e.document_number}</td>
                                                                    <td style="padding: 10px;">{&e.document_date}</td>
                                                                    <td style="padding: 10px;">{e.counterpart_name.clone().unwrap_or("-".to_string())}</td>
                                                                    <td style="padding: 10px;">{e.counterpart_vat.clone().unwrap_or("-".to_string())}</td>
                                                                    <td style="padding: 10px; text-align: right;">{format!("{:.2}", e.base_amount.unwrap_or(0.0))}</td>
                                                                    <td style="padding: 10px; text-align: right;">{format!("{:.2}", e.vat_amount.unwrap_or(0.0))}</td>
                                                                    <td style="padding: 10px; text-align: center;">
                                                                        <button
                                                                            style="background: #e74c3c; color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 11px;"
                                                                            title="Изтрий"
                                                                            on:click=move |_| {
                                                                                let cid = ctx.selected_company_id.get().unwrap_or(1);
                                                                                let url = format!("/api/companies/{}/vat_returns/{}/journal_entries/{}", cid, return_id, entry_id);
                                                                                spawn_local(async move {
                                                                                    match api_delete(&url).await {
                                                                                        Ok(_) => {
                                                                                            selected_sales_ids.update(|ids| { ids.remove(&entry_id); });
                                                                                            load_entries(return_id, "sales");
                                                                                        }
                                                                                        Err(e) => error_msg.set(format!("Грешка: {}", e)),
                                                                                    }
                                                                                });
                                                                            }
                                                                        >
                                                                            "X"
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                        {if items.is_empty() {
                                                            view! {
                                                                <tr><td colspan="10" style="padding: 20px; text-align: center; color: #7f8c8d;">"Няма записи. Натиснете \"Импорт от дневник\" за да импортирате записи."</td></tr>
                                                            }.into_view()
                                                        } else {
                                                            view! {
                                                                <tr style="background: #e3f2fd; font-weight: bold;">
                                                                    <td></td>
                                                                    <td colspan="6" style="padding: 10px; text-align: right;">"ОБЩО:"</td>
                                                                    <td style="padding: 10px; text-align: right;">{format!("{:.2}", total_base)}</td>
                                                                    <td style="padding: 10px; text-align: right;">{format!("{:.2}", total_vat)}</td>
                                                                    <td></td>
                                                                </tr>
                                                            }.into_view()
                                                        }}
                                                    </tbody>
                                                </table>
                                            </div>
                                        }.into_view()
                                    },
                                    "declaration" => {
                                        let return_id = vr.id;
                                        let vr_clone = vr.clone();
                                        view! {
                                            <DeclarationForm vat_return=vr_clone on_save=move || {
                                                if let Some(vr) = selected_return.get() {
                                                    load_entries(vr.id, "purchase");
                                                    load_entries(vr.id, "sales");
                                                }
                                                load_returns();
                                                success_msg.set("Декларацията е запазена".to_string());
                                            } />
                                            <div style="margin-top: 30px; padding: 20px; background: #f0f4f8; border-radius: 8px;">
                                                <h5 style="margin-top: 0;">"Експорт на файлове за НАП"</h5>
                                                <div style="display: flex; gap: 10px; flex-wrap: wrap; margin-top: 15px;">
                                                    <button
                                                        style="background: #27ae60; color: white; border: none; padding: 12px 24px; border-radius: 4px; cursor: pointer; font-weight: 500;"
                                                        on:click=move |_| export_file(return_id, "pokupki")
                                                    >
                                                        "POKUPKI.TXT"
                                                    </button>
                                                    <button
                                                        style="background: #3498db; color: white; border: none; padding: 12px 24px; border-radius: 4px; cursor: pointer; font-weight: 500;"
                                                        on:click=move |_| export_file(return_id, "prodajbi")
                                                    >
                                                        "PRODAJBI.TXT"
                                                    </button>
                                                    <button
                                                        style="background: #9b59b6; color: white; border: none; padding: 12px 24px; border-radius: 4px; cursor: pointer; font-weight: 500;"
                                                        on:click=move |_| export_file(return_id, "deklar")
                                                    >
                                                        "DEKLAR.TXT"
                                                    </button>
                                                </div>
                                            </div>
                                        }.into_view()
                                    },
                                    "vies" => view! {
                                        <div>
                                            <h4>"VIES декларация"</h4>
                                            <p style="color: #7f8c8d;">"Вътреобщностни доставки - в разработка"</p>
                                        </div>
                                    }.into_view(),
                                    _ => view! { <div></div> }.into_view()
                                }}
                            </div>
                        </div>
                    }.into_view()
                } else {
                    // List view
                    view! {
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                <h1 style="margin: 0;">"ДДС Дневници"</h1>
                                <button
                                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                                    on:click=move |_| show_new_modal.set(true)
                                >
                                    "+ Нов период"
                                </button>
                            </div>

                            <div style="background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: hidden;">
                                <table style="width: 100%; border-collapse: collapse;">
                                    <thead>
                                        <tr style="background: #f8f9fa;">
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"Период"</th>
                                            <th style="padding: 15px; text-align: left; border-bottom: 2px solid #dee2e6;">"Статус"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"ДДС"</th>
                                            <th style="padding: 15px; text-align: right; border-bottom: 2px solid #dee2e6;">"Действия"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || returns.get().iter().map(|r| {
                                            let r_for_click = r.clone();
                                            let r_for_calc = r.clone();
                                            let can_calculate = r.status == "DRAFT";
                                            view! {
                                                <tr style="border-bottom: 1px solid #dee2e6;">
                                                    <td style="padding: 15px;">{format!("{:02}/{}", r.period_month, r.period_year)}</td>
                                                    <td style="padding: 15px;">
                                                        <span style=format!("padding: 4px 12px; border-radius: 4px; font-size: 0.85em; {}",
                                                            match r.status.as_str() {
                                                                "SUBMITTED" => "background: #27ae60; color: white;",
                                                                "CALCULATED" => "background: #3498db; color: white;",
                                                                _ => "background: #bdc3c7;"
                                                            }
                                                        )>{&r.status}</span>
                                                    </td>
                                                    <td style="padding: 15px; text-align: right;">
                                                        {r.vat_due.map(|v| format!("{:.2} EUR", v)).unwrap_or("-".to_string())}
                                                    </td>
                                                    <td style="padding: 15px; text-align: right;">
                                                        {if can_calculate {
                                                            view! {
                                                                <button
                                                                    style="background: #27ae60; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; margin-right: 8px;"
                                                                    on:click=move |_| calculate_return(r_for_calc.id)
                                                                >
                                                                    "Изчисли"
                                                                </button>
                                                            }.into_view()
                                                        } else {
                                                            view! { <span></span> }.into_view()
                                                        }}
                                                        <button
                                                            style="background: #3498db; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer;"
                                                            on:click=move |_| open_detail(r_for_click.clone())
                                                        >
                                                            "Преглед"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                        {move || if returns.get().is_empty() && !loading.get() {
                                            view! {
                                                <tr>
                                                    <td colspan="4" style="padding: 30px; text-align: center; color: #7f8c8d;">
                                                        "Няма ДДС периоди. Натиснете \"+ Нов период\" за да създадете."
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

            // New period modal
            {move || if show_new_modal.get() {
                view! {
                    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                        <div style="background: white; border-radius: 8px; padding: 30px; min-width: 350px; box-shadow: 0 4px 20px rgba(0,0,0,0.3);">
                            <h3 style="margin-top: 0;">"Нов ДДС период"</h3>
                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin: 20px 0;">
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Година"</label>
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
                                <div>
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Месец"</label>
                                    <input
                                        type="text"
                                        inputmode="numeric"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        prop:value=move || new_month.get()
                                        on:input=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse() {
                                                new_month.set(v);
                                            }
                                        }
                                    />
                                </div>
                            </div>
                            <div style="display: flex; justify-content: flex-end; gap: 10px;">
                                <button
                                    style="padding: 10px 20px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; background: white;"
                                    on:click=move |_| show_new_modal.set(false)
                                >
                                    "Отказ"
                                </button>
                                <button
                                    style="padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; background: #3498db; color: white;"
                                    on:click=create_return
                                    disabled=move || creating.get()
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
    now.get_full_year() as i32
}

fn get_current_month() -> i32 {
    let now = js_sys::Date::new_0();
    (now.get_month() + 1) as i32
}

// Declaration Form Component
#[component]
fn DeclarationForm<F>(vat_return: VatReturn, on_save: F) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let saving = create_rw_signal(false);
    let ctx = use_company_context();

    // Create signals for all editable fields
    let cell_01_50 = create_rw_signal(vat_return.cell_01_50.unwrap_or(0.0));
    let cell_01_60 = create_rw_signal(vat_return.cell_01_60.unwrap_or(0.0));
    let cell_01_43 = create_rw_signal(vat_return.cell_01_43.unwrap_or(0.0));
    // Load representative info from company settings (read-only display)
    let representative_name = create_rw_signal(String::new());
    let representative_egn = create_rw_signal(String::new());
    let representative_type_label = create_rw_signal(String::new());
    {
        let cid = ctx.selected_company_id.get().unwrap_or(1);
        spawn_local(async move {
            let url = format!("/api/companies/{}", cid);
            if let Ok(data) = api_get(&url).await {
                if let Some(company) = data.get("data") {
                    let rep_type = company.get("representative_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MANAGER");
                    let (name, egn) = if rep_type == "AUTHORIZED_PERSON" {
                        representative_type_label.set("Пълномощник".to_string());
                        (
                            company.get("authorized_person_name").and_then(|v| v.as_str()).unwrap_or(""),
                            company.get("authorized_person_egn").and_then(|v| v.as_str()).unwrap_or(""),
                        )
                    } else {
                        representative_type_label.set("Управител".to_string());
                        (
                            company.get("manager_name").and_then(|v| v.as_str()).unwrap_or(""),
                            company.get("manager_egn").and_then(|v| v.as_str()).unwrap_or(""),
                        )
                    };
                    representative_name.set(name.to_string());
                    representative_egn.set(egn.to_string());
                }
            }
        });
    }

    let vr_id = vat_return.id;
    let on_save_clone = on_save.clone();

    let save_declaration = move |_| {
        saving.set(true);
        let on_save = on_save_clone.clone();

        spawn_local(async move {
            let payload = serde_json::json!({
                "cell_01_50": cell_01_50.get(),
                "cell_01_60": cell_01_60.get(),
                "cell_01_43": cell_01_43.get()
            });

            let cid = ctx.selected_company_id.get().unwrap_or(1);
            let url = format!("/api/companies/{}/vat_returns/{}", cid, vr_id);
            match crate::api::api_put(&url, &payload).await {
                Ok(response) => {
                    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        on_save();
                    }
                }
                Err(_) => {}
            }
            saving.set(false);
        });
    };

    let field_style = "padding: 8px; border: 1px solid #ddd; border-radius: 4px; width: 300px;";
    let label_style = "font-weight: 500; min-width: 250px;";
    let row_style = "display: flex; justify-content: space-between; align-items: center; padding: 8px 0; border-bottom: 1px solid #eee;";
    let section_style = "margin-bottom: 25px; padding: 15px; background: #fafafa; border-radius: 8px;";

    view! {
        <div>
            // Section 0 - Representative info (from company settings, read-only)
            <div style="margin-bottom: 25px; padding: 15px; background: #e3f2fd; border-radius: 8px; border: 1px solid #2196f3;">
                <h5 style="margin-top: 0; color: #0d47a1; border-bottom: 2px solid #2196f3; padding-bottom: 8px;">
                    "Информация за представителя (от настройки на фирмата)"
                </h5>
                <div style=row_style>
                    <span style=label_style>{move || format!("Име на представителя ({})", representative_type_label.get())}</span>
                    <span style="font-weight: bold;">{move || representative_name.get()}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"ЕГН на представителя"</span>
                    <span style="font-weight: bold;">{move || representative_egn.get()}</span>
                </div>
            </div>

            // Section A - Output VAT (Read-only calculated values)
            <div style=section_style>
                <h5 style="margin-top: 0; color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 8px;">
                    "РАЗДЕЛ А - Начислен данък (Продажби)"
                </h5>
                <div style=row_style>
                    <span style=label_style>"01-01: ДО облагаеми доставки 20%"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_01.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-02: Начислен ДДС 20%"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_02.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-05: ДО облагаеми доставки 9%"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_05.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-06: Начислен ДДС 9%"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_06.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-07: ДО облагаеми 0% по глава 3"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_07.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-11: ДО ВОД (вътреобщностни доставки)"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_11.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-16: ДО освободени доставки"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_16.unwrap_or(0.0))}</span>
                </div>
            </div>

            // Section B - Input VAT (Read-only calculated values)
            <div style=section_style>
                <h5 style="margin-top: 0; color: #2c3e50; border-bottom: 2px solid #27ae60; padding-bottom: 8px;">
                    "РАЗДЕЛ Б - Данъчен кредит (Покупки)"
                </h5>
                <div style=row_style>
                    <span style=label_style>"01-20: ДО без право на данъчен кредит"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_20.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-21: ДО с право на пълен ДК"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_21.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-22: ДДС с право на пълен ДК"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_22.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-23: ДО с право на частичен ДК"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_23.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-24: ДДС с право на частичен ДК"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_24.unwrap_or(0.0))}</span>
                </div>
            </div>

            // Section C - Result (Editable fields)
            <div style="margin-bottom: 25px; padding: 15px; background: #fff3cd; border-radius: 8px; border: 1px solid #ffc107;">
                <h5 style="margin-top: 0; color: #856404; border-bottom: 2px solid #ffc107; padding-bottom: 8px;">
                    "РАЗДЕЛ В - Резултат (Ръчно попълване)"
                </h5>
                <div style=row_style>
                    <span style=label_style>"01-40: Общо начислен ДДС"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_40.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-41: Данък за внасяне"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_41.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-42: ДДС за възстановяване"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_42.unwrap_or(0.0))}</span>
                </div>

                // Editable fields
                <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 0; border-bottom: 1px solid #eee; background: #fffde7;">
                    <span style=label_style>"01-43: Ефективно внесен данък"</span>
                    <input
                        type="text"
                        inputmode="decimal"
                        style=field_style
                        prop:value=move || format!("{:.2}", cell_01_43.get())
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                cell_01_43.set(v);
                            }
                        }
                    />
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 0; border-bottom: 1px solid #eee; background: #fffde7;">
                    <span style=label_style>"01-50: ДДС за внасяне от предходен период"</span>
                    <input
                        type="text"
                        inputmode="decimal"
                        style=field_style
                        prop:value=move || format!("{:.2}", cell_01_50.get())
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                cell_01_50.set(v);
                            }
                        }
                    />
                </div>
                <div style=row_style>
                    <span style=label_style>"01-51: Данък за внасяне общо"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_51.unwrap_or(0.0))}</span>
                </div>
                <div style=row_style>
                    <span style=label_style>"01-52: ДДС за възстановяване общо"</span>
                    <span style="font-weight: bold;">{format!("{:.2}", vat_return.cell_01_52.unwrap_or(0.0))}</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 0; background: #fffde7;">
                    <span style=label_style>"01-60: Данъчен кредит за приспадане"</span>
                    <input
                        type="text"
                        inputmode="decimal"
                        style=field_style
                        prop:value=move || format!("{:.2}", cell_01_60.get())
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                cell_01_60.set(v);
                            }
                        }
                    />
                </div>
            </div>

            // Save button
            <div style="margin-top: 20px; text-align: right;">
                <button
                    style="background: #27ae60; color: white; border: none; padding: 12px 30px; border-radius: 4px; cursor: pointer; font-weight: 500; font-size: 1em;"
                    on:click=save_declaration
                    disabled=move || saving.get()
                >
                    {move || if saving.get() { "Запазване..." } else { "Запази декларацията" }}
                </button>
            </div>
        </div>
    }
}

// === VAT Journal Report Helpers ===

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

fn format_date_bg(date: &str) -> String {
    if date.len() >= 10 {
        format!("{}/{}/{}", &date[8..10], &date[5..7], &date[0..4])
    } else {
        date.to_string()
    }
}

fn build_purchase_journal_html(
    items: &[VatJournalEntry],
    company_name: &str,
    company_vat: &str,
    period: &str,
) -> String {
    let mut s = String::with_capacity(16384);

    // Header
    s.push_str("<div style='text-align:center;margin-bottom:15px'>");
    s.push_str("<h2 style='margin:5px 0;font-size:16px'>ДНЕВНИК ЗА ПОКУПКИТЕ</h2>");
    s.push_str(&format!("<p style='margin:3px 0;font-size:12px'>{}</p>", html_escape(company_name)));
    s.push_str(&format!("<p style='margin:3px 0;font-size:12px'>{}</p>", html_escape(company_vat)));
    s.push_str(&format!("<p style='margin:3px 0;font-size:12px'>{}</p>", html_escape(period)));
    s.push_str("</div>");

    // Table
    let th = "border:1px solid #2c3e50;padding:4px;font-size:8px;text-align:center";
    let th_num = "border:1px solid #2c3e50;padding:2px;font-size:8px;text-align:center";
    s.push_str("<table style='width:100%;border-collapse:collapse'>");

    // Header row 1
    s.push_str("<thead>");
    s.push_str("<tr style='background:#34495e;color:white'>");
    s.push_str(&format!("<th rowspan='2' style='{};width:30px'>№ по ред</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{};width:30px'>Клон</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{};width:30px'>Вид</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{}'>№ на документа</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{};width:70px'>Дата</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{}'>ИН на контрагента</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{}'>Име на контрагента</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{}'>Вид на стоката/ услугата</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{};width:25px'>8а</th>", th));
    s.push_str(&format!("<th style='{}'>ДО без право на ДК</th>", th));
    s.push_str(&format!("<th style='{}'>ДО с право на пълен ДК</th>", th));
    s.push_str(&format!("<th style='{}'>ДДС с право на пълен ДК</th>", th));
    s.push_str(&format!("<th style='{}'>ДО с право на частичен ДК</th>", th));
    s.push_str(&format!("<th style='{}'>ДДС с право на частичен ДК</th>", th));
    s.push_str(&format!("<th style='{}'>Годишна корекция по чл.73, ал.8 ЗДДС</th>", th));
    s.push_str(&format!("<th style='{}'>ДО при тристранна операция</th>", th));
    s.push_str("</tr>");

    // Header row 2 - column numbers
    s.push_str("<tr style='background:#34495e;color:white'>");
    for num in &["9", "10", "11", "12", "13", "14", "15"] {
        s.push_str(&format!("<th style='{}'>{}</th>", th_num, num));
    }
    s.push_str("</tr>");
    s.push_str("</thead>");

    // Body
    s.push_str("<tbody>");
    let td = "border:1px solid #ddd;padding:3px;font-size:9px";
    let td_num = "border:1px solid #ddd;padding:3px;font-size:9px;text-align:right;font-family:monospace";
    let mut totals = [0.0f64; 7];

    for e in items {
        let vals = [
            e.base_no_credit.unwrap_or(0.0),
            e.base_full_credit.unwrap_or(0.0),
            e.vat_full_credit.unwrap_or(0.0),
            e.base_partial_credit.unwrap_or(0.0),
            e.vat_partial_credit.unwrap_or(0.0),
            e.annual_adjustment_base.unwrap_or(0.0),
            e.annual_adjustment_vat.unwrap_or(0.0),
        ];
        for i in 0..7 { totals[i] += vals[i]; }

        s.push_str("<tr>");
        s.push_str(&format!("<td style='{};text-align:center'>{}</td>", td, e.row_number));
        s.push_str(&format!("<td style='{};text-align:center'>0</td>", td));
        s.push_str(&format!("<td style='{};text-align:center'>{}</td>", td, html_escape(&e.document_type)));
        s.push_str(&format!("<td style='{}'>{}</td>", td, html_escape(&e.document_number)));
        s.push_str(&format!("<td style='{}'>{}</td>", td, format_date_bg(&e.document_date)));
        s.push_str(&format!("<td style='{}'>{}</td>", td, html_escape(e.counterpart_vat.as_deref().unwrap_or(""))));
        s.push_str(&format!("<td style='{}'>{}</td>", td, html_escape(e.counterpart_name.as_deref().unwrap_or(""))));
        s.push_str(&format!("<td style='{}'>{}</td>", td, html_escape(e.goods_description.as_deref().unwrap_or(""))));
        s.push_str(&format!("<td style='{};text-align:center'>{}</td>", td, html_escape(e.delivery_code.as_deref().unwrap_or(""))));
        for v in &vals {
            s.push_str(&format!("<td style='{}'>{:.2}</td>", td_num, v));
        }
        s.push_str("</tr>");
    }
    s.push_str("</tbody>");

    // Footer
    s.push_str("<tfoot><tr style='background:#ecf0f1;font-weight:bold'>");
    s.push_str(&format!("<td colspan='9' style='{};text-align:right;font-weight:bold'>ОБЩО</td>", td));
    for v in &totals {
        s.push_str(&format!("<td style='{};font-weight:bold'>{:.2}</td>", td_num, v));
    }
    s.push_str("</tr></tfoot>");
    s.push_str("</table>");

    s
}

fn build_sales_journal_html(
    items: &[VatJournalEntry],
    company_name: &str,
    company_vat: &str,
    period: &str,
) -> String {
    let mut s = String::with_capacity(32768);

    // Define all sales amount columns: (label, col_number, getter_fn)
    type Getter = fn(&VatJournalEntry) -> f64;
    let all_columns: Vec<(&str, &str, Getter)> = vec![
        ("Общ размер на ДО за облагане с ДДС", "9", (|e: &VatJournalEntry| e.total_base.unwrap_or(0.0)) as Getter),
        ("Всичко начислен ДДС", "10", |e| e.total_vat.unwrap_or(0.0)),
        ("ДО на обл. доставки 20%", "11", |e| e.sales_base_20.unwrap_or(0.0)),
        ("Начислен ДДС 20%", "12", |e| e.sales_vat_20.unwrap_or(0.0)),
        ("ДО на ВОП", "13", |e| e.sales_base_vop.unwrap_or(0.0)),
        ("Начислен ДДС за ВОП", "15", |e| e.sales_vat_vop.unwrap_or(0.0)),
        ("ДО облагаеми 9%", "16", |e| e.sales_base_9.unwrap_or(0.0)),
        ("ДДС 9%", "17", |e| e.sales_vat_9.unwrap_or(0.0)),
        ("ДО ВОП 9%", "18", |e| e.sales_base_vop_9.unwrap_or(0.0)),
        ("ДО 0% по глава трета", "19", |e| e.sales_base_0_chapter3.unwrap_or(0.0)),
        ("ДО ВОД", "20", |e| e.sales_base_vod.unwrap_or(0.0)),
        ("ДО чл.140, 146, 173", "21", |e| e.sales_base_0_articles.unwrap_or(0.0)),
        ("Услуги чл.21 ал.2", "22", |e| e.sales_base_services_21.unwrap_or(0.0)),
        ("ДО чл.69 ал.2", "23", |e| e.sales_base_69_2.unwrap_or(0.0)),
        ("ДО чл.69 ал.2 ЕС", "24", |e| e.sales_base_69_2_eu.unwrap_or(0.0)),
        ("Освободени доставки", "25", |e| e.sales_base_exempt.unwrap_or(0.0)),
        ("ДДС лични нужди", "26", |e| e.sales_vat_personal.unwrap_or(0.0)),
    ];

    // Pre-compute values for each column and filter out all-zero columns
    let column_data: Vec<(&str, &str, Vec<f64>)> = all_columns.iter().map(|(label, col, getter)| {
        let vals: Vec<f64> = items.iter().map(|e| getter(e)).collect();
        (*label, *col, vals)
    }).collect();

    let visible: Vec<(&str, &str, &Vec<f64>)> = column_data.iter()
        .filter(|(_, _, vals)| vals.iter().any(|&v| v != 0.0))
        .map(|(l, c, v)| (*l, *c, v))
        .collect();

    // Header
    s.push_str("<div style='text-align:center;margin-bottom:15px'>");
    s.push_str("<h2 style='margin:5px 0;font-size:16px'>ДНЕВНИК ЗА ПРОДАЖБИТЕ</h2>");
    s.push_str(&format!("<p style='margin:3px 0;font-size:12px'>{}</p>", html_escape(company_name)));
    s.push_str(&format!("<p style='margin:3px 0;font-size:12px'>{}</p>", html_escape(company_vat)));
    s.push_str(&format!("<p style='margin:3px 0;font-size:12px'>{}</p>", html_escape(period)));
    s.push_str("</div>");

    let th = "border:1px solid #2c3e50;padding:4px;font-size:8px;text-align:center";
    let th_num = "border:1px solid #2c3e50;padding:2px;font-size:8px;text-align:center";
    let td = "border:1px solid #ddd;padding:3px;font-size:9px";
    let td_num = "border:1px solid #ddd;padding:3px;font-size:9px;text-align:right;font-family:monospace";

    s.push_str("<table style='width:100%;border-collapse:collapse'>");

    // Header row 1
    s.push_str("<thead>");
    s.push_str("<tr style='background:#34495e;color:white'>");
    s.push_str(&format!("<th rowspan='2' style='{};width:30px'>№ по ред</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{};width:30px'>Клон</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{};width:30px'>Вид</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{}'>№ на документа</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{};width:70px'>Дата</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{}'>ИН на контрагента</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{}'>Име на контрагента</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{}'>Вид на стоката/ услугата</th>", th));
    s.push_str(&format!("<th rowspan='2' style='{};width:25px'>8а</th>", th));
    for (label, _, _) in &visible {
        s.push_str(&format!("<th style='{}'>{}</th>", th, label));
    }
    s.push_str("</tr>");

    // Header row 2 - column numbers
    s.push_str("<tr style='background:#34495e;color:white'>");
    for (_, col_num, _) in &visible {
        s.push_str(&format!("<th style='{}'>{}</th>", th_num, col_num));
    }
    s.push_str("</tr>");
    s.push_str("</thead>");

    // Body
    s.push_str("<tbody>");
    for (idx, e) in items.iter().enumerate() {
        s.push_str("<tr>");
        s.push_str(&format!("<td style='{};text-align:center'>{}</td>", td, e.row_number));
        s.push_str(&format!("<td style='{};text-align:center'>0</td>", td));
        s.push_str(&format!("<td style='{};text-align:center'>{}</td>", td, html_escape(&e.document_type)));
        s.push_str(&format!("<td style='{}'>{}</td>", td, html_escape(&e.document_number)));
        s.push_str(&format!("<td style='{}'>{}</td>", td, format_date_bg(&e.document_date)));
        s.push_str(&format!("<td style='{}'>{}</td>", td, html_escape(e.counterpart_vat.as_deref().unwrap_or(""))));
        s.push_str(&format!("<td style='{}'>{}</td>", td, html_escape(e.counterpart_name.as_deref().unwrap_or(""))));
        s.push_str(&format!("<td style='{}'>{}</td>", td, html_escape(e.goods_description.as_deref().unwrap_or(""))));
        s.push_str(&format!("<td style='{};text-align:center'>{}</td>", td, html_escape(e.delivery_code.as_deref().unwrap_or(""))));
        for (_, _, vals) in &visible {
            s.push_str(&format!("<td style='{}'>{:.2}</td>", td_num, vals[idx]));
        }
        s.push_str("</tr>");
    }
    s.push_str("</tbody>");

    // Footer - totals
    s.push_str("<tfoot><tr style='background:#ecf0f1;font-weight:bold'>");
    let info_cols = 9;
    s.push_str(&format!("<td colspan='{}' style='{};text-align:right;font-weight:bold'>ОБЩО</td>", info_cols, td));
    for (_, _, vals) in &visible {
        let total: f64 = vals.iter().sum();
        s.push_str(&format!("<td style='{};font-weight:bold'>{:.2}</td>", td_num, total));
    }
    s.push_str("</tr></tfoot>");
    s.push_str("</table>");

    s
}

fn open_journal_print(content: &str, title: &str) {
    let escaped = content
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");
    let title_escaped = title
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    let js = format!(
        r#"(function(){{var w=window.open('','_blank');w.document.write(`<!DOCTYPE html><html><head><meta charset="UTF-8"><title>{title_escaped}</title><style>body{{font-family:Arial,sans-serif;padding:15px;margin:0}}table{{width:100%;border-collapse:collapse}}th,td{{border:1px solid #333;padding:4px}}th{{background:#34495e;color:white;text-align:center}}tfoot td{{background:#ecf0f1;font-weight:bold}}h2{{margin:5px 0}}p{{margin:3px 0;color:#333}}@media print{{@page{{size:landscape;margin:0.5cm}}body{{padding:5px}}}}</style></head><body>{escaped}<script>window.onload=function(){{window.print()}}<\/script></body></html>`);w.document.close()}})();"#
    );
    let _ = js_sys::eval(&js);
}

fn export_journal_excel(content: &str, sheet_name: &str, filename: &str) {
    let escaped = content
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");
    let sheet_escaped = sheet_name
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('"', "\\\"");
    let file_escaped = filename
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('\'', "\\'");

    let js = format!(
        r#"(function(){{if(typeof XLSX==='undefined'){{alert('SheetJS не е зареден. Моля, опреснете страницата.');return}}var d=document.createElement('div');d.innerHTML=`{escaped}`;var t=d.querySelector('table');if(!t){{alert('Няма данни за експорт.');return}}var wb=XLSX.utils.table_to_book(t,{{sheet:"{sheet_escaped}"}});XLSX.writeFile(wb,'{file_escaped}')}})();"#
    );
    let _ = js_sys::eval(&js);
}
