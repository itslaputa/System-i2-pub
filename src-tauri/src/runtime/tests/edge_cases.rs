#![cfg(test)]

use super::super::bundle::create_runtime_bundle_in_folder;
use super::{RuntimeBundleFolderInput, RuntimeStatusSource};
use std::fs;

#[test]
fn rejects_runtime_bundle_creation_when_target_files_already_exist() {
    let bundle_dir = crate::test_support::unique_temp_dir("runtime-existing-file");
    fs::write(bundle_dir.join("tasks.sqlite3"), "occupied").expect("expected preexisting file");

    let error = create_runtime_bundle_in_folder(RuntimeBundleFolderInput {
        bundle_dir: bundle_dir.display().to_string(),
    })
    .expect_err("expected bundle creation to reject occupied dir");

    assert!(error.contains("refusing to create runtime bundle"));
    crate::test_support::cleanup_dir(&bundle_dir);
}

#[test]
fn deserializes_none_runtime_status_source() {
    let serialized = serde_json::to_value(RuntimeStatusSource::None)
        .expect("expected runtime source to serialize");

    assert_eq!(serialized, serde_json::json!("none"));
}
