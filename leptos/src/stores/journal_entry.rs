use leptos::*;
use crate::models::{JournalEntry, JournalEntryLine};
use crate::api::{api_get, api_post, api_put, api_delete};

#[derive(Clone)]
pub struct JournalEntryStore {
    pub entries: RwSignal<Vec<JournalEntry>>,
    pub selected_entry: RwSignal<Option<JournalEntry>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<String>,
}

impl JournalEntryStore {
    pub fn new() -> Self {
        Self {
            entries: RwSignal::new(Vec::new()),
            selected_entry: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(String::new()),
        }
    }

    pub fn fetch_entries(&self, company_id: i64) {
        let entries = self.entries;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get(&format!("/api/companies/{}/journal-entries", company_id)).await {
                Ok(data) => {
                    if let Some(entries_data) = data["journal_entries"].as_array() {
                        let parsed: Result<Vec<JournalEntry>, _> = entries_data
                            .iter()
                            .map(|j| serde_json::from_value(j.clone()))
                            .collect();
                        
                        match parsed {
                            Ok(e) => entries.set(e),
                            Err(e) => error.set(e.to_string()),
                        }
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn create_entry(&self, company_id: i64, entry: JournalEntry) {
        let entries = self.entries;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_post(&format!("/api/companies/{}/journal-entries", company_id), &entry).await {
                Ok(_) => {
                    if let Some(mut e) = entries.get_untracked().pop() {
                        e.id = entries.get_untracked().len() as i64 + 1;
                        entries.update(|es| es.push(e));
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn update_entry(&self, company_id: i64, id: i64, entry: JournalEntry) {
        let entries = self.entries;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_put(&format!("/api/companies/{}/journal-entries/{}", company_id, id), &entry).await {
                Ok(_) => {
                    entries.update(|es| {
                        if let Some(e) = es.iter_mut().find(|e| e.id == id) {
                            *e = entry;
                        }
                    });
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn delete_entry(&self, company_id: i64, id: i64) {
        let entries = self.entries;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete(&format!("/api/companies/{}/journal-entries/{}", company_id, id)).await {
                Ok(_) => {
                    entries.update(|es| es.retain(|e| e.id != id));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn select_entry(&self, id: i64) {
        let entries = self.entries;
        let selected = self.selected_entry;
        
        if let Some(entry) = entries.get_untracked().iter().find(|e| e.id == id) {
            selected.set(Some(entry.clone()));
        }
    }
}
