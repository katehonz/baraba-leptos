use leptos::*;
use crate::models::Company;
use crate::api::{api_get, api_post, api_put, api_delete};

#[derive(Clone)]
pub struct CompanyStore {
    pub companies: RwSignal<Vec<Company>>,
    pub selected_company: RwSignal<Option<Company>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<String>,
}

impl CompanyStore {
    pub fn new() -> Self {
        Self {
            companies: RwSignal::new(Vec::new()),
            selected_company: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(String::new()),
        }
    }

    pub fn fetch_companies(&self) {
        let companies = self.companies;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get("/api/companies").await {
                Ok(data) => {
                    if let Some(companies_data) = data["companies"].as_array() {
                        let parsed: Result<Vec<Company>, _> = companies_data
                            .iter()
                            .map(|c| serde_json::from_value(c.clone()))
                            .collect();
                        
                        match parsed {
                            Ok(c) => companies.set(c),
                            Err(e) => error.set(e.to_string()),
                        }
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn create_company(&self, company: Company) {
        let companies = self.companies;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_post("/api/companies", &company).await {
                Ok(_) => {
                    if let Some(mut c) = companies.get_untracked().pop() {
                        c.id = companies.get_untracked().len() as i64 + 1;
                        companies.update(|cs| cs.push(c));
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn update_company(&self, id: i64, company: Company) {
        let companies = self.companies;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_put(&format!("/api/companies/{}", id), &company).await {
                Ok(_) => {
                    companies.update(|cs| {
                        if let Some(c) = cs.iter_mut().find(|c| c.id == id) {
                            *c = company;
                        }
                    });
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn delete_company(&self, id: i64) {
        let companies = self.companies;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete(&format!("/api/companies/{}", id)).await {
                Ok(_) => {
                    companies.update(|cs| cs.retain(|c| c.id != id));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn select_company(&self, id: i64) {
        let companies = self.companies;
        let selected = self.selected_company;
        
        if let Some(company) = companies.get_untracked().iter().find(|c| c.id == id) {
            selected.set(Some(company.clone()));
        }
    }
}
