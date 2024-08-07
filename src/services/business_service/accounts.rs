use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub account_name: String,
    pub name: String,
    #[serde(rename = "type")]
    pub the_type: String,
    pub verification_state: String,
    pub vetted_state: String,
    pub account_number: Option<String>,
    pub permission_level: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accounts {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PageAdmins {
    pub page_name: String,
    pub page_title: String,
    #[serde(rename = "storeCode")]
    pub store_code: String,
    pub admin_count: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Admins {
    pub admins: Vec<Admin>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Admin {
    pub account: String,
    pub admin: String,
    pub name: String,
    pub role: String,
}
