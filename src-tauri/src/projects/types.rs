use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProjectNode {
    pub id: String,
    pub label: String,
    pub project_category_id: Option<String>,
    pub sum_time_length: i64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_done: bool,
    pub tasks: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskProjectInput {
    pub label: String,
    pub project_category_id: Option<String>,
}
