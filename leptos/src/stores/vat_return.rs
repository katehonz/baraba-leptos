use leptos::*;
use crate::models::{VatReturn, VatJournalEntry};
use crate::api::{api_get, api_post, api_put, api_delete};

#[derive(Clone)]
pub struct VatReturnStore {
    pub returns: RwSignal<Vec<VatReturn>>,
    pub current_return: RwSignal<Option<VatReturn>>,
    pub purchases: RwSignal<Vec<VatJournalEntry>>,
    pub sales: RwSignal<Vec<VatJournalEntry>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<String>,
}

impl VatReturnStore {
    pub fn new() -> Self {
        Self {
            returns: RwSignal::new(Vec::new()),
            current_return: RwSignal::new(None),
            purchases: RwSignal::new(Vec::new()),
            sales: RwSignal::new(Vec::new()),
            loading: RwSignal::new(false),
            error: RwSignal::new(String::new()),
        }
    }

    pub fn fetch_returns(&self, company_id: i64) {
        let returns = self.returns;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get(&format!("/api/companies/{}/vat-returns", company_id)).await {
                Ok(data) => {
                    if let Some(returns_data) = data["vat_returns"].as_array() {
                        let parsed: Result<Vec<VatReturn>, _> = returns_data
                            .iter()
                            .map(|v| serde_json::from_value(v.clone()))
                            .collect();
                        
                        match parsed {
                            Ok(r) => returns.set(r),
                            Err(e) => error.set(e.to_string()),
                        }
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn fetch_return(&self, company_id: i64, return_id: i64) {
        let current_return = self.current_return;
        let purchases = self.purchases;
        let sales = self.sales;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get(&format!("/api/companies/{}/vat-returns/{}", company_id, return_id)).await {
                Ok(data) => {
                    if let Some(return_data) = data["vat_return"].as_object() {
                        let parsed: Result<VatReturn, _> = serde_json::from_value(serde_json::to_value(return_data).unwrap());
                        if let Ok(r) = parsed {
                            current_return.set(Some(r));
                        }
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn create_return(&self, company_id: i64, vat_return: VatReturn) {
        let returns = self.returns;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_post(&format!("/api/companies/{}/vat-returns", company_id), &vat_return).await {
                Ok(_) => {
                    returns.update(|rs| rs.push(vat_return));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn delete_return(&self, company_id: i64, id: i64) {
        let returns = self.returns;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete(&format!("/api/companies/{}/vat-returns/{}", company_id, id)).await {
                Ok(_) => {
                    returns.update(|rs| rs.retain(|r| r.id != id));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }
}
