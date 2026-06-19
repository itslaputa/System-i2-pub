#![cfg(test)]

use super::super::io::save_task_category_tree_to_paths;
use super::super::normalize::normalize_category_tree;
use super::TaskCategoryNode;

#[test]
fn rejects_duplicate_category_ids() {
    let category_file_path = crate::test_support::unique_temp_json_path("categories");
    let change_log_path = crate::test_support::unique_temp_log_path("category-change-log");
    let tree = vec![
        TaskCategoryNode {
            id: "logic".to_string(),
            label: "Логика".to_string(),
            children: None,
        },
        TaskCategoryNode {
            id: "logic".to_string(),
            label: "Математика".to_string(),
            children: None,
        },
    ];

    let error = save_task_category_tree_to_paths(
        path_to_str(&category_file_path),
        path_to_str(&change_log_path),
        tree,
        "2026-03-12T12:00:00.000Z",
    )
    .expect_err("expected duplicate ids to be rejected");
    assert_eq!(error, "duplicate category id 'logic'");

    crate::test_support::cleanup_file(&category_file_path);
    crate::test_support::cleanup_file(&change_log_path);
}

#[test]
fn rejects_category_depth_above_five() {
    let category_file_path = crate::test_support::unique_temp_json_path("categories");
    let change_log_path = crate::test_support::unique_temp_log_path("category-change-log");
    let tree = vec![TaskCategoryNode {
        id: "root".to_string(),
        label: "Root".to_string(),
        children: Some(vec![TaskCategoryNode {
            id: "level-2".to_string(),
            label: "Level 2".to_string(),
            children: Some(vec![TaskCategoryNode {
                id: "level-3".to_string(),
                label: "Level 3".to_string(),
                children: Some(vec![TaskCategoryNode {
                    id: "level-4".to_string(),
                    label: "Level 4".to_string(),
                    children: Some(vec![TaskCategoryNode {
                        id: "level-5".to_string(),
                        label: "Level 5".to_string(),
                        children: Some(vec![TaskCategoryNode {
                            id: "level-6".to_string(),
                            label: "Level 6".to_string(),
                            children: None,
                        }]),
                    }]),
                }]),
            }]),
        }]),
    }];

    let error = save_task_category_tree_to_paths(
        path_to_str(&category_file_path),
        path_to_str(&change_log_path),
        tree,
        "2026-03-12T12:00:00.000Z",
    )
    .expect_err("expected depth limit to be enforced");
    assert!(error.contains("category depth exceeds 5"));

    crate::test_support::cleanup_file(&category_file_path);
    crate::test_support::cleanup_file(&change_log_path);
}

#[test]
fn normalizes_trimmed_tree_and_drops_empty_children() {
    let normalized = normalize_category_tree(&[TaskCategoryNode {
        id: " root ".to_string(),
        label: " Деньги ".to_string(),
        children: Some(vec![]),
    }])
    .expect("expected tree normalization");

    assert_eq!(normalized.len(), 1);
    assert_eq!(normalized[0].id, "root");
    assert_eq!(normalized[0].label, "Деньги");
    assert!(normalized[0].children.is_none());
}

fn path_to_str(path: &std::path::Path) -> &str {
    path.to_str().expect("temp file path should be valid utf-8")
}
