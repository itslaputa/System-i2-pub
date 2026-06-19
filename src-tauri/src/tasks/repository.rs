use super::TaskRecord;
use rusqlite::{params, Connection, OptionalExtension};

pub(crate) fn insert_task_into_connection(
    connection: &Connection,
    category_id: &str,
    time_length: i64,
    date: &str,
    project_id: Option<i64>,
    is_project_closing_task: bool,
    note: Option<&str>,
) -> Result<i64, String> {
    if let Some(project_id) = project_id {
        let project_exists = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
                [project_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| format!("failed to validate project {project_id}: {error}"))?
            != 0;

        if !project_exists {
            return Err(format!("project id {project_id} does not exist"));
        }
    }

    connection
        .execute(
            "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![category_id, time_length, date, project_id, if is_project_closing_task { 1_i64 } else { 0_i64 }, note],
        )
        .map_err(|error| format!("failed to insert task into sqlite: {error}"))?;

    Ok(connection.last_insert_rowid())
}

pub(crate) fn load_task_records_from_connection(
    connection: &Connection,
) -> Result<Vec<TaskRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT t.id, t.category_id, t.time_length, t.date, t.note, t.project_id, p.name, t.is_project_closing_task
             FROM tasks t
             LEFT JOIN projects p ON p.id = t.project_id
             ORDER BY t.date DESC, t.id DESC",
        )
        .map_err(|error| format!("failed to prepare task list query: {error}"))?;

    let task_rows = statement
        .query_map([], map_task_row)
        .map_err(|error| format!("failed to load task rows: {error}"))?;

    task_rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect task rows: {error}"))
}

pub(crate) fn load_task_record_by_id(
    connection: &Connection,
    task_id: i64,
) -> Result<Option<TaskRecord>, String> {
    connection
        .query_row(
            "SELECT t.id, t.category_id, t.time_length, t.date, t.note, t.project_id, p.name, t.is_project_closing_task
             FROM tasks t
             LEFT JOIN projects p ON p.id = t.project_id
             WHERE t.id = ?1",
            [task_id],
            map_task_row,
        )
        .optional()
        .map_err(|error| format!("failed to load task {task_id}: {error}"))
}

fn map_task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskRecord> {
    Ok(TaskRecord {
        id: row.get::<_, i64>(0)?.to_string(),
        category_id: row.get(1)?,
        time_length: row.get(2)?,
        date: row.get(3)?,
        note: row.get(4)?,
        project_id: row.get::<_, Option<i64>>(5)?.map(|value| value.to_string()),
        project_label: row.get(6)?,
        is_project_closing_task: row.get::<_, i64>(7)? != 0,
    })
}
