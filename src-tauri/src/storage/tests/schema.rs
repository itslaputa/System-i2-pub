#![cfg(test)]

use super::connection::{init_database_at_path, open_database};
use super::schema_management::{
    load_table_columns, schema_object_exists, EXPECTED_PROJECTS_COLUMNS, EXPECTED_TASKS_COLUMNS,
    PROJECTS_LEGACY_BACKUP_TABLE_NAME, PROJECTS_TABLE_NAME, TASKS_LEGACY_BACKUP_TABLE_NAME,
    TASKS_TABLE_NAME,
};

#[test]
fn initializes_expected_tables_views_and_indexes() {
    let database_path = crate::test_support::unique_temp_database_path("storage");
    init_database_at_path(&database_path).expect("expected sqlite schema to initialize");

    let connection =
        open_database(&database_path).expect("expected sqlite database to be readable");
    assert!(schema_object_exists(&connection, "table", PROJECTS_TABLE_NAME).unwrap());
    assert!(schema_object_exists(&connection, "table", TASKS_TABLE_NAME).unwrap());
    assert!(schema_object_exists(&connection, "view", "project_with_tasks").unwrap());
    assert!(schema_object_exists(&connection, "index", "idx_tasks_date").unwrap());
    assert!(
        schema_object_exists(&connection, "trigger", "tasks_after_insert_sync_project").unwrap()
    );
    assert_eq!(
        load_table_columns(&connection, PROJECTS_TABLE_NAME).unwrap(),
        EXPECTED_PROJECTS_COLUMNS
    );
    assert_eq!(
        load_table_columns(&connection, TASKS_TABLE_NAME).unwrap(),
        EXPECTED_TASKS_COLUMNS
    );

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn preserves_legacy_tasks_table_before_creating_new_schema() {
    let database_path = crate::test_support::unique_temp_database_path("storage");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category TEXT NOT NULL,
            task_date TEXT NOT NULL,
            project TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
        )
        .expect("expected legacy tasks table to be created");

    drop(connection);
    init_database_at_path(&database_path).expect("expected legacy table preservation");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    assert!(schema_object_exists(&connection, "table", TASKS_LEGACY_BACKUP_TABLE_NAME).unwrap());
    assert!(schema_object_exists(&connection, "table", TASKS_TABLE_NAME).unwrap());
    assert_eq!(
        load_table_columns(&connection, TASKS_TABLE_NAME).unwrap(),
        EXPECTED_TASKS_COLUMNS
    );

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn keeps_project_rollups_and_task_list_derived_from_tasks() {
    let database_path = crate::test_support::unique_temp_database_path("storage");
    init_database_at_path(&database_path).expect("expected sqlite schema to initialize");

    let connection =
        open_database(&database_path).expect("expected sqlite database to be readable");
    connection
        .execute(
            "INSERT INTO projects (name, project_category_id) VALUES (?1, ?2)",
            rusqlite::params!["Юнг - АИКБ", "book"],
        )
        .expect("expected project insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["logic-math", 40_i64, "2026-03-12", 1_i64, 0_i64],
    ).expect("expected first task insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["logic-math", 55_i64, "2026-03-13", 1_i64, 1_i64],
    ).expect("expected second task insert");

    let project_state: (i64, Option<String>, Option<String>, i64) = connection
        .query_row(
            "SELECT sum_time_length, start_date, end_date, is_done FROM projects WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("expected project state");
    assert_eq!(project_state.0, 95);
    assert_eq!(project_state.1.as_deref(), Some("2026-03-12"));
    assert_eq!(project_state.2.as_deref(), Some("2026-03-13"));
    assert_eq!(project_state.3, 1);

    let project_view: (Option<String>, String) = connection
        .query_row(
            "SELECT project_category_id, tasks FROM project_with_tasks WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("expected derived project task list");
    assert_eq!(project_view.0.as_deref(), Some("book"));
    assert_eq!(project_view.1, "1,2");

    connection
        .execute("DELETE FROM tasks WHERE id = 2", [])
        .expect("expected task delete");
    let project_state_after_delete: (i64, Option<String>, Option<String>, i64) = connection
        .query_row(
            "SELECT sum_time_length, start_date, end_date, is_done FROM projects WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("expected project state after delete");
    assert_eq!(project_state_after_delete.0, 40);
    assert_eq!(project_state_after_delete.1.as_deref(), Some("2026-03-12"));
    assert_eq!(project_state_after_delete.2, None);
    assert_eq!(project_state_after_delete.3, 0);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn migrates_category_scoped_projects_to_global_projects_without_losing_links() {
    let database_path = crate::test_support::unique_temp_database_path("storage-project-migration");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL REFERENCES projects(id) ON DELETE SET NULL,
            is_project_closing_task INTEGER NOT NULL DEFAULT 0,
            note TEXT NULL
        );

        INSERT INTO projects (id, category_id, name, sum_time_length, start_date, end_date, is_done)
        VALUES (7, 'logic-math', 'System-I2', 0, NULL, NULL, 0);

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES ('logic-math', 40, '2026-03-12', 7, 0, NULL);

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES ('mind-reading', 55, '2026-03-13', 7, 1, NULL);
        "#,
        )
        .expect("expected legacy category-scoped schema");

    drop(connection);
    init_database_at_path(&database_path).expect("expected project category migration");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    assert_eq!(
        load_table_columns(&connection, PROJECTS_TABLE_NAME).unwrap(),
        EXPECTED_PROJECTS_COLUMNS
    );
    assert!(!load_table_columns(&connection, PROJECTS_TABLE_NAME)
        .unwrap()
        .contains(&"category_id".to_string()));

    let project_state: (String, i64, Option<String>, Option<String>, i64, String) = connection
        .query_row(
            "SELECT name, sum_time_length, start_date, end_date, is_done, tasks FROM project_with_tasks WHERE id = 7",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        )
        .expect("expected migrated project row");

    assert_eq!(project_state.0, "System-I2");
    assert_eq!(project_state.1, 95);
    assert_eq!(project_state.2.as_deref(), Some("2026-03-12"));
    assert_eq!(project_state.3.as_deref(), Some("2026-03-13"));
    assert_eq!(project_state.4, 1);
    assert_eq!(project_state.5, "1,2");

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn upgrades_existing_tasks_table_with_optional_note_without_backup() {
    let database_path = crate::test_support::unique_temp_database_path("storage");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL,
            is_project_closing_task INTEGER NOT NULL DEFAULT 0
        );

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task)
        VALUES ('logic-math', 45, '2026-03-12', NULL, 0);
        "#,
        )
        .expect("expected pre-note tasks schema to be created");

    drop(connection);
    init_database_at_path(&database_path).expect("expected task note migration to succeed");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    assert!(!schema_object_exists(&connection, "table", TASKS_LEGACY_BACKUP_TABLE_NAME).unwrap());
    assert_eq!(
        load_table_columns(&connection, TASKS_TABLE_NAME).unwrap(),
        EXPECTED_TASKS_COLUMNS
    );

    let migrated_row: (String, i64, String, Option<String>) = connection
        .query_row(
            "SELECT category_id, time_length, date, note FROM tasks WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("expected migrated task row");
    assert_eq!(migrated_row.0, "logic-math");
    assert_eq!(migrated_row.1, 45);
    assert_eq!(migrated_row.2, "2026-03-12");
    assert_eq!(migrated_row.3, None);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn upgrades_existing_projects_table_with_nullable_project_category_without_backup() {
    let database_path = crate::test_support::unique_temp_database_path("storage-project-category");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0
        );

        INSERT INTO projects (name, sum_time_length, start_date, end_date, is_done)
        VALUES ('System-I2', 45, '2026-03-12', NULL, 0);
        "#,
        )
        .expect("expected pre-project-category schema to be created");

    drop(connection);
    init_database_at_path(&database_path)
        .expect("expected project category column migration to succeed");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    assert!(
        !schema_object_exists(&connection, "table", PROJECTS_LEGACY_BACKUP_TABLE_NAME).unwrap()
    );
    assert_eq!(
        load_table_columns(&connection, PROJECTS_TABLE_NAME).unwrap(),
        EXPECTED_PROJECTS_COLUMNS
    );

    let migrated_row: (String, i64, Option<String>) = connection
        .query_row(
            "SELECT name, sum_time_length, project_category_id FROM projects WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("expected migrated project row");
    assert_eq!(migrated_row.0, "System-I2");
    assert_eq!(migrated_row.1, 0);
    assert_eq!(migrated_row.2, None);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn refreshes_stale_project_with_tasks_view_after_project_category_migration() {
    let database_path = crate::test_support::unique_temp_database_path("storage-project-view");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL,
            is_project_closing_task INTEGER NOT NULL DEFAULT 0,
            note TEXT NULL
        );

        INSERT INTO projects (id, name, sum_time_length, start_date, end_date, is_done)
        VALUES (3, 'System-I2', 0, NULL, NULL, 0);

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES ('logic-math', 45, '2026-03-12', 3, 0, NULL);

        CREATE VIEW project_with_tasks AS
        SELECT
            p.id,
            p.name,
            p.sum_time_length,
            p.start_date,
            p.end_date,
            p.is_done,
            '' AS tasks
        FROM projects p;
        "#,
        )
        .expect("expected pre-project-category schema with stale view");

    drop(connection);
    init_database_at_path(&database_path).expect("expected schema init to refresh view");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    let project_view: (Option<String>, String) = connection
        .query_row(
            "SELECT project_category_id, tasks FROM project_with_tasks WHERE id = 3",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("expected refreshed project view");

    assert_eq!(project_view.0, None);
    assert_eq!(project_view.1, "1");

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn restores_projects_from_compatible_legacy_backup_when_current_table_is_empty() {
    let database_path =
        crate::test_support::unique_temp_database_path("storage-project-backup-restore");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE projects_legacy_backup (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0,
            project_category_id TEXT NULL
        );

        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL,
            is_project_closing_task INTEGER NOT NULL DEFAULT 0,
            note TEXT NULL
        );

        INSERT INTO projects_legacy_backup (
            id,
            name,
            sum_time_length,
            start_date,
            end_date,
            is_done,
            project_category_id
        )
        VALUES (9, 'Recovered project', 999, '2026-01-01', '2026-01-02', 1, 'film');

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES ('logic-math', 45, '2026-03-12', 9, 1, NULL);
        "#,
        )
        .expect("expected schema with empty current projects and populated backup");

    drop(connection);
    init_database_at_path(&database_path).expect("expected schema init to restore projects");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    assert_eq!(
        load_table_columns(&connection, PROJECTS_TABLE_NAME).unwrap(),
        EXPECTED_PROJECTS_COLUMNS
    );

    let restored_row: (String, i64, Option<String>, Option<String>, i64, Option<String>) =
        connection
            .query_row(
                "SELECT name, sum_time_length, start_date, end_date, is_done, project_category_id FROM projects WHERE id = 9",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .expect("expected restored project row");

    assert_eq!(restored_row.0, "Recovered project");
    assert_eq!(restored_row.1, 45);
    assert_eq!(restored_row.2.as_deref(), Some("2026-03-12"));
    assert_eq!(restored_row.3.as_deref(), Some("2026-03-12"));
    assert_eq!(restored_row.4, 1);
    assert_eq!(restored_row.5, None);
    assert!(schema_object_exists(&connection, "table", PROJECTS_LEGACY_BACKUP_TABLE_NAME).unwrap());

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn rewrites_tasks_foreign_key_back_to_current_projects_table() {
    let database_path = crate::test_support::unique_temp_database_path("storage-task-fk-restore");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0,
            project_category_id TEXT NULL
        );

        CREATE TABLE projects_legacy_backup (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0,
            project_category_id TEXT NULL
        );

        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL REFERENCES projects_legacy_backup(id) ON DELETE SET NULL,
            is_project_closing_task INTEGER NOT NULL DEFAULT 0,
            note TEXT NULL
        );

        INSERT INTO projects (id, name, sum_time_length, start_date, end_date, is_done, project_category_id)
        VALUES
            (8, 'Old project', 0, NULL, NULL, 0, NULL),
            (25, 'New project', 0, NULL, NULL, 0, 'book');

        INSERT INTO projects_legacy_backup (id, name, sum_time_length, start_date, end_date, is_done, project_category_id)
        VALUES (8, 'Old project', 0, NULL, NULL, 0, NULL);

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES ('logic-math', 45, '2026-03-12', 8, 0, NULL);
        "#,
        )
        .expect("expected schema with tasks FK pointing at legacy project backup");

    drop(connection);
    init_database_at_path(&database_path).expect("expected schema init to rewrite task FK");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    let task_project_fk_target: String = connection
        .query_row(
            "SELECT \"table\" FROM pragma_foreign_key_list('tasks') WHERE \"from\" = 'project_id'",
            [],
            |row| row.get(0),
        )
        .expect("expected task project foreign key");
    assert_eq!(task_project_fk_target, "projects");

    connection
        .execute(
            "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["logic-math", 35_i64, "2026-03-13", 25_i64, 0_i64, Option::<String>::None],
        )
        .expect("expected insert linked to new project to satisfy rewritten FK");

    let project_minutes: i64 = connection
        .query_row(
            "SELECT sum_time_length FROM projects WHERE id = 25",
            [],
            |row| row.get(0),
        )
        .expect("expected new project rollup");
    assert_eq!(project_minutes, 35);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn rebuilds_tasks_table_when_project_foreign_key_is_missing_and_clears_orphans() {
    let database_path = crate::test_support::unique_temp_database_path("storage-task-fk-missing");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0,
            project_category_id TEXT NULL
        );

        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL,
            is_project_closing_task INTEGER NOT NULL DEFAULT 0,
            note TEXT NULL
        );

        INSERT INTO projects (id, name, sum_time_length, start_date, end_date, is_done, project_category_id)
        VALUES (2, 'Current project', 0, NULL, NULL, 0, NULL);

        INSERT INTO tasks (id, category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES
            (1, 'logic-math', 15, '2026-03-12', 2, 0, NULL),
            (2, 'logic-math', 20, '2026-03-13', 404, 1, NULL),
            (3, 'logic-math', 25, '2026-03-14', NULL, 1, NULL);
        "#,
        )
        .expect("expected schema without task project FK");

    drop(connection);
    init_database_at_path(&database_path).expect("expected schema init to rebuild task FK");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    let task_project_fk: (String, String) = connection
        .query_row(
            "SELECT \"table\", on_delete FROM pragma_foreign_key_list('tasks') WHERE \"from\" = 'project_id'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("expected task project foreign key");
    assert_eq!(
        task_project_fk,
        ("projects".to_string(), "SET NULL".to_string())
    );

    let orphan_state: (Option<i64>, i64) = connection
        .query_row(
            "SELECT project_id, is_project_closing_task FROM tasks WHERE id = 2",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("expected orphan task row");
    assert_eq!(orphan_state, (None, 0));

    let projectless_state: (Option<i64>, i64) = connection
        .query_row(
            "SELECT project_id, is_project_closing_task FROM tasks WHERE id = 3",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("expected projectless task row");
    assert_eq!(projectless_state, (None, 0));

    connection
        .execute(
            "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["logic-math", 35_i64, "2026-03-15", 2_i64, 0_i64, Option::<String>::None],
        )
        .expect("expected insert linked to current project to satisfy rebuilt FK");

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn rebuilds_tasks_table_when_project_foreign_key_does_not_set_null_on_delete() {
    let database_path =
        crate::test_support::unique_temp_database_path("storage-task-fk-delete-action");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0,
            project_category_id TEXT NULL
        );

        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL REFERENCES projects(id),
            is_project_closing_task INTEGER NOT NULL DEFAULT 0,
            note TEXT NULL
        );

        INSERT INTO projects (id, name, sum_time_length, start_date, end_date, is_done, project_category_id)
        VALUES (3, 'Delete target', 0, NULL, NULL, 0, NULL);

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES ('logic-math', 30, '2026-03-12', 3, 0, NULL);
        "#,
        )
        .expect("expected schema with non-nullifying FK");

    drop(connection);
    init_database_at_path(&database_path).expect("expected schema init to rebuild delete action");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    let on_delete: String = connection
        .query_row(
            "SELECT on_delete FROM pragma_foreign_key_list('tasks') WHERE \"from\" = 'project_id'",
            [],
            |row| row.get(0),
        )
        .expect("expected task project foreign key");
    assert_eq!(on_delete, "SET NULL");

    connection
        .execute("DELETE FROM projects WHERE id = 3", [])
        .expect("expected project delete to nullify task link");
    let task_project_id: Option<i64> = connection
        .query_row("SELECT project_id FROM tasks WHERE id = 1", [], |row| {
            row.get(0)
        })
        .expect("expected task row");
    assert_eq!(task_project_id, None);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn merges_missing_projects_from_legacy_backup_without_overwriting_current_rows() {
    let database_path =
        crate::test_support::unique_temp_database_path("storage-project-backup-merge");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0,
            project_category_id TEXT NULL
        );

        CREATE TABLE projects_legacy_backup (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0,
            project_category_id TEXT NULL
        );

        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL,
            is_project_closing_task INTEGER NOT NULL DEFAULT 0,
            note TEXT NULL
        );

        INSERT INTO projects (id, name, sum_time_length, start_date, end_date, is_done, project_category_id)
        VALUES (5, 'Current wins', 0, NULL, NULL, 0, 'book');

        INSERT INTO projects_legacy_backup (id, name, sum_time_length, start_date, end_date, is_done, project_category_id)
        VALUES
            (5, 'Backup duplicate should not overwrite', 999, '2026-01-01', '2026-01-02', 1, 'film'),
            (9, 'Recovered missing project', 999, '2026-02-01', '2026-02-02', 1, 'film');

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES ('logic-math', 40, '2026-03-12', 9, 1, NULL);

        UPDATE sqlite_sequence SET seq = 1 WHERE name = 'projects';
        "#,
        )
        .expect("expected partial backup schema");

    drop(connection);
    init_database_at_path(&database_path).expect("expected schema init to merge backup projects");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    let current_name: String = connection
        .query_row("SELECT name FROM projects WHERE id = 5", [], |row| {
            row.get(0)
        })
        .expect("expected current project");
    assert_eq!(current_name, "Current wins");

    let recovered: (String, i64, Option<String>) = connection
        .query_row(
            "SELECT name, sum_time_length, project_category_id FROM projects WHERE id = 9",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("expected recovered project");
    assert_eq!(recovered.0, "Recovered missing project");
    assert_eq!(recovered.1, 40);
    assert_eq!(recovered.2, None);

    connection
        .execute(
            "INSERT INTO projects (name, project_category_id) VALUES (?1, ?2)",
            rusqlite::params!["Next project", Option::<String>::None],
        )
        .expect("expected insert after sequence sync");
    assert_eq!(connection.last_insert_rowid(), 10);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn recalculates_stale_project_rollups_and_restores_triggers() {
    let database_path =
        crate::test_support::unique_temp_database_path("storage-project-rollup-heal");
    let connection = open_database(&database_path).expect("expected temp sqlite database to open");

    connection
        .execute_batch(
            r#"
        CREATE TABLE projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sum_time_length INTEGER NOT NULL DEFAULT 0,
            start_date TEXT NULL,
            end_date TEXT NULL,
            is_done INTEGER NOT NULL DEFAULT 0,
            project_category_id TEXT NULL
        );

        CREATE TABLE tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id TEXT NOT NULL,
            time_length INTEGER NOT NULL,
            date TEXT NOT NULL,
            project_id INTEGER NULL REFERENCES projects(id) ON DELETE SET NULL,
            is_project_closing_task INTEGER NOT NULL DEFAULT 0,
            note TEXT NULL
        );

        INSERT INTO projects (id, name, sum_time_length, start_date, end_date, is_done, project_category_id)
        VALUES (4, 'Stale rollup', 999, '2020-01-01', '2020-01-02', 1, NULL);

        INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
        VALUES ('logic-math', 30, '2026-03-12', 4, 0, NULL);
        "#,
        )
        .expect("expected stale rollup schema");

    drop(connection);
    init_database_at_path(&database_path).expect("expected schema init to recalculate rollups");

    let connection = open_database(&database_path).expect("expected sqlite database to reopen");
    let healed: (i64, Option<String>, Option<String>, i64) = connection
        .query_row(
            "SELECT sum_time_length, start_date, end_date, is_done FROM projects WHERE id = 4",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("expected healed project rollup");
    assert_eq!(healed, (30, Some("2026-03-12".to_string()), None, 0));

    assert!(
        schema_object_exists(&connection, "trigger", "tasks_after_insert_sync_project").unwrap()
    );
    assert!(
        schema_object_exists(&connection, "trigger", "tasks_after_update_sync_project").unwrap()
    );
    assert!(
        schema_object_exists(&connection, "trigger", "tasks_after_delete_sync_project").unwrap()
    );

    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params!["logic-math", 20_i64, "2026-03-13", 4_i64, 1_i64, Option::<String>::None],
    ).expect("expected insert trigger to update rollup");

    let after_insert: (i64, Option<String>, i64) = connection
        .query_row(
            "SELECT sum_time_length, end_date, is_done FROM projects WHERE id = 4",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("expected rollup after insert trigger");
    assert_eq!(after_insert, (50, Some("2026-03-13".to_string()), 1));

    crate::test_support::cleanup_database_artifacts(&database_path);
}
