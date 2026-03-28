use leptos::*;
use crate::api::{api_get, api_post};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportStep {
    pub step: String,
    pub progress: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccountMapping {
    pub xml_code: String,
    pub xml_name: String,
    pub mapped_account_id: Option<i64>,
    pub mapped_account_code: Option<String>,
    pub mapped_account_name: Option<String>,
    pub auto_mapped: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct ControlisyImportStore {
    pub imports: RwSignal<Vec<serde_json::Value>>,
    pub preview: RwSignal<Option<serde_json::Value>>,
    pub account_mappings: RwSignal<Vec<AccountMapping>>,
    pub validation_issues: RwSignal<Vec<ValidationIssue>>,
    pub progress: RwSignal<f32>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<String>,
}

impl ControlisyImportStore {
    pub fn new() -> Self {
        Self {
            imports: RwSignal::new(Vec::new()),
            preview: RwSignal::new(None),
            account_mappings: RwSignal::new(Vec::new()),
            validation_issues: RwSignal::new(Vec::new()),
            progress: RwSignal::new(0.0),
            loading: RwSignal::new(false),
            error: RwSignal::new(String::new()),
        }
    }

    pub fn fetch_imports(&self, company_id: i64) {
        let imports = self.imports;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get(&format!("/api/companies/{}/controlisy-imports", company_id)).await {
                Ok(data) => {
                    if let Some(imports_data) = data["imports"].as_array() {
                        imports.set(imports_data.clone());
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn preview_import(&self, company_id: i64, file_data: &[u8]) {
        let preview = self.preview;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        // Placeholder for actual preview logic
        wasm_bindgen_futures::spawn_local(async move {
            loading.set(false);
        });
    }

    pub fn execute_import(&self, company_id: i64) {
        let progress = self.progress;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        progress.set(0.0);
        
        // Placeholder for actual import logic
        wasm_bindgen_futures::spawn_local(async move {
            for i in 0..=100 {
                progress.set(i as f32);
            }
            loading.set(false);
        });
    }
}
