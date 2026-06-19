use super::{RuntimeStatus, RuntimeStatusSource};
use crate::categories::TaskCategoryNode;
use crate::project_categories::ProjectCategoryNode;
use crate::storage;
use crate::storage::runtime_bundle_paths::{self, ResolvedRuntimePaths};
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;

pub fn load_runtime_status() -> RuntimeStatus {
    match resolve_runtime_candidate() {
        RuntimeCandidate::Resolved(paths, details) => {
            validate_runtime_bundle_from_resolved_paths(paths, details)
        }
        RuntimeCandidate::Invalid(source, bundle_dir, invalid, details) => RuntimeStatus {
            is_ready: false,
            bundle_dir,
            source,
            missing: vec![],
            invalid,
            details,
        },
    }
}

enum RuntimeCandidate {
    Resolved(ResolvedRuntimePaths, Vec<String>),
    Invalid(
        RuntimeStatusSource,
        Option<String>,
        Vec<String>,
        Vec<String>,
    ),
}

fn resolve_runtime_candidate() -> RuntimeCandidate {
    match storage::resolve_user_runtime_config_path() {
        Ok(user_config_path) if user_config_path.exists() => {
            match runtime_bundle_paths::load_resolved_runtime_paths() {
                Ok(paths) => RuntimeCandidate::Resolved(
                    paths,
                    vec![format!(
                        "using user runtime config at {}",
                        user_config_path.display()
                    )],
                ),
                Err(error) => RuntimeCandidate::Invalid(
                    RuntimeStatusSource::UserConfig,
                    None,
                    vec![error],
                    vec![format!(
                        "user runtime config path: {}",
                        user_config_path.display()
                    )],
                ),
            }
        }
        Ok(_) => RuntimeCandidate::Invalid(
            RuntimeStatusSource::None,
            None,
            vec!["runtime bundle is not configured; complete first-run setup".to_string()],
            vec![],
        ),
        Err(error) => {
            RuntimeCandidate::Invalid(RuntimeStatusSource::None, None, vec![error], vec![])
        }
    }
}

pub(crate) fn load_runtime_status_for_bundle_dir(
    bundle_dir: &Path,
    source: RuntimeStatusSource,
    details: Vec<String>,
) -> RuntimeStatus {
    validate_runtime_bundle(
        ResolvedRuntimePaths {
            bundle_dir: bundle_dir.to_path_buf(),
            sqlite_db_path: bundle_dir.join(runtime_bundle_paths::SQLITE_DB_FILE_NAME),
            category_tree_path: bundle_dir.join(runtime_bundle_paths::CATEGORY_TREE_FILE_NAME),
            category_change_log_path: bundle_dir
                .join(runtime_bundle_paths::CATEGORY_CHANGE_LOG_FILE_NAME),
            project_category_list_path: bundle_dir
                .join(runtime_bundle_paths::PROJECT_CATEGORY_LIST_FILE_NAME),
        },
        details,
        source,
    )
}

fn validate_runtime_bundle_from_resolved_paths(
    paths: ResolvedRuntimePaths,
    details: Vec<String>,
) -> RuntimeStatus {
    validate_runtime_bundle(paths, details, RuntimeStatusSource::UserConfig)
}

fn validate_runtime_bundle(
    paths: ResolvedRuntimePaths,
    mut details: Vec<String>,
    source: RuntimeStatusSource,
) -> RuntimeStatus {
    let mut missing = Vec::<String>::new();
    let mut invalid = Vec::<String>::new();
    let bundle_dir = validate_bundle_dir(&paths, &mut invalid);

    validate_existing_file(
        &paths.sqlite_db_path,
        runtime_bundle_paths::SQLITE_DB_FILE_NAME,
        &mut missing,
    );
    validate_existing_file(
        &paths.category_tree_path,
        runtime_bundle_paths::CATEGORY_TREE_FILE_NAME,
        &mut missing,
    );
    validate_existing_file(
        &paths.category_change_log_path,
        runtime_bundle_paths::CATEGORY_CHANGE_LOG_FILE_NAME,
        &mut missing,
    );

    if paths.sqlite_db_path.exists() {
        validate_sqlite_database(&paths.sqlite_db_path, &mut invalid);
    }

    if paths.category_tree_path.exists() {
        validate_category_tree(&paths.category_tree_path, &mut invalid);
    }

    if paths.category_change_log_path.exists() {
        validate_change_log(&paths.category_change_log_path, &mut invalid);
    }

    if missing.is_empty() && invalid.is_empty() {
        ensure_project_category_list_file(&paths, &mut details, &mut invalid);
    }

    validate_existing_file(
        &paths.project_category_list_path,
        runtime_bundle_paths::PROJECT_CATEGORY_LIST_FILE_NAME,
        &mut missing,
    );

    if paths.project_category_list_path.exists() {
        validate_project_category_list(&paths.project_category_list_path, &mut invalid);
    }

    if let Some(bundle_dir) = &bundle_dir {
        details.push(format!("bundle dir: {}", bundle_dir));
    }

    RuntimeStatus {
        is_ready: missing.is_empty() && invalid.is_empty(),
        bundle_dir,
        source,
        missing,
        invalid,
        details,
    }
}

fn validate_bundle_dir(paths: &ResolvedRuntimePaths, invalid: &mut Vec<String>) -> Option<String> {
    let sqlite_dir = paths.sqlite_db_path.parent().map(Path::to_path_buf);
    let category_dir = paths.category_tree_path.parent().map(Path::to_path_buf);
    let log_dir = paths
        .category_change_log_path
        .parent()
        .map(Path::to_path_buf);
    let project_category_dir = paths
        .project_category_list_path
        .parent()
        .map(Path::to_path_buf);

    match (sqlite_dir, category_dir, log_dir, project_category_dir) {
        (Some(sqlite_dir), Some(category_dir), Some(log_dir), Some(project_category_dir))
            if sqlite_dir == category_dir
                && sqlite_dir == log_dir
                && sqlite_dir == project_category_dir =>
        {
            Some(sqlite_dir.display().to_string())
        }
        _ => {
            invalid.push("runtime bundle files must live in one folder".to_string());
            Some(paths.bundle_dir.display().to_string())
        }
    }
}

fn validate_existing_file(path: &Path, expected_file_name: &str, missing: &mut Vec<String>) {
    if !path.exists() {
        missing.push(expected_file_name.to_string());
    }
}

fn ensure_project_category_list_file(
    paths: &ResolvedRuntimePaths,
    details: &mut Vec<String>,
    invalid: &mut Vec<String>,
) {
    if paths.project_category_list_path.exists() {
        return;
    }

    match crate::project_categories::write_default_project_category_list(
        &paths.project_category_list_path,
    ) {
        Ok(()) => details.push(format!(
            "created default {}",
            runtime_bundle_paths::PROJECT_CATEGORY_LIST_FILE_NAME
        )),
        Err(error) => invalid.push(format!(
            "failed to create default project category list at {}: {error}",
            paths.project_category_list_path.display()
        )),
    }
}

fn validate_sqlite_database(path: &Path, invalid: &mut Vec<String>) {
    if let Err(error) = rusqlite::Connection::open(path) {
        invalid.push(format!(
            "failed to open sqlite database at {}: {error}",
            path.display()
        ));
    }
}

fn validate_category_tree(path: &Path, invalid: &mut Vec<String>) {
    let file_contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            invalid.push(format!(
                "failed to read category tree at {}: {error}",
                path.display()
            ));
            return;
        }
    };

    let parsed_tree = match serde_json::from_str::<Vec<TaskCategoryNode>>(&file_contents) {
        Ok(tree) => tree,
        Err(error) => {
            invalid.push(format!(
                "failed to parse category tree at {}: {error}",
                path.display()
            ));
            return;
        }
    };

    if let Err(error) = crate::categories::normalize_category_tree(&parsed_tree) {
        invalid.push(format!(
            "failed to validate category tree at {}: {error}",
            path.display()
        ));
    }
}

fn validate_change_log(path: &Path, invalid: &mut Vec<String>) {
    if let Err(error) = fs::read_to_string(path) {
        invalid.push(format!(
            "failed to read category change log at {}: {error}",
            path.display()
        ));
        return;
    }

    if let Err(error) = OpenOptions::new().append(true).open(path) {
        invalid.push(format!(
            "failed to append to category change log at {}: {error}",
            path.display()
        ));
    }
}

fn validate_project_category_list(path: &Path, invalid: &mut Vec<String>) {
    let file_contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            invalid.push(format!(
                "failed to read project category list at {}: {error}",
                path.display()
            ));
            return;
        }
    };

    let parsed_categories = match serde_json::from_str::<Vec<ProjectCategoryNode>>(&file_contents) {
        Ok(categories) => categories,
        Err(error) => {
            invalid.push(format!(
                "failed to parse project category list at {}: {error}",
                path.display()
            ));
            return;
        }
    };

    if let Err(error) =
        crate::project_categories::normalize_project_category_list(&parsed_categories)
    {
        invalid.push(format!(
            "failed to validate project category list at {}: {error}",
            path.display()
        ));
    }
}
