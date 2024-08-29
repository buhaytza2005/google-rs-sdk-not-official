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
    pub accounts: Option<Vec<Account>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PageAdmins {
    pub page_name: String,
    pub page_title: String,
    #[serde(rename = "storeCode")]
    pub store_code: String,
    pub admin_count: usize,
    pub admins: Vec<Admin>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Admins {
    pub admins: Vec<Admin>,
}
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Admin {
    pub account: Option<String>,
    pub admin: Option<String>,
    pub name: Option<String>,
    pub role: Option<AdminRole>,
    #[serde(rename = "pendingInvitation")]
    pub pending_invitation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum AdminRole {
    #[serde(rename = "ADMIN_ROLE_UNSPECIFIED")]
    #[default]
    AdminRoleUnspecified,
    #[serde(rename = "PRIMARY_OWNER")]
    PrimaryOwner,
    Owner,
    Manager,
    #[serde(rename = "SITE_MANAGER")]
    SiteManager,
}

impl AdminRole {
    pub fn as_str(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
