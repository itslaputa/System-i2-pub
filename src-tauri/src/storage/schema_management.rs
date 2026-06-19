use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub(crate) const PROJECTS_TABLE_NAME: &str = "projects";
pub(crate) const TASKS_TABLE_NAME: &str = "tasks";
pub(crate) const PROJECTS_LEGACY_BACKUP_TABLE_NAME: &str = "projects_legacy_backup";
pub(crate) const TASKS_LEGACY_BACKUP_TABLE_NAME: &str = "tasks_legacy_backup";
pub(crate) const EXPECTED_PROJECTS_COLUMNS: &[&str] = &[
    "id",
    "name",
    "sum_time_length",
    "start_date",
    "end_date",
    "is_done",
    "project_category_id",
];
const EXPECTED_PROJECTS_COLUMNS_BEFORE_PROJECT_CATEGORY: &[&str] = &[
    "id",
    "name",
    "sum_time_length",
    "start_date",
    "end_date",
    "is_done",
];
pub(crate) const EXPECTED_TASKS_COLUMNS: &[&str] = &[
    "id",
    "category_id",
    "time_length",
    "date",
    "project_id",
    "is_project_closing_task",
    "note",
];
const LEGACY_TASKS_COLUMNS_BEFORE_NOTE: &[&str] = &[
    "id",
    "category_id",
    "time_length",
    "date",
    "project_id",
    "is_project_closing_task",
];
const LEGACY_PROJECTS_COLUMNS_WITH_CATEGORY: &[&str] = &[
    "id",
    "category_id",
    "name",
    "sum_time_length",
    "start_date",
    "end_date",
    "is_done",
];
const DATABASE_SCHEMA_SQL: &str = include_str!("schema.sql");

pub(crate) fn initialize_database_schema(
    connection: &Connection,
    database_path: &Path,
) -> Result<(), String> {
    upgrade_tasks_table_with_note_column(connection)?;
    upgrade_projects_table_without_category(connection)?;
    upgrade_projects_table_with_project_category_column(connection)?;
    preserve_legacy_table_if_schema_mismatch(
        connection,
        PROJECTS_TABLE_NAME,
        EXPECTED_PROJECTS_COLUMNS,
        PROJECTS_LEGACY_BACKUP_TABLE_NAME,
    )?;
    preserve_legacy_table_if_schema_mismatch(
        connection,
        TASKS_TABLE_NAME,
        EXPECTED_TASKS_COLUMNS,
        TASKS_LEGACY_BACKUP_TABLE_NAME,
    )?;

    connection
        .execute_batch(DATABASE_SCHEMA_SQL)
        .map_err(|error| {
            format!(
                "failed to initialize sqlite schema at {}: {error}",
                database_path.display()
            )
        })?;

    restore_or_merge_projects_from_legacy_backup(connection)?;
    if rebuild_tasks_table_with_current_project_fk_if_needed(connection)? {
        connection
            .execute_batch(DATABASE_SCHEMA_SQL)
            .map_err(|error| {
                format!(
                    "failed to rebuild sqlite schema after task foreign key migration at {}: {error}",
                    database_path.display()
                )
            })?;
    }
    clear_projectless_closing_tasks(connection)?;
    recalculate_project_rollups(connection)?;
    sync_autoincrement_sequence(connection, PROJECTS_TABLE_NAME)?;
    sync_autoincrement_sequence(connection, TASKS_TABLE_NAME)?;

    Ok(())
}

fn upgrade_projects_table_without_category(connection: &Connection) -> Result<(), String> {
    if !schema_object_exists(connection, "table", PROJECTS_TABLE_NAME)? {
        return Ok(());
    }

    let current_columns = load_table_columns(connection, PROJECTS_TABLE_NAME)?;
    if current_columns == EXPECTED_PROJECTS_COLUMNS {
        return Ok(());
    }

    if current_columns != LEGACY_PROJECTS_COLUMNS_WITH_CATEGORY {
        return Ok(());
    }

    connection
        .execute_batch(
            r#"
            PRAGMA foreign_keys = OFF;

            DROP VIEW IF EXISTS project_with_tasks;
            DROP TRIGGER IF EXISTS tasks_after_insert_sync_project;
            DROP TRIGGER IF EXISTS tasks_after_update_sync_project;
            DROP TRIGGER IF EXISTS tasks_after_delete_sync_project;

            CREATE TABLE projects_next (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL CHECK(length(trim(name)) > 0),
                sum_time_length INTEGER NOT NULL DEFAULT 0 CHECK(sum_time_length >= 0),
                start_date TEXT NULL CHECK(
                    start_date IS NULL
                    OR (length(start_date) = 10 AND substr(start_date, 5, 1) = '-' AND substr(start_date, 8, 1) = '-')
                ),
                end_date TEXT NULL CHECK(
                    end_date IS NULL
                    OR (length(end_date) = 10 AND substr(end_date, 5, 1) = '-' AND substr(end_date, 8, 1) = '-')
                ),
                is_done INTEGER NOT NULL DEFAULT 0 CHECK(is_done IN (0, 1))
            );

            INSERT INTO projects_next (id, name, sum_time_length, start_date, end_date, is_done)
            SELECT id, name, sum_time_length, start_date, end_date, is_done
            FROM projects;

            DROP TABLE projects;
            ALTER TABLE projects_next RENAME TO projects;

            PRAGMA foreign_keys = ON;
            "#,
        )
        .map_err(|error| {
            format!("failed to migrate projects table away from category ownership: {error}")
        })
}

fn upgrade_projects_table_with_project_category_column(
    connection: &Connection,
) -> Result<(), String> {
    if !schema_object_exists(connection, "table", PROJECTS_TABLE_NAME)? {
        return Ok(());
    }

    let current_columns = load_table_columns(connection, PROJECTS_TABLE_NAME)?;
    if current_columns == EXPECTED_PROJECTS_COLUMNS {
        return Ok(());
    }

    if current_columns != EXPECTED_PROJECTS_COLUMNS_BEFORE_PROJECT_CATEGORY {
        return Ok(());
    }

    connection
        .execute_batch(
            r#"
            ALTER TABLE projects
            ADD COLUMN project_category_id TEXT NULL CHECK(project_category_id IS NULL OR length(trim(project_category_id)) > 0);
            "#,
        )
        .map_err(|error| {
            format!("failed to add project_category_id column to projects table: {error}")
        })
}

fn upgrade_tasks_table_with_note_column(connection: &Connection) -> Result<(), String> {
    if !schema_object_exists(connection, "table", TASKS_TABLE_NAME)? {
        return Ok(());
    }

    let current_columns = load_table_columns(connection, TASKS_TABLE_NAME)?;
    if current_columns == EXPECTED_TASKS_COLUMNS {
        return Ok(());
    }

    if current_columns != LEGACY_TASKS_COLUMNS_BEFORE_NOTE {
        return Ok(());
    }

    connection
        .execute_batch(
            r#"
            ALTER TABLE tasks
            ADD COLUMN note TEXT NULL CHECK(note IS NULL OR length(trim(note)) > 0);
            "#,
        )
        .map_err(|error| format!("failed to add optional note column to tasks table: {error}"))
}

pub(crate) fn schema_object_exists(
    connection: &Connection,
    object_type: &str,
    object_name: &str,
) -> Result<bool, String> {
    connection
        .query_row(
            "SELECT name FROM sqlite_master WHERE type = ?1 AND name = ?2 LIMIT 1",
            [object_type, object_name],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map(|value| value.is_some())
        .map_err(|error| {
            format!(
                "failed to inspect sqlite schema object '{object_name}' of type '{object_type}': {error}"
            )
        })
}

pub(crate) fn load_table_columns(
    connection: &Connection,
    table_name: &str,
) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info(\"{table_name}\")"))
        .map_err(|error| format!("failed to inspect sqlite table '{table_name}': {error}"))?;

    let column_rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("failed to load column list for '{table_name}': {error}"))?;

    column_rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect column list for '{table_name}': {error}"))
}

fn preserve_legacy_table_if_schema_mismatch(
    connection: &Connection,
    table_name: &str,
    expected_columns: &[&str],
    backup_table_name: &str,
) -> Result<(), String> {
    if !schema_object_exists(connection, "table", table_name)? {
        return Ok(());
    }

    if load_table_columns(connection, table_name)? == expected_columns {
        return Ok(());
    }

    if schema_object_exists(connection, "table", backup_table_name)? {
        return Err(format!(
            "refusing to rename legacy table '{table_name}' because backup table '{backup_table_name}' already exists"
        ));
    }

    connection
        .execute_batch(&format!(
            "ALTER TABLE \"{table_name}\" RENAME TO \"{backup_table_name}\";"
        ))
        .map_err(|error| {
            format!(
                "failed to preserve legacy sqlite table '{table_name}' as '{backup_table_name}': {error}"
            )
        })?;

    Ok(())
}

fn restore_or_merge_projects_from_legacy_backup(connection: &Connection) -> Result<(), String> {
    if !schema_object_exists(connection, "table", PROJECTS_TABLE_NAME)?
        || !schema_object_exists(connection, "table", PROJECTS_LEGACY_BACKUP_TABLE_NAME)?
    {
        return Ok(());
    }

    let backup_columns = load_table_columns(connection, PROJECTS_LEGACY_BACKUP_TABLE_NAME)?;
    if backup_columns != EXPECTED_PROJECTS_COLUMNS
        && backup_columns != EXPECTED_PROJECTS_COLUMNS_BEFORE_PROJECT_CATEGORY
    {
        return Ok(());
    }

    connection
        .execute_batch(&format!(
            r#"
            INSERT INTO "{PROJECTS_TABLE_NAME}" (
                id,
                name,
                sum_time_length,
                start_date,
                end_date,
                is_done,
                project_category_id
            )
            SELECT
                backup.id,
                backup.name,
                backup.sum_time_length,
                backup.start_date,
                backup.end_date,
                backup.is_done,
                NULL
            FROM "{PROJECTS_LEGACY_BACKUP_TABLE_NAME}" backup
            WHERE NOT EXISTS (
                SELECT 1
                FROM "{PROJECTS_TABLE_NAME}" current
                WHERE current.id = backup.id
            );
            "#
        ))
        .map_err(|error| {
            format!(
                "failed to restore projects from '{PROJECTS_LEGACY_BACKUP_TABLE_NAME}': {error}"
            )
        })?;

    Ok(())
}

fn rebuild_tasks_table_with_current_project_fk_if_needed(
    connection: &Connection,
) -> Result<bool, String> {
    if !schema_object_exists(connection, "table", TASKS_TABLE_NAME)?
        || !schema_object_exists(connection, "table", PROJECTS_TABLE_NAME)?
    {
        return Ok(false);
    }

    if load_table_columns(connection, TASKS_TABLE_NAME)? != EXPECTED_TASKS_COLUMNS {
        return Ok(false);
    }

    if load_task_project_fk(connection)?.is_some_and(|foreign_key| {
        foreign_key.target_table == PROJECTS_TABLE_NAME
            && foreign_key.on_delete.eq_ignore_ascii_case("SET NULL")
    }) {
        return Ok(false);
    }

    connection
        .execute_batch(
            r#"
            PRAGMA foreign_keys = OFF;

            DROP VIEW IF EXISTS project_with_tasks;
            DROP TRIGGER IF EXISTS tasks_after_insert_sync_project;
            DROP TRIGGER IF EXISTS tasks_after_update_sync_project;
            DROP TRIGGER IF EXISTS tasks_after_delete_sync_project;

            CREATE TABLE tasks_next (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_id TEXT NOT NULL CHECK(length(trim(category_id)) > 0),
                time_length INTEGER NOT NULL CHECK(time_length >= 0),
                date TEXT NOT NULL CHECK(
                    length(date) = 10
                    AND substr(date, 5, 1) = '-'
                    AND substr(date, 8, 1) = '-'
                ),
                project_id INTEGER NULL REFERENCES projects(id) ON DELETE SET NULL,
                is_project_closing_task INTEGER NOT NULL DEFAULT 0 CHECK(is_project_closing_task IN (0, 1)),
                note TEXT NULL CHECK(note IS NULL OR length(trim(note)) > 0)
            );

            INSERT INTO tasks_next (
                id,
                category_id,
                time_length,
                date,
                project_id,
                is_project_closing_task,
                note
            )
            SELECT
                id,
                category_id,
                time_length,
                date,
                CASE
                    WHEN project_id IS NULL THEN NULL
                    WHEN EXISTS(SELECT 1 FROM projects WHERE id = tasks.project_id) THEN project_id
                    ELSE NULL
                END,
                CASE
                    WHEN project_id IS NULL THEN 0
                    WHEN EXISTS(SELECT 1 FROM projects WHERE id = tasks.project_id) THEN is_project_closing_task
                    ELSE 0
                END,
                note
            FROM tasks;

            DROP TABLE tasks;
            ALTER TABLE tasks_next RENAME TO tasks;

            PRAGMA foreign_keys = ON;
            "#,
        )
        .map_err(|error| {
            format!("failed to rebuild tasks table with current project foreign key: {error}")
        })?;

    Ok(true)
}

struct TaskProjectForeignKey {
    target_table: String,
    on_delete: String,
}

fn load_task_project_fk(connection: &Connection) -> Result<Option<TaskProjectForeignKey>, String> {
    let mut statement = connection
        .prepare("PRAGMA foreign_key_list(\"tasks\")")
        .map_err(|error| format!("failed to inspect tasks foreign keys: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|error| format!("failed to load tasks foreign keys: {error}"))?;

    for row in rows {
        let (target_table, from_column, on_delete) =
            row.map_err(|error| format!("failed to read tasks foreign key row: {error}"))?;
        if from_column == "project_id" {
            return Ok(Some(TaskProjectForeignKey {
                target_table,
                on_delete,
            }));
        }
    }

    Ok(None)
}

fn clear_projectless_closing_tasks(connection: &Connection) -> Result<(), String> {
    if !schema_object_exists(connection, "table", TASKS_TABLE_NAME)? {
        return Ok(());
    }

    connection
        .execute(
            "UPDATE tasks
             SET is_project_closing_task = 0
             WHERE project_id IS NULL
               AND is_project_closing_task = 1",
            [],
        )
        .map_err(|error| format!("failed to clear projectless closing tasks: {error}"))?;

    Ok(())
}

fn recalculate_project_rollups(connection: &Connection) -> Result<(), String> {
    if !schema_object_exists(connection, "table", PROJECTS_TABLE_NAME)?
        || !schema_object_exists(connection, "table", TASKS_TABLE_NAME)?
    {
        return Ok(());
    }

    connection
        .execute_batch(
            r#"
            UPDATE projects
            SET
                sum_time_length = COALESCE(
                    (SELECT SUM(time_length) FROM tasks WHERE project_id = projects.id),
                    0
                ),
                start_date = (SELECT MIN(date) FROM tasks WHERE project_id = projects.id),
                end_date = (
                    SELECT MAX(date)
                    FROM tasks
                    WHERE project_id = projects.id
                      AND is_project_closing_task = 1
                ),
                is_done = CASE
                    WHEN EXISTS(
                        SELECT 1
                        FROM tasks
                        WHERE project_id = projects.id
                          AND is_project_closing_task = 1
                    ) THEN 1
                    ELSE 0
                END;
            "#,
        )
        .map_err(|error| format!("failed to recalculate project rollups: {error}"))
}

fn sync_autoincrement_sequence(connection: &Connection, table_name: &str) -> Result<(), String> {
    if !schema_object_exists(connection, "table", table_name)?
        || !schema_object_exists(connection, "table", "sqlite_sequence")?
    {
        return Ok(());
    }

    let max_id = connection
        .query_row(
            &format!("SELECT COALESCE(MAX(id), 0) FROM \"{table_name}\""),
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to load max id for '{table_name}': {error}"))?;

    let updated_rows = connection
        .execute(
            "UPDATE sqlite_sequence
             SET seq = CASE WHEN seq < ?2 THEN ?2 ELSE seq END
             WHERE name = ?1",
            params![table_name, max_id],
        )
        .map_err(|error| format!("failed to update sqlite sequence for '{table_name}': {error}"))?;

    if updated_rows == 0 {
        connection
            .execute(
                "INSERT INTO sqlite_sequence (name, seq) VALUES (?1, ?2)",
                params![table_name, max_id],
            )
            .map_err(|error| {
                format!("failed to insert sqlite sequence for '{table_name}': {error}")
            })?;
    }

    Ok(())
}
