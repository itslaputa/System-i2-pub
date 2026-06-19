#![cfg(test)]

use super::super::bundle::create_runtime_bundle_backup_to_dir;
use super::{
    attach_existing_runtime_bundle, create_runtime_bundle_in_app_data_dir,
    create_runtime_bundle_in_folder, RuntimeBundleFolderInput, RuntimeStatus, RuntimeStatusSource,
};
use serde_json::json;
use std::fs;
use std::path::Path;

#[test]
fn deserializes_runtime_bundle_folder_input_from_camel_case_json() {
    let input = serde_json::from_value::<RuntimeBundleFolderInput>(json!({
        "bundleDir": "/tmp/system-i2-bundle"
    }))
    .expect("expected runtime bundle folder input");

    assert_eq!(input.bundle_dir, "/tmp/system-i2-bundle");
}

#[test]
fn serializes_runtime_status_using_camel_case_and_kebab_case_source() {
    let serialized = serde_json::to_value(RuntimeStatus {
        is_ready: true,
        bundle_dir: Some("/tmp/system-i2-bundle".to_string()),
        source: RuntimeStatusSource::UserConfig,
        missing: vec![],
        invalid: vec![],
        details: vec!["bundle is ready".to_string()],
    })
    .expect("expected runtime status to serialize");

    assert_eq!(
        serialized,
        json!({
            "isReady": true,
            "bundleDir": "/tmp/system-i2-bundle",
            "source": "user-config",
            "missing": [],
            "invalid": [],
            "details": ["bundle is ready"]
        })
    );
}

#[test]
fn creates_runtime_bundle_in_explicit_folder_and_persists_user_config() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-create-folder");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-create-folder-config");

    let status =
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            create_runtime_bundle_in_folder(RuntimeBundleFolderInput {
                bundle_dir: bundle_dir.display().to_string(),
            })
            .expect("expected runtime bundle creation")
        });

    assert!(status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);
    assert!(bundle_dir.join("tasks.sqlite3").exists());
    assert!(bundle_dir.join("task_categories.json").exists());
    assert!(bundle_dir.join("task_category_change_log.log").exists());
    assert!(bundle_dir.join("project_categories.json").exists());

    let user_config_contents =
        fs::read_to_string(&user_runtime_config_path).expect("expected user runtime config file");
    assert!(user_config_contents.contains(&bundle_dir.display().to_string()));

    cleanup_runtime_bundle_fixture(&bundle_dir, &user_runtime_config_path);
}

#[test]
fn attaches_existing_runtime_bundle_and_persists_user_config() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-attach-existing");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-attach-existing-config");
    crate::storage::init_database_at_path(&bundle_dir.join("tasks.sqlite3"))
        .expect("expected temp sqlite init");
    crate::test_support::seed_minimal_category_file(&bundle_dir.join("task_categories.json"));
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(bundle_dir.join("task_category_change_log.log"), "")
        .expect("expected change log file");

    let status =
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            attach_existing_runtime_bundle(RuntimeBundleFolderInput {
                bundle_dir: bundle_dir.display().to_string(),
            })
            .expect("expected existing runtime bundle attach")
        });

    assert!(status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);

    let user_config_contents =
        fs::read_to_string(&user_runtime_config_path).expect("expected user runtime config file");
    assert!(user_config_contents.contains(&bundle_dir.display().to_string()));

    cleanup_runtime_bundle_fixture(&bundle_dir, &user_runtime_config_path);
}

#[test]
fn creates_runtime_bundle_in_default_app_data_dir_under_overridden_config_parent() {
    let app_data_dir = crate::test_support::unique_temp_dir("runtime-app-data");
    let user_runtime_config_path = app_data_dir.join("runtime-bundle.json");

    let status =
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            create_runtime_bundle_in_app_data_dir()
                .expect("expected app data runtime bundle creation")
        });

    assert!(status.is_ready);
    assert_eq!(status.source, RuntimeStatusSource::UserConfig);
    assert_eq!(
        status.bundle_dir.as_deref(),
        Some(app_data_dir.to_str().unwrap())
    );
    assert!(app_data_dir.join("tasks.sqlite3").exists());
    assert!(app_data_dir.join("task_categories.json").exists());
    assert!(app_data_dir.join("task_category_change_log.log").exists());
    assert!(app_data_dir.join("project_categories.json").exists());
    assert!(user_runtime_config_path.exists());

    cleanup_runtime_bundle_fixture(&app_data_dir, &user_runtime_config_path);
}

#[test]
fn creates_zip_backup_for_current_runtime_bundle() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-backup-bundle");
    let backup_dir = crate::test_support::unique_temp_dir("runtime-backup-target");
    let user_runtime_config_path =
        crate::test_support::unique_temp_json_path("runtime-backup-config");

    crate::storage::init_database_at_path(&bundle_dir.join("tasks.sqlite3"))
        .expect("expected temp sqlite init");
    crate::test_support::seed_minimal_category_file(&bundle_dir.join("task_categories.json"));
    crate::test_support::seed_minimal_project_category_file(
        &bundle_dir.join("project_categories.json"),
    );
    fs::write(bundle_dir.join("task_category_change_log.log"), "")
        .expect("expected change log file");
    fs::write(bundle_dir.join("tasks.sqlite3-wal"), "wal data").expect("expected wal sidecar file");
    fs::write(
        &user_runtime_config_path,
        format!(
            "{{\"schemaVersion\":1,\"bundleDir\":\"{}\"}}",
            bundle_dir.display()
        ),
    )
    .expect("expected user runtime config");

    let result =
        crate::storage::with_test_user_runtime_config_path(&user_runtime_config_path, || {
            create_runtime_bundle_backup_to_dir(&backup_dir)
        })
        .expect("expected runtime backup");

    let backup_path = Path::new(&result.backup_path);
    assert!(backup_path.exists());
    assert_eq!(backup_path.parent(), Some(backup_dir.as_path()));

    let backup_bytes = fs::read(backup_path).expect("expected backup zip to read");
    assert!(backup_bytes.starts_with(&[0x50, 0x4b, 0x03, 0x04]));
    assert!(contains_bytes(&backup_bytes, b"/tasks.sqlite3"));
    assert!(contains_bytes(&backup_bytes, b"/task_categories.json"));
    assert!(contains_bytes(
        &backup_bytes,
        b"/task_category_change_log.log",
    ));
    assert!(contains_bytes(&backup_bytes, b"/project_categories.json"));
    assert!(contains_bytes(&backup_bytes, b"/tasks.sqlite3-wal"));

    cleanup_runtime_bundle_fixture(&bundle_dir, &user_runtime_config_path);
    crate::test_support::cleanup_dir(&backup_dir);
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn cleanup_runtime_bundle_fixture(bundle_dir: &Path, user_runtime_config_path: &Path) {
    crate::test_support::cleanup_dir(bundle_dir);
    crate::test_support::cleanup_file(user_runtime_config_path);
}
