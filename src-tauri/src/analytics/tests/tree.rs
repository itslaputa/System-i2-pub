#![cfg(test)]

use super::super::tree::{build_category_summaries, build_root_share_items};
use super::repository::AnalyticsTaskRow;
use crate::categories::TaskCategoryNode;

#[test]
fn merges_same_project_across_child_categories_under_one_root() {
    let category_tree = vec![TaskCategoryNode {
        id: "logic".to_string(),
        label: "Логика".to_string(),
        children: Some(vec![
            TaskCategoryNode {
                id: "logic-math".to_string(),
                label: "Математика".to_string(),
                children: None,
            },
            TaskCategoryNode {
                id: "logic-reading".to_string(),
                label: "Чтение".to_string(),
                children: None,
            },
        ]),
    }];
    let task_rows = vec![
        AnalyticsTaskRow {
            id: "1".to_string(),
            category_id: "logic-math".to_string(),
            time_length: 25,
            date: "2026-03-13".to_string(),
            note: None,
            project_id: Some("5".to_string()),
            project_label: Some("System-I2".to_string()),
            project_is_done: false,
            is_project_closing_task: false,
        },
        AnalyticsTaskRow {
            id: "2".to_string(),
            category_id: "logic-reading".to_string(),
            time_length: 35,
            date: "2026-03-13".to_string(),
            note: None,
            project_id: Some("5".to_string()),
            project_label: Some("System-I2".to_string()),
            project_is_done: true,
            is_project_closing_task: true,
        },
    ];

    let summaries = build_category_summaries(&category_tree, &task_rows);

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].projects.len(), 1);
    assert_eq!(summaries[0].projects[0].project_id, "5");
    assert_eq!(summaries[0].projects[0].total_minutes, 60);
    assert!(summaries[0].projects[0].finished_in_period);
}

#[test]
fn keeps_all_root_share_items_separate_instead_of_collapsing_into_other() {
    let root_summaries = vec![
        root("money", "Деньги", 10),
        root("health", "Здоровье.", 20),
        root("logic", "Логика", 30),
        root("mind", "Психика", 40),
        root("relationships", "Отношения", 50),
        root("routine", "Рутина", 60),
        root("system", "Система", 70),
    ];

    let items = build_root_share_items(&root_summaries);

    assert_eq!(items.len(), 7);
    assert_eq!(items[0].label, "Система");
    assert_eq!(items[4].label, "Логика");
    assert_eq!(items[5].label, "Здоровье");
    assert_eq!(items[5].total_minutes, 20);
    assert_eq!(items[6].label, "Деньги");
}

#[test]
fn normalizes_punctuated_root_labels_for_root_share_items() {
    let items = build_root_share_items(&[root("health", "Здоровье.", 25)]);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].label, "Здоровье");
}

fn root(category_id: &str, label: &str, total_minutes: i64) -> super::AnalyticsCategorySummaryNode {
    super::AnalyticsCategorySummaryNode {
        category_id: category_id.to_string(),
        label: label.to_string(),
        total_minutes,
        task_count: 1,
        projects: vec![],
        children: vec![],
    }
}
