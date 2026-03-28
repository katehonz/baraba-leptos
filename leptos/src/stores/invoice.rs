use leptos::*;
use crate::models::Invoice;
use crate::api::{api_get, api_post, api_put, api_delete};

#[derive(Clone)]
pub struct InvoiceStore {
    pub invoices: RwSignal<Vec<Invoice>>,
    pub selected_invoice: RwSignal<Option<Invoice>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<String>,
}

impl InvoiceStore {
    pub fn new() -> Self {
        Self {
            invoices: RwSignal::new(Vec::new()),
            selected_invoice: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(String::new()),
        }
    }

    pub fn fetch_invoices(&self, company_id: i64) {
        let invoices = self.invoices;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get(&format!("/api/companies/{}/invoices", company_id)).await {
                Ok(data) => {
                    if let Some(invoices_data) = data["invoices"].as_array() {
                        let parsed: Result<Vec<Invoice>, _> = invoices_data
                            .iter()
                            .map(|i| serde_json::from_value(i.clone()))
                            .collect();
                        
                        match parsed {
                            Ok(i) => invoices.set(i),
                            Err(e) => error.set(e.to_string()),
                        }
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn create_invoice(&self, company_id: i64, invoice: Invoice) {
        let invoices = self.invoices;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_post(&format!("/api/companies/{}/invoices", company_id), &invoice).await {
                Ok(_) => {
                    invoices.update(|is_| is_.push(invoice));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn update_invoice(&self, company_id: i64, id: i64, invoice: Invoice) {
        let invoices = self.invoices;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_put(&format!("/api/companies/{}/invoices/{}", company_id, id), &invoice).await {
                Ok(_) => {
                    invoices.update(|is_| {
                        if let Some(i) = is_.iter_mut().find(|i| i.id == id) {
                            *i = invoice;
                        }
                    });
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn delete_invoice(&self, company_id: i64, id: i64) {
        let invoices = self.invoices;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete(&format!("/api/companies/{}/invoices/{}", company_id, id)).await {
                Ok(_) => {
                    invoices.update(|is_| is_.retain(|i| i.id != id));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn select_invoice(&self, id: i64) {
        let invoices = self.invoices;
        let selected = self.selected_invoice;
        
        if let Some(invoice) = invoices.get_untracked().iter().find(|i| i.id == id) {
            selected.set(Some(invoice.clone()));
        }
    }
}
