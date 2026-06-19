use super::AnalyticsDailyPoint;
use rusqlite::{Connection, OptionalExtension};

#[derive(Debug, Clone)]
pub(super) struct AnalyticsTaskRow {
    pub id: String,
    pub category_id: String,
    pub time_length: i64,
    pub date: String,
    pub note: Option<String>,
    pub project_id: Option<String>,
    pub project_label: Option<String>,
    pub project_is_done: bool,
    pub is_project_closing_task: bool,
}

pub(super) fn load_ranged_task_rows(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<AnalyticsTaskRow>, String> {
    let mut statement = connection
        .prepare(
            "SELECT t.id, t.category_id, t.time_length, t.date, t.note, t.project_id, p.name, p.is_done, t.is_project_closing_task
             FROM tasks t
             LEFT JOIN projects p ON p.id = t.project_id
             WHERE t.date >= ?1 AND t.date <= ?2
             ORDER BY t.date ASC, t.id ASC",
        )
        .map_err(|error| format!("failed to prepare analytics task query: {error}"))?;

    let rows = statement
        .query_map([start_date, end_date], |row| {
            let project_id = row.get::<_, Option<i64>>(5)?.map(|value| value.to_string());

            Ok(AnalyticsTaskRow {
                id: row.get::<_, i64>(0)?.to_string(),
                category_id: row.get(1)?,
                time_length: row.get(2)?,
                date: row.get(3)?,
                note: row.get(4)?,
                project_id,
                project_label: row.get::<_, Option<String>>(6)?,
                project_is_done: row.get::<_, Option<i64>>(7)?.unwrap_or(0) != 0,
                is_project_closing_task: row.get::<_, i64>(8)? != 0,
            })
        })
        .map_err(|error| format!("failed to query analytics task rows: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect analytics task rows: {error}"))
}

pub(super) fn load_daily_series(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<AnalyticsDailyPoint>, String> {
    let mut statement = connection
        .prepare(
            r#"
            WITH RECURSIVE date_range(day) AS (
                SELECT ?1
                UNION ALL
                SELECT date(day, '+1 day')
                FROM date_range
                WHERE day < ?2
            )
            SELECT
                date_range.day,
                COALESCE(SUM(tasks.time_length), 0) AS total_minutes,
                COALESCE(COUNT(tasks.id), 0) AS task_count
            FROM date_range
            LEFT JOIN tasks ON tasks.date = date_range.day
            GROUP BY date_range.day
            ORDER BY date_range.day ASC
            "#,
        )
        .map_err(|error| format!("failed to prepare analytics daily series query: {error}"))?;

    let rows = statement
        .query_map([start_date, end_date], |row| {
            Ok(AnalyticsDailyPoint {
                date: row.get(0)?,
                total_minutes: row.get(1)?,
                task_count: row.get(2)?,
                category_breakdown: vec![],
            })
        })
        .map_err(|error| format!("failed to query analytics daily series: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect analytics daily series: {error}"))
}

pub(super) fn load_total_minutes(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<i64, String> {
    connection
        .query_row(
            "SELECT COALESCE(SUM(time_length), 0) FROM tasks WHERE date >= ?1 AND date <= ?2",
            [start_date, end_date],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| {
            format!("failed to load analytics total minutes for {start_date}..{end_date}: {error}")
        })?
        .ok_or_else(|| "failed to read analytics total minutes".to_string())
}
