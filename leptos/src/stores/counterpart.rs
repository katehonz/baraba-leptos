use leptos::*;
use crate::models::Counterpart;
use crate::api::{api_get, api_post, api_put, api_delete};

#[derive(Clone)]
pub struct CounterpartStore {
    pub counterparts: RwSignal<Vec<Counterpart>>,
    pub selected_counterpart: RwSignal<Option<Counterpart>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<String>,
}

impl CounterpartStore {
    pub fn new() -> Self {
        Self {
            counterparts: RwSignal::new(Vec::new()),
            selected_counterpart: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(String::new()),
        }
    }

    pub fn fetch_counterparts(&self, company_id: i64) {
        let counterparts = self.counterparts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get(&format!("/api/companies/{}/counterparts", company_id)).await {
                Ok(data) => {
                    if let Some(counterparts_data) = data["counterparts"].as_array() {
                        let parsed: Result<Vec<Counterpart>, _> = counterparts_data
                            .iter()
                            .map(|c| serde_json::from_value(c.clone()))
                            .collect();
                        
                        match parsed {
                            Ok(c) => counterparts.set(c),
                            Err(e) => error.set(e.to_string()),
                        }
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn create_counterpart(&self, company_id: i64, counterpart: Counterpart) {
        let counterparts = self.counterparts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_post(&format!("/api/companies/{}/counterparts", company_id), &counterpart).await {
                Ok(_) => {
                    counterparts.update(|cs| cs.push(counterpart));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn update_counterpart(&self, company_id: i64, id: i64, counterpart: Counterpart) {
        let counterparts = self.counterparts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_put(&format!("/api/companies/{}/counterparts/{}", company_id, id), &counterpart).await {
                Ok(_) => {
                    counterparts.update(|cs| {
                        if let Some(c) = cs.iter_mut().find(|c| c.id == id) {
                            *c = counterpart;
                        }
                    });
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn delete_counterpart(&self, company_id: i64, id: i64) {
        let counterparts = self.counterparts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete(&format!("/api/companies/{}/counterparts/{}", company_id, id)).await {
                Ok(_) => {
                    counterparts.update(|cs| cs.retain(|c| c.id != id));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn select_counterpart(&self, id: i64) {
        let counterparts = self.counterparts;
        let selected = self.selected_counterpart;
        
        if let Some(counterpart) = counterparts.get_untracked().iter().find(|c| c.id == id) {
            selected.set(Some(counterpart.clone()));
        }
    }
}
