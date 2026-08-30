use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StaffRole {
    Admin,
    Staff,
}

impl StaffRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Staff => "staff",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, crate::error::AppError> {
        match raw.trim() {
            "admin" => Ok(Self::Admin),
            "staff" => Ok(Self::Staff),
            _ => Err(crate::error::AppError::Validation {
                field: Some("role".into()),
                message: "Role must be admin or staff.".into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Staff {
    pub id: String,
    pub team_id: Option<String>,
    pub name: String,
    pub role: StaffRole,
    pub deactivated_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffInput {
    pub name: String,
    pub pin: String,
    pub role: Option<StaffRole>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffNameInput {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffPinInput {
    pub current_pin: Option<String>,
    pub new_pin: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffListQuery {
    pub query: Option<String>,
    pub include_deactivated: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffListResult {
    pub items: Vec<Staff>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub staff: Staff,
    pub device_id: String,
    pub device_name: String,
    pub device_code: Option<String>,
    pub started_at: String,
}
