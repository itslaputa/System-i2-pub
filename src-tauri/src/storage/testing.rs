#![cfg(test)]

use rusqlite::Connection;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

thread_local! {
    static TEST_DB_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static TEST_CATEGORY_TREE_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static TEST_USER_RUNTIME_CONFIG_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

pub fn open_app_database_connection_for_tests(database_path: &Path) -> Result<Connection, String> {
    super::connection::open_database(database_path)
}

pub fn with_test_database_path<T, F>(database_path: &Path, callback: F) -> T
where
    F: FnOnce() -> T,
{
    TEST_DB_OVERRIDE.with(|slot| {
        let previous_value = slot.replace(Some(database_path.to_path_buf()));
        let result = callback();
        slot.replace(previous_value);
        result
    })
}

pub fn with_test_category_tree_path<T, F>(category_tree_path: &Path, callback: F) -> T
where
    F: FnOnce() -> T,
{
    TEST_CATEGORY_TREE_OVERRIDE.with(|slot| {
        let previous_value = slot.replace(Some(category_tree_path.to_path_buf()));
        let result = callback();
        slot.replace(previous_value);
        result
    })
}

pub fn with_test_user_runtime_config_path<T, F>(config_path: &Path, callback: F) -> T
where
    F: FnOnce() -> T,
{
    TEST_USER_RUNTIME_CONFIG_OVERRIDE.with(|slot| {
        let previous_value = slot.replace(Some(config_path.to_path_buf()));
        let result = callback();
        slot.replace(previous_value);
        result
    })
}

pub(crate) fn load_test_database_override_path() -> Option<PathBuf> {
    TEST_DB_OVERRIDE.with(|slot| slot.borrow().clone())
}

pub(crate) fn load_test_category_tree_override_path() -> Option<PathBuf> {
    TEST_CATEGORY_TREE_OVERRIDE.with(|slot| slot.borrow().clone())
}

pub(crate) fn load_test_user_runtime_config_override_path() -> Option<PathBuf> {
    TEST_USER_RUNTIME_CONFIG_OVERRIDE.with(|slot| slot.borrow().clone())
}
