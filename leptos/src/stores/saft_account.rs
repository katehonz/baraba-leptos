use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::api_get;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaftAccount {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub section_code: String,
    pub section_name: String,
    pub group_code: String,
    pub group_name: String,
    pub display_name: String,
}

impl SaftAccount {
    pub fn empty() -> Self {
        Self::default()
    }
}

#[derive(Clone)]
pub struct SaftAccountStore {
    pub accounts: RwSignal<Vec<SaftAccount>>,
    pub filtered_accounts: RwSignal<Vec<SaftAccount>>,
    pub selected_account: RwSignal<Option<SaftAccount>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<String>,
    pub loaded: RwSignal<bool>,
}

impl SaftAccountStore {
    pub fn new() -> Self {
        Self {
            accounts: create_rw_signal(Vec::new()),
            filtered_accounts: create_rw_signal(Vec::new()),
            selected_account: create_rw_signal(None),
            loading: create_rw_signal(false),
            error: create_rw_signal(String::new()),
            loaded: create_rw_signal(false),
        }
    }

    pub async fn fetch_accounts(&self) {
        if self.loaded.get() {
            return;
        }

        self.loading.set(true);
        self.error.set(String::new());

        match api_get("/api/saft_accounts").await {
            Ok(data) => {
                if let Some(accounts_data) = data.get("data") {
                    if let Ok(accounts) = serde_json::from_value::<Vec<SaftAccount>>(accounts_data.clone()) {
                        let accounts_with_display: Vec<SaftAccount> = accounts.into_iter().map(|mut acc| {
                            if acc.display_name.is_empty() {
                                acc.display_name = format!("{} - {}", acc.code, acc.name);
                            }
                            acc
                        }).collect();
                        self.accounts.set(accounts_with_display.clone());
                        self.filtered_accounts.set(accounts_with_display);
                        self.loaded.set(true);
                    }
                }
            }
            Err(e) => {
                self.error.set(format!("Failed to fetch SAF-T accounts: {}", e));
            }
        }

        self.loading.set(false);
    }

    pub async fn fetch_by_prefix(&self, prefix: &str) {
        self.loading.set(true);
        self.error.set(String::new());

        match api_get(&format!("/api/saft_accounts/prefix/{}", prefix)).await {
            Ok(data) => {
                if let Some(accounts_data) = data.get("data") {
                    if let Ok(accounts) = serde_json::from_value::<Vec<SaftAccount>>(accounts_data.clone()) {
                        let accounts_with_display: Vec<SaftAccount> = accounts.into_iter().map(|mut acc| {
                            if acc.display_name.is_empty() {
                                acc.display_name = format!("{} - {}", acc.code, acc.name);
                            }
                            acc
                        }).collect();
                        self.filtered_accounts.set(accounts_with_display);
                    }
                }
            }
            Err(e) => {
                self.error.set(format!("Failed to fetch SAF-T accounts: {}", e));
            }
        }

        self.loading.set(false);
    }

    pub fn filter_by_prefix(&self, prefix: &str) {
        if prefix.is_empty() {
            self.filtered_accounts.set(self.accounts.get());
        } else {
            let filtered: Vec<SaftAccount> = self.accounts.get()
                .into_iter()
                .filter(|acc| acc.code.starts_with(prefix))
                .collect();
            self.filtered_accounts.set(filtered);
        }
    }

    pub fn filter_by_search(&self, term: &str) {
        if term.is_empty() {
            self.filtered_accounts.set(self.accounts.get());
        } else {
            let lower_term = term.to_lowercase();
            let filtered: Vec<SaftAccount> = self.accounts.get()
                .into_iter()
                .filter(|acc| {
                    acc.code.to_lowercase().contains(&lower_term) ||
                    acc.name.to_lowercase().contains(&lower_term)
                })
                .collect();
            self.filtered_accounts.set(filtered);
        }
    }

    pub fn get_by_code(&self, code: &str) -> Option<SaftAccount> {
        self.accounts.get()
            .into_iter()
            .find(|acc| acc.code == code)
    }

    pub fn get_by_id(&self, id: i32) -> Option<SaftAccount> {
        self.accounts.get()
            .into_iter()
            .find(|acc| acc.id == id)
    }

    pub fn get_by_section(&self, section_code: &str) -> Vec<SaftAccount> {
        self.accounts.get()
            .into_iter()
            .filter(|acc| acc.section_code == section_code)
            .collect()
    }

    pub fn get_by_group(&self, group_code: &str) -> Vec<SaftAccount> {
        self.accounts.get()
            .into_iter()
            .filter(|acc| acc.group_code == group_code)
            .collect()
    }

    pub fn select_account(&self, id: i32) {
        self.selected_account.set(self.get_by_id(id));
    }

    pub fn clear_selection(&self) {
        self.selected_account.set(None);
    }
}
