use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub is_ready: bool,
    pub bundle_dir: Option<String>,
    pub source: RuntimeStatusSource,
    pub missing: Vec<String>,
    pub invalid: Vec<String>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeStatusSource {
    None,
    UserConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeBundleFolderInput {
    pub bundle_dir: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeBackupResult {
    pub backup_path: String,
}
