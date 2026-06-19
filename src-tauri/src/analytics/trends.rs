use super::periods;
use super::repository::AnalyticsTaskRow;
use super::{tree, AnalyticsTrendComparison, AnalyticsTrendPoint, AnalyticsTrendSeries};
use crate::categories::TaskCategoryNode;
use rusqlite::Connection;
use std::collections::BTreeMap;

const ROOT_TREND_SERIES: [(&str, &str); 7] = [
    ("money", "Деньги"),
    ("health", "Здоровье"),
    ("logic", "Логика"),
    ("mind", "Психика"),
    ("relationships", "Отношения"),
    ("routine", "Рутина"),
    ("system", "Система"),
];

pub(super) fn build_trend_comparison(
    connection: &Connection,
    category_tree: &[TaskCategoryNode],
    start_date: &str,
    end_date: &str,
) -> Result<AnalyticsTrendComparison, String> {
    let (mode, periods) = periods::build_trend_periods(connection, start_date, end_date)?;
    let root_category_map = tree::build_root_category_map(category_tree);
    let mut total_points = Vec::<AnalyticsTrendPoint>::with_capacity(periods.len());
    let mut category_points = ROOT_TREND_SERIES
        .iter()
        .map(|(_, label)| {
            (
                label.to_string(),
                Vec::<AnalyticsTrendPoint>::with_capacity(periods.len()),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for period in &periods {
        let task_rows = super::repository::load_ranged_task_rows(
            connection,
            &period.start_date,
            &period.end_date,
        )?;
        let total_minutes = task_rows.iter().map(|row| row.time_length).sum::<i64>();
        let root_totals = collect_root_totals(&task_rows, &root_category_map);

        total_points.push(build_point(period, total_minutes));

        for (_, label) in ROOT_TREND_SERIES {
            let total_minutes = root_totals.get(label).copied().unwrap_or(0);
            category_points
                .entry((*label).to_string())
                .or_default()
                .push(build_point(period, total_minutes));
        }
    }

    Ok(AnalyticsTrendComparison {
        mode,
        total: AnalyticsTrendSeries {
            series_id: "total".to_string(),
            label: "Суммарное время".to_string(),
            points: total_points,
        },
        categories: ROOT_TREND_SERIES
            .iter()
            .map(|(series_id, label)| AnalyticsTrendSeries {
                series_id: (*series_id).to_string(),
                label: (*label).to_string(),
                points: category_points.remove(*label).unwrap_or_default(),
            })
            .collect(),
    })
}

fn collect_root_totals(
    task_rows: &[AnalyticsTaskRow],
    root_category_map: &BTreeMap<String, tree::RootCategoryRef>,
) -> BTreeMap<String, i64> {
    let mut totals = BTreeMap::<String, i64>::new();

    for row in task_rows {
        let Some(root_category) = root_category_map.get(&row.category_id) else {
            continue;
        };

        let normalized_label = tree::normalize_root_label(&root_category.label);
        *totals.entry(normalized_label).or_default() += row.time_length;
    }

    totals
}

fn build_point(period: &periods::ComparisonPeriod, total_minutes: i64) -> AnalyticsTrendPoint {
    AnalyticsTrendPoint {
        label: period.label.clone(),
        start_date: period.start_date.clone(),
        end_date: period.end_date.clone(),
        total_minutes,
        is_current: period.is_current,
    }
}
