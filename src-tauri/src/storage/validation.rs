use crate::categories::TaskCategoryNode;

pub fn normalize_non_empty_text(raw_value: &str, field_name: &str) -> Result<String, String> {
    let normalized = raw_value.trim();
    if normalized.is_empty() {
        return Err(format!("{field_name} cannot be empty"));
    }
    Ok(normalized.to_string())
}

pub fn normalize_optional_text(raw_value: Option<String>) -> Option<String> {
    raw_value.and_then(|value| {
        let normalized = value.trim();

        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        }
    })
}

pub fn normalize_iso_date(date_value: &str, field_name: &str) -> Result<String, String> {
    let normalized = normalize_non_empty_text(date_value, field_name)?;

    if normalized.len() != 10
        || !normalized.is_char_boundary(4)
        || !normalized.is_char_boundary(7)
        || &normalized[4..5] != "-"
        || &normalized[7..8] != "-"
        || !normalized
            .chars()
            .enumerate()
            .all(|(index, character)| matches!(index, 4 | 7) || character.is_ascii_digit())
    {
        return Err(format!(
            "{field_name} must use YYYY-MM-DD format, got '{normalized}'"
        ));
    }

    let year = normalized[0..4]
        .parse::<i32>()
        .map_err(|_| format!("{field_name} must use YYYY-MM-DD format, got '{normalized}'"))?;
    let month = normalized[5..7]
        .parse::<u32>()
        .map_err(|_| format!("{field_name} must use YYYY-MM-DD format, got '{normalized}'"))?;
    let day = normalized[8..10]
        .parse::<u32>()
        .map_err(|_| format!("{field_name} must use YYYY-MM-DD format, got '{normalized}'"))?;

    if !is_valid_calendar_date(year, month, day) {
        return Err(format!(
            "{field_name} must be a valid calendar date, got '{normalized}'"
        ));
    }

    Ok(normalized)
}

fn is_valid_calendar_date(year: i32, month: u32, day: u32) -> bool {
    if month == 0 || month > 12 || day == 0 {
        return false;
    }

    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };

    day <= max_day
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn parse_optional_i64_id(
    raw_value: Option<String>,
    field_name: &str,
) -> Result<Option<i64>, String> {
    let Some(raw_value) = raw_value else {
        return Ok(None);
    };

    let normalized = raw_value.trim();
    if normalized.is_empty() {
        return Ok(None);
    }

    normalized.parse::<i64>().map(Some).map_err(|error| {
        format!("failed to parse {field_name} '{normalized}' as integer id: {error}")
    })
}

pub fn validate_category_id_exists(category_id: &str) -> Result<(), String> {
    let normalized_category_id = normalize_non_empty_text(category_id, "category_id")?;
    let category_tree = load_category_tree_for_validation()?;

    if category_tree_contains_id(&category_tree, &normalized_category_id) {
        return Ok(());
    }

    Err(format!(
        "category_id '{}' does not exist in the current codificator",
        normalized_category_id
    ))
}

fn load_category_tree_for_validation() -> Result<Vec<TaskCategoryNode>, String> {
    #[cfg(test)]
    {
        if let Some(override_path) = super::testing::load_test_category_tree_override_path() {
            let file_contents = std::fs::read_to_string(&override_path).map_err(|error| {
                format!(
                    "failed to read test override task category file at {}: {error}",
                    override_path.display()
                )
            })?;

            return serde_json::from_str::<Vec<TaskCategoryNode>>(&file_contents).map_err(
                |error| {
                    format!(
                        "failed to parse test override task category file at {}: {error}",
                        override_path.display()
                    )
                },
            );
        }
    }

    crate::categories::load_task_category_tree()
}

fn category_tree_contains_id(tree: &[TaskCategoryNode], target_id: &str) -> bool {
    tree.iter().any(|node| {
        node.id == target_id
            || node
                .children
                .as_ref()
                .is_some_and(|children| category_tree_contains_id(children, target_id))
    })
}
