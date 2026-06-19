#![cfg(test)]

use super::{load_dashboard, AnalyticsDashboardInput};
use crate::projects::repository::insert_task_project_into_connection;
use std::fs;

#[test]
fn builds_dashboard_totals_tree_root_share_and_daily_series() {
    let database_path = crate::test_support::unique_temp_database_path("analytics");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let project_id = insert_task_project_into_connection(&connection, "Юнг - АИКБ", None)
        .expect("expected project insert");

    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params!["logic-math", 45_i64, "2026-03-12", project_id, 0_i64, "Теория графов"],
    ).expect("expected first task insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params!["logic-math", 30_i64, "2026-03-13", project_id, 1_i64, "Финальная практика"],
    ).expect("expected second task insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, NULL, 0, NULL)",
        rusqlite::params!["mind-reading", 20_i64, "2026-03-13"],
    ).expect("expected third task insert");
    drop(connection);

    let dashboard = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            load_dashboard(AnalyticsDashboardInput {
                start_date: "2026-03-12".to_string(),
                end_date: "2026-03-14".to_string(),
            })
        })
    })
    .expect("expected analytics dashboard");

    assert_eq!(dashboard.total_minutes, 95);
    assert_eq!(
        dashboard.daily_chart_kind,
        super::AnalyticsDailyChartKind::Bar
    );
    assert_eq!(dashboard.daily_series.len(), 3);
    assert_eq!(dashboard.daily_series[0].total_minutes, 45);
    assert_eq!(dashboard.daily_series[1].total_minutes, 50);
    assert_eq!(dashboard.daily_series[2].total_minutes, 0);
    assert_eq!(dashboard.daily_series[2].task_count, 0);
    assert_eq!(dashboard.daily_series[0].category_breakdown.len(), 1);
    assert_eq!(
        dashboard.daily_series[0].category_breakdown[0].label,
        "Логика"
    );
    assert_eq!(
        dashboard.daily_series[0].category_breakdown[0].total_minutes,
        45
    );
    assert_eq!(dashboard.daily_series[1].category_breakdown.len(), 2);
    assert_eq!(
        dashboard.daily_series[1].category_breakdown[0].label,
        "Логика"
    );
    assert_eq!(
        dashboard.daily_series[1].category_breakdown[0].total_minutes,
        30
    );
    assert_eq!(
        dashboard.daily_series[1].category_breakdown[1].label,
        "Психика"
    );
    assert_eq!(
        dashboard.daily_series[1].category_breakdown[1].total_minutes,
        20
    );
    assert!(dashboard.daily_series[2].category_breakdown.is_empty());
    assert_eq!(dashboard.commented_tasks.len(), 2);
    assert_eq!(dashboard.commented_tasks[0].note, "Финальная практика");
    assert_eq!(dashboard.commented_tasks[0].category_label, "Математика");
    assert_eq!(
        dashboard.commented_tasks[0].project_label.as_deref(),
        Some("Юнг - АИКБ")
    );
    assert_eq!(dashboard.commented_tasks[1].note, "Теория графов");
    assert_eq!(dashboard.project_summaries.len(), 1);
    assert_eq!(dashboard.project_summaries[0].label, "Юнг - АИКБ");
    assert_eq!(dashboard.project_summaries[0].total_minutes, 75);
    assert_eq!(dashboard.project_summaries[0].task_count, 2);
    assert!(dashboard.project_summaries[0].is_done);
    assert!(dashboard.project_summaries[0].finished_in_period);
    assert_eq!(
        dashboard.project_summaries[0].category_breakdown[0].label,
        "Математика"
    );

    assert_eq!(dashboard.root_share_items.len(), 2);
    assert_eq!(dashboard.root_share_items[0].label, "Логика");
    assert_eq!(dashboard.root_share_items[0].total_minutes, 75);
    assert_eq!(dashboard.root_share_items[1].label, "Психика");
    assert_eq!(dashboard.root_share_items[1].total_minutes, 20);

    assert_eq!(dashboard.category_summaries.len(), 2);
    let logic_root = &dashboard.category_summaries[0];
    assert_eq!(logic_root.label, "Логика");
    assert_eq!(logic_root.total_minutes, 75);
    assert_eq!(logic_root.task_count, 2);
    assert_eq!(logic_root.projects.len(), 1);
    assert_eq!(logic_root.projects[0].label, "Юнг - АИКБ");
    assert!(logic_root.projects[0].finished_in_period);
    assert_eq!(logic_root.children.len(), 1);
    assert_eq!(logic_root.children[0].label, "Математика");
    assert_eq!(logic_root.children[0].projects.len(), 1);

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn builds_global_project_summary_across_task_categories() {
    let database_path = crate::test_support::unique_temp_database_path("analytics-global-project");
    let category_file_path =
        crate::test_support::unique_temp_json_path("task-categories-global-project");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    let project_id = insert_task_project_into_connection(&connection, "System-I2", None)
        .expect("expected project insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, 0, NULL)",
        rusqlite::params!["logic-math", 60_i64, "2026-03-12", project_id],
    ).expect("expected logic task insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, 0, NULL)",
        rusqlite::params!["mind-reading", 30_i64, "2026-03-13", project_id],
    ).expect("expected mind task insert");
    drop(connection);

    let dashboard = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            load_dashboard(AnalyticsDashboardInput {
                start_date: "2026-03-12".to_string(),
                end_date: "2026-03-13".to_string(),
            })
        })
    })
    .expect("expected analytics dashboard");

    assert_eq!(dashboard.project_summaries.len(), 1);
    assert_eq!(
        dashboard.project_summaries[0].project_id,
        project_id.to_string()
    );
    assert_eq!(dashboard.project_summaries[0].total_minutes, 90);
    assert_eq!(dashboard.project_summaries[0].category_breakdown.len(), 2);
    assert_eq!(
        dashboard.project_summaries[0].category_breakdown[0].label,
        "Математика"
    );
    assert_eq!(
        dashboard.project_summaries[0].category_breakdown[0].total_minutes,
        60
    );
    assert_eq!(
        dashboard.project_summaries[0].category_breakdown[1].label,
        "Читал"
    );
    assert_eq!(
        dashboard.project_summaries[0].category_breakdown[1].total_minutes,
        30
    );

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn uses_line_chart_kind_for_ranges_above_one_month() {
    let database_path = crate::test_support::unique_temp_database_path("analytics");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories");
    crate::test_support::seed_minimal_category_file(&category_file_path);
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");

    let dashboard = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            load_dashboard(AnalyticsDashboardInput {
                start_date: "2026-01-01".to_string(),
                end_date: "2026-02-10".to_string(),
            })
        })
    })
    .expect("expected analytics dashboard");

    assert_eq!(
        dashboard.daily_chart_kind,
        super::AnalyticsDailyChartKind::Line
    );
    assert_eq!(dashboard.daily_series.len(), 41);
    assert_eq!(dashboard.total_minutes, 0);

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn rejects_reversed_ranges() {
    let error = load_dashboard(AnalyticsDashboardInput {
        start_date: "2026-03-14".to_string(),
        end_date: "2026-03-12".to_string(),
    })
    .expect_err("expected reversed range to fail");

    assert!(error.contains("start_date cannot be later than end_date"));
}

#[test]
fn keeps_root_categories_in_fixed_business_order() {
    let database_path = crate::test_support::unique_temp_database_path("analytics-order");
    let category_file_path = crate::test_support::unique_temp_json_path("task-categories-order");
    fs::write(
        &category_file_path,
        r#"[
  { "id": "system", "label": "Система" },
  { "id": "relationships", "label": "Отношения" },
  { "id": "money", "label": "Деньги" },
  { "id": "health", "label": "Здоровье." },
  { "id": "logic", "label": "Логика" },
  { "id": "routine", "label": "Рутина" },
  { "id": "mind", "label": "Психика" }
]
"#,
    )
    .expect("expected temp category file to be writable");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");

    for (category_id, minutes) in [
        ("system", 10_i64),
        ("relationships", 20_i64),
        ("money", 30_i64),
        ("health", 40_i64),
        ("logic", 50_i64),
        ("routine", 60_i64),
        ("mind", 70_i64),
    ] {
        connection
            .execute(
                "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, NULL, 0, NULL)",
                rusqlite::params![category_id, minutes, "2026-03-13"],
            )
            .expect("expected task insert");
    }
    drop(connection);

    let dashboard = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            load_dashboard(AnalyticsDashboardInput {
                start_date: "2026-03-13".to_string(),
                end_date: "2026-03-13".to_string(),
            })
        })
    })
    .expect("expected analytics dashboard");

    let labels = dashboard
        .category_summaries
        .iter()
        .map(|node| node.label.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "Деньги",
            "Здоровье.",
            "Логика",
            "Психика",
            "Отношения",
            "Рутина",
            "Система",
        ]
    );

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn keeps_known_health_category_and_buckets_only_unknown_categories_as_other() {
    let database_path =
        crate::test_support::unique_temp_database_path("analytics-unknown-category");
    let category_file_path =
        crate::test_support::unique_temp_json_path("task-categories-unknown-category");
    fs::write(
        &category_file_path,
        r#"[
  {
    "id": "health",
    "label": "Здоровье.",
    "children": [{ "id": "health-reading", "label": "Читал про здоровье" }]
  }
]
"#,
    )
    .expect("expected temp category file to be writable");
    crate::storage::init_database_at_path(&database_path).expect("expected temp sqlite init");
    let connection = crate::storage::open_app_database_connection_for_tests(&database_path)
        .expect("expected temp sqlite connection");
    let project_id = insert_task_project_into_connection(&connection, "Health Project", None)
        .expect("expected project insert");

    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, 0, NULL)",
        rusqlite::params!["health-reading", 40_i64, "2026-03-13", project_id],
    ).expect("expected known health task insert");
    connection.execute(
        "INSERT INTO tasks (category_id, time_length, date, project_id, is_project_closing_task, note) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        rusqlite::params!["deleted-category", 25_i64, "2026-03-13", project_id, "Старый классификатор"],
    ).expect("expected unknown category task insert");
    drop(connection);

    let dashboard = crate::storage::with_test_database_path(&database_path, || {
        crate::storage::with_test_category_tree_path(&category_file_path, || {
            load_dashboard(AnalyticsDashboardInput {
                start_date: "2026-03-13".to_string(),
                end_date: "2026-03-13".to_string(),
            })
        })
    })
    .expect("expected analytics dashboard");

    assert_eq!(dashboard.total_minutes, 65);
    assert_eq!(dashboard.root_share_items.len(), 2);
    assert_eq!(dashboard.root_share_items[0].label, "Здоровье");
    assert_eq!(dashboard.root_share_items[0].total_minutes, 40);
    assert_eq!(dashboard.root_share_items[1].label, "Остальное");
    assert_eq!(dashboard.root_share_items[1].total_minutes, 25);

    let daily_labels = dashboard.daily_series[0]
        .category_breakdown
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(daily_labels, vec!["Здоровье", "Остальное"]);

    assert_eq!(
        dashboard.project_summaries[0].category_breakdown[1].label,
        "Остальное"
    );
    assert_eq!(dashboard.commented_tasks[0].category_label, "Остальное");

    crate::test_support::cleanup_database_artifacts(&database_path);
    crate::test_support::cleanup_file(&category_file_path);
}
