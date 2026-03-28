use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::api_get;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Option<String>,
    pub permissions_list: Vec<String>,
    pub user_count: i64,
    pub is_super_admin: bool,
}

fn get_role_label(role: &str) -> &'static str {
    match role {
        "super_admin" => "Супер Администратор",
        "admin" => "Администратор",
        "accountant" => "Счетоводител",
        "viewer" => "Наблюдател",
        _ => "Друга",
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

fn get_permission_label(permission: &str) -> String {
    match permission {
        "*" => "Всички права".to_string(),
        "admin:read" => "Админ: Четене".to_string(),
        "admin:create" => "Админ: Създаване".to_string(),
        "admin:update" => "Админ: Редактиране".to_string(),
        "admin:delete" => "Админ: Изтриване".to_string(),
        "accounting:read" => "Счетоводство: Четене".to_string(),
        "accounting:write" => "Счетоводство: Писане".to_string(),
        "accounting:post" => "Счетоводство: Осчетоводяване".to_string(),
        "document:read" => "Документи: Четене".to_string(),
        "document:upload" => "Документи: Качване".to_string(),
        "document:process" => "Документи: Обработка".to_string(),
        "report:read" => "Справки: Четене".to_string(),
        "report:export" => "Справки: Експорт".to_string(),
        "invoice:read" => "Фактури: Четене".to_string(),
        "invoice:create" => "Фактури: Създаване".to_string(),
        "invoice:update" => "Фактури: Редактиране".to_string(),
        "invoice:delete" => "Фактури: Изтриване".to_string(),
        "invoice:send" => "Фактури: Изпращане".to_string(),
        "payment:read" => "Плащания: Четене".to_string(),
        "payment:create" => "Плащания: Създаване".to_string(),
        "payment:update" => "Плащания: Редактиране".to_string(),
        "payment:delete" => "Плащания: Изтриване".to_string(),
        "vat:read" => "ДДС: Четене".to_string(),
        "vat:create" => "ДДС: Създаване".to_string(),
        "vat:submit" => "ДДС: Подаване".to_string(),
        "settings:read" => "Настройки: Четене".to_string(),
        "settings:update" => "Настройки: Промяна".to_string(),
        "user:read" => "Потребители: Четене".to_string(),
        "user:create" => "Потребители: Създаване".to_string(),
        "user:update" => "Потребители: Редактиране".to_string(),
        "user:delete" => "Потребители: Изтриване".to_string(),
        "company:read" => "Фирми: Четене".to_string(),
        "company:create" => "Фирми: Създаване".to_string(),
        "company:update" => "Фирми: Редактиране".to_string(),
        "company:delete" => "Фирми: Изтриване".to_string(),
        "role:read" => "Роли: Четене".to_string(),
        "role:create" => "Роли: Създаване".to_string(),
        "role:update" => "Роли: Редактиране".to_string(),
        "role:delete" => "Роли: Изтриване".to_string(),
        _ => permission.to_string(),
    }
}

fn get_permission_category(permission: &str) -> &'static str {
    if permission == "*" {
        return "system";
    }
    match permission.split(':').next().unwrap_or("other") {
        "admin" => "admin",
        "accounting" => "accounting",
        "document" => "document",
        "report" => "report",
        "invoice" => "invoice",
        "payment" => "payment",
        "vat" => "vat",
        "settings" => "settings",
        "user" => "user",
        "company" => "company",
        "role" => "role",
        _ => "other",
    }
}

fn get_category_label(category: &str) -> &'static str {
    match category {
        "system" => "Системни",
        "admin" => "Администрация",
        "accounting" => "Счетоводство",
        "document" => "Документи",
        "report" => "Справки",
        "invoice" => "Фактури",
        "payment" => "Плащания",
        "vat" => "ДДС",
        "settings" => "Настройки",
        "user" => "Потребители",
        "company" => "Фирми",
        "role" => "Роли",
        _ => "Други",
    }
}

#[component]
pub fn Roles() -> impl IntoView {
    let roles = create_rw_signal(Vec::<Role>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let selected_role = create_rw_signal(Option::<Role>::None);

    // Load roles
    create_effect(move |_| {
        spawn_local(async move {
            match api_get("/api/roles").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Role> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        roles.set(items);
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
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Роли и права"</h1>
            </div>

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

            // Roles list
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                let role_list = roles.get();

                view! {
                    <div style="display: grid; grid-template-columns: 300px 1fr; gap: 20px; margin-top: 20px;">
                        // Roles sidebar
                        <div style="background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            <h3 style="margin: 0 0 15px 0; color: #2c3e50;">"Системни роли"</h3>
                            <div style="display: flex; flex-direction: column; gap: 10px;">
                                {role_list.clone().into_iter().map(|role| {
                                    let role_for_click = role.clone();
                                    let role_name = role.name.clone();
                                    let is_selected = move || {
                                        selected_role.get().map(|r| r.id == role.id).unwrap_or(false)
                                    };

                                    view! {
                                        <div
                                            style=move || format!(
                                                "padding: 15px; border-radius: 8px; cursor: pointer; transition: background 0.2s; {}",
                                                if is_selected() { "background: #ecf0f1;" } else { "background: #f8f9fa;" }
                                            )
                                            on:click=move |_| selected_role.set(Some(role_for_click.clone()))
                                        >
                                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                                <div style="display: flex; align-items: center; gap: 10px;">
                                                    <span style=format!(
                                                        "display: inline-block; width: 12px; height: 12px; border-radius: 50%; background: {};",
                                                        get_role_color(&role_name)
                                                    )></span>
                                                    <span style="font-weight: 500;">{get_role_label(&role_name)}</span>
                                                </div>
                                                <span style="background: #ddd; padding: 2px 8px; border-radius: 10px; font-size: 12px;">
                                                    {role.user_count}
                                                </span>
                                            </div>
                                            {role.description.clone().map(|d| {
                                                view! {
                                                    <p style="margin: 8px 0 0 22px; font-size: 13px; color: #7f8c8d;">{d}</p>
                                                }
                                            })}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // Permissions panel
                        <div style="background: white; border-radius: 8px; padding: 25px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                            {move || {
                                match selected_role.get() {
                                    Some(role) => {
                                        let permissions = role.permissions_list.clone();

                                        // Group permissions by category
                                        let mut categories: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
                                        for perm in permissions.iter() {
                                            let category = get_permission_category(perm);
                                            categories.entry(category).or_insert_with(Vec::new).push(perm);
                                        }

                                        // Sort categories
                                        let mut sorted_categories: Vec<_> = categories.into_iter().collect();
                                        sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

                                        view! {
                                            <div>
                                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 25px;">
                                                    <div>
                                                        <h2 style="margin: 0; color: #2c3e50;">{get_role_label(&role.name)}</h2>
                                                        <p style="margin: 5px 0 0 0; color: #7f8c8d;">
                                                            {role.description.clone().unwrap_or_else(|| "Без описание".to_string())}
                                                        </p>
                                                    </div>
                                                    {if role.is_super_admin {
                                                        view! {
                                                            <span style="background: #e74c3c; color: white; padding: 8px 15px; border-radius: 20px; font-size: 13px;">
                                                                "Пълни права"
                                                            </span>
                                                        }.into_view()
                                                    } else {
                                                        view! { <span></span> }.into_view()
                                                    }}
                                                </div>

                                                <h3 style="margin: 0 0 15px 0; color: #34495e; font-size: 16px;">"Права"</h3>

                                                {if role.is_super_admin {
                                                    view! {
                                                        <div style="background: #fdf2f2; border: 1px solid #e74c3c; border-radius: 8px; padding: 20px; text-align: center;">
                                                            <p style="margin: 0; color: #e74c3c; font-size: 16px;">
                                                                "Тази роля има достъп до всички функции на системата"
                                                            </p>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <div style="display: flex; flex-direction: column; gap: 15px;">
                                                            {sorted_categories.into_iter().map(|(category, perms)| {
                                                                view! {
                                                                    <div style="border: 1px solid #ecf0f1; border-radius: 8px; overflow: hidden;">
                                                                        <div style="background: #f8f9fa; padding: 12px 15px; font-weight: 500; color: #34495e;">
                                                                            {get_category_label(category)}
                                                                        </div>
                                                                        <div style="padding: 15px; display: flex; flex-wrap: wrap; gap: 10px;">
                                                                            {perms.into_iter().map(|perm| {
                                                                                view! {
                                                                                    <span style="background: #ecf0f1; padding: 6px 12px; border-radius: 15px; font-size: 13px; color: #34495e;">
                                                                                        {get_permission_label(perm)}
                                                                                    </span>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_view()
                                                }}
                                            </div>
                                        }.into_view()
                                    }
                                    None => {
                                        view! {
                                            <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 300px; color: #7f8c8d;">
                                                <span style="font-size: 48px; margin-bottom: 15px;">"🔐"</span>
                                                <p style="margin: 0; font-size: 16px;">"Изберете роля, за да видите правата"</p>
                                            </div>
                                        }.into_view()
                                    }
                                }
                            }}
                        </div>
                    </div>
                }.into_view()
            }}
        </Layout>
    }
}
