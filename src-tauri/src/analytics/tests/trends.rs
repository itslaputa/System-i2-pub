#![cfg(test)]

use super::periods::build_trend_periods_for_reference_date;
use super::{load_dashboard, AnalyticsDashboardInput, AnalyticsTrendMode};
use crate::projects::repository::insert_task_project_into_connection;

#[test]
fn current_calendar_month_selection_compares_against_previous_full_months() {
    let connection = rusqlite::Connection::open_in_memory().expect("expected in-memory sqlite");

    let (mode, periods) = build_trend_periods_for_reference_date(
        &connection,
        "2026-03-01",
        "2026-03-31",
        "2026-03-13",
    )
    .expect("expected trend periods");

    assert_eq!(mode, AnalyticsTrendMode::Month);
    assert_eq!(periods.len(), 6);
    assert_eq!(periods[0].start_date, "2025-10-01");
    assert_eq!(periods[0].end_date, "2025-10-31");
    assert_eq!(periods[4].start_date, "2026-02-01");
    assert_eq!(periods[4].end_date, "2026-02-28");
    assert_eq!(periods[5].start_date, "2026-03-01");
    assert_eq!(periods[5].end_date, "2026-03-13");
    assert!(periods[5].is_current);
}

#[test]
fn dashboard_builds_total_and_root_category_trend_series() {
    let database_path = crate::test_support::unique_temp_database_path("analytics-trends");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories-trends");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let project_id = insert_task_project_into_connection(&connection, "System-I2", None)
        .expect("expected project insert");

    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, 0, NULL)",
        rusqlite::params!["logic-math", 120_i64, "2026-03-10", project_id],
    ).expect("expected current-period task insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, 0, NULL)",
        rusqlite::params!["logic-math", 90_i64, "2026-03-03", project_id],
    ).expect("expected previous-period task insert");
    drop(connection);

    let dashboard = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            load_dashboard(AnalyticsDashboardInput {
                start_date: "2026-03-09".to_string(),
                end_date: "2026-03-15".to_string(),
            })
        })
    })
    .expect("expected analytics dashboard");

    assert_eq!(dashboard.trend_comparison.mode, AnalyticsTrendMode::Week);
    assert_eq!(dashboard.trend_comparison.total.points.len(), 6);
    assert_eq!(
        dashboard
            .trend_comparison
            .total
            .points
            .last()
            .map(|point| point.total_minutes),
        Some(120),
    );
    assert_eq!(
        dashboard
            .trend_comparison
            .categories
            .iter()
            .find(|series| series.label == "Логика")
            .and_then(|series| series.points.last())
            .map(|point| point.total_minutes),
        Some(120),
    );

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}
