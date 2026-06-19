use super::ProjectCategoryNode;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::Path;

pub fn load_project_category_list() -> Result<Vec<ProjectCategoryNode>, String> {
    let categories = super::io::load_project_category_list()?;

    if crate::storage::resolve_database_path().is_ok_and(|database_path| database_path.exists()) {
        let connection = crate::storage::open_app_database_connection()?;
        remap_projects_with_unknown_category_ids(&connection, &categories)?;
    }

    Ok(categories)
}

pub fn save_project_category_list(categories: Vec<ProjectCategoryNode>) -> Result<(), String> {
    let project_category_list_path =
        crate::storage::runtime_bundle_paths::load_project_category_list_path()?;
    let mut connection = crate::storage::open_app_database_connection()?;

    save_project_category_list_to_path(&mut connection, &project_category_list_path, categories)
}

pub(crate) fn validate_project_category_id_exists(category_id: &str) -> Result<(), String> {
    let normalized_category_id =
        crate::storage::normalize_non_empty_text(category_id, "project_category_id")?;
    let categories = load_project_category_list()?;

    if categories
        .iter()
        .any(|category| category.id == normalized_category_id)
    {
        return Ok(());
    }

    Err(format!(
        "project_category_id '{normalized_category_id}' does not exist"
    ))
}

pub(crate) fn save_project_category_list_to_path(
    connection: &mut Connection,
    path: &Path,
    categories: Vec<ProjectCategoryNode>,
) -> Result<(), String> {
    let path = path
        .to_str()
        .ok_or_else(|| format!("path '{}' is not valid utf-8", path.display()))?;
    let previous_categories = super::io::load_project_category_list_from_path(path)?;
    let normalized_categories = super::normalize::normalize_project_category_list(&categories)?;
    let next_ids = normalized_categories
        .iter()
        .map(|category| category.id.as_str())
        .collect::<HashSet<_>>();
    let removed_ids = previous_categories
        .iter()
        .filter_map(|category| {
            if next_ids.contains(category.id.as_str()) {
                None
            } else {
                Some(category.id.clone())
            }
        })
        .collect::<Vec<_>>();

    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start project category save transaction: {error}"))?;

    if !removed_ids.is_empty() {
        let mut statement = transaction
            .prepare(
                "UPDATE projects
                 SET project_category_id = NULL
                 WHERE project_category_id = ?1",
            )
            .map_err(|error| {
                format!("failed to prepare project category remap statement: {error}")
            })?;

        for removed_id in &removed_ids {
            statement
                .execute(params![removed_id])
                .map_err(|error| {
                    format!(
                        "failed to remap projects away from removed project category '{removed_id}': {error}"
                    )
                })?;
        }
    }
    remap_projects_with_unknown_category_ids(&transaction, &normalized_categories)?;

    super::io::save_project_category_list_to_path(path, &normalized_categories)?;
    transaction
        .commit()
        .map_err(|error| format!("failed to commit project category save transaction: {error}"))
}

pub(crate) fn set_project_category_id(
    connection: &Connection,
    project_id: i64,
    project_category_id: Option<&str>,
) -> Result<(), String> {
    let updated_rows = connection
        .execute(
            "UPDATE projects
             SET project_category_id = ?2
             WHERE id = ?1",
            params![project_id, project_category_id],
        )
        .map_err(|error| {
            format!("failed to update project category for project {project_id}: {error}")
        })?;

    if updated_rows == 0 {
        return Err(format!("project id {project_id} does not exist"));
    }

    Ok(())
}

fn remap_projects_with_unknown_category_ids(
    connection: &Connection,
    categories: &[ProjectCategoryNode],
) -> Result<(), String> {
    let projects_table_exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'projects')",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to inspect projects table for category remap: {error}"))?
        != 0;

    if !projects_table_exists {
        return Ok(());
    }

    let known_ids = categories
        .iter()
        .map(|category| category.id.as_str())
        .collect::<HashSet<_>>();
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT project_category_id
             FROM projects
             WHERE project_category_id IS NOT NULL",
        )
        .map_err(|error| format!("failed to prepare project category scan: {error}"))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("failed to scan project category ids: {error}"))?;
    let stale_ids = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect project category ids: {error}"))?
        .into_iter()
        .filter(|category_id| !known_ids.contains(category_id.as_str()))
        .collect::<Vec<_>>();
    drop(statement);

    for stale_id in stale_ids {
        connection
            .execute(
                "UPDATE projects
                 SET project_category_id = NULL
                 WHERE project_category_id = ?1",
                params![stale_id.as_str()],
            )
            .map_err(|error| {
                format!("failed to remap unknown project category '{stale_id}': {error}")
            })?;
    }

    Ok(())
}
