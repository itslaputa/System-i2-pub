mod io;
mod normalize;
mod service;
mod template;
mod types;

pub(crate) use normalize::normalize_project_category_list;
pub use service::{load_project_category_list, save_project_category_list};
pub(crate) use service::{set_project_category_id, validate_project_category_id_exists};
pub(crate) use template::write_default_project_category_list;
pub use types::ProjectCategoryNode;

#[cfg(test)]
mod tests;
