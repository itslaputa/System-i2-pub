use super::TaskCategoryNode;
use std::fs;
use std::path::Path;

const DEFAULT_TASK_CATEGORY_TREE_JSON: &str = include_str!("default_tree.json");

pub(crate) fn load_default_task_category_tree() -> Result<Vec<TaskCategoryNode>, String> {
    serde_json::from_str::<Vec<TaskCategoryNode>>(DEFAULT_TASK_CATEGORY_TREE_JSON)
        .map_err(|error| format!("failed to parse bundled default category tree: {error}"))
}

pub(crate) fn write_default_task_category_tree(path: &Path) -> Result<(), String> {
    let tree = load_default_task_category_tree()?;
    let serialized = serde_json::to_string_pretty(&tree)
        .map_err(|error| format!("failed to serialize bundled default category tree: {error}"))?;

    fs::write(path, format!("{serialized}\n")).map_err(|error| {
        format!(
            "failed to write bundled default category tree at {}: {error}",
            path.display()
        )
    })?;

    super::io::load_task_category_tree_from_path(
        path.to_str()
            .ok_or_else(|| format!("path '{}' is not valid utf-8", path.display()))?,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::load_default_task_category_tree;

    #[test]
    fn bundled_default_tree_is_valid() {
        let tree = load_default_task_category_tree().expect("expected bundled category tree");
        assert!(!tree.is_empty());
    }
}
