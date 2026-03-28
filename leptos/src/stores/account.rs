use leptos::*;
use crate::models::{Account, SaftAccount};
use crate::api::{api_get, api_post, api_put, api_delete};

#[derive(Clone)]
pub struct AccountStore {
    pub accounts: RwSignal<Vec<Account>>,
    pub saft_accounts: RwSignal<Vec<SaftAccount>>,
    pub selected_account: RwSignal<Option<Account>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<String>,
}

impl AccountStore {
    pub fn new() -> Self {
        Self {
            accounts: RwSignal::new(Vec::new()),
            saft_accounts: RwSignal::new(Vec::new()),
            selected_account: RwSignal::new(None),
            loading: RwSignal::new(false),
            error: RwSignal::new(String::new()),
        }
    }

    pub fn fetch_accounts(&self, company_id: i64) {
        let accounts = self.accounts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get(&format!("/api/companies/{}/accounts", company_id)).await {
                Ok(data) => {
                    if let Some(accounts_data) = data["accounts"].as_array() {
                        let parsed: Result<Vec<Account>, _> = accounts_data
                            .iter()
                            .map(|a| serde_json::from_value(a.clone()))
                            .collect();
                        
                        match parsed {
                            Ok(a) => accounts.set(a),
                            Err(e) => error.set(e.to_string()),
                        }
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn fetch_saft_accounts(&self) {
        let saft_accounts = self.saft_accounts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_get("/api/saft-accounts").await {
                Ok(data) => {
                    if let Some(saft_data) = data["saft_accounts"].as_array() {
                        let parsed: Result<Vec<SaftAccount>, _> = saft_data
                            .iter()
                            .map(|s| serde_json::from_value(s.clone()))
                            .collect();
                        
                        match parsed {
                            Ok(s) => saft_accounts.set(s),
                            Err(e) => error.set(e.to_string()),
                        }
                    }
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn create_account(&self, company_id: i64, account: Account) {
        let accounts = self.accounts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_post(&format!("/api/companies/{}/accounts", company_id), &account).await {
                Ok(_) => {
                    accounts.update(|as_| as_.push(account));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn update_account(&self, company_id: i64, id: i64, account: Account) {
        let accounts = self.accounts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_put(&format!("/api/companies/{}/accounts/{}", company_id, id), &account).await {
                Ok(_) => {
                    accounts.update(|as_| {
                        if let Some(a) = as_.iter_mut().find(|a| a.id == id) {
                            *a = account;
                        }
                    });
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn delete_account(&self, company_id: i64, id: i64) {
        let accounts = self.accounts;
        let loading = self.loading;
        let error = self.error;
        
        loading.set(true);
        error.set(String::new());
        
        wasm_bindgen_futures::spawn_local(async move {
            match api_delete(&format!("/api/companies/{}/accounts/{}", company_id, id)).await {
                Ok(_) => {
                    accounts.update(|as_| as_.retain(|a| a.id != id));
                }
                Err(e) => error.set(e),
            }
            loading.set(false);
        });
    }

    pub fn get_account_by_id(&self, id: i64) -> Option<Account> {
        self.accounts.get_untracked().into_iter().find(|a| a.id == id)
    }

    pub fn get_account_by_code(&self, code: &str) -> Option<Account> {
        self.accounts.get_untracked().into_iter().find(|a| a.code == code)
    }

    pub fn get_accounts_by_type(&self, account_type: &str) -> Vec<Account> {
        self.accounts.get_untracked()
            .into_iter()
            .filter(|a| a.account_type == account_type)
            .collect()
    }

    pub fn select_account(&self, id: i64) {
        if let Some(account) = self.get_account_by_id(id) {
            self.selected_account.set(Some(account));
        }
    }
}
