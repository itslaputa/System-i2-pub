use super::runtime_user_config::{load_optional_user_runtime_config, UserRuntimeConfig};
use std::path::PathBuf;

pub(crate) const SQLITE_DB_FILE_NAME: &str = "tasks.sqlite3";
pub(crate) const CATEGORY_TREE_FILE_NAME: &str = "task_categories.json";
pub(crate) const CATEGORY_CHANGE_LOG_FILE_NAME: &str = "task_category_change_log.log";
pub(crate) const PROJECT_CATEGORY_LIST_FILE_NAME: &str = "project_categories.json";

#[derive(Debug, Clone)]
pub(crate) struct ResolvedRuntimePaths {
    pub bundle_dir: PathBuf,
    pub sqlite_db_path: PathBuf,
    pub category_tree_path: PathBuf,
    pub category_change_log_path: PathBuf,
    pub project_category_list_path: PathBuf,
}

pub(crate) fn load_resolved_runtime_paths() -> Result<ResolvedRuntimePaths, String> {
    let Some(user_config) = load_optional_user_runtime_config()? else {
        return Err("runtime bundle is not configured; complete first-run setup".to_string());
    };

    Ok(build_paths_from_user_config(&user_config))
}

pub fn load_sqlite_db_path() -> Result<PathBuf, String> {
    Ok(load_resolved_runtime_paths()?.sqlite_db_path)
}

pub fn load_category_tree_path() -> Result<PathBuf, String> {
    Ok(load_resolved_runtime_paths()?.category_tree_path)
}

pub fn load_category_change_log_path() -> Result<PathBuf, String> {
    Ok(load_resolved_runtime_paths()?.category_change_log_path)
}

pub fn load_project_category_list_path() -> Result<PathBuf, String> {
    Ok(load_resolved_runtime_paths()?.project_category_list_path)
}

fn build_paths_from_user_config(config: &UserRuntimeConfig) -> ResolvedRuntimePaths {
    let bundle_dir = PathBuf::from(config.bundle_dir.trim());

    ResolvedRuntimePaths {
        sqlite_db_path: bundle_dir.join(SQLITE_DB_FILE_NAME),
        category_tree_path: bundle_dir.join(CATEGORY_TREE_FILE_NAME),
        category_change_log_path: bundle_dir.join(CATEGORY_CHANGE_LOG_FILE_NAME),
        project_category_list_path: bundle_dir.join(PROJECT_CATEGORY_LIST_FILE_NAME),
        bundle_dir,
    }
}
