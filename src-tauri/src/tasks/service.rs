use super::{repository, CreateTaskInput, TaskRecord};

pub fn create_task_record(input: CreateTaskInput) -> Result<TaskRecord, String> {
    let normalized_category_id =
        crate::storage::normalize_non_empty_text(&input.category_id, "category_id")?;
    crate::storage::validate_category_id_exists(&normalized_category_id)?;
    let normalized_task_date = crate::storage::normalize_iso_date(&input.task_date, "task_date")?;
    let normalized_note = crate::storage::normalize_optional_text(input.note);

    if input.duration_minutes < 0 {
        return Err("duration_minutes cannot be negative".to_string());
    }

    let parsed_project_id = crate::storage::parse_optional_i64_id(input.project_id, "project_id")?;
    if input.is_project_closing_task && parsed_project_id.is_none() {
        return Err(
            "isProjectClosingTask cannot be true when the task has no selected project".to_string(),
        );
    }

    let connection = crate::storage::open_app_database_connection()?;
    let created_task_id = repository::insert_task_into_connection(
        &connection,
        &normalized_category_id,
        input.duration_minutes,
        &normalized_task_date,
        parsed_project_id,
        input.is_project_closing_task,
        normalized_note.as_deref(),
    )?;

    repository::load_task_record_by_id(&connection, created_task_id)?
        .ok_or_else(|| format!("failed to reload newly inserted task id {created_task_id}"))
}

pub fn load_task_records() -> Result<Vec<TaskRecord>, String> {
    let connection = crate::storage::open_app_database_connection()?;
    repository::load_task_records_from_connection(&connection)
}

pub fn delete_task_record(task_id: String) -> Result<(), String> {
    let parsed_task_id = crate::storage::parse_optional_i64_id(Some(task_id), "task_id")?
        .ok_or_else(|| "task_id cannot be empty".to_string())?;
    let connection = crate::storage::open_app_database_connection()?;
    let deleted_rows = connection
        .execute("DELETE FROM tasks WHERE id = ?1", [parsed_task_id])
        .map_err(|error| format!("failed to delete task {parsed_task_id}: {error}"))?;

    if deleted_rows == 0 {
        return Err(format!("task id {parsed_task_id} does not exist"));
    }

    Ok(())
}
