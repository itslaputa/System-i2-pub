use super::ProjectCategoryNode;
use std::fs;
use std::path::Path;

const DEFAULT_PROJECT_CATEGORY_LIST_JSON: &str = include_str!("default_list.json");

pub(crate) fn load_default_project_category_list() -> Result<Vec<ProjectCategoryNode>, String> {
    serde_json::from_str::<Vec<ProjectCategoryNode>>(DEFAULT_PROJECT_CATEGORY_LIST_JSON)
        .map_err(|error| format!("failed to parse bundled default project category list: {error}"))
}

pub(crate) fn write_default_project_category_list(path: &Path) -> Result<(), String> {
    let categories = load_default_project_category_list()?;
    let serialized = serde_json::to_string_pretty(&categories).map_err(|error| {
        format!("failed to serialize bundled default project category list: {error}")
    })?;

    fs::write(path, format!("{serialized}\n")).map_err(|error| {
        format!(
            "failed to write bundled default project category list at {}: {error}",
            path.display()
        )
    })?;

    super::io::load_project_category_list_from_path(
        path.to_str()
            .ok_or_else(|| format!("path '{}' is not valid utf-8", path.display()))?,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::load_default_project_category_list;

    #[test]
    fn bundled_default_project_categories_are_valid() {
        let categories =
            load_default_project_category_list().expect("expected bundled project categories");
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].id, "book");
        assert_eq!(categories[1].id, "film");
    }
}
