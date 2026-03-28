use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::Layout;
use crate::api::{api_get, api_post, api_delete};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupInfo {
    pub filename: String,
    pub size: String,
    pub last_modified: String,
}

#[component]
pub fn SystemSettings() -> impl IntoView {
    let active_tab = create_rw_signal(0); // 0=SMTP, 1=Security, 2=App, 3=Backup
    let loading = create_rw_signal(true);
    let saving = create_rw_signal(false);
    let error_msg = create_rw_signal(String::new());
    let success_msg = create_rw_signal(String::new());

    // SMTP settings
    let smtp_host = create_rw_signal(String::new());
    let smtp_port = create_rw_signal(587i32);
    let smtp_username = create_rw_signal(String::new());
    let smtp_password = create_rw_signal(String::new());
    let smtp_from_email = create_rw_signal(String::new());
    let smtp_from_name = create_rw_signal(String::new());
    let smtp_use_tls = create_rw_signal(true);
    let smtp_enabled = create_rw_signal(false);
    let show_smtp_password = create_rw_signal(false);
    let smtp_test_email = create_rw_signal(String::new());

    // Security settings
    let session_timeout = create_rw_signal(60i32);
    let max_login_attempts = create_rw_signal(5i32);
    let lockout_duration = create_rw_signal(15i32);
    let password_min_length = create_rw_signal(8i32);
    let require_2fa = create_rw_signal(false);

    // App settings
    let app_name = create_rw_signal("Baraba".to_string());
    let app_url = create_rw_signal(String::new());
    let default_language = create_rw_signal("bg".to_string());
    let registration_enabled = create_rw_signal(true);

    // SEO settings
    let site_description = create_rw_signal(String::new());
    let meta_keywords = create_rw_signal(String::new());
    let og_image_url = create_rw_signal(String::new());
    let favicon_url = create_rw_signal(String::new());
    let footer_text = create_rw_signal(String::new());

    // Backup settings
    let s3_endpoint = create_rw_signal(String::new());
    let s3_bucket = create_rw_signal(String::new());
    let s3_access_key = create_rw_signal(String::new());
    let s3_secret_key = create_rw_signal(String::new());
    let s3_region = create_rw_signal("us-east-1".to_string());
    let s3_prefix = create_rw_signal("backups/".to_string());
    let s3_retention_days = create_rw_signal(30i32);
    let s3_has_secret = create_rw_signal(false);
    let show_s3_secret = create_rw_signal(false);
    let backups = create_rw_signal(Vec::<BackupInfo>::new());
    let backup_loading = create_rw_signal(false);
    let restoring = create_rw_signal(false);
    let confirm_restore = create_rw_signal(String::new()); // filename to confirm
    let schedule_enabled = create_rw_signal(false);
    let schedule_time = create_rw_signal("03:00".to_string());
    let schedule_days = create_rw_signal("daily".to_string());

    // Load settings
    create_effect(move |_| {
        spawn_local(async move {
            // Load SMTP settings
            if let Ok(data) = api_get("/api/settings/smtp").await {
                if let Some(settings) = data.get("data") {
                    smtp_host.set(settings.get("smtp_host").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    smtp_port.set(settings.get("smtp_port").and_then(|v| v.as_i64()).unwrap_or(587) as i32);
                    smtp_username.set(settings.get("smtp_username").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    smtp_from_email.set(settings.get("smtp_from_email").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    smtp_from_name.set(settings.get("smtp_from_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    smtp_use_tls.set(settings.get("smtp_use_tls").and_then(|v| v.as_bool()).unwrap_or(true));
                    smtp_enabled.set(settings.get("smtp_enabled").and_then(|v| v.as_bool()).unwrap_or(false));
                }
            }

            // Load security settings
            if let Ok(data) = api_get("/api/system_settings/security").await {
                if let Some(settings) = data.get("data").and_then(|d| d.get("value")) {
                    session_timeout.set(settings.get("session_timeout_minutes").and_then(|v| v.as_i64()).unwrap_or(60) as i32);
                    max_login_attempts.set(settings.get("max_login_attempts").and_then(|v| v.as_i64()).unwrap_or(5) as i32);
                    lockout_duration.set(settings.get("lockout_duration_minutes").and_then(|v| v.as_i64()).unwrap_or(15) as i32);
                    password_min_length.set(settings.get("password_min_length").and_then(|v| v.as_i64()).unwrap_or(8) as i32);
                    require_2fa.set(settings.get("require_2fa").and_then(|v| v.as_bool()).unwrap_or(false));
                }
            }

            // Load app settings
            if let Ok(data) = api_get("/api/system_settings/app").await {
                if let Some(settings) = data.get("data").and_then(|d| d.get("value")) {
                    app_name.set(settings.get("name").and_then(|v| v.as_str()).unwrap_or("Baraba").to_string());
                    app_url.set(settings.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    default_language.set(settings.get("default_language").and_then(|v| v.as_str()).unwrap_or("bg").to_string());
                    registration_enabled.set(settings.get("registration_enabled").and_then(|v| v.as_bool()).unwrap_or(true));
                    // SEO settings
                    site_description.set(settings.get("site_description").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    meta_keywords.set(settings.get("meta_keywords").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    og_image_url.set(settings.get("og_image_url").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    favicon_url.set(settings.get("favicon_url").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    footer_text.set(settings.get("footer_text").and_then(|v| v.as_str()).unwrap_or("").to_string());
                }
            }

            // Load backup settings
            if let Ok(data) = api_get("/api/backup/settings").await {
                if let Some(settings) = data.get("data") {
                    s3_endpoint.set(settings.get("endpoint").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    s3_bucket.set(settings.get("bucket").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    s3_access_key.set(settings.get("access_key").and_then(|v| v.as_str()).unwrap_or("").to_string());
                    s3_region.set(settings.get("region").and_then(|v| v.as_str()).unwrap_or("us-east-1").to_string());
                    s3_prefix.set(settings.get("prefix").and_then(|v| v.as_str()).unwrap_or("backups/").to_string());
                    s3_retention_days.set(settings.get("retention_days").and_then(|v| v.as_i64()).unwrap_or(30) as i32);
                    s3_has_secret.set(settings.get("has_secret_key").and_then(|v| v.as_bool()).unwrap_or(false));
                    schedule_enabled.set(settings.get("schedule_enabled").and_then(|v| v.as_bool()).unwrap_or(false));
                    schedule_time.set(settings.get("schedule_time").and_then(|v| v.as_str()).unwrap_or("03:00").to_string());
                    schedule_days.set(settings.get("schedule_days").and_then(|v| v.as_str()).unwrap_or("daily").to_string());
                }
            }

            loading.set(false);
        });
    });

    // Load backups list
    let load_backups = move || {
        backup_loading.set(true);
        spawn_local(async move {
            if let Ok(data) = api_get("/api/backup/list").await {
                if let Some(items) = data.get("data").and_then(|v| v.as_array()) {
                    let parsed: Vec<BackupInfo> = items.iter().filter_map(|v| {
                        Some(BackupInfo {
                            filename: v.get("filename")?.as_str()?.to_string(),
                            size: v.get("size")?.as_str()?.to_string(),
                            last_modified: v.get("last_modified")?.as_str()?.to_string(),
                        })
                    }).collect();
                    backups.set(parsed);
                }
            }
            backup_loading.set(false);
        });
    };

    // Save SMTP settings
    let save_smtp = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "smtp_host": smtp_host.get(),
                "smtp_port": smtp_port.get(),
                "smtp_username": smtp_username.get(),
                "smtp_password": smtp_password.get(),
                "smtp_from_email": smtp_from_email.get(),
                "smtp_from_name": smtp_from_name.get(),
                "smtp_use_tls": smtp_use_tls.get(),
                "smtp_enabled": smtp_enabled.get(),
            });

            match api_post("/api/settings/smtp", &body).await {
                Ok(_) => {
                    smtp_password.set(String::new());
                    success_msg.set("SMTP настройките са запазени".to_string());
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Test SMTP
    let test_smtp = move |_| {
        let test_to = smtp_test_email.get();
        if test_to.is_empty() {
            error_msg.set("Моля въведете email за тест".to_string());
            return;
        }

        saving.set(true);
        error_msg.set(String::new());
        spawn_local(async move {
            let body = serde_json::json!({
                "test_email": test_to,
                "smtp_host": smtp_host.get(),
                "smtp_port": smtp_port.get(),
                "smtp_username": smtp_username.get(),
                "smtp_password": smtp_password.get(),
                "smtp_from_email": smtp_from_email.get(),
                "smtp_from_name": smtp_from_name.get(),
                "smtp_use_tls": smtp_use_tls.get(),
            });

            match api_post("/api/settings/smtp/test", &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Тестов email изпратен успешно!");
                        success_msg.set(msg.to_string());
                    } else {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Грешка при изпращане");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Save security settings
    let save_security = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "value": serde_json::json!({
                    "session_timeout_minutes": session_timeout.get(),
                    "max_login_attempts": max_login_attempts.get(),
                    "lockout_duration_minutes": lockout_duration.get(),
                    "password_min_length": password_min_length.get(),
                    "require_2fa": require_2fa.get(),
                }).to_string()
            });

            match api_post("/api/system_settings/security", &body).await {
                Ok(_) => success_msg.set("Настройките за сигурност са запазени".to_string()),
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Save app settings
    let save_app = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "value": serde_json::json!({
                    "name": app_name.get(),
                    "url": app_url.get(),
                    "default_language": default_language.get(),
                    "registration_enabled": registration_enabled.get(),
                    "site_description": site_description.get(),
                    "meta_keywords": meta_keywords.get(),
                    "og_image_url": og_image_url.get(),
                    "favicon_url": favicon_url.get(),
                    "footer_text": footer_text.get(),
                }).to_string()
            });

            match api_post("/api/system_settings/app", &body).await {
                Ok(_) => success_msg.set("Настройките на приложението са запазени".to_string()),
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Save backup settings
    let save_backup = move |_| {
        saving.set(true);
        spawn_local(async move {
            let body = serde_json::json!({
                "endpoint": s3_endpoint.get(),
                "bucket": s3_bucket.get(),
                "access_key": s3_access_key.get(),
                "secret_key": s3_secret_key.get(),
                "region": s3_region.get(),
                "prefix": s3_prefix.get(),
                "retention_days": s3_retention_days.get(),
                "schedule_enabled": schedule_enabled.get(),
                "schedule_time": schedule_time.get(),
                "schedule_days": schedule_days.get(),
            });

            match api_post("/api/backup/settings", &body).await {
                Ok(_) => {
                    s3_secret_key.set(String::new());
                    s3_has_secret.set(true);
                    success_msg.set("Настройките за архивиране са запазени".to_string());
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Test S3 connection
    let test_s3 = move |_| {
        saving.set(true);
        spawn_local(async move {
            match api_post("/api/backup/test", &serde_json::json!({})).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Връзката е успешна");
                        success_msg.set(msg.to_string());
                    } else {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Грешка при връзка");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
            saving.set(false);
        });
    };

    // Create backup
    let create_backup = move |_| {
        saving.set(true);
        error_msg.set(String::new());
        success_msg.set("Създаване на backup...".to_string());
        spawn_local(async move {
            match api_post("/api/backup/create", &serde_json::json!({})).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Backup създаден");
                        success_msg.set(msg.to_string());
                        load_backups();
                    } else {
                        let msg = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при backup");
                        error_msg.set(msg.to_string());
                        success_msg.set(String::new());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                    success_msg.set(String::new());
                }
            }
            saving.set(false);
        });
    };

    // Delete backup
    let delete_backup = move |filename: String| {
        spawn_local(async move {
            match api_delete(&format!("/api/backup/{}", filename)).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        success_msg.set("Backup изтрит".to_string());
                        load_backups();
                    } else {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Грешка");
                        error_msg.set(msg.to_string());
                    }
                }
                Err(e) => error_msg.set(format!("Грешка: {}", e)),
            }
        });
    };

    // Restore backup
    let restore_backup = move |filename: String| {
        restoring.set(true);
        confirm_restore.set(String::new());
        error_msg.set(String::new());
        success_msg.set("Възстановяване на базата данни...".to_string());
        spawn_local(async move {
            let body = serde_json::json!({"filename": filename});
            match api_post("/api/backup/restore", &body).await {
                Ok(data) => {
                    if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("Базата е възстановена");
                        success_msg.set(msg.to_string());
                    } else {
                        let msg = data.get("error").and_then(|v| v.as_str()).unwrap_or("Грешка при възстановяване");
                        error_msg.set(msg.to_string());
                        success_msg.set(String::new());
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Грешка: {}", e));
                    success_msg.set(String::new());
                }
            }
            restoring.set(false);
        });
    };

    // Auto-hide messages
    create_effect(move |_| {
        if !success_msg.get().is_empty() && !success_msg.get().contains("...") {
            set_timeout(move || success_msg.set(String::new()), std::time::Duration::from_secs(3));
        }
    });

    view! {
        <Layout>
            <h1>"Системни настройки"</h1>

            // Tabs
            <div style="display: flex; gap: 0; margin-top: 20px; border-bottom: 2px solid #9b59b6;">
                {[(0, "📧 Email"), (1, "🔒 Сигурност"), (2, "⚙️ Приложение"), (3, "💾 Архивиране")].into_iter().map(|(idx, label)| {
                    view! {
                        <button
                            style=move || format!(
                                "padding: 12px 24px; border: none; cursor: pointer; font-size: 14px; font-weight: 500; transition: all 0.2s; {}",
                                if active_tab.get() == idx {
                                    "background: #9b59b6; color: white; border-radius: 4px 4px 0 0;"
                                } else {
                                    "background: #ecf0f1; color: #34495e; border-radius: 4px 4px 0 0;"
                                }
                            )
                            on:click=move |_| {
                                active_tab.set(idx);
                                if idx == 3 {
                                    load_backups();
                                }
                            }
                        >
                            {label}
                        </button>
                    }
                }).collect_view()}
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

            // Tab content
            {move || {
                if loading.get() {
                    return view! { <span></span> }.into_view();
                }

                match active_tab.get() {
                    0 => {
                        // SMTP settings
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 700px;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 25px;">
                                    <div>
                                        <h3 style="margin: 0 0 5px 0; color: #2c3e50;">"SMTP настройки за email"</h3>
                                        <p style="color: #7f8c8d; margin: 0; font-size: 14px;">"Конфигурирайте изходящата поща"</p>
                                    </div>
                                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                        <span style="font-size: 14px; color: #7f8c8d;">"Активен"</span>
                                        <input type="checkbox" style="width: 18px; height: 18px;" checked=move || smtp_enabled.get() on:change=move |ev| smtp_enabled.set(event_target_checked(&ev)) />
                                    </label>
                                </div>

                                <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 20px;">
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"SMTP сървър"</label>
                                        <input type="text" placeholder="smtp.gmail.com" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || smtp_host.get() on:input=move |ev| smtp_host.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Порт"</label>
                                        <input type="text" inputmode="numeric" placeholder="587" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || smtp_port.get().to_string() on:input=move |ev| { smtp_port.set(event_target_value(&ev).parse().unwrap_or(587)); } />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Потребителско име"</label>
                                        <input type="text" placeholder="your@email.com" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || smtp_username.get() on:input=move |ev| smtp_username.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Парола"</label>
                                        <div style="position: relative;">
                                            <input type=move || if show_smtp_password.get() { "text" } else { "password" } placeholder="********" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || smtp_password.get() on:input=move |ev| smtp_password.set(event_target_value(&ev)) />
                                        </div>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Email подател"</label>
                                        <input type="email" placeholder="noreply@yourcompany.com" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || smtp_from_email.get() on:input=move |ev| smtp_from_email.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Име подател"</label>
                                        <input type="text" placeholder="Вашата Фирма" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || smtp_from_name.get() on:input=move |ev| smtp_from_name.set(event_target_value(&ev)) />
                                    </div>
                                </div>

                                <div style="margin-top: 20px;">
                                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                        <input type="checkbox" style="width: 18px; height: 18px;" checked=move || smtp_use_tls.get() on:change=move |ev| smtp_use_tls.set(event_target_checked(&ev)) />
                                        <span>"Използвай TLS криптиране"</span>
                                    </label>
                                </div>

                                <div style="display: flex; gap: 10px; margin-top: 25px;">
                                    <button style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer;" on:click=save_smtp disabled=move || saving.get()>
                                        {move || if saving.get() { "Запазване..." } else { "Запази" }}
                                    </button>
                                </div>

                                // Test email section
                                <div style="margin-top: 25px; padding-top: 25px; border-top: 1px solid #eee;">
                                    <h4 style="margin: 0 0 15px 0; color: #2c3e50;">"Тест на SMTP"</h4>
                                    <div style="display: flex; gap: 10px; align-items: flex-end;">
                                        <div style="flex: 1;">
                                            <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Email за тест"</label>
                                            <input
                                                type="email"
                                                placeholder="test@example.com"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                prop:value=move || smtp_test_email.get()
                                                on:input=move |ev| smtp_test_email.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <button
                                            style="background: #9b59b6; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; white-space: nowrap;"
                                            on:click=test_smtp
                                            disabled=move || saving.get() || smtp_host.get().is_empty() || smtp_test_email.get().is_empty()
                                        >
                                            {move || if saving.get() { "Изпращане..." } else { "Изпрати тест" }}
                                        </button>
                                    </div>
                                    <p style="color: #7f8c8d; font-size: 13px; margin: 10px 0 0 0;">
                                        "Изпраща тестов email с текущите настройки"
                                    </p>
                                </div>
                            </div>
                        }.into_view()
                    }
                    1 => {
                        // Security settings
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 700px;">
                                <h3 style="margin: 0 0 5px 0; color: #2c3e50;">"Настройки за сигурност"</h3>
                                <p style="color: #7f8c8d; margin: 0 0 25px 0; font-size: 14px;">"Параметри за сигурност на системата"</p>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Изтичане на сесия (мин)"</label>
                                        <input type="text" inputmode="numeric" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || session_timeout.get().to_string() on:input=move |ev| { session_timeout.set(event_target_value(&ev).parse().unwrap_or(60)); } />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Макс. опити за вход"</label>
                                        <input type="text" inputmode="numeric" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || max_login_attempts.get().to_string() on:input=move |ev| { max_login_attempts.set(event_target_value(&ev).parse().unwrap_or(5)); } />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Заключване (мин)"</label>
                                        <input type="text" inputmode="numeric" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || lockout_duration.get().to_string() on:input=move |ev| { lockout_duration.set(event_target_value(&ev).parse().unwrap_or(15)); } />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Мин. дължина парола"</label>
                                        <input type="text" inputmode="numeric" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || password_min_length.get().to_string() on:input=move |ev| { password_min_length.set(event_target_value(&ev).parse().unwrap_or(8)); } />
                                    </div>
                                </div>

                                <div style="margin-top: 20px; padding: 15px; background: #f8f9fa; border-radius: 8px;">
                                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                        <input type="checkbox" style="width: 18px; height: 18px;" checked=move || require_2fa.get() on:change=move |ev| require_2fa.set(event_target_checked(&ev)) />
                                        <span style="font-weight: 500;">"Изискване на 2FA"</span>
                                    </label>
                                </div>

                                <button style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; margin-top: 25px;" on:click=save_security disabled=move || saving.get()>
                                    {move || if saving.get() { "Запазване..." } else { "Запази" }}
                                </button>
                            </div>
                        }.into_view()
                    }
                    2 => {
                        // App settings
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 800px;">
                                <h3 style="margin: 0 0 5px 0; color: #2c3e50;">"Настройки на приложението"</h3>
                                <p style="color: #7f8c8d; margin: 0 0 25px 0; font-size: 14px;">"Общи настройки и SEO"</p>

                                // Basic settings
                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Име на сайта"</label>
                                        <input type="text" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || app_name.get() on:input=move |ev| app_name.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Език по подразбиране"</label>
                                        <select style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;" on:change=move |ev| default_language.set(event_target_value(&ev))>
                                            <option value="bg" selected=move || default_language.get() == "bg">"Български"</option>
                                            <option value="en" selected=move || default_language.get() == "en">"English"</option>
                                        </select>
                                    </div>
                                </div>

                                // URL - Important for email links
                                <div style="margin-top: 20px; padding: 15px; background: #e8f4fd; border: 1px solid #3498db; border-radius: 8px;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: 500; color: #2980b9;">"URL на сайта (важно!)"</label>
                                    <input type="url" placeholder="https://app.example.com" style="width: 100%; padding: 10px; border: 1px solid #3498db; border-radius: 4px; box-sizing: border-box;" prop:value=move || app_url.get() on:input=move |ev| app_url.set(event_target_value(&ev)) />
                                    <small style="color: #7f8c8d; display: block; margin-top: 5px;">
                                        "Използва се за линковете в email-ите (потвърждение, възстановяване на парола и др.)"
                                    </small>
                                </div>

                                // SEO Section
                                <div style="margin-top: 25px; padding-top: 20px; border-top: 1px solid #eee;">
                                    <h4 style="margin: 0 0 15px 0; color: #2c3e50;">"SEO настройки"</h4>

                                    <div style="margin-bottom: 15px;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Описание на сайта (meta description)"</label>
                                        <textarea
                                            rows="3"
                                            placeholder="Кратко описание на вашия сайт за търсачките..."
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; resize: vertical;"
                                            prop:value=move || site_description.get()
                                            on:input=move |ev| site_description.set(event_target_value(&ev))
                                        ></textarea>
                                        <small style="color: #7f8c8d;">"Препоръчително: 150-160 символа"</small>
                                    </div>

                                    <div style="margin-bottom: 15px;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Ключови думи (meta keywords)"</label>
                                        <input
                                            type="text"
                                            placeholder="счетоводство, фактури, ДДС, България"
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || meta_keywords.get()
                                            on:input=move |ev| meta_keywords.set(event_target_value(&ev))
                                        />
                                        <small style="color: #7f8c8d;">"Разделени със запетая"</small>
                                    </div>

                                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Open Graph изображение"</label>
                                            <input
                                                type="url"
                                                placeholder="https://example.com/og-image.jpg"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                prop:value=move || og_image_url.get()
                                                on:input=move |ev| og_image_url.set(event_target_value(&ev))
                                            />
                                            <small style="color: #7f8c8d;">"Изображение за социални мрежи (1200x630px)"</small>
                                        </div>
                                        <div>
                                            <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Favicon URL"</label>
                                            <input
                                                type="url"
                                                placeholder="https://example.com/favicon.ico"
                                                style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                                prop:value=move || favicon_url.get()
                                                on:input=move |ev| favicon_url.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>

                                    <div style="margin-top: 15px;">
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Текст във футъра"</label>
                                        <input
                                            type="text"
                                            placeholder="© 2024 Вашата Фирма. Всички права запазени."
                                            style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
                                            prop:value=move || footer_text.get()
                                            on:input=move |ev| footer_text.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>

                                // Registration toggle
                                <div style="margin-top: 20px; padding: 15px; background: #f8f9fa; border-radius: 8px;">
                                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                        <input type="checkbox" style="width: 18px; height: 18px;" checked=move || registration_enabled.get() on:change=move |ev| registration_enabled.set(event_target_checked(&ev)) />
                                        <span style="font-weight: 500;">"Разреши регистрация на нови потребители"</span>
                                    </label>
                                </div>

                                <button style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer; margin-top: 25px;" on:click=save_app disabled=move || saving.get()>
                                    {move || if saving.get() { "Запазване..." } else { "Запази настройки" }}
                                </button>
                            </div>
                        }.into_view()
                    }
                    3 => {
                        // Backup settings
                        view! {
                            <div style="background: white; border-radius: 8px; padding: 25px; margin-top: 20px; max-width: 900px;">
                                <h3 style="margin: 0 0 5px 0; color: #2c3e50;">"Архивиране на базата данни"</h3>
                                <p style="color: #7f8c8d; margin: 0 0 25px 0; font-size: 14px;">
                                    "Конфигурирайте S3-съвместимо хранилище за автоматични архиви (AWS S3, MinIO, DigitalOcean Spaces, Backblaze B2)"
                                </p>

                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"S3 Endpoint (за non-AWS)"</label>
                                        <input type="text" placeholder="https://s3.eu-central-1.amazonaws.com" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || s3_endpoint.get() on:input=move |ev| s3_endpoint.set(event_target_value(&ev)) />
                                        <small style="color: #7f8c8d;">"Оставете празно за AWS S3"</small>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Bucket"</label>
                                        <input type="text" placeholder="my-backups" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || s3_bucket.get() on:input=move |ev| s3_bucket.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Access Key"</label>
                                        <input type="text" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || s3_access_key.get() on:input=move |ev| s3_access_key.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">
                                            "Secret Key"
                                            {move || if s3_has_secret.get() {
                                                view! { <span style="color: #27ae60; margin-left: 10px;">"✓ запазен"</span> }.into_view()
                                            } else {
                                                view! { <span></span> }.into_view()
                                            }}
                                        </label>
                                        <div style="position: relative;">
                                            <input type=move || if show_s3_secret.get() { "text" } else { "password" } placeholder="********" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || s3_secret_key.get() on:input=move |ev| s3_secret_key.set(event_target_value(&ev)) />
                                        </div>
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Region"</label>
                                        <input type="text" placeholder="us-east-1" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || s3_region.get() on:input=move |ev| s3_region.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Prefix (път в bucket)"</label>
                                        <input type="text" placeholder="backups/" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || s3_prefix.get() on:input=move |ev| s3_prefix.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Задържане (дни)"</label>
                                        <input type="text" inputmode="numeric" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || s3_retention_days.get().to_string() on:input=move |ev| { s3_retention_days.set(event_target_value(&ev).parse().unwrap_or(30)); } />
                                    </div>
                                </div>

                                // Schedule settings
                                <div style="margin-top: 25px; padding: 20px; background: #f8f9fa; border-radius: 8px;">
                                    <div style="display: flex; align-items: center; gap: 15px; margin-bottom: 15px;">
                                        <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                                            <input type="checkbox" style="width: 18px; height: 18px;" checked=move || schedule_enabled.get() on:change=move |ev| schedule_enabled.set(event_target_checked(&ev)) />
                                            <span style="font-weight: 500;">"Автоматично архивиране"</span>
                                        </label>
                                    </div>
                                    {move || if schedule_enabled.get() {
                                        view! {
                                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Час (UTC)"</label>
                                                    <input type="time" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;" prop:value=move || schedule_time.get() on:input=move |ev| schedule_time.set(event_target_value(&ev)) />
                                                </div>
                                                <div>
                                                    <label style="display: block; margin-bottom: 5px; font-weight: 500;">"Честота"</label>
                                                    <select style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px;" on:change=move |ev| schedule_days.set(event_target_value(&ev))>
                                                        <option value="daily" selected=move || schedule_days.get() == "daily">"Всеки ден"</option>
                                                        <option value="mon,tue,wed,thu,fri" selected=move || schedule_days.get() == "mon,tue,wed,thu,fri">"Делнични дни"</option>
                                                        <option value="mon,wed,fri" selected=move || schedule_days.get() == "mon,wed,fri">"Пон, Сря, Пет"</option>
                                                        <option value="mon" selected=move || schedule_days.get() == "mon">"Всеки понеделник"</option>
                                                        <option value="sun" selected=move || schedule_days.get() == "sun">"Всяка неделя"</option>
                                                    </select>
                                                </div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                </div>

                                <div style="display: flex; gap: 10px; margin-top: 25px;">
                                    <button style="background: #27ae60; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer;" on:click=save_backup disabled=move || saving.get()>
                                        {move || if saving.get() { "Запазване..." } else { "Запази настройки" }}
                                    </button>
                                    <button style="background: #9b59b6; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer;" on:click=test_s3 disabled=move || saving.get() || s3_bucket.get().is_empty()>
                                        "Тест връзка"
                                    </button>
                                    <button style="background: #3498db; color: white; border: none; padding: 12px 25px; border-radius: 4px; cursor: pointer;" on:click=create_backup disabled=move || saving.get() || s3_bucket.get().is_empty()>
                                        "Създай backup сега"
                                    </button>
                                </div>

                                // Backups list
                                <div style="margin-top: 30px; border-top: 1px solid #eee; padding-top: 20px;">
                                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                                        <h4 style="margin: 0; color: #2c3e50;">"Налични архиви"</h4>
                                        <button style="background: #ecf0f1; border: none; padding: 8px 15px; border-radius: 4px; cursor: pointer;" on:click=move |_| load_backups() disabled=move || backup_loading.get()>
                                            {move || if backup_loading.get() { "Зареждане..." } else { "Опресни" }}
                                        </button>
                                    </div>

                                    {move || {
                                        let items = backups.get();
                                        if items.is_empty() {
                                            view! {
                                                <p style="color: #7f8c8d; text-align: center; padding: 20px;">"Няма налични архиви"</p>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <table style="width: 100%; border-collapse: collapse;">
                                                    <thead>
                                                        <tr style="background: #f8f9fa;">
                                                            <th style="padding: 12px; text-align: left; border-bottom: 1px solid #ddd;">"Файл"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 1px solid #ddd;">"Размер"</th>
                                                            <th style="padding: 12px; text-align: left; border-bottom: 1px solid #ddd;">"Дата"</th>
                                                            <th style="padding: 12px; text-align: center; border-bottom: 1px solid #ddd; width: 80px;">"Действия"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {items.into_iter().map(|b| {
                                                            let filename = b.filename.clone();
                                                            let filename_for_delete = b.filename.clone();
                                                            let filename_for_restore = b.filename.clone();
                                                            view! {
                                                                <tr style="border-bottom: 1px solid #eee;">
                                                                    <td style="padding: 12px; font-family: monospace; font-size: 13px;">{filename}</td>
                                                                    <td style="padding: 12px;">{b.size}</td>
                                                                    <td style="padding: 12px;">{b.last_modified}</td>
                                                                    <td style="padding: 12px; text-align: center;">
                                                                        <button style="background: none; border: none; cursor: pointer; color: #3498db; font-size: 16px;" title="Възстанови" disabled=move || restoring.get() on:click={let f = filename_for_restore.clone(); move |_| confirm_restore.set(f.clone())}>
                                                                            "↩"
                                                                        </button>
                                                                        <button style="background: none; border: none; cursor: pointer; color: #e74c3c;" title="Изтрий" on:click=move |_| delete_backup(filename_for_delete.clone())>
                                                                            "🗑"
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            }.into_view()
                                        }
                                    }}
                                </div>

                                // Restore confirmation dialog
                                {move || {
                                    let fname = confirm_restore.get();
                                    if !fname.is_empty() {
                                        let fname_for_btn = fname.clone();
                                        view! {
                                            <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                                                <div style="background: white; border-radius: 8px; padding: 30px; max-width: 500px; width: 90%;">
                                                    <h3 style="margin: 0 0 15px 0; color: #e74c3c;">"Възстановяване на базата данни"</h3>
                                                    <p style="color: #2c3e50; margin-bottom: 10px;">
                                                        "Сигурни ли сте, че искате да възстановите базата от:"
                                                    </p>
                                                    <p style="font-family: monospace; background: #f8f9fa; padding: 10px; border-radius: 4px; word-break: break-all;">
                                                        {fname}
                                                    </p>
                                                    <p style="color: #e74c3c; font-weight: 500;">
                                                        "Текущите данни ще бъдат заменени!"
                                                    </p>
                                                    <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px;">
                                                        <button style="background: #ecf0f1; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;" on:click=move |_| confirm_restore.set(String::new())>
                                                            "Отказ"
                                                        </button>
                                                        <button style="background: #e74c3c; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;" on:click=move |_| restore_backup(fname_for_btn.clone())>
                                                            "Възстанови"
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }
                                }}
                            </div>
                        }.into_view()
                    }
                    _ => view! { <span></span> }.into_view()
                }
            }}
        </Layout>
    }
}
