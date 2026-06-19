mod periods;
mod repository;
mod service;
#[cfg(test)]
mod tests;
mod tree;
mod trends;
mod types;

pub use service::load_dashboard;
pub use types::{
    AnalyticsCategorySummaryNode, AnalyticsCommentedTaskItem, AnalyticsDailyCategorySlice,
    AnalyticsDailyChartKind, AnalyticsDailyPoint, AnalyticsDashboardData, AnalyticsDashboardInput,
    AnalyticsProjectCategoryBreakdownItem, AnalyticsProjectPeriodItem, AnalyticsProjectSummaryItem,
    AnalyticsRootShareItem, AnalyticsTrendComparison, AnalyticsTrendMode, AnalyticsTrendPoint,
    AnalyticsTrendSeries,
};
