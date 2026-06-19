use super::TaskProjectNode;
use rusqlite::{params, Connection, OptionalExtension};

pub(crate) fn load_project_by_id(
    connection: &Connection,
    project_id: i64,
) -> Result<Option<TaskProjectNode>, String> {
    connection
        .query_row(
            "SELECT id, name, project_category_id, sum_time_length, start_date, end_date, is_done, tasks FROM project_with_tasks WHERE id = ?1",
            [project_id],
            map_task_project_row,
        )
        .optional()
        .map_err(|error| format!("failed to load project {project_id}: {error}"))
}

pub(crate) fn load_task_project_list_from_connection(
    connection: &Connection,
) -> Result<Vec<TaskProjectNode>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, project_category_id, sum_time_length, start_date, end_date, is_done, tasks
             FROM project_with_tasks
             ORDER BY is_done ASC, id DESC",
        )
        .map_err(|error| format!("failed to prepare project list query: {error}"))?;

    let rows = statement
        .query_map([], map_task_project_row)
        .map_err(|error| format!("failed to load project rows: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect project rows: {error}"))?;

    Ok(rows)
}

pub(crate) fn load_active_task_project_list_from_connection(
    connection: &Connection,
) -> Result<Vec<TaskProjectNode>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, project_category_id, sum_time_length, start_date, end_date, is_done, tasks
             FROM project_with_tasks
             WHERE is_done = 0
             ORDER BY id DESC",
        )
        .map_err(|error| format!("failed to prepare active project list query: {error}"))?;

    let rows = statement
        .query_map([], map_task_project_row)
        .map_err(|error| format!("failed to load active project rows: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect active project rows: {error}"))?;

    Ok(rows)
}

pub(crate) fn insert_task_project_into_connection(
    connection: &Connection,
    label: &str,
    project_category_id: Option<&str>,
) -> Result<i64, String> {
    let existing_project = connection
        .query_row(
            "SELECT id FROM projects WHERE trim(name) = ?1 LIMIT 1",
            params![label],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| format!("failed to check existing projects: {error}"))?;

    if existing_project.is_some() {
        return Err(format!("project '{label}' already exists"));
    }

    connection
        .execute(
            "INSERT INTO projects (name, project_category_id) VALUES (?1, ?2)",
            params![label, project_category_id],
        )
        .map_err(|error| format!("failed to insert project into sqlite: {error}"))?;

    Ok(connection.last_insert_rowid())
}

fn map_task_project_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskProjectNode> {
    let tasks_csv = row.get::<_, String>(7)?;
    let tasks = if tasks_csv.trim().is_empty() {
        Vec::new()
    } else {
        tasks_csv
            .split(',')
            .filter_map(|task_id| {
                let normalized = task_id.trim();
                if normalized.is_empty() {
                    None
                } else {
                    Some(normalized.to_string())
                }
            })
            .collect()
    };

    Ok(TaskProjectNode {
        id: row.get::<_, i64>(0)?.to_string(),
        label: row.get(1)?,
        project_category_id: row.get(2)?,
        sum_time_length: row.get(3)?,
        start_date: row.get(4)?,
        end_date: row.get(5)?,
        is_done: row.get::<_, i64>(6)? != 0,
        tasks,
    })
}
