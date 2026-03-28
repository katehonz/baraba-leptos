use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::{api_get, get_selected_company_id, set_selected_company_id, get_token};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CompanyInfo {
    pub id: i64,
    pub name: String,
    pub eik: Option<String>,
}

#[derive(Clone, Copy)]
pub struct CompanyContext {
    pub companies: RwSignal<Vec<CompanyInfo>>,
    pub selected_company_id: RwSignal<Option<i64>>,
    pub loading: RwSignal<bool>,
}

impl CompanyContext {
    pub fn new() -> Self {
        Self {
            companies: create_rw_signal(Vec::new()),
            selected_company_id: create_rw_signal(None),
            loading: create_rw_signal(false),
        }
    }

    pub fn selected_company(&self) -> Option<CompanyInfo> {
        let id = self.selected_company_id.get()?;
        self.companies.get().into_iter().find(|c| c.id == id)
    }

    pub fn load_companies(&self) {
        // Only load companies if user is logged in (has token)
        if get_token().is_none() {
            return;
        }

        let companies = self.companies;
        let selected_id = self.selected_company_id;
        let loading = self.loading;

        // Load saved company ID from localStorage
        let saved_company_id = get_selected_company_id();

        loading.set(true);

        spawn_local(async move {
            match api_get("/api/my_companies").await {
                Ok(data) => {
                    // API returns { data: [...] }
                    if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                        let items: Vec<CompanyInfo> = arr.iter()
                            .filter_map(|v| {
                                Some(CompanyInfo {
                                    id: v.get("id")?.as_i64()?,
                                    name: v.get("name")?.as_str()?.to_string(),
                                    eik: v.get("eik").and_then(|e| e.as_str()).map(|s| s.to_string()),
                                })
                            })
                            .collect();

                        // Use saved company ID if valid, otherwise select first
                        if let Some(saved_id) = saved_company_id {
                            if items.iter().any(|c| c.id == saved_id) {
                                selected_id.set(Some(saved_id));
                            } else if let Some(first) = items.first() {
                                selected_id.set(Some(first.id));
                                set_selected_company_id(first.id);
                            }
                        } else if let Some(first) = items.first() {
                            selected_id.set(Some(first.id));
                            set_selected_company_id(first.id);
                        }

                        companies.set(items);
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load companies: {}", e).into());
                }
            }
            loading.set(false);
        });
    }

    pub fn select_company(&self, id: i64) {
        self.selected_company_id.set(Some(id));
        set_selected_company_id(id);
    }
}

pub fn provide_company_context() {
    let ctx = CompanyContext::new();
    provide_context(ctx);
    ctx.load_companies();
}

pub fn use_company_context() -> CompanyContext {
    use_context::<CompanyContext>().expect("CompanyContext not provided")
}
