#![cfg(test)]

use super::{CreateTaskInput, TaskRecord};
use serde_json::json;

#[test]
fn deserializes_create_task_input_from_camel_case_json() {
    let deserialized = serde_json::from_value::<CreateTaskInput>(json!({
        "categoryId": "10061",
        "projectId": "5",
        "taskDate": "2026-03-13",
        "durationMinutes": 35,
        "isProjectClosingTask": false,
        "note": "Deep work session"
    }))
    .expect("expected create task input to deserialize");

    assert_eq!(deserialized.category_id, "10061");
    assert_eq!(deserialized.project_id.as_deref(), Some("5"));
    assert_eq!(deserialized.task_date, "2026-03-13");
    assert_eq!(deserialized.duration_minutes, 35);
    assert!(!deserialized.is_project_closing_task);
    assert_eq!(deserialized.note.as_deref(), Some("Deep work session"));
}

#[test]
fn serializes_task_record_using_snake_case_keys() {
    let serialized = serde_json::to_value(TaskRecord {
        id: "4".to_string(),
        category_id: "10061".to_string(),
        time_length: 30,
        date: "2026-03-13".to_string(),
        note: Some("Quick sync".to_string()),
        project_id: Some("5".to_string()),
        project_label: Some("system-i2".to_string()),
        is_project_closing_task: false,
    })
    .expect("expected task record to serialize");

    assert_eq!(
        serialized,
        json!({
            "id": "4",
            "category_id": "10061",
            "time_length": 30,
            "date": "2026-03-13",
            "note": "Quick sync",
            "project_id": "5",
            "project_label": "system-i2",
            "is_project_closing_task": false
        })
    );
}
