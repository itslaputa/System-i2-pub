use super::{change_log, normalize, TaskCategoryNode};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn load_task_category_tree() -> Result<Vec<TaskCategoryNode>, String> {
    let category_tree_path = resolve_category_tree_path()?;
    load_task_category_tree_from_path(path_to_str(&category_tree_path)?)
}

pub fn load_task_category_change_log() -> Result<Vec<String>, String> {
    let change_log_path = crate::storage::runtime_bundle_paths::load_category_change_log_path()?;
    load_task_category_change_log_from_path(path_to_str(&change_log_path)?)
}

pub fn save_task_category_tree(
    tree: Vec<TaskCategoryNode>,
    changed_at: String,
) -> Result<(), String> {
    let category_tree_path = crate::storage::runtime_bundle_paths::load_category_tree_path()?;
    let change_log_path = crate::storage::runtime_bundle_paths::load_category_change_log_path()?;

    save_task_category_tree_to_paths(
        path_to_str(&category_tree_path)?,
        path_to_str(&change_log_path)?,
        tree,
        &changed_at,
    )
}

pub(crate) fn load_task_category_tree_from_path(
    path: &str,
) -> Result<Vec<TaskCategoryNode>, String> {
    let file_contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read task category file at {}: {error}", path))?;

    let parsed_tree = serde_json::from_str::<Vec<TaskCategoryNode>>(&file_contents)
        .map_err(|error| format!("failed to parse task category file at {}: {error}", path))?;

    normalize::normalize_category_tree(&parsed_tree)
        .map_err(|error| format!("failed to validate task category file at {}: {error}", path))
}

pub(crate) fn load_task_category_change_log_from_path(path: &str) -> Result<Vec<String>, String> {
    if !Path::new(path).exists() {
        fs::write(path, "").map_err(|error| {
            format!(
                "failed to create task category change log at {}: {error}",
                path
            )
        })?;
        return Ok(vec![]);
    }

    let file_contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read task category change log at {}: {error}",
            path
        )
    })?;

    Ok(file_contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .rev()
        .collect())
}

pub(crate) fn save_task_category_tree_to_paths(
    path: &str,
    change_log_path: &str,
    tree: Vec<TaskCategoryNode>,
    changed_at: &str,
) -> Result<(), String> {
    let previous_tree = load_task_category_tree_from_path(path).unwrap_or_default();
    let normalized_tree = normalize::normalize_category_tree(&tree)?;
    let change_log_lines =
        change_log::collect_category_change_log_lines(&previous_tree, &normalized_tree, changed_at);
    let serialized = serde_json::to_string_pretty(&normalized_tree)
        .map_err(|error| format!("failed to serialize task category tree: {error}"))?;

    write_atomic(path, &format!("{serialized}\n"))?;

    if !change_log_lines.is_empty() {
        change_log::append_category_change_log_lines(change_log_path, &change_log_lines)?;
    }

    Ok(())
}

fn path_to_str(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("path '{}' is not valid utf-8", path.display()))
}

fn write_atomic(path: &str, contents: &str) -> Result<(), String> {
    let path = Path::new(path);
    let temp_path = build_temp_path(path)?;

    fs::write(&temp_path, contents).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        format!(
            "failed to write temporary task category file at {}: {error}",
            temp_path.display()
        )
    })?;

    fs::rename(&temp_path, path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        format!(
            "failed to replace task category file at {}: {error}",
            path.display()
        )
    })?;

    Ok(())
}

fn build_temp_path(path: &Path) -> Result<PathBuf, String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("path '{}' has no valid file name", path.display()))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to build task category temp path: {error}"))?
        .as_nanos();

    Ok(parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        timestamp
    )))
}

fn resolve_category_tree_path() -> Result<std::path::PathBuf, String> {
    #[cfg(test)]
    {
        if let Some(override_path) = crate::storage::load_test_category_tree_override_path() {
            return Ok(override_path);
        }
    }

    crate::storage::runtime_bundle_paths::load_category_tree_path()
}
