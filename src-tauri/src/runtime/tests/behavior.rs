#![cfg(test)]

use super::{load_runtime_status, RuntimeStatusSource};
use std::fs;
use std::path::Path;

#[test]
fn reports_ready_user_config_bundle() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-ready");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-ready-user-config");
    write_user_runtime_config(&user_runtime_config_path, &bundle_dir);
    seed_ready_bundle(&bundle_dir);

    let status = crate::storage::with_test_user_runtime_config_path(
        &user_runtime_config_path,
        load_runtime_status,
    );

    assert!(status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);
    assert_eq!(
        status.bundle_dir.as_deref(),
        Some(bundle_dir.to_str().unwrap())
    );
    assert_eq!(status.missing, Vec::<String>::new());
    assert_eq!(status.invalid, Vec::<String>::new());

    cleanup_runtime_fixture(&bundle_dir, &user_runtime_config_path);
}

#[test]
fn reports_not_configured_when_user_config_is_missing() {
    let missing_user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-missing-user-config");

    let status = crate::storage::with_test_user_runtime_config_path(
        &missing_user_runtime_config_path,
        load_runtime_status,
    );

    assert!(!status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::None);
    assert!(status.missing.is_empty());
    assert!(status
        .invalid
        .contains(&"runtime bundle is not configured; complete first-run setup".to_string()));
}

#[test]
fn upgrades_ready_legacy_bundle_with_default_project_categories() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-upgrade-project-categories");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-upgrade-user-config");
    write_user_runtime_config(&user_runtime_config_path, &bundle_dir);
    crate::storage::init_database_at_path(&bundle_dir.join("tasks.sqlite3"))
        .expect("expected temp sqlite init");
    crate::test_support::seed_minimal_category_file(&bundle_dir.join("task_categories.json"));
    fs::write(bundle_dir.join("task_category_change_log.log"), "").expect("expected temp log file");

    let status = crate::storage::with_test_user_runtime_config_path(
        &user_runtime_config_path,
        load_runtime_status,
    );

    assert!(status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);
    assert!(bundle_dir.join("project_categories.json").exists());
    assert!(status
        .details
        .iter()
        .any(|entry| entry.contains("created default project_categories.json")));

    cleanup_runtime_fixture(&bundle_dir, &user_runtime_config_path);
}

#[test]
fn reports_missing_bundle_files() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-missing");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-missing-user-config");
    write_user_runtime_config(&user_runtime_config_path, &bundle_dir);

    let status = crate::storage::with_test_user_runtime_config_path(
        &user_runtime_config_path,
        load_runtime_status,
    );

    assert!(!status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);
    assert!(status.missing.contains(&"tasks.sqlite3".to_string()));
    assert!(status.missing.contains(&"task_categories.json".to_string()));
    assert!(status
        .missing
        .contains(&"task_category_change_log.log".to_string()));
    assert!(status
        .missing
        .contains(&"project_categories.json".to_string()));

    cleanup_runtime_fixture(&bundle_dir, &user_runtime_config_path);
}

#[test]
fn reports_invalid_category_json() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-bad-json");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-bad-json-user-config");
    write_user_runtime_config(&user_runtime_config_path, &bundle_dir);
    crate::storage::init_database_at_path(&bundle_dir.join("tasks.sqlite3"))
        .expect("expected temp sqlite init");
    fs::write(bundle_dir.join("task_categories.json"), "{not-json")
        .expect("expected broken category file");
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(bundle_dir.join("task_category_change_log.log"), "").expect("expected temp log file");

    let status = crate::storage::with_test_user_runtime_config_path(
        &user_runtime_config_path,
        load_runtime_status,
    );

    assert!(!status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);
    assert!(status
        .invalid
        .iter()
        .any(|entry| entry.contains("failed to parse category tree")));

    cleanup_runtime_fixture(&bundle_dir, &user_runtime_config_path);
}

#[test]
fn reports_invalid_category_tree_when_json_has_duplicate_ids() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-duplicate-categories");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-duplicate-categories-user-config");
    write_user_runtime_config(&user_runtime_config_path, &bundle_dir);
    crate::storage::init_database_at_path(&bundle_dir.join("tasks.sqlite3"))
        .expect("expected temp sqlite init");
    fs::write(
        bundle_dir.join("task_categories.json"),
        r#"[
  { "id": "logic", "label": "Логика" },
  { "id": "logic", "label": "Дубликат" }
]
"#,
    )
    .expect("expected duplicate category file");
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(bundle_dir.join("task_category_change_log.log"), "").expect("expected temp log file");

    let status = crate::storage::with_test_user_runtime_config_path(
        &user_runtime_config_path,
        load_runtime_status,
    );

    assert!(!status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);
    assert!(status
        .invalid
        .iter()
        .any(|entry| entry.contains("failed to validate category tree")));
    assert!(status
        .invalid
        .iter()
        .any(|entry| entry.contains("duplicate category id 'logic'")));

    cleanup_runtime_fixture(&bundle_dir, &user_runtime_config_path);
}

#[test]
fn reports_invalid_project_category_json() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-bad-project-categories");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-bad-project-categories-user-config");
    write_user_runtime_config(&user_runtime_config_path, &bundle_dir);
    crate::storage::init_database_at_path(&bundle_dir.join("tasks.sqlite3"))
        .expect("expected temp sqlite init");
    crate::test_support::seed_minimal_category_file(&bundle_dir.join("task_categories.json"));
    fs::write(bundle_dir.join("project_categories.json"), "{not-json")
        .expect("expected broken project category file");
    fs::write(bundle_dir.join("task_category_change_log.log"), "").expect("expected temp log file");

    let status = crate::storage::with_test_user_runtime_config_path(
        &user_runtime_config_path,
        load_runtime_status,
    );

    assert!(!status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);
    assert!(status
        .invalid
        .iter()
        .any(|entry| entry.contains("failed to parse project category list")));

    cleanup_runtime_fixture(&bundle_dir, &user_runtime_config_path);
}

fn seed_ready_bundle(bundle_dir: &Path) {
    crate::storage::init_database_at_path(&bundle_dir.join("tasks.sqlite3"))
        .expect("expected temp sqlite init");
    crate::test_support::seed_minimal_category_file(&bundle_dir.join("task_categories.json"));
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(bundle_dir.join("task_category_change_log.log"), "").expect("expected temp log file");
}

fn write_user_runtime_config(config_path: &Path, bundle_dir: &Path) {
    fs::write(
        config_path,
        format!(
            "{{\"schemaVersion\":1,\"bundleDir\":\"{}\"}}",
            bundle_dir.display()
        ),
    )
    .expect("expected user runtime config file");
}

fn cleanup_runtime_fixture(bundle_dir: &Path, user_runtime_config_path: &Path) {
    crate::test_support::cleanup_dir(bundle_dir);
    crate::test_support::cleanup_file(user_runtime_config_path);
}
