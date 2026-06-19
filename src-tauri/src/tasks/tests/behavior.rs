#![cfg(test)]

use super::repository::{insert_task_into_connection, load_task_records_from_connection};
use super::service::{create_task_record, delete_task_record, load_task_records};
use crate::projects::repository::insert_task_project_into_connection;
use crate::projects::{add_task_project, CreateTaskProjectInput};

#[test]
fn inserts_and_lists_tasks_from_sqlite() {
    let database_path = crate::test_support::unique_temp_database_path("tasks");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let project_id = insert_task_project_into_connection(&connection, "Юнг - АИКБ", None)
        .expect("expected sqlite project insert");
    insert_task_into_connection(
        &connection,
        "logic-math",
        90,
        "2026-03-12",
        Some(project_id),
        true,
        Some("Финальная глава"),
    )
    .expect("expected sqlite task insert");

    let tasks = load_task_records_from_connection(&connection).expect("expected task list to load");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].category_id, "logic-math");
    assert_eq!(tasks[0].time_length, 90);
    assert_eq!(tasks[0].note.as_deref(), Some("Финальная глава"));
    assert_eq!(tasks[0].project_label.as_deref(), Some("Юнг - АИКБ"));
    assert!(tasks[0].is_project_closing_task);

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn public_create_and_delete_task_update_project_rollup() {
    let database_path = crate::test_support::unique_temp_database_path("tasks");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");

    let created_project = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            add_task_project(CreateTaskProjectInput {
                label: "Тестовый проект".to_string(),
                project_category_id: None,
            })
        })
    })
    .expect("expected project create through public API");

    let created_task = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            create_task_record(super::CreateTaskInput {
                category_id: "logic-math".to_string(),
                project_id: Some(created_project.id.clone()),
                task_date: "2026-03-12".to_string(),
                duration_minutes: 45,
                is_project_closing_task: true,
                note: Some("Первая заметка".to_string()),
            })
        })
    })
    .expect("expected task create through public API");

    let tasks = crate::storage::with_test_database_path(&database_path, load_task_records)
        .expect("expected task list through public API");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, created_task.id);
    assert_eq!(tasks[0].note.as_deref(), Some("Первая заметка"));

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");
    let project_state: (i64, Option<String>, Option<String>, i64) = connection
        .query_row(
            "SELECT sum_time_length, start_date, end_date, is_done FROM projects WHERE id = ?1",
            [created_project
                .id
                .parse::<i64>()
                .expect("project id should parse")],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("expected project state after insert");
    assert_eq!(project_state.0, 45);
    assert_eq!(project_state.1.as_deref(), Some("2026-03-12"));
    assert_eq!(project_state.2.as_deref(), Some("2026-03-12"));
    assert_eq!(project_state.3, 1);

    crate::storage::with_test_database_path(&database_path, || delete_task_record(created_task.id))
        .expect("expected task delete through public API");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection after delete");
    let project_state_after_delete: (i64, Option<String>, Option<String>, i64) = connection
        .query_row(
            "SELECT sum_time_length, start_date, end_date, is_done FROM projects WHERE id = ?1",
            [created_project
                .id
                .parse::<i64>()
                .expect("project id should parse")],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("expected project state after task delete");
    assert_eq!(project_state_after_delete.0, 0);
    assert_eq!(project_state_after_delete.1, None);
    assert_eq!(project_state_after_delete.2, None);
    assert_eq!(project_state_after_delete.3, 0);

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn allows_task_to_link_project_across_categories() {
    let database_path = crate::test_support::unique_temp_database_path("tasks");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");

    let created_project = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            add_task_project(CreateTaskProjectInput {
                label: "Глобальный проект".to_string(),
                project_category_id: None,
            })
        })
    })
    .expect("expected project create through public API");

    let created_task = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            create_task_record(super::CreateTaskInput {
                category_id: "mind-reading".to_string(),
                project_id: Some(created_project.id.clone()),
                task_date: "2026-03-12".to_string(),
                duration_minutes: 30,
                is_project_closing_task: false,
                note: Some("Кросс-категорийная связь".to_string()),
            })
        })
    })
    .expect("expected cross-category project link to work");

    assert_eq!(created_task.category_id, "mind-reading");
    assert_eq!(
        created_task.project_id.as_deref(),
        Some(created_project.id.as_str())
    );
    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn normalizes_blank_task_note_to_none() {
    let database_path = crate::test_support::unique_temp_database_path("tasks");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");

    let created_task = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            create_task_record(super::CreateTaskInput {
                category_id: "mind-reading".to_string(),
                project_id: None,
                task_date: "2026-03-12".to_string(),
                duration_minutes: 15,
                is_project_closing_task: false,
                note: Some("   ".to_string()),
            })
        })
    })
    .expect("expected task create with blank note");

    assert_eq!(created_task.note, None);
    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn rejects_closing_task_without_selected_project() {
    let database_path = crate::test_support::unique_temp_database_path("tasks");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");

    let error = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            create_task_record(super::CreateTaskInput {
                category_id: "mind-reading".to_string(),
                project_id: None,
                task_date: "2026-03-12".to_string(),
                duration_minutes: 20,
                is_project_closing_task: true,
                note: Some("Нельзя закрыть без проекта".to_string()),
            })
        })
    })
    .expect_err("expected closing task without project to fail");

    assert!(error.contains("isProjectClosingTask cannot be true"));
    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}
