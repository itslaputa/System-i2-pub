#![cfg(test)]

use super::{CreateTaskProjectInput, TaskProjectNode};
use serde_json::json;

#[test]
fn deserializes_create_task_project_input_from_camel_case_json() {
    let deserialized = serde_json::from_value::<CreateTaskProjectInput>(json!({
        "label": "system-i2",
        "projectCategoryId": "book"
    }))
    .expect("expected create project input to deserialize");

    assert_eq!(deserialized.label, "system-i2");
    assert_eq!(deserialized.project_category_id.as_deref(), Some("book"));
}

#[test]
fn serializes_task_project_node_using_snake_case_keys() {
    let serialized = serde_json::to_value(TaskProjectNode {
        id: "5".to_string(),
        label: "system-i2".to_string(),
        project_category_id: Some("film".to_string()),
        sum_time_length: 85,
        start_date: Some("2026-03-13".to_string()),
        end_date: None,
        is_done: false,
        tasks: vec!["4".to_string(), "8".to_string()],
    })
    .expect("expected task project node to serialize");

    assert_eq!(
        serialized,
        json!({
            "id": "5",
            "label": "system-i2",
            "project_category_id": "film",
            "sum_time_length": 85,
            "start_date": "2026-03-13",
            "end_date": null,
            "is_done": false,
            "tasks": ["4", "8"]
        })
    );
}
