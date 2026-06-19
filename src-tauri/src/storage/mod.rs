mod connection;
pub mod runtime_bundle_paths;
mod runtime_user_config;
mod schema_management;
#[cfg(test)]
mod testing;
#[cfg(test)]
mod tests;
mod validation;

pub(crate) use connection::resolve_database_path;
pub use connection::{init_database, init_database_at_path, open_app_database_connection};
pub(crate) use runtime_user_config::{
    resolve_default_runtime_bundle_dir, resolve_user_runtime_config_path, save_user_runtime_config,
};
pub use validation::{
    normalize_iso_date, normalize_non_empty_text, normalize_optional_text, parse_optional_i64_id,
    validate_category_id_exists,
};

#[cfg(test)]
pub(crate) use testing::load_test_category_tree_override_path;
#[cfg(test)]
pub(crate) use testing::load_test_user_runtime_config_override_path;
#[cfg(test)]
pub use testing::{
    open_app_database_connection_for_tests, with_test_category_tree_path, with_test_database_path,
    with_test_user_runtime_config_path,
};
