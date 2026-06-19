#![cfg(test)]

use super::super::io::{
    load_task_category_change_log_from_path, load_task_category_tree_from_path,
    save_task_category_tree_to_paths,
};
use super::TaskCategoryNode;
use serde_json::json;
use std::fs;

#[test]
fn saves_trimmed_category_tree_to_json() {
    let category_file_path = crate::test_support::unique_temp_json_path("categories");
    let change_log_path = crate::test_support::unique_temp_log_path("category-change-log");
    let tree = vec![TaskCategoryNode {
        id: "  health  ".to_string(),
        label: "  Здоровье  ".to_string(),
        children: Some(vec![TaskCategoryNode {
            id: "  health-sport ".to_string(),
            label: " Спорт ".to_string(),
            children: Some(vec![]),
        }]),
    }];

    save_task_category_tree_to_paths(
        path_to_str(&category_file_path),
        path_to_str(&change_log_path),
        tree,
        "2026-03-12T12:00:00.000Z",
    )
    .expect("expected category tree to be written");
    let file_contents =
        fs::read_to_string(&category_file_path).expect("expected written category file");

    assert!(file_contents.contains("\"id\": \"health\""));
    assert!(file_contents.contains("\"label\": \"Здоровье\""));
    assert!(file_contents.contains("\"id\": \"health-sport\""));
    assert!(!file_contents.contains("\"children\": []"));

    crate::test_support::cleanup_file(&category_file_path);
    crate::test_support::cleanup_file(&change_log_path);
}

#[test]
fn appends_added_removed_and_renamed_category_log_lines() {
    let category_file_path = crate::test_support::unique_temp_json_path("categories");
    let change_log_path = crate::test_support::unique_temp_log_path("category-change-log");
    let initial_tree = vec![
        TaskCategoryNode {
            id: "logic".to_string(),
            label: "Логика".to_string(),
            children: Some(vec![TaskCategoryNode {
                id: "logic-math".to_string(),
                label: "Математика".to_string(),
                children: None,
            }]),
        },
        TaskCategoryNode {
            id: "relationships".to_string(),
            label: "Отношения".to_string(),
            children: None,
        },
    ];

    save_task_category_tree_to_paths(
        path_to_str(&category_file_path),
        path_to_str(&change_log_path),
        initial_tree,
        "2026-03-12T12:00:00.000Z",
    )
    .expect("expected initial category tree to be written");

    let updated_tree = vec![
        TaskCategoryNode {
            id: "logic".to_string(),
            label: "Мышление".to_string(),
            children: None,
        },
        TaskCategoryNode {
            id: "10000".to_string(),
            label: "Здоровье".to_string(),
            children: None,
        },
    ];

    save_task_category_tree_to_paths(
        path_to_str(&category_file_path),
        path_to_str(&change_log_path),
        updated_tree,
        "2026-03-12T13:00:00.000Z",
    )
    .expect("expected updated category tree to be written");

    let change_log_lines = load_task_category_change_log_from_path(path_to_str(&change_log_path))
        .expect("expected category change log to be readable");
    assert!(change_log_lines
        .iter()
        .any(|line| line.contains("added | Здоровье [10000]")));
    assert!(change_log_lines
        .iter()
        .any(|line| line.contains("removed | Логика / Математика [logic-math]")));
    assert!(change_log_lines
        .iter()
        .any(|line| line.contains("removed | Отношения [relationships]")));
    assert!(change_log_lines
        .iter()
        .any(|line| line.contains("renamed | Логика -> Мышление [logic]")));

    crate::test_support::cleanup_file(&category_file_path);
    crate::test_support::cleanup_file(&change_log_path);
}

#[test]
fn serializes_category_node_without_empty_children_field() {
    let serialized = serde_json::to_value(TaskCategoryNode {
        id: "10061".to_string(),
        label: "Рутина".to_string(),
        children: None,
    })
    .expect("expected category node to serialize");

    assert_eq!(serialized, json!({ "id": "10061", "label": "Рутина" }));
}

#[test]
fn loading_category_tree_rejects_duplicate_ids() {
    let category_file_path =
        crate::test_support::unique_temp_json_path("categories-duplicate-load");
    fs::write(
        &category_file_path,
        r#"[
  { "id": "logic", "label": "Логика" },
  { "id": "logic", "label": "Дубликат" }
]
"#,
    )
    .expect("expected duplicate category fixture");

    let error = load_task_category_tree_from_path(path_to_str(&category_file_path))
        .expect_err("expected duplicate category ids to fail on load");

    assert!(error.contains("failed to validate task category file"));
    assert!(error.contains("duplicate category id 'logic'"));

    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn failed_category_tree_save_does_not_leave_temp_files() {
    let missing_parent = crate::test_support::unique_temp_dir("missing-category-parent");
    crate::test_support::cleanup_dir(&missing_parent);
    let category_file_path = missing_parent.join("task_categories.json");
    let change_log_path = missing_parent.join("task_category_change_log.log");

    let error = save_task_category_tree_to_paths(
        path_to_str(&category_file_path),
        path_to_str(&change_log_path),
        vec![TaskCategoryNode {
            id: "logic".to_string(),
            label: "Логика".to_string(),
            children: None,
        }],
        "2026-03-12T12:00:00.000Z",
    )
    .expect_err("expected save into missing parent to fail");

    assert!(error.contains("failed to write temporary task category file"));
    assert!(!missing_parent.exists());
}

#[test]
fn deserializes_category_node_with_nested_children() {
    let deserialized = serde_json::from_value::<TaskCategoryNode>(json!({
        "id": "10018",
        "label": "Здоровье",
        "children": [{ "id": "10023", "label": "Медицина", "children": [{ "id": "10026", "label": "Поход на ЛФК" }] }]
    }))
    .expect("expected category node tree to deserialize");

    let child = deserialized
        .children
        .as_ref()
        .and_then(|children| children.first())
        .expect("expected first child");
    let grandchild = child
        .children
        .as_ref()
        .and_then(|children| children.first())
        .expect("expected nested child");
    assert_eq!(deserialized.id, "10018");
    assert_eq!(child.id, "10023");
    assert_eq!(grandchild.id, "10026");
    assert_eq!(grandchild.label, "Поход на ЛФК");
}

fn path_to_str(path: &std::path::Path) -> &str {
    path.to_str().expect("temp file path should be valid utf-8")
}
