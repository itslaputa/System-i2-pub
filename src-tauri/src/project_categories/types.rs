use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectCategoryNode {
    pub id: String,
    pub label: String,
    pub order: usize,
}
