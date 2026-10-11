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

/// Get the API URL prefix for the current (selected) company.
/// Uses localStorage `selected_company_id`. Never hardcodes a company — if none is
/// selected yet returns `/api/companies/0` so the backend rejects instead of writing
/// into someone else's firm (critical for multi-company / счетоводни кантори).
pub fn company_api_url() -> String {
    let id = get_selected_company_id().unwrap_or(0);
    format!("/api/companies/{}", id)
}

/// Active company id from localStorage, if any.
pub fn require_company_id() -> Result<i64, String> {
    get_selected_company_id().ok_or_else(|| "Няма избрана фирма. Натиснете „Смени фирма“ и я направете активна.".to_string())
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

pub async fn api_get_public(endpoint: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}{}", API_BASE, endpoint);

    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Extract a human-readable error from common API error response shapes.
fn extract_api_error(body: &serde_json::Value) -> String {
    // Prefer top-level message
    if let Some(msg) = body.get("message").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        return msg.to_string();
    }
    // Single error string
    if let Some(msg) = body.get("error").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        return msg.to_string();
    }
    // Avram/Lucky style: errors: [{ field, messages: [...] }, ...]
    if let Some(arr) = body.get("errors").and_then(|v| v.as_array()) {
        let parts: Vec<String> = arr
            .iter()
            .filter_map(|e| {
                let field = e.get("field").and_then(|f| f.as_str()).unwrap_or("");
                let messages = e
                    .get("messages")
                    .and_then(|m| m.as_array())
                    .map(|msgs| {
                        msgs.iter()
                            .filter_map(|m| m.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                if messages.is_empty() {
                    None
                } else if field.is_empty() || field == "base" {
                    Some(messages)
                } else {
                    // Translate common field names for journal entries
                    let field_bg = match field {
                        "description" => "описание",
                        "entry_date" => "счетоводна дата",
                        "document_date" => "дата на документа",
                        "document_number" => "номер на документ",
                        "status" => "статус",
                        "lines" => "редове",
                        _ => field,
                    };
                    Some(format!("{}: {}", field_bg, messages))
                }
            })
            .collect();
        if !parts.is_empty() {
            return parts.join("; ");
        }
    }
    "Грешка при заявка".to_string()
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

    let response = req.send().await.map_err(|e| e.to_string())?;

    if !response.ok() {
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        return Err(extract_api_error(&body));
    }

    response.json().await.map_err(|e| e.to_string())
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

    let response = req.json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        return Err(extract_api_error(&body));
    }

    response.json().await.map_err(|e| e.to_string())
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

    let response = req.json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        return Err(extract_api_error(&body));
    }

    response.json().await.map_err(|e| e.to_string())
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

    let response = req.send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        return Err(extract_api_error(&body));
    }

    response.json().await.map_err(|e| e.to_string())
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

pub async fn api_get_text(endpoint: &str) -> Result<String, String> {
    let token = get_token().ok_or("No auth token")?;
    let url = format!("{}{}", API_BASE, endpoint);

    let mut req = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token));

    if let Some(company_id) = get_selected_company_id() {
        req = req.header("X-Company-ID", &company_id.to_string());
    }

    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        return Err(extract_api_error(&body));
    }
    response.text().await.map_err(|e| e.to_string())
}

pub async fn api_post_formdata(endpoint: &str, form: web_sys::FormData) -> Result<serde_json::Value, String> {
    let token = get_token().ok_or("No auth token")?;
    let url = format!("{}{}", API_BASE, endpoint);

    let mut req = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token));

    if let Some(company_id) = get_selected_company_id() {
        req = req.header("X-Company-ID", &company_id.to_string());
    }

    let response = req
        .body(form)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        return Err(extract_api_error(&body));
    }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn api_get_bytes(endpoint: &str) -> Result<(Vec<u8>, Option<String>), String> {
    let token = get_token().ok_or("No auth token")?;
    let url = format!("{}{}", API_BASE, endpoint);

    let mut req = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token));

    if let Some(company_id) = get_selected_company_id() {
        req = req.header("X-Company-ID", &company_id.to_string());
    }

    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        return Err(extract_api_error(&body));
    }

    let filename = response
        .headers()
        .get("content-disposition")
        .as_ref()
        .and_then(|header| content_disposition_filename(header));
    let bytes = response.binary().await.map_err(|e| e.to_string())?;
    Ok((bytes, filename))
}

fn content_disposition_filename(header: &str) -> Option<String> {
    let lower = header.to_ascii_lowercase();
    let key = "filename=";
    let idx = lower.find(key)?;
    let rest = header[idx + key.len()..].trim().trim_matches('"');
    let name = rest.split(';').next()?.trim().trim_matches('"');
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Save raw bytes. Used for NAP files, which must stay Windows-1251.
pub fn download_bytes(filename: &str, bytes: &[u8], mime: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array);

    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts)
        .map_err(|_| "Неуспешно създаване на файла".to_string())?;
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Неуспешно създаване на връзка за сваляне".to_string())?;

    let window = web_sys::window().ok_or_else(|| "Няма прозорец на браузъра".to_string())?;
    let document = window.document().ok_or_else(|| "Няма документ".to_string())?;
    let anchor = document
        .create_element("a")
        .map_err(|_| "Неуспешен линк за сваляне".to_string())?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "Неуспешен линк за сваляне".to_string())?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    } else {
        anchor.click();
    }

    let url_for_revoke = url.clone();
    gloo_timers::callback::Timeout::new(60_000, move || {
        let _ = Url::revoke_object_url(&url_for_revoke);
    })
    .forget();
    Ok(())
}

pub fn download_text_file(filename: &str, content: &str, mime: &str) {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlAnchorElement;

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(a) = document.create_element("a") {
                if let Ok(anchor) = a.dyn_into::<HtmlAnchorElement>() {
                    let encoded = js_sys::encode_uri_component(content);
                    let data_url = format!("data:{};charset=utf-8,{}", mime, encoded);
                    anchor.set_href(&data_url);
                    anchor.set_download(filename);
                    anchor.click();
                }
            }
        }
    }
}
