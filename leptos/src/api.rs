use crate::models::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::Storage;

pub const API_BASE: &str = "";

pub fn get_token() -> Option<String> {
    use web_sys::window;
    let storage = window()?.local_storage().ok()??;
    storage.get_item("auth_token").ok()?
}

pub fn set_token(token: &str) {
    use web_sys::window;
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("auth_token", token);
        }
    }
}

pub fn clear_token() {
    use web_sys::window;
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth_token");
            let _ = storage.remove_item("selected_company_id");
        }
    }
}

pub fn get_selected_company_id() -> Option<i64> {
    use web_sys::window;
    let storage = window()?.local_storage().ok()??;
    storage.get_item("selected_company_id").ok()?.and_then(|s| s.parse().ok())
}

pub fn set_selected_company_id(id: i64) {
    use web_sys::window;
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("selected_company_id", &id.to_string());
        }
    }
}

/// Get the API URL prefix for the current company
/// Returns "/api/companies/{id}" or defaults to "/api/companies/1" if no company selected
pub fn company_api_url() -> String {
    let id = get_selected_company_id().unwrap_or(1);
    format!("/api/companies/{}", id)
}

pub async fn api_post_auth<T: serde::Serialize>(
    endpoint: &str,
    body: &T,
) -> Result<serde_json::Value, String> {
    let url = format!("{}{}", API_BASE, endpoint);
    
    Request::post(&url)
        .json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn api_get(endpoint: &str) -> Result<serde_json::Value, String> {
    let token = get_token().ok_or("No auth token")?;
    let url = format!("{}{}", API_BASE, endpoint);

    let mut req = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token));

    // Add company ID header if selected
    if let Some(company_id) = get_selected_company_id() {
        req = req.header("X-Company-ID", &company_id.to_string());
    }

    req.send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn api_post<T: serde::Serialize>(
    endpoint: &str,
    body: &T,
) -> Result<serde_json::Value, String> {
    let token = get_token().ok_or("No auth token")?;
    let url = format!("{}{}", API_BASE, endpoint);

    let mut req = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token));

    // Add company ID header if selected
    if let Some(company_id) = get_selected_company_id() {
        req = req.header("X-Company-ID", &company_id.to_string());
    }

    req.json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn api_put<T: serde::Serialize>(
    endpoint: &str,
    body: &T,
) -> Result<serde_json::Value, String> {
    let token = get_token().ok_or("No auth token")?;
    let url = format!("{}{}", API_BASE, endpoint);

    let mut req = Request::put(&url)
        .header("Authorization", &format!("Bearer {}", token));

    // Add company ID header if selected
    if let Some(company_id) = get_selected_company_id() {
        req = req.header("X-Company-ID", &company_id.to_string());
    }

    req.json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn api_delete(endpoint: &str) -> Result<serde_json::Value, String> {
    let token = get_token().ok_or("No auth token")?;
    let url = format!("{}{}", API_BASE, endpoint);

    let mut req = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token));

    // Add company ID header if selected
    if let Some(company_id) = get_selected_company_id() {
        req = req.header("X-Company-ID", &company_id.to_string());
    }

    req.send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Post raw content (XML/text) to an endpoint
pub async fn api_post_raw(
    endpoint: &str,
    body: &str,
) -> Result<serde_json::Value, String> {
    let token = get_token().unwrap_or_default();
    let url = format!("{}{}", API_BASE, endpoint);

    let mut req = Request::post(&url)
        .header("Content-Type", "application/xml");

    if !token.is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }

    req.body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
