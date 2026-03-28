use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::{api_get, get_token};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CompanyRole {
    pub company_id: i64,
    pub company_name: String,
    pub role: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    pub id: i64,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_super_admin: bool,
    pub company_roles: Vec<CompanyRole>,
}

#[derive(Clone, Copy)]
pub struct UserContext {
    pub user: RwSignal<Option<UserInfo>>,
    pub loading: RwSignal<bool>,
}

impl UserContext {
    pub fn new() -> Self {
        Self {
            user: create_rw_signal(None),
            loading: create_rw_signal(false),
        }
    }

    pub fn is_super_admin(&self) -> bool {
        self.user.get().map(|u| u.is_super_admin).unwrap_or(false)
    }

    pub fn role_for_company(&self, company_id: i64) -> Option<String> {
        self.user.get().and_then(|u| {
            u.company_roles
                .iter()
                .find(|cr| cr.company_id == company_id)
                .map(|cr| cr.role.clone())
        })
    }

    pub fn is_admin_for_company(&self, company_id: i64) -> bool {
        if self.is_super_admin() {
            return true;
        }
        self.role_for_company(company_id)
            .map(|r| r == "admin")
            .unwrap_or(false)
    }

    pub fn load_user(&self) {
        if get_token().is_none() {
            return;
        }

        let user = self.user;
        let loading = self.loading;

        loading.set(true);

        spawn_local(async move {
            match api_get("/api/auth/me").await {
                Ok(data) => {
                    if let Some(user_data) = data.get("data") {
                        if let Ok(u) = serde_json::from_value::<UserInfo>(user_data.clone()) {
                            user.set(Some(u));
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load user: {}", e).into());
                }
            }
            loading.set(false);
        });
    }

    pub fn clear(&self) {
        self.user.set(None);
    }
}

pub fn provide_user_context() {
    let ctx = UserContext::new();
    provide_context(ctx);
}

pub fn use_user_context() -> UserContext {
    use_context::<UserContext>().expect("UserContext not provided")
}
