#![cfg(test)]

use super::{super::change_log::collect_category_change_log_lines, TaskCategoryNode};

#[test]
fn collects_added_removed_and_renamed_lines_with_normalized_timestamp() {
    let previous_tree = vec![
        TaskCategoryNode {
            id: "10000".to_string(),
            label: "Деньги".to_string(),
            children: Some(vec![TaskCategoryNode {
                id: "10001".to_string(),
                label: "Работа".to_string(),
                children: None,
            }]),
        },
        TaskCategoryNode {
            id: "10002".to_string(),
            label: "Психика".to_string(),
            children: None,
        },
    ];
    let next_tree = vec![TaskCategoryNode {
        id: "10000".to_string(),
        label: "Деньги".to_string(),
        children: Some(vec![
            TaskCategoryNode {
                id: "10001".to_string(),
                label: "Основная работа".to_string(),
                children: None,
            },
            TaskCategoryNode {
                id: "10003".to_string(),
                label: "Поиск работы".to_string(),
                children: None,
            },
        ]),
    }];

    let lines = collect_category_change_log_lines(&previous_tree, &next_tree, "   ");

    assert_eq!(
        lines,
        vec![
            "unknown-time | added | Деньги / Поиск работы [10003]".to_string(),
            "unknown-time | removed | Психика [10002]".to_string(),
            "unknown-time | renamed | Деньги / Работа -> Деньги / Основная работа [10001]"
                .to_string(),
        ]
    );
}
