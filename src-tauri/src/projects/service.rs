use super::{repository, CreateTaskProjectInput, TaskProjectNode};
use rusqlite::OptionalExtension;

pub fn load_task_project_list() -> Result<Vec<TaskProjectNode>, String> {
    let connection = crate::storage::open_app_database_connection()?;

    repository::load_task_project_list_from_connection(&connection)
}

pub fn load_active_task_project_list() -> Result<Vec<TaskProjectNode>, String> {
    let connection = crate::storage::open_app_database_connection()?;

    repository::load_active_task_project_list_from_connection(&connection)
}

pub fn add_task_project(input: CreateTaskProjectInput) -> Result<TaskProjectNode, String> {
    let normalized_label = crate::storage::normalize_non_empty_text(&input.label, "project name")?;
    let normalized_project_category_id =
        crate::storage::normalize_optional_text(input.project_category_id);

    if let Some(project_category_id) = normalized_project_category_id.as_deref() {
        crate::project_categories::validate_project_category_id_exists(project_category_id)?;
    }

    let connection = crate::storage::open_app_database_connection()?;

    let created_project_id = repository::insert_task_project_into_connection(
        &connection,
        &normalized_label,
        normalized_project_category_id.as_deref(),
    )?;

    repository::load_project_by_id(&connection, created_project_id)?
        .ok_or_else(|| format!("failed to reload newly inserted project id {created_project_id}"))
}

pub fn set_task_project_category(
    project_id: String,
    project_category_id: Option<String>,
) -> Result<(), String> {
    let parsed_project_id = crate::storage::parse_optional_i64_id(Some(project_id), "project_id")?
        .ok_or_else(|| "project_id cannot be empty".to_string())?;
    let normalized_project_category_id =
        crate::storage::normalize_optional_text(project_category_id);

    if let Some(category_id) = normalized_project_category_id.as_deref() {
        crate::project_categories::validate_project_category_id_exists(category_id)?;
    }

    let connection = crate::storage::open_app_database_connection()?;
    crate::project_categories::set_project_category_id(
        &connection,
        parsed_project_id,
        normalized_project_category_id.as_deref(),
    )
}

pub fn delete_task_project(project_id: String) -> Result<(), String> {
    let parsed_project_id = crate::storage::parse_optional_i64_id(Some(project_id), "project_id")?
        .ok_or_else(|| "project_id cannot be empty".to_string())?;
    let mut connection = crate::storage::open_app_database_connection()?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start project delete transaction: {error}"))?;

    transaction
        .execute(
            "UPDATE tasks
             SET project_id = NULL,
                 is_project_closing_task = 0
             WHERE project_id = ?1",
            [parsed_project_id],
        )
        .map_err(|error| {
            format!("failed to detach tasks from project {parsed_project_id}: {error}")
        })?;

    let deleted_rows = transaction
        .execute("DELETE FROM projects WHERE id = ?1", [parsed_project_id])
        .map_err(|error| format!("failed to delete project {parsed_project_id}: {error}"))?;

    if deleted_rows == 0 {
        return Err(format!("project id {parsed_project_id} does not exist"));
    }

    transaction
        .commit()
        .map_err(|error| format!("failed to commit project delete transaction: {error}"))?;

    Ok(())
}

pub fn close_task_project(project_id: String) -> Result<(), String> {
    let parsed_project_id = crate::storage::parse_optional_i64_id(Some(project_id), "project_id")?
        .ok_or_else(|| "project_id cannot be empty".to_string())?;
    let connection = crate::storage::open_app_database_connection()?;

    let project_exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
            [parsed_project_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to verify project {parsed_project_id}: {error}"))?
        != 0;

    if !project_exists {
        return Err(format!("project id {parsed_project_id} does not exist"));
    }

    let latest_task_id = connection
        .query_row(
            "SELECT id
             FROM tasks
             WHERE project_id = ?1
             ORDER BY date DESC, id DESC
             LIMIT 1",
            [parsed_project_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| {
            format!("failed to load latest task for project {parsed_project_id}: {error}")
        })?;

    if let Some(latest_task_id) = latest_task_id {
        connection
            .execute(
                "UPDATE tasks
                 SET is_project_closing_task = CASE
                     WHEN id = ?2 THEN 1
                     ELSE 0
                 END
                 WHERE project_id = ?1",
                [parsed_project_id, latest_task_id],
            )
            .map_err(|error| format!("failed to close project {parsed_project_id}: {error}"))?;
    } else {
        connection
            .execute(
                "UPDATE projects
                 SET is_done = 1
                 WHERE id = ?1",
                [parsed_project_id],
            )
            .map_err(|error| {
                format!("failed to close empty project {parsed_project_id}: {error}")
            })?;
    }

    Ok(())
}

pub fn reopen_task_project(project_id: String) -> Result<(), String> {
    let parsed_project_id = crate::storage::parse_optional_i64_id(Some(project_id), "project_id")?
        .ok_or_else(|| "project_id cannot be empty".to_string())?;
    let connection = crate::storage::open_app_database_connection()?;

    let project_exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
            [parsed_project_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to verify project {parsed_project_id}: {error}"))?
        != 0;

    if !project_exists {
        return Err(format!("project id {parsed_project_id} does not exist"));
    }

    connection
        .execute(
            "UPDATE tasks
             SET is_project_closing_task = 0
             WHERE project_id = ?1
               AND is_project_closing_task = 1",
            [parsed_project_id],
        )
        .map_err(|error| format!("failed to reopen project {parsed_project_id}: {error}"))?;

    connection
        .execute(
            "UPDATE projects
             SET is_done = 0,
                 end_date = NULL
             WHERE id = ?1",
            [parsed_project_id],
        )
        .map_err(|error| format!("failed to sync reopened project {parsed_project_id}: {error}"))?;

    Ok(())
}
