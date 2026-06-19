use super::AnalyticsTrendMode;
use rusqlite::Connection;

const TREND_PERIOD_COUNT: usize = 6;
const SHORT_RUSSIAN_MONTHS: [&str; 12] = [
    "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ComparisonPeriod {
    pub label: String,
    pub start_date: String,
    pub end_date: String,
    pub is_current: bool,
}

pub(super) fn build_trend_periods(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<(AnalyticsTrendMode, Vec<ComparisonPeriod>), String> {
    let today = load_today(connection)?;
    build_trend_periods_for_reference_date(connection, start_date, end_date, &today)
}

pub(super) fn build_trend_periods_for_reference_date(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
    today: &str,
) -> Result<(AnalyticsTrendMode, Vec<ComparisonPeriod>), String> {
    if is_current_calendar_week(connection, start_date, end_date, today)? {
        return Ok((
            AnalyticsTrendMode::Week,
            build_current_week_periods(connection, today)?,
        ));
    }

    if is_current_calendar_month(connection, start_date, end_date, today)? {
        return Ok((
            AnalyticsTrendMode::Month,
            build_current_month_periods(connection, today)?,
        ));
    }

    if is_current_calendar_year(connection, start_date, end_date, today)? {
        return Ok((
            AnalyticsTrendMode::Year,
            build_current_year_periods(connection, today)?,
        ));
    }

    if is_full_week(connection, start_date, end_date)? {
        return Ok((
            AnalyticsTrendMode::Week,
            build_full_week_periods(connection, start_date, end_date)?,
        ));
    }

    if is_full_month(connection, start_date, end_date)? {
        return Ok((
            AnalyticsTrendMode::Month,
            build_full_month_periods(connection, start_date, end_date)?,
        ));
    }

    if is_full_year(connection, start_date, end_date)? {
        return Ok((
            AnalyticsTrendMode::Year,
            build_full_year_periods(connection, start_date, end_date)?,
        ));
    }

    Ok((
        AnalyticsTrendMode::Custom,
        build_custom_periods(connection, start_date, end_date)?,
    ))
}

fn build_current_week_periods(
    connection: &Connection,
    today: &str,
) -> Result<Vec<ComparisonPeriod>, String> {
    let current_start = start_of_week(connection, today)?;
    let mut periods = Vec::with_capacity(TREND_PERIOD_COUNT);

    for offset in (1..TREND_PERIOD_COUNT).rev() {
        let start_date = shift_days(connection, &current_start, -7 * offset as i64)?;
        let end_date = shift_days(connection, &start_date, 6)?;
        periods.push(comparison_period(
            &start_date,
            &end_date,
            false,
            AnalyticsTrendMode::Week,
        ));
    }

    periods.push(comparison_period(
        &current_start,
        today,
        true,
        AnalyticsTrendMode::Week,
    ));
    Ok(periods)
}

fn build_current_month_periods(
    connection: &Connection,
    today: &str,
) -> Result<Vec<ComparisonPeriod>, String> {
    let current_start = start_of_month(connection, today)?;
    let mut periods = Vec::with_capacity(TREND_PERIOD_COUNT);

    for offset in (1..TREND_PERIOD_COUNT).rev() {
        let start_date = shift_months(connection, &current_start, -(offset as i64))?;
        let end_date = end_of_month(connection, &start_date)?;
        periods.push(comparison_period(
            &start_date,
            &end_date,
            false,
            AnalyticsTrendMode::Month,
        ));
    }

    periods.push(comparison_period(
        &current_start,
        today,
        true,
        AnalyticsTrendMode::Month,
    ));
    Ok(periods)
}

fn build_current_year_periods(
    connection: &Connection,
    today: &str,
) -> Result<Vec<ComparisonPeriod>, String> {
    let current_start = start_of_year(connection, today)?;
    let mut periods = Vec::with_capacity(TREND_PERIOD_COUNT);

    for offset in (1..TREND_PERIOD_COUNT).rev() {
        let start_date = shift_years(connection, &current_start, -(offset as i64))?;
        let end_date = end_of_year(connection, &start_date)?;
        periods.push(comparison_period(
            &start_date,
            &end_date,
            false,
            AnalyticsTrendMode::Year,
        ));
    }

    periods.push(comparison_period(
        &current_start,
        today,
        true,
        AnalyticsTrendMode::Year,
    ));
    Ok(periods)
}

fn build_full_week_periods(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<ComparisonPeriod>, String> {
    let mut periods = Vec::with_capacity(TREND_PERIOD_COUNT);

    for offset in (1..TREND_PERIOD_COUNT).rev() {
        let previous_start = shift_days(connection, start_date, -(7 * offset as i64))?;
        let previous_end = shift_days(connection, &previous_start, 6)?;
        periods.push(comparison_period(
            &previous_start,
            &previous_end,
            false,
            AnalyticsTrendMode::Week,
        ));
    }

    periods.push(comparison_period(
        start_date,
        end_date,
        true,
        AnalyticsTrendMode::Week,
    ));
    Ok(periods)
}

fn build_full_month_periods(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<ComparisonPeriod>, String> {
    let mut periods = Vec::with_capacity(TREND_PERIOD_COUNT);

    for offset in (1..TREND_PERIOD_COUNT).rev() {
        let previous_start = shift_months(connection, start_date, -(offset as i64))?;
        let previous_end = end_of_month(connection, &previous_start)?;
        periods.push(comparison_period(
            &previous_start,
            &previous_end,
            false,
            AnalyticsTrendMode::Month,
        ));
    }

    periods.push(comparison_period(
        start_date,
        end_date,
        true,
        AnalyticsTrendMode::Month,
    ));
    Ok(periods)
}

fn build_full_year_periods(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<ComparisonPeriod>, String> {
    let mut periods = Vec::with_capacity(TREND_PERIOD_COUNT);

    for offset in (1..TREND_PERIOD_COUNT).rev() {
        let previous_start = shift_years(connection, start_date, -(offset as i64))?;
        let previous_end = end_of_year(connection, &previous_start)?;
        periods.push(comparison_period(
            &previous_start,
            &previous_end,
            false,
            AnalyticsTrendMode::Year,
        ));
    }

    periods.push(comparison_period(
        start_date,
        end_date,
        true,
        AnalyticsTrendMode::Year,
    ));
    Ok(periods)
}

fn build_custom_periods(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<ComparisonPeriod>, String> {
    let range_length_days = day_diff(connection, start_date, end_date)? + 1;
    let mut periods = Vec::with_capacity(TREND_PERIOD_COUNT);

    for offset in (1..TREND_PERIOD_COUNT).rev() {
        let previous_start =
            shift_days(connection, start_date, -(range_length_days * offset as i64))?;
        let previous_end = shift_days(
            connection,
            start_date,
            -((range_length_days * (offset as i64 - 1)) + 1),
        )?;

        periods.push(comparison_period(
            &previous_start,
            &previous_end,
            false,
            AnalyticsTrendMode::Custom,
        ));
    }

    periods.push(comparison_period(
        start_date,
        end_date,
        true,
        AnalyticsTrendMode::Custom,
    ));
    Ok(periods)
}

fn comparison_period(
    start_date: &str,
    end_date: &str,
    is_current: bool,
    mode: AnalyticsTrendMode,
) -> ComparisonPeriod {
    ComparisonPeriod {
        label: format_period_label(start_date, end_date, mode),
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        is_current,
    }
}

fn format_period_label(start_date: &str, end_date: &str, mode: AnalyticsTrendMode) -> String {
    match mode {
        AnalyticsTrendMode::Month => format_month_label(start_date),
        AnalyticsTrendMode::Year => start_date[..4].to_string(),
        AnalyticsTrendMode::Week | AnalyticsTrendMode::Custom => {
            format!(
                "{}-{}",
                short_day_label(start_date),
                short_day_label(end_date)
            )
        }
    }
}

fn short_day_label(date: &str) -> String {
    let [_year, month, day] = split_date(date);
    let month_index = month
        .parse::<usize>()
        .ok()
        .and_then(|value| value.checked_sub(1));

    match month_index.and_then(|value| SHORT_RUSSIAN_MONTHS.get(value)) {
        Some(label) => format!("{day} {label}"),
        None => date.to_string(),
    }
}

fn format_month_label(date: &str) -> String {
    let [year, month, _] = split_date(date);
    let month_index = month
        .parse::<usize>()
        .ok()
        .and_then(|value| value.checked_sub(1));

    match month_index.and_then(|value| SHORT_RUSSIAN_MONTHS.get(value)) {
        Some(label) => format!("{label} {year}"),
        None => date.to_string(),
    }
}

fn split_date(date: &str) -> [&str; 3] {
    let mut parts = date.split('-');
    [
        parts.next().unwrap_or(date),
        parts.next().unwrap_or(""),
        parts.next().unwrap_or(""),
    ]
}

fn is_current_calendar_week(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
    today: &str,
) -> Result<bool, String> {
    Ok(start_date == start_of_week(connection, today)?
        && end_date == end_of_week(connection, today)?
        && today < end_date)
}

fn is_current_calendar_month(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
    today: &str,
) -> Result<bool, String> {
    Ok(start_date == start_of_month(connection, today)?
        && end_date == end_of_month(connection, today)?
        && today < end_date)
}

fn is_current_calendar_year(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
    today: &str,
) -> Result<bool, String> {
    Ok(start_date == start_of_year(connection, today)?
        && end_date == end_of_year(connection, today)?
        && today < end_date)
}

fn is_full_week(connection: &Connection, start_date: &str, end_date: &str) -> Result<bool, String> {
    Ok(start_date == start_of_week(connection, start_date)?
        && end_date == end_of_week(connection, start_date)?)
}

fn is_full_month(
    connection: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<bool, String> {
    Ok(start_date == start_of_month(connection, start_date)?
        && end_date == end_of_month(connection, start_date)?)
}

fn is_full_year(connection: &Connection, start_date: &str, end_date: &str) -> Result<bool, String> {
    Ok(start_date == start_of_year(connection, start_date)?
        && end_date == end_of_year(connection, start_date)?)
}

fn load_today(connection: &Connection) -> Result<String, String> {
    scalar_date(connection, "SELECT date('now', 'localtime')", [])
}

fn start_of_week(connection: &Connection, date: &str) -> Result<String, String> {
    scalar_date(
        connection,
        "SELECT date(?1, '-' || ((CAST(strftime('%w', ?1) AS INTEGER) + 6) % 7) || ' days')",
        [date],
    )
}

fn end_of_week(connection: &Connection, date: &str) -> Result<String, String> {
    let start = start_of_week(connection, date)?;
    shift_days(connection, &start, 6)
}

fn start_of_month(connection: &Connection, date: &str) -> Result<String, String> {
    scalar_date(connection, "SELECT date(?1, 'start of month')", [date])
}

fn end_of_month(connection: &Connection, date: &str) -> Result<String, String> {
    scalar_date(
        connection,
        "SELECT date(?1, 'start of month', '+1 month', '-1 day')",
        [date],
    )
}

fn start_of_year(connection: &Connection, date: &str) -> Result<String, String> {
    scalar_date(connection, "SELECT date(?1, 'start of year')", [date])
}

fn end_of_year(connection: &Connection, date: &str) -> Result<String, String> {
    scalar_date(
        connection,
        "SELECT date(?1, 'start of year', '+1 year', '-1 day')",
        [date],
    )
}

fn shift_days(connection: &Connection, date: &str, days: i64) -> Result<String, String> {
    let modifier = build_modifier(days, "days");
    scalar_date(connection, "SELECT date(?1, ?2)", [date, modifier.as_str()])
}

fn shift_months(connection: &Connection, date: &str, months: i64) -> Result<String, String> {
    let modifier = build_modifier(months, "months");
    scalar_date(connection, "SELECT date(?1, ?2)", [date, modifier.as_str()])
}

fn shift_years(connection: &Connection, date: &str, years: i64) -> Result<String, String> {
    let modifier = build_modifier(years, "years");
    scalar_date(connection, "SELECT date(?1, ?2)", [date, modifier.as_str()])
}

fn day_diff(connection: &Connection, start_date: &str, end_date: &str) -> Result<i64, String> {
    connection
        .query_row(
            "SELECT CAST(julianday(?2) - julianday(?1) AS INTEGER)",
            [start_date, end_date],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to calculate analytics trend day diff: {error}"))
}

fn scalar_date<const N: usize>(
    connection: &Connection,
    sql: &str,
    params: [&str; N],
) -> Result<String, String> {
    connection
        .query_row(sql, rusqlite::params_from_iter(params), |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| format!("failed to evaluate analytics date expression: {error}"))
}

fn build_modifier(offset: i64, unit: &str) -> String {
    if offset >= 0 {
        format!("+{offset} {unit}")
    } else {
        format!("{offset} {unit}")
    }
}
