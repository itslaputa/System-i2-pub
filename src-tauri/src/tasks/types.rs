use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub category_id: String,
    pub time_length: i64,
    pub date: String,
    pub note: Option<String>,
    pub project_id: Option<String>,
    pub project_label: Option<String>,
    pub is_project_closing_task: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskInput {
    pub category_id: String,
    pub project_id: Option<String>,
    pub task_date: String,
    pub duration_minutes: i64,
    pub is_project_closing_task: bool,
    pub note: Option<String>,
}
