use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_delete};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub is_super_admin: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub pages: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Company {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserCompanyRole {
    pub company_id: i64,
    pub company_name: String,
    pub role: String,
}

fn get_role_color(role: &str) -> &'static str {
    match role {
        "superadmin" | "SUPERADMIN" => "#e74c3c",
        "admin" | "ADMIN" => "#f39c12",
        "accountant" | "ACCOUNTANT" => "#3498db",
        "observer" | "OBSERVER" => "#95a5a6",
        _ => "#7f8c8d",
    }
}

fn get_role_label(role: &str) -> &'static str {
    match role {
        "superadmin" | "SUPERADMIN" => "Супер Админ",
        "admin" | "ADMIN" => "Администратор",
        "accountant" | "ACCOUNTANT" => "Счетоводител",
        "observer" | "OBSERVER" => "Наблюдател",
        _ => "Друг",
    }
}

#[component]
pub fn Users() -> impl IntoView {
    let users = create_rw_signal(Vec::<User>::new());
    let companies = create_rw_signal(Vec::<Company>::new());
    let loading = create_rw_signal(true);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // Search and filter state
    let search_query = create_rw_signal(String::new());
    let role_filter = create_rw_signal(String::new());
    let status_filter = create_rw_signal(String::new());
    let company_filter = create_rw_signal(0i64);
    let pagination = create_rw_signal(PaginationMeta::default());
    let current_page = create_rw_signal(1i64);

    // Modal states
    let show_company_modal = create_rw_signal(false);
    let selected_user = create_rw_signal(Option::<User>::None);
    let user_company_roles = create_rw_signal(Vec::<UserCompanyRole>::new());
    let selected_company = create_rw_signal(0i64);
    let selected_role = create_rw_signal(String::new());

    // Invite user modal
    let show_invite_modal = create_rw_signal(false);
    let invite_email = create_rw_signal(String::new());
    let invite_role = create_rw_signal("accountant".to_string());

    // Load users with filters
    let load_users = move || {
        loading.set(true);
        spawn_local(async move {
            let mut url = format!("/api/users?page={}", current_page.get());

            let q = search_query.get();
            if !q.is_empty() {
                url.push_str(&format!("&q={}", q));
            }

            let role = role_filter.get();
            if !role.is_empty() {
                url.push_str(&format!("&role={}", role));
            }

            let status = status_filter.get();
            if !status.is_empty() {
                url.push_str(&format!("&is_active={}", status));
            }

            let company_id = company_filter.get();
            if company_id > 0 {
                url.push_str(&format!("&company_id={}", company_id));
            }

            match api_get(&url).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<User> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        users.set(items);
                    }
                    if let Some(meta) = data.get("meta") {
                        if let Ok(m) = serde_json::from_value::<PaginationMeta>(meta.clone()) {
                            pagination.set(m);
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка при зареждане: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Initial load
    create_effect(move |_| {
        // Load companies first (for filter dropdown)
        spawn_local(async move {
            match api_get("/api/companies").await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<Company> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        companies.set(items);
                    }
                }
                _ => {}
            }
        });
        load_users();
    });

    // Reload when filters change
    let do_search = move || {
        current_page.set(1);
        load_users();
    };

    // Toggle user active status
    let toggle_active = move |user_id: i64, current_status: bool| {
        spawn_local(async move {
            let body = serde_json::json!({ "is_active": !current_status });
            match api_post(&format!("/api/users/{}/toggle_active", user_id), &body).await {
                Ok(_) => {
                    users.update(|list| {
                        if let Some(user) = list.iter_mut().find(|u| u.id == user_id) {
                            user.is_active = !user.is_active;
                        }
                    });
                    success_msg.set("Статусът е обновен".to_string());
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Change user role
    let change_role = move |user_id: i64, new_role: String| {
        spawn_local(async move {
            let body = serde_json::json!({ "role": new_role.clone() });
            match api_post(&format!("/api/users/{}/role", user_id), &body).await {
                Ok(_) => {
                    users.update(|list| {
                        if let Some(user) = list.iter_mut().find(|u| u.id == user_id) {
                            user.role = new_role;
                        }
                    });
                    success_msg.set("Ролята е обновена".to_string());
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Open company roles modal
    let open_company_modal = move |user: User| {
        selected_user.set(Some(user.clone()));
        selected_company.set(0);
        selected_role.set(String::new());
        user_company_roles.set(Vec::new());

        // Load user's company roles
        let user_id = user.id;
        spawn_local(async move {
            match api_get(&format!("/api/users/{}/company_roles", user_id)).await {
                Ok(data) => {
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let roles: Vec<UserCompanyRole> = arr.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
                        user_company_roles.set(roles);
                    }
                }
                _ => {}
            }
        });

        show_company_modal.set(true);
    };

    // Assign company role
    let assign_company_role = move |_| {
        let user = selected_user.get();
        let company_id = selected_company.get();
        let role = selected_role.get();

        if user.is_none() || company_id == 0 || role.is_empty() {
            return;
        }

        let user_id = user.unwrap().id;
        let company_name = companies.get()
            .iter()
            .find(|c| c.id == company_id)
            .map(|c| c.name.clone())
            .unwrap_or_default();

        spawn_local(async move {
            let body = serde_json::json!({
                "company_id": company_id,
                "role": role.clone()
            });

            match api_post(&format!("/api/users/{}/company_roles", user_id), &body).await {
                Ok(_) => {
                    user_company_roles.update(|list| {
                        list.push(UserCompanyRole {
                            company_id,
                            company_name,
                            role: role.clone(),
                        });
                    });
                    selected_company.set(0);
                    selected_role.set(String::new());
                    success_msg.set("Ролята е присвоена".to_string());
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Remove company role
    let remove_company_role = move |company_id: i64| {
        let user = selected_user.get();
        if user.is_none() {
            return;
        }
        let user_id = user.unwrap().id;

        spawn_local(async move {
            match api_delete(&format!("/api/users/{}/company_roles/{}", user_id, company_id)).await {
                Ok(_) => {
                    user_company_roles.update(|list| {
                        list.retain(|r| r.company_id != company_id);
                    });
                    success_msg.set("Ролята е премахната".to_string());
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Delete user
    let delete_user = move |user_id: i64, user_email: String| {
        if !web_sys::window()
            .unwrap()
            .confirm_with_message(&format!("Сигурни ли сте, че искате да изтриете {}?", user_email))
            .unwrap_or(false)
        {
            return;
        }

        spawn_local(async move {
            match api_delete(&format!("/api/users/{}", user_id)).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        users.update(|list| {
                            list.retain(|u| u.id != user_id);
                        });
                        success_msg.set("Потребителят е изтрит".to_string());
                    } else {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Грешка");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Invite user
    let invite_user = move |_| {
        let email = invite_email.get();
        let role = invite_role.get();

        if email.is_empty() {
            error_msg.set("Моля въведете email".to_string());
            return;
        }

        spawn_local(async move {
            let body = serde_json::json!({
                "email": email,
                "role": role
            });

            match api_post("/api/users/invite", &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        show_invite_modal.set(false);
                        invite_email.set(String::new());
                        success_msg.set("Поканата е изпратена".to_string());
                    } else {
                        let msg = data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Грешка при изпращане");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Auto-hide messages
    create_effect(move |_| {
        if !success_msg.get().is_empty() {
            set_timeout(move || success_msg.set(String::new()), std::time::Duration::from_secs(3));
        }
    });

    view! {
        <Layout>
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 15px;">
                <h1>"Потребители"</h1>
                <button
                    style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                    on:click=move |_| show_invite_modal.set(true)
                >
                    "+ Покани потребител"
                </button>
            </div>

            // Search and filters
            <div style="background: white; border-radius: 8px; padding: 20px; margin-top: 20px;">
                <div style="display: flex; gap: 15px; flex-wrap: wrap; align-items: flex-end;">
                    // Search input
                    <div style="flex: 1; min-width: 200px;">
                        <label style="display: block; margin-bottom: 5px; font-size: 13px; color: #7f8c8d;">"Търсене"</label>
                        <input
                            type="text"
                            placeholder="Email, име..."
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                            prop:value=move || search_query.get()
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                            on:keypress=move |ev| {
                                if ev.key() == "Enter" {
                                    do_search();
                                }
                            }
                        />
                    </div>

                    // Role filter
                    <div style="min-width: 150px;">
                        <label style="display: block; margin-bottom: 5px; font-size: 13px; color: #7f8c8d;">"Роля"</label>
                        <select
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; background: white;"
                            on:change=move |ev| {
                                role_filter.set(event_target_value(&ev));
                                do_search();
                            }
                        >
                            <option value="">"Всички"</option>
                            <option value="super_admin">"Супер Админ"</option>
                            <option value="admin">"Администратор"</option>
                            <option value="accountant">"Счетоводител"</option>
                            <option value="viewer">"Наблюдател"</option>
                        </select>
                    </div>

                    // Status filter
                    <div style="min-width: 130px;">
                        <label style="display: block; margin-bottom: 5px; font-size: 13px; color: #7f8c8d;">"Статус"</label>
                        <select
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; background: white;"
                            on:change=move |ev| {
                                status_filter.set(event_target_value(&ev));
                                do_search();
                            }
                        >
                            <option value="">"Всички"</option>
                            <option value="true">"Активни"</option>
                            <option value="false">"Неактивни"</option>
                        </select>
                    </div>

                    // Company filter
                    <div style="min-width: 180px;">
                        <label style="display: block; margin-bottom: 5px; font-size: 13px; color: #7f8c8d;">"Фирма"</label>
                        <select
                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; background: white;"
                            on:change=move |ev| {
                                company_filter.set(event_target_value(&ev).parse().unwrap_or(0));
                                do_search();
                            }
                        >
                            <option value="0">"Всички"</option>
                            {move || companies.get().into_iter().map(|c| {
                                view! { <option value=c.id.to_string()>{c.name}</option> }
                            }).collect_view()}
                        </select>
                    </div>

                    // Search button
                    <button
                        style="background: #3498db; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=move |_| do_search()
                    >
                        "Търси"
                    </button>

                    // Clear filters button
                    <button
                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                        on:click=move |_| {
                            search_query.set(String::new());
                            role_filter.set(String::new());
                            status_filter.set(String::new());
                            company_filter.set(0);
                            current_page.set(1);
                            load_users();
                        }
                    >
                        "Изчисти"
                    </button>
                </div>

                // Results count
                {move || {
                    let meta = pagination.get();
                    if meta.total > 0 {
                        view! {
                            <p style="margin: 15px 0 0 0; color: #7f8c8d; font-size: 13px;">
                                {format!("Намерени: {} потребители", meta.total)}
                            </p>
                        }.into_view()
                    } else {
                        view! { <span></span> }.into_view()
                    }
                }}
            </div>

            // Messages
            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div style="background: #e74c3c; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {err}
                            <button style="float: right; background: transparent; border: none; color: white; cursor: pointer;" on:click=move |_| error_msg.set(String::new())>"×"</button>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
            {move || {
                let msg = success_msg.get();
                if !msg.is_empty() {
                    view! {
                        <div style="background: #27ae60; color: white; padding: 15px; border-radius: 8px; margin-top: 20px;">
                            {msg}
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

            // Users table
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                view! {
                    <div style="background: white; border-radius: 8px; margin-top: 20px; overflow: hidden;">
                        <table style="width: 100%; border-collapse: collapse;">
                            <thead style="background: #34495e; color: white;">
                                <tr>
                                    <th style="padding: 12px; text-align: left;">"Email"</th>
                                    <th style="padding: 12px; text-align: left;">"Име"</th>
                                    <th style="padding: 12px; text-align: left; width: 150px;">"Роля"</th>
                                    <th style="padding: 12px; text-align: center; width: 100px;">"Статус"</th>
                                    <th style="padding: 12px; text-align: center; width: 150px;">"Действия"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    let user_list = users.get();
                                    if user_list.is_empty() {
                                        view! {
                                            <tr>
                                                <td colspan="5" style="padding: 40px; text-align: center; color: #7f8c8d;">
                                                    "Няма регистрирани потребители"
                                                </td>
                                            </tr>
                                        }.into_view()
                                    } else {
                                        user_list.into_iter().map(|user| {
                                            let user_id = user.id;
                                            let user_email = user.email.clone();
                                            let user_email_for_delete = user.email.clone();
                                            let user_for_modal = user.clone();
                                            let is_active = user.is_active;
                                            let role = user.role.clone();
                                            let role_for_select = user.role.clone();
                                            let full_name = format!("{} {}",
                                                user.first_name.clone().unwrap_or_default(),
                                                user.last_name.clone().unwrap_or_default()
                                            ).trim().to_string();

                                            view! {
                                                <tr style=format!(
                                                    "border-bottom: 1px solid #ecf0f1; {}",
                                                    if user.is_active { "" } else { "opacity: 0.5;" }
                                                )>
                                                    <td style="padding: 12px;">{user_email}</td>
                                                    <td style="padding: 12px;">
                                                        {if full_name.is_empty() { "-".to_string() } else { full_name }}
                                                    </td>
                                                    <td style="padding: 12px;">
                                                        <select
                                                            style=format!(
                                                                "padding: 6px 10px; border-radius: 4px; border: none; color: white; cursor: pointer; background: {};",
                                                                get_role_color(&role)
                                                            )
                                                            on:change=move |ev| {
                                                                let new_role = event_target_value(&ev);
                                                                change_role(user_id, new_role);
                                                            }
                                                        >
                                                            <option value="superadmin" selected=role_for_select == "superadmin">"Супер Админ"</option>
                                                            <option value="admin" selected=role_for_select == "admin">"Администратор"</option>
                                                            <option value="accountant" selected=role_for_select == "accountant">"Счетоводител"</option>
                                                            <option value="observer" selected=role_for_select == "observer">"Наблюдател"</option>
                                                        </select>
                                                    </td>
                                                    <td style="padding: 12px; text-align: center;">
                                                        <span style=format!(
                                                            "display: inline-block; padding: 4px 10px; border-radius: 12px; font-size: 12px; color: white; background: {};",
                                                            if user.is_active { "#27ae60" } else { "#95a5a6" }
                                                        )>
                                                            {if user.is_active { "Активен" } else { "Неактивен" }}
                                                        </span>
                                                    </td>
                                                    <td style="padding: 12px; text-align: center;">
                                                        <button
                                                            style=format!(
                                                                "border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer; margin-right: 5px; color: white; background: {};",
                                                                if is_active { "#e74c3c" } else { "#27ae60" }
                                                            )
                                                            title=if is_active { "Деактивирай" } else { "Активирай" }
                                                            on:click=move |_| toggle_active(user_id, is_active)
                                                        >
                                                            {if is_active { "🚫" } else { "✓" }}
                                                        </button>
                                                        <button
                                                            style="background: #3498db; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer; margin-right: 5px;"
                                                            title="Управление на фирми"
                                                            on:click=move |_| open_company_modal(user_for_modal.clone())
                                                        >
                                                            "🏢"
                                                        </button>
                                                        <button
                                                            style="background: #c0392b; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                            title="Изтрий потребител"
                                                            on:click=move |_| delete_user(user_id, user_email_for_delete.clone())
                                                        >
                                                            "🗑"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()
                                    }
                                }}
                            </tbody>
                        </table>
                    </div>

                    // Pagination
                    {move || {
                        let meta = pagination.get();
                        if meta.pages > 1 {
                            let current = current_page.get();
                            view! {
                                <div style="display: flex; justify-content: center; align-items: center; gap: 10px; padding: 20px;">
                                    <button
                                        style="background: #3498db; color: white; border: none; padding: 8px 15px; border-radius: 4px; cursor: pointer; disabled:opacity: 0.5;"
                                        disabled=move || current_page.get() <= 1
                                        on:click=move |_| {
                                            if current_page.get() > 1 {
                                                current_page.update(|p| *p -= 1);
                                                load_users();
                                            }
                                        }
                                    >
                                        "« Предишна"
                                    </button>
                                    <span style="color: #7f8c8d;">
                                        {format!("Страница {} от {}", current, meta.pages)}
                                    </span>
                                    <button
                                        style="background: #3498db; color: white; border: none; padding: 8px 15px; border-radius: 4px; cursor: pointer;"
                                        disabled=move || current_page.get() >= pagination.get().pages
                                        on:click=move |_| {
                                            if current_page.get() < pagination.get().pages {
                                                current_page.update(|p| *p += 1);
                                                load_users();
                                            }
                                        }
                                    >
                                        "Следваща »"
                                    </button>
                                </div>
                            }.into_view()
                        } else {
                            view! { <span></span> }.into_view()
                        }
                    }}
                }.into_view()
            }}

            // Company Roles Modal
            {move || {
                if show_company_modal.get() {
                    let user = selected_user.get();
                    let user_name = user.as_ref().map(|u| u.email.clone()).unwrap_or_default();

                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 500px; max-height: 80vh; overflow-y: auto;">
                                <h2 style="margin: 0 0 5px 0;">"Фирми на потребителя"</h2>
                                <p style="color: #7f8c8d; margin: 0 0 20px 0;">{user_name}</p>

                                // Add new company role
                                <div style="background: #ecf0f1; padding: 15px; border-radius: 8px; margin-bottom: 20px;">
                                    <h4 style="margin: 0 0 15px 0; color: #34495e;">"Добави достъп до фирма"</h4>
                                    <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                        <select
                                            style="flex: 1; min-width: 150px; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                selected_company.set(val.parse().unwrap_or(0));
                                            }
                                        >
                                            <option value="0">"-- Избери фирма --"</option>
                                            {move || companies.get().into_iter().map(|c| {
                                                let id = c.id;
                                                view! {
                                                    <option value=id.to_string()>{c.name}</option>
                                                }
                                            }).collect_view()}
                                        </select>
                                        <select
                                            style="min-width: 130px; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                            on:change=move |ev| selected_role.set(event_target_value(&ev))
                                        >
                                            <option value="">"-- Роля --"</option>
                                            <option value="admin">"Администратор"</option>
                                            <option value="accountant">"Счетоводител"</option>
                                            <option value="observer">"Наблюдател"</option>
                                        </select>
                                        <button
                                            style="background: #3498db; color: white; border: none; padding: 10px 15px; border-radius: 4px; cursor: pointer;"
                                            on:click=assign_company_role
                                            disabled=move || selected_company.get() == 0 || selected_role.get().is_empty()
                                        >
                                            "Добави"
                                        </button>
                                    </div>
                                </div>

                                // Current company roles
                                <h4 style="margin: 0 0 10px 0; color: #34495e;">"Текущи роли"</h4>
                                {move || {
                                    let roles = user_company_roles.get();
                                    if roles.is_empty() {
                                        view! {
                                            <p style="color: #7f8c8d; font-style: italic;">"Няма присвоени фирми"</p>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div style="border: 1px solid #ecf0f1; border-radius: 8px; overflow: hidden;">
                                                {roles.into_iter().map(|role| {
                                                    let company_id = role.company_id;
                                                    view! {
                                                        <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 15px; border-bottom: 1px solid #ecf0f1;">
                                                            <div>
                                                                <div style="font-weight: 500;">{role.company_name}</div>
                                                                <span style=format!(
                                                                    "display: inline-block; padding: 2px 8px; border-radius: 10px; font-size: 11px; color: white; background: {};",
                                                                    get_role_color(&role.role)
                                                                )>
                                                                    {get_role_label(&role.role)}
                                                                </span>
                                                            </div>
                                                            <button
                                                                style="background: #e74c3c; color: white; border: none; padding: 6px 10px; border-radius: 4px; cursor: pointer;"
                                                                on:click=move |_| remove_company_role(company_id)
                                                            >
                                                                "🗑"
                                                            </button>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_view()
                                    }
                                }}

                                <div style="text-align: right; margin-top: 20px;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_company_modal.set(false)
                                    >
                                        "Затвори"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}

            // Invite Modal
            {move || {
                if show_invite_modal.get() {
                    view! {
                        <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                            <div style="background: white; border-radius: 8px; padding: 30px; width: 400px;">
                                <h2 style="margin: 0 0 20px 0;">"Покани потребител"</h2>

                                <div style="margin-bottom: 15px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Email *"</label>
                                    <input
                                        type="email"
                                        placeholder="user@example.com"
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                        on:input=move |ev| invite_email.set(event_target_value(&ev))
                                        prop:value=move || invite_email.get()
                                    />
                                </div>

                                <div style="margin-bottom: 20px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Роля"</label>
                                    <select
                                        style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                                        on:change=move |ev| invite_role.set(event_target_value(&ev))
                                    >
                                        <option value="accountant">"Счетоводител"</option>
                                        <option value="admin">"Администратор"</option>
                                        <option value="observer">"Наблюдател"</option>
                                    </select>
                                </div>

                                <div style="display: flex; justify-content: flex-end; gap: 10px;">
                                    <button
                                        style="background: #95a5a6; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=move |_| show_invite_modal.set(false)
                                    >
                                        "Отказ"
                                    </button>
                                    <button
                                        style="background: #27ae60; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
                                        on:click=invite_user
                                    >
                                        "Изпрати покана"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
        </Layout>
    }
}
