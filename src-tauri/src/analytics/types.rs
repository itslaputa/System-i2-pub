use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDashboardData {
    pub start_date: String,
    pub end_date: String,
    pub total_minutes: i64,
    pub category_summaries: Vec<AnalyticsCategorySummaryNode>,
    pub project_summaries: Vec<AnalyticsProjectSummaryItem>,
    pub root_share_items: Vec<AnalyticsRootShareItem>,
    pub daily_series: Vec<AnalyticsDailyPoint>,
    pub commented_tasks: Vec<AnalyticsCommentedTaskItem>,
    pub daily_chart_kind: AnalyticsDailyChartKind,
    pub trend_comparison: AnalyticsTrendComparison,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDashboardInput {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsCategorySummaryNode {
    pub category_id: String,
    pub label: String,
    pub total_minutes: i64,
    pub task_count: i64,
    pub projects: Vec<AnalyticsProjectPeriodItem>,
    pub children: Vec<AnalyticsCategorySummaryNode>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsProjectPeriodItem {
    pub project_id: String,
    pub label: String,
    pub total_minutes: i64,
    pub finished_in_period: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsProjectSummaryItem {
    pub project_id: String,
    pub label: String,
    pub total_minutes: i64,
    pub task_count: i64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_done: bool,
    pub finished_in_period: bool,
    pub category_breakdown: Vec<AnalyticsProjectCategoryBreakdownItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsProjectCategoryBreakdownItem {
    pub category_id: String,
    pub label: String,
    pub total_minutes: i64,
    pub task_count: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsRootShareItem {
    pub category_id: String,
    pub label: String,
    pub total_minutes: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDailyPoint {
    pub date: String,
    pub total_minutes: i64,
    pub task_count: i64,
    pub category_breakdown: Vec<AnalyticsDailyCategorySlice>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDailyCategorySlice {
    pub category_id: String,
    pub label: String,
    pub total_minutes: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsCommentedTaskItem {
    pub task_id: String,
    pub date: String,
    pub category_label: String,
    pub project_label: Option<String>,
    pub time_length: i64,
    pub note: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsDailyChartKind {
    Bar,
    Line,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsTrendComparison {
    pub mode: AnalyticsTrendMode,
    pub total: AnalyticsTrendSeries,
    pub categories: Vec<AnalyticsTrendSeries>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsTrendMode {
    Week,
    Month,
    Year,
    Custom,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsTrendSeries {
    pub series_id: String,
    pub label: String,
    pub points: Vec<AnalyticsTrendPoint>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsTrendPoint {
    pub label: String,
    pub start_date: String,
    pub end_date: String,
    pub total_minutes: i64,
    pub is_current: bool,
}
