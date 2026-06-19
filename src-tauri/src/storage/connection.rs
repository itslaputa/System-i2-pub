use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

pub fn init_database() -> Result<(), String> {
    let database_path = resolve_database_path()?;
    init_database_at_path(&database_path)
}

pub fn init_database_at_path(database_path: &Path) -> Result<(), String> {
    if let Some(parent_dir) = database_path.parent() {
        fs::create_dir_all(parent_dir).map_err(|error| {
            format!(
                "failed to create database directory {}: {error}",
                parent_dir.display()
            )
        })?;
    }

    let connection = open_database(database_path)?;
    super::schema_management::initialize_database_schema(&connection, database_path)?;

    println!("SQLite database initialized at {}", database_path.display());
    Ok(())
}

pub fn open_app_database_connection() -> Result<Connection, String> {
    let database_path = resolve_database_path()?;
    open_database(&database_path)
}

pub(crate) fn open_database(database_path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(database_path).map_err(|error| {
        format!(
            "failed to open sqlite database at {}: {error}",
            database_path.display()
        )
    })?;

    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| {
            format!(
                "failed to configure sqlite pragmas at {}: {error}",
                database_path.display()
            )
        })?;

    Ok(connection)
}

pub(crate) fn resolve_database_path() -> Result<PathBuf, String> {
    #[cfg(test)]
    {
        if let Some(override_path) = super::testing::load_test_database_override_path() {
            return Ok(override_path);
        }
    }

    super::runtime_bundle_paths::load_sqlite_db_path()
}
