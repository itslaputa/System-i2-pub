use super::{normalize, ProjectCategoryNode};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn load_project_category_list() -> Result<Vec<ProjectCategoryNode>, String> {
    let project_category_list_path =
        crate::storage::runtime_bundle_paths::load_project_category_list_path()?;
    load_project_category_list_from_path(path_to_str(&project_category_list_path)?)
}

pub(crate) fn load_project_category_list_from_path(
    path: &str,
) -> Result<Vec<ProjectCategoryNode>, String> {
    if !Path::new(path).exists() {
        super::template::write_default_project_category_list(Path::new(path))?;
    }

    let file_contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read project category list at {}: {error}", path))?;

    let parsed_categories = serde_json::from_str::<Vec<ProjectCategoryNode>>(&file_contents)
        .map_err(|error| format!("failed to parse project category list at {}: {error}", path))?;

    normalize::normalize_project_category_list(&parsed_categories).map_err(|error| {
        format!(
            "failed to validate project category list at {}: {error}",
            path
        )
    })
}

pub(crate) fn save_project_category_list_to_path(
    path: &str,
    categories: &[ProjectCategoryNode],
) -> Result<(), String> {
    let normalized_categories = normalize::normalize_project_category_list(categories)?;
    let serialized = serde_json::to_string_pretty(&normalized_categories)
        .map_err(|error| format!("failed to serialize project category list: {error}"))?;
    let path = Path::new(path);
    let temp_path = build_temp_path(path)?;

    fs::write(&temp_path, format!("{serialized}\n")).map_err(|error| {
        format!(
            "failed to write temporary project category list at {}: {error}",
            temp_path.display()
        )
    })?;

    fs::rename(&temp_path, path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        format!(
            "failed to replace project category list at {}: {error}",
            path.display()
        )
    })?;

    Ok(())
}

fn path_to_str(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("path '{}' is not valid utf-8", path.display()))
}

fn build_temp_path(path: &Path) -> Result<PathBuf, String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("path '{}' has no valid file name", path.display()))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to build project category temp path: {error}"))?
        .as_nanos();

    Ok(parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        timestamp
    )))
}
