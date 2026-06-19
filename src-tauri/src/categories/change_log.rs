use super::TaskCategoryNode;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;

pub(super) fn append_category_change_log_lines(path: &str, lines: &[String]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| {
            format!(
                "failed to open task category change log at {}: {error}",
                path
            )
        })?;

    for line in lines {
        writeln!(file, "{line}").map_err(|error| {
            format!(
                "failed to append task category change log at {}: {error}",
                path
            )
        })?;
    }

    Ok(())
}

pub(super) fn collect_category_change_log_lines(
    previous_tree: &[TaskCategoryNode],
    next_tree: &[TaskCategoryNode],
    changed_at: &str,
) -> Vec<String> {
    let normalized_timestamp = if changed_at.trim().is_empty() {
        "unknown-time"
    } else {
        changed_at.trim()
    };

    let previous_by_id = flatten_category_tree_for_log(previous_tree)
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    let next_by_id = flatten_category_tree_for_log(next_tree)
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let previous_ids = previous_by_id.keys().cloned().collect::<BTreeSet<_>>();
    let next_ids = next_by_id.keys().cloned().collect::<BTreeSet<_>>();
    let mut lines = Vec::new();

    for added_id in next_ids.difference(&previous_ids) {
        if let Some(entry) = next_by_id.get(added_id) {
            lines.push(format!(
                "{} | added | {} [{}]",
                normalized_timestamp, entry.display_path, entry.id
            ));
        }
    }

    for removed_id in previous_ids.difference(&next_ids) {
        if let Some(entry) = previous_by_id.get(removed_id) {
            lines.push(format!(
                "{} | removed | {} [{}]",
                normalized_timestamp, entry.display_path, entry.id
            ));
        }
    }

    for shared_id in previous_ids.intersection(&next_ids) {
        let Some(previous_entry) = previous_by_id.get(shared_id) else {
            continue;
        };
        let Some(next_entry) = next_by_id.get(shared_id) else {
            continue;
        };

        if previous_entry.label != next_entry.label {
            lines.push(format!(
                "{} | renamed | {} -> {} [{}]",
                normalized_timestamp,
                previous_entry.display_path,
                next_entry.display_path,
                shared_id
            ));
        }
    }

    lines
}

#[derive(Debug, Clone)]
struct CategoryLogNode {
    id: String,
    label: String,
    display_path: String,
}

fn flatten_category_tree_for_log(tree: &[TaskCategoryNode]) -> Vec<CategoryLogNode> {
    flatten_category_tree_for_log_with_path(tree, &[])
}

fn flatten_category_tree_for_log_with_path(
    tree: &[TaskCategoryNode],
    parent_labels: &[String],
) -> Vec<CategoryLogNode> {
    tree.iter()
        .flat_map(|node| {
            let mut next_labels = parent_labels.to_vec();
            next_labels.push(node.label.clone());
            let current_entry = CategoryLogNode {
                id: node.id.clone(),
                label: node.label.clone(),
                display_path: next_labels.join(" / "),
            };

            let mut entries = vec![current_entry];
            if let Some(children) = node.children.as_ref() {
                entries.extend(flatten_category_tree_for_log_with_path(
                    children,
                    &next_labels,
                ));
            }

            entries
        })
        .collect()
}
