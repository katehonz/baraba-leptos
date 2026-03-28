use leptos::*;
use crate::components::Layout;
use crate::api::{get_token, API_BASE};
use crate::i18n::use_i18n;
use crate::context::use_company_context;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlAnchorElement};

#[derive(Clone, Debug, serde::Deserialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
    summary: ValidationSummary,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ValidationSummary {
    company: String,
    period: String,
    journal_entries: i32,
    invoices: i32,
    counterparts: i32,
    accounts: i32,
}

#[component]
pub fn SaftExport() -> impl IntoView {
    let i18n = use_i18n();
    let company_ctx = use_company_context();

    let selected_year = create_rw_signal(2026i32);
    let selected_month = create_rw_signal(1i32);
    let report_type = create_rw_signal("monthly".to_string());

    let validation_result = create_rw_signal(Option::<ValidationResult>::None);
    let loading = create_rw_signal(false);
    let error_msg = create_rw_signal(Option::<String>::None);

    let validate = move |_| {
        let company_id = company_ctx.selected_company_id.get().unwrap_or(0);
        let year = selected_year.get();
        let month = selected_month.get();

        if company_id == 0 {
            error_msg.set(Some(i18n.t("saft.selectCompanyFirst")));
            return;
        }

        loading.set(true);
        error_msg.set(None);
        validation_result.set(None);

        spawn_local(async move {
            let token = get_token().unwrap_or_default();
            let url = format!(
                "{}/api/saft/validate?company_id={}&year={}&month={}",
                API_BASE, company_id, year, month
            );

            match gloo_net::http::Request::get(&url)
                .header("Authorization", &format!("Bearer {}", token))
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<ValidationResult>().await {
                            Ok(result) => {
                                validation_result.set(Some(result));
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Failed to parse response: {}", e)));
                            }
                        }
                    } else {
                        error_msg.set(Some(format!("Validation failed: HTTP {}", response.status())));
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Validation failed: {}", e)));
                }
            }
            loading.set(false);
        });
    };

    let export_xml = move |_| {
        let company_id = company_ctx.selected_company_id.get().unwrap_or(0);
        let year = selected_year.get();
        let month = selected_month.get();
        let rtype = report_type.get();

        if company_id == 0 {
            error_msg.set(Some(i18n.t("saft.selectCompanyFirst")));
            return;
        }

        loading.set(true);
        error_msg.set(None);

        let companies = company_ctx.companies.get();
        let company_eik = companies.iter()
            .find(|c| c.id == company_id)
            .and_then(|c| c.eik.clone())
            .unwrap_or_default();

        spawn_local(async move {
            let token = get_token().unwrap_or_default();
            let url = format!(
                "{}/api/saft/export?company_id={}&year={}&month={}&report_type={}",
                API_BASE, company_id, year, month, rtype
            );

            match gloo_net::http::Request::get(&url)
                .header("Authorization", &format!("Bearer {}", token))
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        match response.text().await {
                            Ok(xml_content) => {
                                // Trigger download using data URL
                                if let Some(window) = window() {
                                    let document = window.document().unwrap();
                                    let a = document.create_element("a").unwrap();
                                    let anchor: HtmlAnchorElement = a.dyn_into().unwrap();

                                    let filename = format!(
                                        "SAFT_{}_{}{:02}_{}.xml",
                                        company_eik,
                                        year, month, rtype
                                    );

                                    // Create a data URL for the XML content
                                    let encoded = js_sys::encode_uri_component(&xml_content);
                                    let data_url = format!("data:application/xml;charset=utf-8,{}", encoded);

                                    anchor.set_href(&data_url);
                                    anchor.set_download(&filename);
                                    anchor.click();
                                }
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Export failed: {}", e)));
                            }
                        }
                    } else {
                        error_msg.set(Some(format!("Export failed: HTTP {}", response.status())));
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Export failed: {}", e)));
                }
            }
            loading.set(false);
        });
    };

    let years: Vec<i32> = (2020..=2030).collect();
    let months: Vec<(i32, &str, &str)> = vec![
        (1, "Януари", "January"),
        (2, "Февруари", "February"),
        (3, "Март", "March"),
        (4, "Април", "April"),
        (5, "Май", "May"),
        (6, "Юни", "June"),
        (7, "Юли", "July"),
        (8, "Август", "August"),
        (9, "Септември", "September"),
        (10, "Октомври", "October"),
        (11, "Ноември", "November"),
        (12, "Декември", "December"),
    ];

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <h1>{move || i18n.t("saft.title")}</h1>
            </div>

            // Info box
            <div style="background: #e3f2fd; border-radius: 8px; padding: 15px; margin: 20px 0; border-left: 4px solid #2196f3;">
                <strong>{move || i18n.t("saft.infoTitle")}</strong>
                <p style="margin: 10px 0 0 0; color: #555;">
                    {move || i18n.t("saft.infoText")}
                </p>
            </div>

            // Current company display
            <div style="background: #fff3e0; border-radius: 8px; padding: 15px; margin-bottom: 20px; border-left: 4px solid #ff9800;">
                <strong>"Избрана фирма: "</strong>
                {move || {
                    if let Some(company) = company_ctx.selected_company() {
                        let display = if let Some(eik) = &company.eik {
                            format!("{} ({})", company.name, eik)
                        } else {
                            company.name.clone()
                        };
                        view! { <span style="font-weight: bold; color: #e65100;">{display}</span> }.into_view()
                    } else {
                        view! { <span style="color: #666;">"Моля изберете фирма от менюто"</span> }.into_view()
                    }
                }}
            </div>

            // Selection form
            <div style="background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                <h3 style="margin-top: 0;">{move || i18n.t("saft.selectPeriod")}</h3>

                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 20px;">
                    // Year select
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">{move || i18n.t("saft.year")}</label>
                        <select
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                            on:change=move |ev| {
                                let value = event_target_value(&ev).parse::<i32>().unwrap_or(2025);
                                selected_year.set(value);
                            }
                        >
                            {years.iter().map(|y| {
                                view! {
                                    <option value={y.to_string()} selected={*y == selected_year.get_untracked()}>
                                        {y.to_string()}
                                    </option>
                                }
                            }).collect_view()}
                        </select>
                    </div>

                    // Month select
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">{move || i18n.t("saft.month")}</label>
                        <select
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                            on:change=move |ev| {
                                let value = event_target_value(&ev).parse::<i32>().unwrap_or(1);
                                selected_month.set(value);
                            }
                        >
                            {months.iter().map(|(m, bg, en)| {
                                let label = if i18n.lang_code() == "bg" { *bg } else { *en };
                                view! {
                                    <option value={m.to_string()} selected={*m == selected_month.get_untracked()}>
                                        {label}
                                    </option>
                                }
                            }).collect_view()}
                        </select>
                    </div>

                    // Report type select
                    <div>
                        <label style="display: block; margin-bottom: 5px; font-weight: bold;">{move || i18n.t("saft.reportType")}</label>
                        <select
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                            on:change=move |ev| {
                                report_type.set(event_target_value(&ev));
                            }
                        >
                            <option value="monthly" selected={report_type.get_untracked() == "monthly"}>
                                {move || i18n.t("saft.typeMonthly")}
                            </option>
                            <option value="ondemand" selected={report_type.get_untracked() == "ondemand"}>
                                {move || i18n.t("saft.typeOnDemand")}
                            </option>
                            <option value="annual" selected={report_type.get_untracked() == "annual"}>
                                {move || i18n.t("saft.typeAnnual")}
                            </option>
                        </select>
                    </div>
                </div>

                <div style="display: flex; gap: 10px;">
                    <button
                        style="background: #f39c12; color: white; border: none; padding: 12px 24px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                        on:click=validate
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { i18n.t("common.loading") } else { i18n.t("saft.validate") }}
                    </button>
                    <button
                        style="background: #27ae60; color: white; border: none; padding: 12px 24px; border-radius: 4px; cursor: pointer; font-size: 1em;"
                        on:click=export_xml
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { i18n.t("common.loading") } else { i18n.t("saft.export") }}
                    </button>
                </div>
            </div>

            // Error message
            {move || error_msg.get().map(|msg| view! {
                <div style="background: #ffebee; border-radius: 8px; padding: 15px; margin-bottom: 20px; border-left: 4px solid #f44336; color: #c62828;">
                    {msg}
                </div>
            })}

            // Validation results
            {move || validation_result.get().map(|result| {
                view! {
                    <div style="background: white; border-radius: 8px; padding: 20px;">
                        <h3 style="margin-top: 0;">
                            {move || i18n.t("saft.validationResults")}
                            {if result.valid {
                                view! { <span style="color: #27ae60; margin-left: 10px;">"\u{2713}"</span> }.into_view()
                            } else {
                                view! { <span style="color: #e74c3c; margin-left: 10px;">"\u{2717}"</span> }.into_view()
                            }}
                        </h3>

                        // Summary
                        <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px; margin-bottom: 20px;">
                            <div style="padding: 15px; background: #f5f5f5; border-radius: 8px; text-align: center;">
                                <div style="font-size: 1.5em; font-weight: bold;">{result.summary.journal_entries}</div>
                                <div style="color: #666;">{move || i18n.t("saft.journalEntries")}</div>
                            </div>
                            <div style="padding: 15px; background: #f5f5f5; border-radius: 8px; text-align: center;">
                                <div style="font-size: 1.5em; font-weight: bold;">{result.summary.invoices}</div>
                                <div style="color: #666;">{move || i18n.t("saft.invoices")}</div>
                            </div>
                            <div style="padding: 15px; background: #f5f5f5; border-radius: 8px; text-align: center;">
                                <div style="font-size: 1.5em; font-weight: bold;">{result.summary.counterparts}</div>
                                <div style="color: #666;">{move || i18n.t("saft.counterparts")}</div>
                            </div>
                            <div style="padding: 15px; background: #f5f5f5; border-radius: 8px; text-align: center;">
                                <div style="font-size: 1.5em; font-weight: bold;">{result.summary.accounts}</div>
                                <div style="color: #666;">{move || i18n.t("saft.accounts")}</div>
                            </div>
                        </div>

                        // Errors
                        {if !result.errors.is_empty() {
                            view! {
                                <div style="margin-bottom: 15px;">
                                    <h4 style="color: #e74c3c; margin-bottom: 10px;">{move || i18n.t("saft.errors")}</h4>
                                    <ul style="margin: 0; padding-left: 20px;">
                                        {result.errors.iter().map(|e| view! {
                                            <li style="color: #c62828; margin-bottom: 5px;">{e.clone()}</li>
                                        }).collect_view()}
                                    </ul>
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }}

                        // Warnings
                        {if !result.warnings.is_empty() {
                            view! {
                                <div>
                                    <h4 style="color: #f39c12; margin-bottom: 10px;">{move || i18n.t("saft.warnings")}</h4>
                                    <ul style="margin: 0; padding-left: 20px;">
                                        {result.warnings.iter().map(|w| view! {
                                            <li style="color: #e67e22; margin-bottom: 5px;">{w.clone()}</li>
                                        }).collect_view()}
                                    </ul>
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }}

                        // Valid message
                        {if result.valid && result.errors.is_empty() && result.warnings.is_empty() {
                            view! {
                                <div style="color: #27ae60; font-weight: bold;">
                                    {move || i18n.t("saft.allValid")}
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }}
                    </div>
                }
            })}
        </Layout>
    }
}
