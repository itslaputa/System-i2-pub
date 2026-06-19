#![cfg(test)]

use super::{
    normalize_iso_date, normalize_non_empty_text, normalize_optional_text, parse_optional_i64_id,
    validate_category_id_exists,
};

#[test]
fn trims_and_normalizes_text_and_dates() {
    assert_eq!(
        normalize_non_empty_text("  Юнг - АИКБ  ", "label").expect("expected normalized text"),
        "Юнг - АИКБ"
    );
    assert_eq!(
        normalize_optional_text(Some("  заметка  ".to_string())),
        Some("заметка".to_string())
    );
    assert_eq!(normalize_optional_text(Some("   ".to_string())), None);
    assert_eq!(
        normalize_iso_date(" 2026-03-13 ", "task_date").expect("expected normalized date"),
        "2026-03-13"
    );
}

#[test]
fn rejects_invalid_dates_and_ids() {
    let date_error =
        normalize_iso_date("13-03-2026", "task_date").expect_err("expected invalid date");
    assert!(date_error.contains("YYYY-MM-DD"));

    let calendar_error =
        normalize_iso_date("2026-02-31", "task_date").expect_err("expected invalid calendar date");
    assert!(calendar_error.contains("valid calendar date"));

    assert_eq!(
        normalize_iso_date("2028-02-29", "task_date").expect("expected leap day to be valid"),
        "2028-02-29"
    );

    let parsed =
        parse_optional_i64_id(Some(" 42 ".to_string()), "project_id").expect("expected integer id");
    assert_eq!(parsed, Some(42));

    let none_from_blank =
        parse_optional_i64_id(Some("   ".to_string()), "project_id").expect("expected blank none");
    assert_eq!(none_from_blank, None);

    let parse_error = parse_optional_i64_id(Some("abc".to_string()), "project_id")
        .expect_err("expected parse failure");
    assert!(parse_error.contains("failed to parse project_id"));
}

#[test]
fn validates_category_ids_against_current_category_tree() {
    let category_file_path = crate::test_support::unique_temp_json_path("validation-categories");
    crate::test_support::seed_minimal_category_file(&category_file_path);

    crate::storage::with_test_category_tree_path(&category_file_path, || {
        validate_category_id_exists(" logic-math ").expect("expected category to exist");
        let error = validate_category_id_exists("missing-category")
            .expect_err("expected missing category to fail");
        assert!(error.contains("does not exist in the current codificator"));
    });

    crate::test_support::cleanup_file(&category_file_path);
}
