use super::super::service::{load_project_category_list, save_project_category_list_to_path};
use super::ProjectCategoryNode;
use std::fs;

#[test]
fn saving_project_categories_remaps_removed_project_ids_to_null() {
    let database_path = crate::test_support::unique_temp_database_path("project-category-remap");
    let category_file_path =
        crate::test_support::unique_temp_json_path("project-category-remap-list");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    crate::test_support::seed_minimal_project_category_file(&category_file_path);

    let mut connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");
    connection
        .execute(
            "INSERT INTO projects (name, project_category_id) VALUES (?1, ?2)",
            rusqlite::params!["Проект", "film"],
        )
        .expect("expected project insert with category");

    save_project_category_list_to_path(
        &mut connection,
        &category_file_path,
        vec![ProjectCategoryNode {
            id: "book".to_string(),
            label: "Книга".to_string(),
            order: 0,
        }],
    )
    .expect("expected project categories save");

    let project_category_id: Option<String> = connection
        .query_row(
            "SELECT project_category_id FROM projects WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("expected remapped project row");

    assert_eq!(project_category_id, None);

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn loading_project_categories_remaps_stale_project_category_ids_to_null() {
    let database_path =
        crate::test_support::unique_temp_database_path("project-category-load-heal");
    let bundle_dir = crate::test_support::unique_temp_dir("project-category-load-heal-bundle");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("project-category-load-heal-user");
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

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");
    connection
        .execute(
            "INSERT INTO projects (name, project_category_id) VALUES (?1, ?2)",
            rusqlite::params!["Старый тип проекта", "missing"],
        )
        .expect("expected stale project category insert");
    drop(connection);

    let categories = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            load_project_category_list()
        })
    })
    .expect("expected project categories to load");
    assert_eq!(categories.len(), 2);

    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection after heal");
    let project_category_id: Option<String> = connection
        .query_row(
            "SELECT project_category_id FROM projects WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("expected healed project category id");
    assert_eq!(project_category_id, None);

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_dir(&bundle_dir);
    crate::test_support::cleanup_file(&user_runtime_config_path);
}
