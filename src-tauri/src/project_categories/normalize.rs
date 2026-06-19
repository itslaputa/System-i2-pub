use super::ProjectCategoryNode;
use std::collections::HashSet;

pub(crate) fn normalize_project_category_list(
    categories: &[ProjectCategoryNode],
) -> Result<Vec<ProjectCategoryNode>, String> {
    let mut known_ids = HashSet::<String>::new();
    let mut known_labels = HashSet::<String>::new();
    let mut ordered_categories = categories.to_vec();
    ordered_categories.sort_by_key(|category| category.order);

    ordered_categories
        .iter()
        .enumerate()
        .map(|(index, category)| {
            let normalized_id = category.id.trim();
            let normalized_label = category.label.trim();

            if normalized_id.is_empty() {
                return Err(format!(
                    "project category id cannot be empty at index {index}"
                ));
            }

            if normalized_label.is_empty() {
                return Err(format!(
                    "project category label cannot be empty at index {index}"
                ));
            }

            if !known_ids.insert(normalized_id.to_string()) {
                return Err(format!("duplicate project category id '{normalized_id}'"));
            }

            let normalized_label_key = normalized_label.to_lowercase();
            if !known_labels.insert(normalized_label_key) {
                return Err(format!(
                    "duplicate project category label '{normalized_label}'"
                ));
            }

            Ok(ProjectCategoryNode {
                id: normalized_id.to_string(),
                label: normalized_label.to_string(),
                order: index,
            })
        })
        .collect()
}
