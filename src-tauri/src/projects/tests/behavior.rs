#![cfg(test)]

use super::repository::{
    insert_task_project_into_connection, load_active_task_project_list_from_connection,
    load_project_by_id, load_task_project_list_from_connection,
};
use super::service::{
    add_task_project, close_task_project, delete_task_project, load_active_task_project_list,
    load_task_project_list, reopen_task_project, set_task_project_category,
};
use super::CreateTaskProjectInput;
use rusqlite::params;
use std::fs;

#[test]
fn inserts_and_lists_projects_from_sqlite_view() {
    let database_path = crate::test_support::unique_temp_database_path("projects");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let created_project_id = insert_task_project_into_connection(&connection, "Юнг - АИКБ", None)
        .expect("expected sqlite project insert to succeed");
    let created_project = load_project_by_id(&connection, created_project_id)
        .expect("expected project query to succeed")
        .expect("expected project to exist");
    let all_projects = load_task_project_list_from_connection(&connection).expect("expected list");

    assert_eq!(created_project.label, "Юнг - АИКБ");
    assert_eq!(created_project.tasks.len(), 0);
    assert_eq!(all_projects.len(), 1);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn deleting_project_sets_linked_task_project_id_to_null() {
    let database_path = crate::test_support::unique_temp_database_path("projects");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let created_project_id =
        insert_task_project_into_connection(&connection, "Тестовый проект", None)
            .expect("expected sqlite project insert");
    connection
        .execute(
            "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["logic-math", 25_i64, "2026-03-12", created_project_id, 1_i64],
        )
        .expect("expected linked task insert");

    drop(connection);
    crate::storage::with_test_database_path(&database_path, || {
        delete_task_project(created_project_id.to_string())
    })
    .expect("expected project delete through public API");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection after delete");
    let orphaned_task: (Option<i64>, i64) = connection
        .query_row(
            "SELECT project_id, is_project_closing_task FROM tasks WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("expected task row to remain");

    assert_eq!(orphaned_task, (None, 0));
    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn rejects_duplicate_project_name_globally() {
    let database_path = crate::test_support::unique_temp_database_path("projects");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let first_id = insert_task_project_into_connection(&connection, "Повтор", None)
        .expect("expected first project insert");
    let duplicate_error = insert_task_project_into_connection(&connection, "Повтор", None)
        .expect_err("expected duplicate project label to fail");

    assert!(first_id > 0);
    assert!(duplicate_error.contains("already exists"));
    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn reopening_project_clears_done_state_and_end_date() {
    let database_path = crate::test_support::unique_temp_database_path("projects");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let created_project_id = insert_task_project_into_connection(&connection, "system-i2", None)
        .expect("expected sqlite project insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["10026", 30_i64, "2026-03-12", created_project_id, 0_i64],
    ).expect("expected task insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["10026", 55_i64, "2026-03-13", created_project_id, 1_i64],
    ).expect("expected closing task insert");

    let completed_project = load_project_by_id(&connection, created_project_id)
        .expect("expected project query before reopen")
        .expect("expected completed project");
    assert!(completed_project.is_done);
    assert_eq!(completed_project.end_date.as_deref(), Some("2026-03-13"));
    assert_eq!(completed_project.sum_time_length, 85);

    drop(connection);
    crate::storage::with_test_database_path(&database_path, || {
        reopen_task_project(created_project_id.to_string())
    })
    .expect("expected project reopen to succeed");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection after reopen");
    let reopened_project = load_project_by_id(&connection, created_project_id)
        .expect("expected project query after reopen")
        .expect("expected reopened project");
    let closing_task_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND is_project_closing_task = 1",
            [created_project_id],
            |row| row.get(0),
        )
        .expect("expected closing task count query");

    assert!(!reopened_project.is_done);
    assert_eq!(reopened_project.end_date, None);
    assert_eq!(reopened_project.sum_time_length, 85);
    assert_eq!(closing_task_count, 0);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn public_project_list_returns_global_projects() {
    let database_path = crate::test_support::unique_temp_database_path("projects");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");
    insert_task_project_into_connection(&connection, "Алгебра", None)
        .expect("expected logic project insert");
    insert_task_project_into_connection(&connection, "Юнг", None)
        .expect("expected mind project insert");
    drop(connection);

    let projects = crate::storage::with_test_database_path(&database_path, load_task_project_list)
        .expect("expected public project list");

    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].label, "Юнг");
    assert_eq!(projects[1].label, "Алгебра");

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn active_project_list_hides_done_projects_globally() {
    let database_path = crate::test_support::unique_temp_database_path("projects");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let active_project_id =
        insert_task_project_into_connection(&connection, "Активный проект", None)
            .expect("expected active project insert");
    let done_project_id =
        insert_task_project_into_connection(&connection, "Завершенный проект", None)
            .expect("expected done project insert");

    connection
        .execute(
            "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["10026", 25_i64, "2026-03-12", active_project_id, 0_i64],
        )
        .expect("expected active task insert");
    connection
        .execute(
            "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["10026", 35_i64, "2026-03-13", done_project_id, 1_i64],
        )
        .expect("expected closing task insert");

    let direct_filtered = load_active_task_project_list_from_connection(&connection)
        .expect("expected direct active project list");
    drop(connection);

    let public_filtered =
        crate::storage::with_test_database_path(&database_path, load_active_task_project_list)
            .expect("expected public active project list");

    assert_eq!(direct_filtered.len(), 1);
    assert_eq!(public_filtered.len(), 1);
    assert_eq!(direct_filtered[0].label, "Активный проект");
    assert_eq!(public_filtered[0].label, "Активный проект");
    assert!(!direct_filtered[0].is_done);
    assert!(!public_filtered[0].is_done);

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn closing_project_hides_it_from_active_list_and_reopen_restores_it() {
    let database_path = crate::test_support::unique_temp_database_path("projects");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let created_project_id = insert_task_project_into_connection(&connection, "Книга", None)
        .expect("expected project insert");
    connection
        .execute(
            "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["mind-read", 40_i64, "2026-03-12", created_project_id, 0_i64],
        )
        .expect("expected linked task insert");
    drop(connection);

    crate::storage::with_test_database_path(&database_path, || {
        close_task_project(created_project_id.to_string())
    })
    .expect("expected close project to succeed");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection after close");
    let closed_project = load_project_by_id(&connection, created_project_id)
        .expect("expected closed project query")
        .expect("expected project to exist");
    let active_after_close = load_active_task_project_list_from_connection(&connection)
        .expect("expected active project list after close");

    assert!(closed_project.is_done);
    assert_eq!(closed_project.end_date.as_deref(), Some("2026-03-12"));
    assert!(active_after_close.is_empty());
    drop(connection);

    crate::storage::with_test_database_path(&database_path, || {
        reopen_task_project(created_project_id.to_string())
    })
    .expect("expected reopen project to succeed");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection after reopen");
    let reopened_project = load_project_by_id(&connection, created_project_id)
        .expect("expected reopened project query")
        .expect("expected reopened project");
    let active_after_reopen = load_active_task_project_list_from_connection(&connection)
        .expect("expected active project list after reopen");

    assert!(!reopened_project.is_done);
    assert_eq!(reopened_project.end_date, None);
    assert_eq!(active_after_reopen.len(), 1);
    assert_eq!(active_after_reopen[0].id, created_project_id.to_string());

    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn closing_empty_project_marks_it_done_and_reopen_restores_it() {
    let database_path = crate::test_support::unique_temp_database_path("projects");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let created_project_id =
        insert_task_project_into_connection(&connection, "Пустой проект", None)
            .expect("expected project insert");
    drop(connection);

    crate::storage::with_test_database_path(&database_path, || {
        close_task_project(created_project_id.to_string())
    })
    .expect("expected empty project close");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection after close");
    let closed_project = load_project_by_id(&connection, created_project_id)
        .expect("expected closed project query")
        .expect("expected project to exist");

    assert!(closed_project.is_done);
    assert!(closed_project.tasks.is_empty());
    drop(connection);

    crate::storage::with_test_database_path(&database_path, || {
        reopen_task_project(created_project_id.to_string())
    })
    .expect("expected empty project reopen");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection after reopen");
    let reopened_project = load_project_by_id(&connection, created_project_id)
        .expect("expected reopened project query")
        .expect("expected project to exist");

    assert!(!reopened_project.is_done);
    crate::test_support::cleanup_database_artifacts(&database_path);
}

#[test]
fn public_project_create_persists_project_category_id() {
    let database_path = crate::test_support::unique_temp_database_path("projects-category-create");
    let bundle_dir = crate::test_support::unique_temp_dir("projects-category-create-bundle");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("projects-category-create-user");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(
        &user_runtime_config_path,
        format!(
            "{{\"schemaVersion\":1,\"bundleDir\":\"{}\"}}",
            bundle_dir.display()
        ),
    )
    .expect("expected user runtime config");

    let created_project = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            add_task_project(CreateTaskProjectInput {
                label: "Книга проекта".to_string(),
                project_category_id: Some("book".to_string()),
            })
        })
    })
    .expect("expected project create with category");

    assert_eq!(created_project.project_category_id.as_deref(), Some("book"));

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_dir(&bundle_dir);
    crate::test_support::cleanup_file(&user_runtime_config_path);
}

#[test]
fn public_project_create_rejects_unknown_project_category_id() {
    let database_path =
        crate::test_support::unique_temp_database_path("projects-category-create-missing");
    let bundle_dir =
        crate::test_support::unique_temp_dir("projects-category-create-missing-bundle");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("projects-category-create-missing-user");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(
        &user_runtime_config_path,
        format!(
            "{{\"schemaVersion\":1,\"bundleDir\":\"{}\"}}",
            bundle_dir.display()
        ),
    )
    .expect("expected user runtime config");

    let error = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            add_task_project(CreateTaskProjectInput {
                label: "Неизвестная категория".to_string(),
                project_category_id: Some("missing".to_string()),
            })
        })
    })
    .expect_err("expected unknown project category id to fail");

    assert!(error.contains("project_category_id 'missing' does not exist"));

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_dir(&bundle_dir);
    crate::test_support::cleanup_file(&user_runtime_config_path);
}

#[test]
fn public_project_category_update_changes_existing_project() {
    let database_path = crate::test_support::unique_temp_database_path("projects-category-update");
    let bundle_dir = crate::test_support::unique_temp_dir("projects-category-update-bundle");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("projects-category-update-user");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(
        &user_runtime_config_path,
        format!(
            "{{\"schemaVersion\":1,\"bundleDir\":\"{}\"}}",
            bundle_dir.display()
        ),
    )
    .expect("expected user runtime config");

    let created_project = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            add_task_project(CreateTaskProjectInput {
                label: "Фильм проекта".to_string(),
                project_category_id: None,
            })
        })
    })
    .expect("expected project create without category");

    crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            set_task_project_category(created_project.id.clone(), Some("film".to_string()))
        })
    })
    .expect("expected project category update");

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");
    let project_category_id: Option<String> = connection
        .query_row(
            "SELECT project_category_id FROM projects WHERE id = ?1",
            [created_project
                .id
                .parse::<i64>()
                .expect("project id should parse")],
            |row| row.get(0),
        )
        .expect("expected project category query");

    assert_eq!(project_category_id.as_deref(), Some("film"));

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_dir(&bundle_dir);
    crate::test_support::cleanup_file(&user_runtime_config_path);
}

#[test]
fn public_project_category_update_rejects_unknown_project_category_id() {
    let database_path =
        crate::test_support::unique_temp_database_path("projects-category-update-missing");
    let bundle_dir =
        crate::test_support::unique_temp_dir("projects-category-update-missing-bundle");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("projects-category-update-missing-user");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(
        &user_runtime_config_path,
        format!(
            "{{\"schemaVersion\":1,\"bundleDir\":\"{}\"}}",
            bundle_dir.display()
        ),
    )
    .expect("expected user runtime config");

    let created_project = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            add_task_project(CreateTaskProjectInput {
                label: "Категория для ошибки".to_string(),
                project_category_id: None,
            })
        })
    })
    .expect("expected project create without category");

    let error = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            set_task_project_category(created_project.id.clone(), Some("missing".to_string()))
        })
    })
    .expect_err("expected unknown project category update to fail");

    assert!(error.contains("project_category_id 'missing' does not exist"));

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_dir(&bundle_dir);
    crate::test_support::cleanup_file(&user_runtime_config_path);
}
