use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::api_get;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub event: String,
    pub email: String,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub details: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventCount {
    pub event: String,
    pub count: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IpCount {
    pub ip: String,
    pub count: i64,
    pub unique_emails: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditStats {
    pub total: i64,
    pub by_event: Vec<EventCount>,
    pub top_ips: Vec<IpCount>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

fn event_label(event: &str) -> &'static str {
    match event {
        "register" => "Регистрация",
        "register_duplicate" => "Дублиран email",
        "login" => "Вход",
        "login_failed" => "Неуспешен вход",
        "login_unverified" => "Неверифициран вход",
        "verify_email" => "Верификация",
        "forgot_password" => "Забравена парола",
        _ => "Друго",
    }
}

fn event_color(event: &str) -> &'static str {
    match event {
        "register" => "#27ae60",
        "register_duplicate" => "#f39c12",
        "login" => "#3498db",
        "login_failed" => "#e74c3c",
        "login_unverified" => "#e67e22",
        "verify_email" => "#9b59b6",
        "forgot_password" => "#95a5a6",
        _ => "#7f8c8d",
    }
}

#[component]
pub fn AuditLogs() -> impl IntoView {
    let logs = create_rw_signal(Vec::<AuditLogEntry>::new());
    let stats = create_rw_signal(Option::<AuditStats>::None);
    let pagination = create_rw_signal(Pagination { page: 1, per_page: 50, total: 0, total_pages: 0 });
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());

    // Filters
    let filter_event = create_rw_signal(String::new());
    let filter_email = create_rw_signal(String::new());
    let filter_ip = create_rw_signal(String::new());

    // Tab: "log" or "stats"
    let active_tab = create_rw_signal("log".to_string());

    let load_data = move |page: i64| {
        loading.set(true);
        error_msg.set(String::new());
        let event = filter_event.get_untracked();
        let email = filter_email.get_untracked();
        let ip = filter_ip.get_untracked();

        spawn_local(async move {
            let mut url = format!("/api/admin/audit_logs?page={}&per_page=50", page);
            if !event.is_empty() {
                url.push_str(&format!("&event={}", event));
            }
            if !email.is_empty() {
                url.push_str(&format!("&email={}", email));
            }
            if !ip.is_empty() {
                url.push_str(&format!("&ip={}", ip));
            }

            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<AuditLogEntry> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        logs.set(items);
                    }
                    if let Some(p) = data.get("pagination") {
                        if let Ok(pg) = serde_json::from_value::<Pagination>(p.clone()) {
                            pagination.set(pg);
                        }
                    }
                    if let Some(s) = data.get("stats") {
                        if let Ok(st) = serde_json::from_value::<AuditStats>(s.clone()) {
                            stats.set(Some(st));
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Initial load
    let load_data_clone = load_data.clone();
    create_effect(move |_| {
        load_data_clone(1);
    });

    let load_page1 = load_data.clone();
    let on_filter = move |_| {
        load_page1(1);
    };

    let load_data_prev = load_data.clone();
    let on_prev = move |_| {
        let p = pagination.get_untracked();
        if p.page > 1 {
            load_data_prev(p.page - 1);
        }
    };

    let load_data_next = load_data.clone();
    let on_next = move |_| {
        let p = pagination.get_untracked();
        if p.page < p.total_pages {
            load_data_next(p.page + 1);
        }
    };

    // Click on IP to filter
    let load_data_ip = load_data.clone();
    let on_ip_click = move |ip: String| {
        filter_ip.set(ip);
        filter_event.set(String::new());
        filter_email.set(String::new());
        active_tab.set("log".to_string());
        load_data_ip(1);
    };

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <h1 style="margin: 0;">"Одит лог"</h1>

                // Tabs
                <div style="display: flex; gap: 5px;">
                    <button
                        style=move || format!(
                            "padding: 8px 20px; border: none; border-radius: 6px 6px 0 0; cursor: pointer; font-size: 14px; {}",
                            if active_tab.get() == "log" { "background: white; color: #2c3e50; font-weight: 600;" } else { "background: #d5dbdb; color: #7f8c8d;" }
                        )
                        on:click=move |_| active_tab.set("log".to_string())
                    >
                        "Записи"
                    </button>
                    <button
                        style=move || format!(
                            "padding: 8px 20px; border: none; border-radius: 6px 6px 0 0; cursor: pointer; font-size: 14px; {}",
                            if active_tab.get() == "stats" { "background: white; color: #2c3e50; font-weight: 600;" } else { "background: #d5dbdb; color: #7f8c8d;" }
                        )
                        on:click=move |_| active_tab.set("stats".to_string())
                    >
                        "Статистика"
                    </button>
                </div>
            </div>

            // Error
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 12px; border-radius: 6px; margin-top: 15px;">
                            {err}
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Log tab
            {move || {
                if active_tab.get() != "log" {
                    return view! { <span></span> }.into_view();
                }

                let on_filter = on_filter.clone();

                view! {
                    // Filters
                    <div style="background: white; border-radius: 8px; padding: 15px; margin-top: 15px; display: flex; gap: 10px; align-items: end; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                        <div style="flex: 1;">
                            <label style="display: block; font-size: 12px; color: #7f8c8d; margin-bottom: 4px;">"Събитие"</label>
                            <select
                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                                prop:value=move || filter_event.get()
                                on:change=move |e| filter_event.set(event_target_value(&e))
                            >
                                <option value="">"Всички"</option>
                                <option value="register">"Регистрация"</option>
                                <option value="register_duplicate">"Дублиран email"</option>
                                <option value="login">"Вход"</option>
                                <option value="login_failed">"Неуспешен вход"</option>
                                <option value="login_unverified">"Неверифициран"</option>
                                <option value="verify_email">"Верификация"</option>
                                <option value="forgot_password">"Забравена парола"</option>
                            </select>
                        </div>
                        <div style="flex: 1;">
                            <label style="display: block; font-size: 12px; color: #7f8c8d; margin-bottom: 4px;">"Email"</label>
                            <input
                                type="text"
                                placeholder="Търсене по email..."
                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                prop:value=move || filter_email.get()
                                on:change=move |e| filter_email.set(event_target_value(&e))
                            />
                        </div>
                        <div style="flex: 1;">
                            <label style="display: block; font-size: 12px; color: #7f8c8d; margin-bottom: 4px;">"IP адрес"</label>
                            <input
                                type="text"
                                placeholder="Търсене по IP..."
                                style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                prop:value=move || filter_ip.get()
                                on:change=move |e| filter_ip.set(event_target_value(&e))
                            />
                        </div>
                        <button
                            style="padding: 8px 20px; background: #3498db; color: white; border: none; border-radius: 4px; cursor: pointer; white-space: nowrap;"
                            on:click=on_filter
                        >
                            "Филтрирай"
                        </button>
                    </div>

                    // Table
                    <div style="background: white; border-radius: 8px; margin-top: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: hidden;">
                        {move || {
                            if loading.get() {
                                return view! {
                                    <div style="padding: 40px; text-align: center; color: #7f8c8d;">"Зареждане..."</div>
                                }.into_view();
                            }

                            let entries = logs.get();
                            let pg = pagination.get();

                            view! {
                                // Pagination header
                                <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 15px; border-bottom: 1px solid #ecf0f1;">
                                    <span style="color: #7f8c8d; font-size: 13px;">
                                        {format!("Общо: {} записа", pg.total)}
                                    </span>
                                    <div style="display: flex; gap: 8px; align-items: center;">
                                        <button
                                            style="padding: 6px 12px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; background: white;"
                                            on:click=on_prev.clone()
                                            disabled=move || pagination.get().page <= 1
                                        >
                                            "< Назад"
                                        </button>
                                        <span style="font-size: 13px; color: #7f8c8d;">
                                            {move || {
                                                let p = pagination.get();
                                                format!("{} / {}", p.page, p.total_pages.max(1))
                                            }}
                                        </span>
                                        <button
                                            style="padding: 6px 12px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; background: white;"
                                            on:click=on_next.clone()
                                            disabled=move || { let p = pagination.get(); p.page >= p.total_pages }
                                        >
                                            "Напред >"
                                        </button>
                                    </div>
                                </div>

                                <table style="width: 100%; border-collapse: collapse;">
                                    <thead style="background: #f8f9fa;">
                                        <tr>
                                            <th style="padding: 10px 12px; text-align: left; color: #7f8c8d; font-weight: 500; font-size: 13px;">"Време"</th>
                                            <th style="padding: 10px 12px; text-align: left; color: #7f8c8d; font-weight: 500; font-size: 13px;">"Събитие"</th>
                                            <th style="padding: 10px 12px; text-align: left; color: #7f8c8d; font-weight: 500; font-size: 13px;">"Email"</th>
                                            <th style="padding: 10px 12px; text-align: left; color: #7f8c8d; font-weight: 500; font-size: 13px;">"IP адрес"</th>
                                            <th style="padding: 10px 12px; text-align: left; color: #7f8c8d; font-weight: 500; font-size: 13px;">"Детайли"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {if entries.is_empty() {
                                            view! {
                                                <tr>
                                                    <td colspan="5" style="padding: 30px; text-align: center; color: #7f8c8d;">"Няма записи"</td>
                                                </tr>
                                            }.into_view()
                                        } else {
                                            entries.into_iter().map(|entry| {
                                                let created = entry.created_at.clone().unwrap_or_default();
                                                let time_str = created.replace("T", " ").chars().take(19).collect::<String>();
                                                let ev = entry.event.clone();
                                                let color = event_color(&ev);
                                                let label = event_label(&ev);
                                                let ip = entry.ip_address.clone();
                                                let ip_for_click = ip.clone();
                                                let on_ip_click = on_ip_click.clone();

                                                view! {
                                                    <tr style="border-bottom: 1px solid #f0f0f0;">
                                                        <td style="padding: 10px 12px; font-size: 13px; color: #555; white-space: nowrap;">
                                                            {time_str}
                                                        </td>
                                                        <td style="padding: 10px 12px;">
                                                            <span style=format!(
                                                                "display: inline-block; padding: 3px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                                color
                                                            )>
                                                                {label}
                                                            </span>
                                                        </td>
                                                        <td style="padding: 10px 12px; font-size: 13px;">{entry.email}</td>
                                                        <td style="padding: 10px 12px;">
                                                            <span
                                                                style="font-size: 13px; font-family: monospace; color: #3498db; cursor: pointer; text-decoration: underline;"
                                                                on:click=move |_| on_ip_click(ip_for_click.clone())
                                                            >
                                                                {ip.clone()}
                                                            </span>
                                                        </td>
                                                        <td style="padding: 10px 12px; font-size: 12px; color: #7f8c8d; max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                                                            {entry.details.unwrap_or_default()}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        }}
                                    </tbody>
                                </table>
                            }.into_view()
                        }}
                    </div>
                }.into_view()
            }}

            // Stats tab
            {move || {
                if active_tab.get() != "stats" {
                    return view! { <span></span> }.into_view();
                }

                let st = stats.get();
                let on_ip_click = on_ip_click.clone();

                match st {
                    None => view! {
                        <div style="background: white; border-radius: 8px; padding: 40px; margin-top: 15px; text-align: center; color: #7f8c8d;">
                            "Няма данни"
                        </div>
                    }.into_view(),
                    Some(s) => view! {
                        // Summary cards
                        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 15px; margin-top: 15px;">
                            {s.by_event.clone().into_iter().map(|ec| {
                                let color = event_color(&ec.event);
                                let label = event_label(&ec.event);
                                view! {
                                    <div style="background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                                        <div style="display: flex; justify-content: space-between; align-items: center;">
                                            <span style=format!(
                                                "display: inline-block; padding: 3px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                color
                                            )>
                                                {label}
                                            </span>
                                            <span style="font-size: 28px; font-weight: 700; color: #2c3e50;">{ec.count}</span>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>

                        // Top IPs table
                        <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            <h3 style="margin: 0 0 15px 0; color: #2c3e50;">"Топ IP адреси"</h3>
                            <table style="width: 100%; border-collapse: collapse;">
                                <thead style="background: #f8f9fa;">
                                    <tr>
                                        <th style="padding: 10px 12px; text-align: left; color: #7f8c8d; font-weight: 500; font-size: 13px;">"IP адрес"</th>
                                        <th style="padding: 10px 12px; text-align: right; color: #7f8c8d; font-weight: 500; font-size: 13px;">"Заявки"</th>
                                        <th style="padding: 10px 12px; text-align: right; color: #7f8c8d; font-weight: 500; font-size: 13px;">"Уникални email-и"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {s.top_ips.clone().into_iter().map(|ip_stat| {
                                        let ip = ip_stat.ip.clone();
                                        let ip_for_click = ip.clone();
                                        let on_ip_click = on_ip_click.clone();
                                        view! {
                                            <tr style="border-bottom: 1px solid #f0f0f0;">
                                                <td style="padding: 10px 12px;">
                                                    <span
                                                        style="font-family: monospace; color: #3498db; cursor: pointer; text-decoration: underline;"
                                                        on:click=move |_| on_ip_click(ip_for_click.clone())
                                                    >
                                                        {ip}
                                                    </span>
                                                </td>
                                                <td style="padding: 10px 12px; text-align: right; font-weight: 600;">
                                                    {ip_stat.count}
                                                </td>
                                                <td style="padding: 10px 12px; text-align: right;">
                                                    {ip_stat.unique_emails}
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }.into_view(),
                }
            }}
        </Layout>
    }
}
