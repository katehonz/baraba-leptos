use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::api_get;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserStats {
    pub total: i64,
    pub active: i64,
    pub inactive: i64,
    pub super_admins: i64,
    pub verified: i64,
    pub unverified: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompanyStats {
    pub total: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleStats {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub user_count: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentUser {
    pub id: i64,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentCompany {
    pub id: i64,
    pub name: String,
    pub eik: Option<String>,
    pub city: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminStats {
    pub users: UserStats,
    pub companies: CompanyStats,
    pub roles: Vec<RoleStats>,
    pub recent_users: Vec<RecentUser>,
    pub recent_companies: Vec<RecentCompany>,
}

fn get_role_label(role: &str) -> &'static str {
    match role {
        "super_admin" => "Супер Админ",
        "admin" => "Администратор",
        "accountant" => "Счетоводител",
        "viewer" => "Наблюдател",
        _ => "Друг",
    }
}

fn get_role_color(role: &str) -> &'static str {
    match role {
        "super_admin" => "#e74c3c",
        "admin" => "#f39c12",
        "accountant" => "#3498db",
        "viewer" => "#95a5a6",
        _ => "#7f8c8d",
    }
}

#[component]
pub fn AdminDashboard() -> impl IntoView {
    let stats = create_rw_signal(AdminStats::default());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());

    // Load stats
    create_effect(move |_| {
        spawn_local(async move {
            match api_get("/api/admin/stats").await {
                Ok(data) => {
                    if let Some(stats_data) = data.get("data") {
                        if let Ok(s) = serde_json::from_value::<AdminStats>(stats_data.clone()) {
                            stats.set(s);
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            loading.set(false);
        });
    });

    view! {
        <Layout>
            <h1>"Администрация"</h1>

            // Error message
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {err}
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Loading
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

            // Stats cards
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                let s = stats.get();

                view! {
                    // Stats cards row
                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-top: 20px;">
                        // Total users card
                        <div style="background: white; border-radius: 8px; padding: 25px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                <div>
                                    <p style="margin: 0; color: #7f8c8d; font-size: 14px;">"Общо потребители"</p>
                                    <h2 style="margin: 10px 0 0 0; color: #2c3e50; font-size: 32px;">{s.users.total}</h2>
                                </div>
                                <div style="width: 50px; height: 50px; background: #3498db; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 24px;">
                                    "👥"
                                </div>
                            </div>
                            <div style="margin-top: 15px; display: flex; gap: 15px; font-size: 13px;">
                                <span style="color: #27ae60;">{format!("{} активни", s.users.active)}</span>
                                <span style="color: #95a5a6;">{format!("{} неактивни", s.users.inactive)}</span>
                            </div>
                        </div>

                        // Companies card
                        <div style="background: white; border-radius: 8px; padding: 25px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                <div>
                                    <p style="margin: 0; color: #7f8c8d; font-size: 14px;">"Общо фирми"</p>
                                    <h2 style="margin: 10px 0 0 0; color: #2c3e50; font-size: 32px;">{s.companies.total}</h2>
                                </div>
                                <div style="width: 50px; height: 50px; background: #27ae60; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 24px;">
                                    "🏢"
                                </div>
                            </div>
                        </div>

                        // Super admins card
                        <div style="background: white; border-radius: 8px; padding: 25px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                <div>
                                    <p style="margin: 0; color: #7f8c8d; font-size: 14px;">"Супер администратори"</p>
                                    <h2 style="margin: 10px 0 0 0; color: #2c3e50; font-size: 32px;">{s.users.super_admins}</h2>
                                </div>
                                <div style="width: 50px; height: 50px; background: #e74c3c; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 24px;">
                                    "🛡️"
                                </div>
                            </div>
                        </div>

                        // Verification card
                        <div style="background: white; border-radius: 8px; padding: 25px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                <div>
                                    <p style="margin: 0; color: #7f8c8d; font-size: 14px;">"Верификация"</p>
                                    <h2 style="margin: 10px 0 0 0; color: #2c3e50; font-size: 32px;">{s.users.verified}</h2>
                                </div>
                                <div style="width: 50px; height: 50px; background: #9b59b6; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 24px;">
                                    "✓"
                                </div>
                            </div>
                            <div style="margin-top: 15px; font-size: 13px;">
                                <span style="color: #e74c3c;">{format!("{} неверифицирани", s.users.unverified)}</span>
                            </div>
                        </div>
                    </div>

                    // Two column layout for roles and recent activity
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-top: 20px;">
                        // Roles breakdown
                        <div style="background: white; border-radius: 8px; padding: 25px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            <h3 style="margin: 0 0 20px 0; color: #2c3e50;">"Разпределение по роли"</h3>
                            <div style="display: flex; flex-direction: column; gap: 12px;">
                                {s.roles.clone().into_iter().map(|role| {
                                    view! {
                                        <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px; background: #f8f9fa; border-radius: 6px;">
                                            <div style="display: flex; align-items: center; gap: 10px;">
                                                <span style=format!(
                                                    "display: inline-block; width: 10px; height: 10px; border-radius: 50%; background: {};",
                                                    get_role_color(&role.name)
                                                )></span>
                                                <span style="font-weight: 500;">{get_role_label(&role.name)}</span>
                                            </div>
                                            <span style="background: #ecf0f1; padding: 4px 12px; border-radius: 12px; font-size: 13px; font-weight: 500;">
                                                {role.user_count}
                                            </span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // Quick actions
                        <div style="background: white; border-radius: 8px; padding: 25px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            <h3 style="margin: 0 0 20px 0; color: #2c3e50;">"Бързи действия"</h3>
                            <div style="display: flex; flex-direction: column; gap: 12px;">
                                <a href="/users" style="display: flex; align-items: center; gap: 12px; padding: 15px; background: #f8f9fa; border-radius: 6px; text-decoration: none; color: #2c3e50; transition: background 0.2s;">
                                    <span style="font-size: 20px;">"👥"</span>
                                    <span>"Управление на потребители"</span>
                                </a>
                                <a href="/companies" style="display: flex; align-items: center; gap: 12px; padding: 15px; background: #f8f9fa; border-radius: 6px; text-decoration: none; color: #2c3e50; transition: background 0.2s;">
                                    <span style="font-size: 20px;">"🏢"</span>
                                    <span>"Управление на фирми"</span>
                                </a>
                                <a href="/roles" style="display: flex; align-items: center; gap: 12px; padding: 15px; background: #f8f9fa; border-radius: 6px; text-decoration: none; color: #2c3e50; transition: background 0.2s;">
                                    <span style="font-size: 20px;">"🔐"</span>
                                    <span>"Управление на роли"</span>
                                </a>
                                <a href="/system-settings" style="display: flex; align-items: center; gap: 12px; padding: 15px; background: #f8f9fa; border-radius: 6px; text-decoration: none; color: #2c3e50; transition: background 0.2s;">
                                    <span style="font-size: 20px;">"⚙️"</span>
                                    <span>"Системни настройки"</span>
                                </a>
                            </div>
                        </div>
                    </div>

                    // Recent users table
                    <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                            <h3 style="margin: 0; color: #2c3e50;">"Последни регистрации"</h3>
                            <a href="/users" style="color: #3498db; text-decoration: none; font-size: 14px;">"Виж всички →"</a>
                        </div>
                        <table style="width: 100%; border-collapse: collapse;">
                            <thead style="background: #f8f9fa;">
                                <tr>
                                    <th style="padding: 12px; text-align: left; color: #7f8c8d; font-weight: 500;">"Email"</th>
                                    <th style="padding: 12px; text-align: left; color: #7f8c8d; font-weight: 500;">"Име"</th>
                                    <th style="padding: 12px; text-align: center; color: #7f8c8d; font-weight: 500;">"Статус"</th>
                                    <th style="padding: 12px; text-align: right; color: #7f8c8d; font-weight: 500;">"Дата"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {s.recent_users.clone().into_iter().map(|user| {
                                    let full_name = format!("{} {}",
                                        user.first_name.unwrap_or_default(),
                                        user.last_name.unwrap_or_default()
                                    ).trim().to_string();
                                    let created = user.created_at.clone().unwrap_or_default();
                                    let date_str = created.split('T').next().unwrap_or(&created).to_string();

                                    view! {
                                        <tr style="border-bottom: 1px solid #ecf0f1;">
                                            <td style="padding: 12px;">{user.email}</td>
                                            <td style="padding: 12px; color: #7f8c8d;">
                                                {if full_name.is_empty() { "-".to_string() } else { full_name }}
                                            </td>
                                            <td style="padding: 12px; text-align: center;">
                                                <span style=format!(
                                                    "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                    if user.is_active { "#27ae60" } else { "#95a5a6" }
                                                )>
                                                    {if user.is_active { "Активен" } else { "Неактивен" }}
                                                </span>
                                            </td>
                                            <td style="padding: 12px; text-align: right; color: #7f8c8d; font-size: 13px;">
                                                {date_str}
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>

                    // Recent companies table
                    <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                            <h3 style="margin: 0; color: #2c3e50;">"Последни фирми"</h3>
                            <a href="/companies" style="color: #3498db; text-decoration: none; font-size: 14px;">"Виж всички →"</a>
                        </div>
                        <table style="width: 100%; border-collapse: collapse;">
                            <thead style="background: #f8f9fa;">
                                <tr>
                                    <th style="padding: 12px; text-align: left; color: #7f8c8d; font-weight: 500;">"Име"</th>
                                    <th style="padding: 12px; text-align: left; color: #7f8c8d; font-weight: 500;">"ЕИК"</th>
                                    <th style="padding: 12px; text-align: left; color: #7f8c8d; font-weight: 500;">"Град"</th>
                                    <th style="padding: 12px; text-align: right; color: #7f8c8d; font-weight: 500;">"Дата"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {s.recent_companies.clone().into_iter().map(|company| {
                                    let created = company.created_at.clone().unwrap_or_default();
                                    let date_str = created.split('T').next().unwrap_or(&created).to_string();

                                    view! {
                                        <tr style="border-bottom: 1px solid #ecf0f1;">
                                            <td style="padding: 12px; font-weight: 500;">{company.name}</td>
                                            <td style="padding: 12px; color: #7f8c8d;">
                                                {company.eik.unwrap_or_else(|| "-".to_string())}
                                            </td>
                                            <td style="padding: 12px; color: #7f8c8d;">
                                                {company.city.unwrap_or_else(|| "-".to_string())}
                                            </td>
                                            <td style="padding: 12px; text-align: right; color: #7f8c8d; font-size: 13px;">
                                                {date_str}
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_view()
            }}
        </Layout>
    }
}
