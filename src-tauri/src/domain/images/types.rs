use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepairImage {
    pub id: i64,
    pub repair_id: i64,
    pub original_path: String,
    pub thumb_path: Option<String>,
    pub caption: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachRepairImagesInput {
    pub repair_id: i64,
    pub source_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRepairImageInput {
    pub caption: Option<Option<String>>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ImageVariant {
    Original,
    Thumb,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveRepairImagePathResult {
    pub absolute_path: String,
}
