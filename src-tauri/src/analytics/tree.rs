use super::{
    repository::AnalyticsTaskRow, AnalyticsCategorySummaryNode, AnalyticsProjectPeriodItem,
    AnalyticsRootShareItem,
};
use crate::categories::TaskCategoryNode;
use std::collections::BTreeMap;

pub(super) const UNKNOWN_CATEGORY_ID: &str = "__unknown__";
pub(super) const UNKNOWN_CATEGORY_LABEL: &str = "Остальное";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RootCategoryRef {
    pub category_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Default)]
struct CategoryAccumulator {
    total_minutes: i64,
    task_count: i64,
    projects: BTreeMap<String, ProjectAccumulator>,
}

#[derive(Debug, Clone, Default)]
struct ProjectAccumulator {
    label: String,
    total_minutes: i64,
    finished_in_period: bool,
}

pub(super) fn build_category_summaries(
    category_tree: &[TaskCategoryNode],
    task_rows: &[AnalyticsTaskRow],
) -> Vec<AnalyticsCategorySummaryNode> {
    let direct_metrics = collect_direct_metrics(task_rows);
    let known_category_ids = collect_category_ids(category_tree);
    let mut root_summaries = category_tree
        .iter()
        .filter_map(|node| build_category_summary(node, &direct_metrics))
        .collect::<Vec<_>>();

    if let Some(unknown_summary) =
        build_unknown_category_summary(&direct_metrics, &known_category_ids)
    {
        root_summaries.push(unknown_summary);
    }

    root_summaries.sort_by(|left, right| {
        root_category_rank(&left.label)
            .cmp(&root_category_rank(&right.label))
            .then_with(|| left.label.cmp(&right.label))
    });

    root_summaries
}

fn collect_category_ids(category_tree: &[TaskCategoryNode]) -> Vec<String> {
    let mut ids = Vec::new();

    for node in category_tree {
        collect_category_id(node, &mut ids);
    }

    ids
}

fn collect_category_id(node: &TaskCategoryNode, target: &mut Vec<String>) {
    target.push(node.id.clone());

    if let Some(children) = node.children.as_deref() {
        for child in children {
            collect_category_id(child, target);
        }
    }
}

pub(super) fn build_root_share_items(
    root_summaries: &[AnalyticsCategorySummaryNode],
) -> Vec<AnalyticsRootShareItem> {
    let mut root_items = root_summaries
        .iter()
        .filter(|summary| summary.total_minutes > 0)
        .map(|summary| AnalyticsRootShareItem {
            category_id: summary.category_id.clone(),
            label: normalize_root_label(&summary.label),
            total_minutes: summary.total_minutes,
        })
        .collect::<Vec<_>>();

    root_items.sort_by(|left, right| {
        right
            .total_minutes
            .cmp(&left.total_minutes)
            .then_with(|| left.label.cmp(&right.label))
    });

    root_items
}

pub(super) fn build_root_category_map(
    category_tree: &[TaskCategoryNode],
) -> BTreeMap<String, RootCategoryRef> {
    let mut root_map = BTreeMap::new();

    for root in category_tree {
        let root_ref = RootCategoryRef {
            category_id: root.id.clone(),
            label: normalize_root_label(&root.label),
        };
        collect_root_category_refs(root, &root_ref, &mut root_map);
    }

    root_map
}

fn collect_direct_metrics(task_rows: &[AnalyticsTaskRow]) -> BTreeMap<String, CategoryAccumulator> {
    let mut metrics = BTreeMap::<String, CategoryAccumulator>::new();

    for row in task_rows {
        let entry = metrics.entry(row.category_id.clone()).or_default();
        entry.total_minutes += row.time_length;
        entry.task_count += 1;

        if let (Some(project_id), Some(project_label)) = (&row.project_id, &row.project_label) {
            let project_entry = entry.projects.entry(project_id.clone()).or_default();
            project_entry.label = project_label.clone();
            project_entry.total_minutes += row.time_length;
            project_entry.finished_in_period |= row.is_project_closing_task;
        }
    }

    metrics
}

fn build_category_summary(
    node: &TaskCategoryNode,
    direct_metrics: &BTreeMap<String, CategoryAccumulator>,
) -> Option<AnalyticsCategorySummaryNode> {
    let child_summaries = node
        .children
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|child| build_category_summary(child, direct_metrics))
        .collect::<Vec<_>>();

    let direct = direct_metrics.get(&node.id).cloned().unwrap_or_default();
    let total_minutes = direct.total_minutes
        + child_summaries
            .iter()
            .map(|child| child.total_minutes)
            .sum::<i64>();
    let task_count = direct.task_count
        + child_summaries
            .iter()
            .map(|child| child.task_count)
            .sum::<i64>();
    let projects = merge_projects(&direct.projects, &child_summaries);

    if total_minutes == 0 && task_count == 0 && projects.is_empty() && child_summaries.is_empty() {
        return None;
    }

    Some(AnalyticsCategorySummaryNode {
        category_id: node.id.clone(),
        label: node.label.clone(),
        total_minutes,
        task_count,
        projects,
        children: child_summaries,
    })
}

fn merge_projects(
    direct_projects: &BTreeMap<String, ProjectAccumulator>,
    child_summaries: &[AnalyticsCategorySummaryNode],
) -> Vec<AnalyticsProjectPeriodItem> {
    let mut merged = direct_projects.clone();

    for child in child_summaries {
        for project in &child.projects {
            let entry = merged.entry(project.project_id.clone()).or_default();
            entry.label = project.label.clone();
            entry.total_minutes += project.total_minutes;
            entry.finished_in_period |= project.finished_in_period;
        }
    }

    project_items_from_accumulators(merged)
}

fn build_unknown_category_summary(
    direct_metrics: &BTreeMap<String, CategoryAccumulator>,
    known_category_ids: &[String],
) -> Option<AnalyticsCategorySummaryNode> {
    let known_category_ids = known_category_ids
        .iter()
        .map(String::as_str)
        .collect::<std::collections::HashSet<_>>();
    let mut total_minutes = 0;
    let mut task_count = 0;
    let mut projects = BTreeMap::<String, ProjectAccumulator>::new();

    for (category_id, metrics) in direct_metrics {
        if known_category_ids.contains(category_id.as_str()) {
            continue;
        }

        total_minutes += metrics.total_minutes;
        task_count += metrics.task_count;

        for (project_id, project) in &metrics.projects {
            let entry = projects.entry(project_id.clone()).or_default();
            entry.label = project.label.clone();
            entry.total_minutes += project.total_minutes;
            entry.finished_in_period |= project.finished_in_period;
        }
    }

    if total_minutes == 0 && task_count == 0 && projects.is_empty() {
        return None;
    }

    Some(AnalyticsCategorySummaryNode {
        category_id: UNKNOWN_CATEGORY_ID.to_string(),
        label: UNKNOWN_CATEGORY_LABEL.to_string(),
        total_minutes,
        task_count,
        projects: project_items_from_accumulators(projects),
        children: vec![],
    })
}

fn project_items_from_accumulators(
    projects: BTreeMap<String, ProjectAccumulator>,
) -> Vec<AnalyticsProjectPeriodItem> {
    let mut projects = projects
        .into_iter()
        .map(|(project_id, project)| AnalyticsProjectPeriodItem {
            project_id,
            label: project.label,
            total_minutes: project.total_minutes,
            finished_in_period: project.finished_in_period,
        })
        .collect::<Vec<_>>();

    projects.sort_by(|left, right| left.label.cmp(&right.label));
    projects
}

fn collect_root_category_refs(
    node: &TaskCategoryNode,
    root_ref: &RootCategoryRef,
    target: &mut BTreeMap<String, RootCategoryRef>,
) {
    target.insert(node.id.clone(), root_ref.clone());

    if let Some(children) = node.children.as_deref() {
        for child in children {
            collect_root_category_refs(child, root_ref, target);
        }
    }
}

pub(super) fn root_category_rank(label: &str) -> usize {
    match normalize_root_label_key(label).as_str() {
        "деньги" => 0,
        "здоровье" => 1,
        "логика" => 2,
        "психика" => 3,
        "отношения" => 4,
        "рутина" => 5,
        "система" => 6,
        _ => usize::MAX,
    }
}

pub(super) fn normalize_root_label(label: &str) -> String {
    match normalize_root_label_key(label).as_str() {
        "деньги" => "Деньги".to_string(),
        "здоровье" => "Здоровье".to_string(),
        "логика" => "Логика".to_string(),
        "психика" => "Психика".to_string(),
        "отношения" => "Отношения".to_string(),
        "рутина" => "Рутина".to_string(),
        "система" => "Система".to_string(),
        _ => label
            .trim()
            .trim_end_matches('.')
            .trim_end_matches('!')
            .trim_end_matches('?')
            .to_string(),
    }
}

fn normalize_root_label_key(label: &str) -> String {
    label
        .trim()
        .trim_end_matches('.')
        .trim_end_matches('!')
        .trim_end_matches('?')
        .to_lowercase()
}
