#![cfg(test)]

use super::{
    AnalyticsDailyCategorySlice, AnalyticsDailyChartKind, AnalyticsDailyPoint,
    AnalyticsDashboardData, AnalyticsDashboardInput, AnalyticsProjectCategoryBreakdownItem,
    AnalyticsProjectSummaryItem, AnalyticsTrendComparison, AnalyticsTrendMode, AnalyticsTrendPoint,
    AnalyticsTrendSeries,
};
use serde_json::json;

#[test]
fn deserializes_dashboard_input_from_camel_case_json() {
    let deserialized = serde_json::from_value::<AnalyticsDashboardInput>(json!({
        "startDate": "2026-03-12",
        "endDate": "2026-03-13"
    }))
    .expect("expected analytics dashboard input to deserialize");

    assert_eq!(deserialized.start_date, "2026-03-12");
    assert_eq!(deserialized.end_date, "2026-03-13");
}

#[test]
fn serializes_dashboard_payload_using_camel_case_keys() {
    let serialized = serde_json::to_value(AnalyticsDashboardData {
        start_date: "2026-03-12".to_string(),
        end_date: "2026-03-13".to_string(),
        total_minutes: 685,
        category_summaries: vec![],
        project_summaries: vec![AnalyticsProjectSummaryItem {
            project_id: "5".to_string(),
            label: "System-I2".to_string(),
            total_minutes: 45,
            task_count: 1,
            start_date: Some("2026-03-12".to_string()),
            end_date: Some("2026-03-12".to_string()),
            is_done: false,
            finished_in_period: false,
            category_breakdown: vec![AnalyticsProjectCategoryBreakdownItem {
                category_id: "logic".to_string(),
                label: "Логика".to_string(),
                total_minutes: 45,
                task_count: 1,
            }],
        }],
        root_share_items: vec![],
        daily_series: vec![AnalyticsDailyPoint {
            date: "2026-03-12".to_string(),
            total_minutes: 45,
            task_count: 2,
            category_breakdown: vec![AnalyticsDailyCategorySlice {
                category_id: "logic".to_string(),
                label: "Логика".to_string(),
                total_minutes: 45,
            }],
        }],
        commented_tasks: vec![],
        daily_chart_kind: AnalyticsDailyChartKind::Bar,
        trend_comparison: AnalyticsTrendComparison {
            mode: AnalyticsTrendMode::Week,
            total: AnalyticsTrendSeries {
                series_id: "total".to_string(),
                label: "Суммарное время".to_string(),
                points: vec![AnalyticsTrendPoint {
                    label: "10 мар-16 мар".to_string(),
                    start_date: "2026-03-10".to_string(),
                    end_date: "2026-03-16".to_string(),
                    total_minutes: 685,
                    is_current: true,
                }],
            },
            categories: vec![],
        },
    })
    .expect("expected analytics dashboard to serialize");

    assert_eq!(
        serialized,
        json!({
            "startDate": "2026-03-12",
            "endDate": "2026-03-13",
            "totalMinutes": 685,
            "categorySummaries": [],
            "projectSummaries": [{
                "projectId": "5",
                "label": "System-I2",
                "totalMinutes": 45,
                "taskCount": 1,
                "startDate": "2026-03-12",
                "endDate": "2026-03-12",
                "isDone": false,
                "finishedInPeriod": false,
                "categoryBreakdown": [{
                    "categoryId": "logic",
                    "label": "Логика",
                    "totalMinutes": 45,
                    "taskCount": 1
                }]
            }],
            "rootShareItems": [],
            "dailySeries": [{
                "date": "2026-03-12",
                "totalMinutes": 45,
                "taskCount": 2,
                "categoryBreakdown": [{
                    "categoryId": "logic",
                    "label": "Логика",
                    "totalMinutes": 45
                }]
            }],
            "commentedTasks": [],
            "dailyChartKind": "bar",
            "trendComparison": {
                "mode": "week",
                "total": {
                    "seriesId": "total",
                    "label": "Суммарное время",
                    "points": [{
                        "label": "10 мар-16 мар",
                        "startDate": "2026-03-10",
                        "endDate": "2026-03-16",
                        "totalMinutes": 685,
                        "isCurrent": true
                    }]
                },
                "categories": []
            }
        })
    );
}
