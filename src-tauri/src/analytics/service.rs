use super::{
    repository, tree, AnalyticsCommentedTaskItem, AnalyticsDailyCategorySlice,
    AnalyticsDailyChartKind, AnalyticsDailyPoint, AnalyticsDashboardData, AnalyticsDashboardInput,
    AnalyticsProjectCategoryBreakdownItem, AnalyticsProjectSummaryItem,
};
use crate::categories::TaskCategoryNode;
use std::collections::BTreeMap;

pub fn load_dashboard(input: AnalyticsDashboardInput) -> Result<AnalyticsDashboardData, String> {
    let start_date = crate::storage::normalize_iso_date(&input.start_date, "start_date")?;
    let end_date = crate::storage::normalize_iso_date(&input.end_date, "end_date")?;

    if start_date > end_date {
        return Err("start_date cannot be later than end_date".to_string());
    }

    let connection = crate::storage::open_app_database_connection()?;
    let category_tree = crate::categories::load_task_category_tree()?;
    let task_rows = repository::load_ranged_task_rows(&connection, &start_date, &end_date)?;
    let total_minutes = repository::load_total_minutes(&connection, &start_date, &end_date)?;
    let category_summaries = tree::build_category_summaries(&category_tree, &task_rows);
    let project_summaries = build_project_summaries(&category_tree, &task_rows);
    let root_share_items = tree::build_root_share_items(&category_summaries);
    let daily_series = build_daily_series(
        &category_tree,
        repository::load_daily_series(&connection, &start_date, &end_date)?,
        &task_rows,
    );
    let commented_tasks = build_commented_tasks(&category_tree, &task_rows);
    let trend_comparison =
        super::trends::build_trend_comparison(&connection, &category_tree, &start_date, &end_date)?;
    let daily_chart_kind = if daily_series.len() <= 31 {
        AnalyticsDailyChartKind::Bar
    } else {
        AnalyticsDailyChartKind::Line
    };

    Ok(AnalyticsDashboardData {
        start_date,
        end_date,
        total_minutes,
        category_summaries,
        project_summaries,
        root_share_items,
        daily_series,
        commented_tasks,
        daily_chart_kind,
        trend_comparison,
    })
}

#[derive(Debug, Clone, Default)]
struct ProjectSummaryAccumulator {
    label: String,
    total_minutes: i64,
    task_count: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    is_done: bool,
    finished_in_period: bool,
    categories: BTreeMap<String, ProjectCategoryAccumulator>,
}

#[derive(Debug, Clone, Default)]
struct ProjectCategoryAccumulator {
    total_minutes: i64,
    task_count: i64,
}

fn build_project_summaries(
    category_tree: &[TaskCategoryNode],
    task_rows: &[repository::AnalyticsTaskRow],
) -> Vec<AnalyticsProjectSummaryItem> {
    let category_labels = build_category_label_map(category_tree);
    let mut projects = BTreeMap::<String, ProjectSummaryAccumulator>::new();

    for row in task_rows {
        let (Some(project_id), Some(project_label)) = (&row.project_id, &row.project_label) else {
            continue;
        };

        let project = projects.entry(project_id.clone()).or_default();
        project.label = project_label.clone();
        project.total_minutes += row.time_length;
        project.task_count += 1;
        project.is_done |= row.project_is_done;
        project.finished_in_period |= row.is_project_closing_task;

        match &project.start_date {
            Some(current_start_date) if current_start_date <= &row.date => {}
            _ => project.start_date = Some(row.date.clone()),
        }

        match &project.end_date {
            Some(current_end_date) if current_end_date >= &row.date => {}
            _ => project.end_date = Some(row.date.clone()),
        }

        let category_id = normalize_breakdown_category_id(&category_labels, &row.category_id);
        let category = project.categories.entry(category_id).or_default();
        category.total_minutes += row.time_length;
        category.task_count += 1;
    }

    let mut summaries = projects
        .into_iter()
        .map(|(project_id, project)| {
            let mut category_breakdown = project
                .categories
                .into_iter()
                .map(
                    |(category_id, category)| AnalyticsProjectCategoryBreakdownItem {
                        label: category_labels
                            .get(&category_id)
                            .cloned()
                            .unwrap_or_else(|| tree::UNKNOWN_CATEGORY_LABEL.to_string()),
                        category_id,
                        total_minutes: category.total_minutes,
                        task_count: category.task_count,
                    },
                )
                .collect::<Vec<_>>();

            category_breakdown.sort_by(|left, right| {
                right
                    .total_minutes
                    .cmp(&left.total_minutes)
                    .then_with(|| left.label.cmp(&right.label))
            });

            AnalyticsProjectSummaryItem {
                project_id,
                label: project.label,
                total_minutes: project.total_minutes,
                task_count: project.task_count,
                start_date: project.start_date,
                end_date: project.end_date,
                is_done: project.is_done,
                finished_in_period: project.finished_in_period,
                category_breakdown,
            }
        })
        .collect::<Vec<_>>();

    summaries.sort_by(|left, right| {
        right
            .total_minutes
            .cmp(&left.total_minutes)
            .then_with(|| left.label.cmp(&right.label))
    });

    summaries
}

fn build_daily_series(
    category_tree: &[TaskCategoryNode],
    mut base_series: Vec<AnalyticsDailyPoint>,
    task_rows: &[repository::AnalyticsTaskRow],
) -> Vec<AnalyticsDailyPoint> {
    let root_category_map = tree::build_root_category_map(category_tree);
    let mut minutes_by_day_and_root = BTreeMap::<(String, String), i64>::new();
    let mut root_labels = BTreeMap::<String, String>::new();

    for row in task_rows {
        let root_category = root_category_map
            .get(&row.category_id)
            .cloned()
            .unwrap_or_else(|| tree::RootCategoryRef {
                category_id: tree::UNKNOWN_CATEGORY_ID.to_string(),
                label: tree::UNKNOWN_CATEGORY_LABEL.to_string(),
            });

        *minutes_by_day_and_root
            .entry((row.date.clone(), root_category.category_id.clone()))
            .or_default() += row.time_length;
        root_labels.insert(root_category.category_id, root_category.label);
    }

    for point in &mut base_series {
        let mut breakdown = root_labels
            .iter()
            .filter_map(|(root_category_id, label)| {
                let total_minutes = minutes_by_day_and_root
                    .get(&(point.date.clone(), root_category_id.clone()))
                    .copied()
                    .unwrap_or(0);

                if total_minutes == 0 {
                    return None;
                }

                Some(AnalyticsDailyCategorySlice {
                    category_id: root_category_id.clone(),
                    label: label.clone(),
                    total_minutes,
                })
            })
            .collect::<Vec<_>>();

        breakdown.sort_by(|left, right| {
            tree::root_category_rank(&left.label)
                .cmp(&tree::root_category_rank(&right.label))
                .then_with(|| left.label.cmp(&right.label))
        });

        point.category_breakdown = breakdown;
    }

    base_series
}

fn build_commented_tasks(
    category_tree: &[TaskCategoryNode],
    task_rows: &[repository::AnalyticsTaskRow],
) -> Vec<AnalyticsCommentedTaskItem> {
    let category_labels = build_category_label_map(category_tree);

    task_rows
        .iter()
        .rev()
        .filter_map(|row| {
            let note = row.note.as_ref()?.trim();

            if note.is_empty() {
                return None;
            }

            Some(AnalyticsCommentedTaskItem {
                task_id: row.id.clone(),
                date: row.date.clone(),
                category_label: category_labels
                    .get(&row.category_id)
                    .cloned()
                    .unwrap_or_else(|| tree::UNKNOWN_CATEGORY_LABEL.to_string()),
                project_label: row.project_label.clone(),
                time_length: row.time_length,
                note: note.to_string(),
            })
        })
        .collect()
}

fn build_category_label_map(category_tree: &[TaskCategoryNode]) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();
    collect_category_labels(category_tree, &mut labels);
    labels
}

fn collect_category_labels(nodes: &[TaskCategoryNode], target: &mut BTreeMap<String, String>) {
    for node in nodes {
        target.insert(node.id.clone(), node.label.clone());

        if let Some(children) = node.children.as_deref() {
            collect_category_labels(children, target);
        }
    }
}

fn normalize_breakdown_category_id(
    category_labels: &BTreeMap<String, String>,
    category_id: &str,
) -> String {
    if category_labels.contains_key(category_id) {
        category_id.to_string()
    } else {
        tree::UNKNOWN_CATEGORY_ID.to_string()
    }
}
