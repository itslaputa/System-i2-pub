use super::TaskCategoryNode;
use std::collections::HashSet;

pub(crate) fn normalize_category_tree(
    tree: &[TaskCategoryNode],
) -> Result<Vec<TaskCategoryNode>, String> {
    let mut known_ids = HashSet::new();

    tree.iter()
        .enumerate()
        .map(|(index, node)| {
            normalize_category_node(node, 1, &mut known_ids, &format!("root[{index}]"))
        })
        .collect()
}

fn normalize_category_node(
    node: &TaskCategoryNode,
    depth: usize,
    known_ids: &mut HashSet<String>,
    location: &str,
) -> Result<TaskCategoryNode, String> {
    if depth > 5 {
        return Err(format!("category depth exceeds 5 at {location}"));
    }

    let normalized_id = node.id.trim();
    let normalized_label = node.label.trim();

    if normalized_id.is_empty() {
        return Err(format!("category id cannot be empty at {location}"));
    }

    if normalized_label.is_empty() {
        return Err(format!("category label cannot be empty at {location}"));
    }

    if !known_ids.insert(normalized_id.to_string()) {
        return Err(format!("duplicate category id '{normalized_id}'"));
    }

    let normalized_children = match node.children.as_ref() {
        Some(children) if !children.is_empty() => Some(
            children
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    normalize_category_node(
                        child,
                        depth + 1,
                        known_ids,
                        &format!("{location}.children[{index}]"),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
        _ => None,
    };

    Ok(TaskCategoryNode {
        id: normalized_id.to_string(),
        label: normalized_label.to_string(),
        children: normalized_children,
    })
}
