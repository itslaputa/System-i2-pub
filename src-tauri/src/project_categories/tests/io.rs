use super::super::io::{load_project_category_list_from_path, save_project_category_list_to_path};
use super::ProjectCategoryNode;

#[test]
fn loads_seeded_project_categories_when_file_is_missing() {
    let category_file_path =
        crate::test_support::unique_temp_json_path("project-categories-seeded");

    let categories = load_project_category_list_from_path(
        category_file_path
            .to_str()
            .expect("expected utf-8 project category path"),
    )
    .expect("expected missing project category file to seed defaults");

    assert_eq!(categories.len(), 2);
    assert_eq!(categories[0].label, "Книга");
    assert_eq!(categories[1].label, "Фильм");
    assert!(category_file_path.exists());

    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn rejects_duplicate_project_category_labels_after_normalization() {
    let category_file_path =
        crate::test_support::unique_temp_json_path("project-categories-validation");

    let error = save_project_category_list_to_path(
        category_file_path
            .to_str()
            .expect("expected utf-8 project category path"),
        &[
            ProjectCategoryNode {
                id: "book".to_string(),
                label: " Книга ".to_string(),
                order: 0,
            },
            ProjectCategoryNode {
                id: "film".to_string(),
                label: "книга".to_string(),
                order: 1,
            },
        ],
    )
    .expect_err("expected duplicate labels to fail");

    assert!(error.contains("duplicate project category label"));
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn rejects_duplicate_project_category_ids_after_normalization() {
    let category_file_path =
        crate::test_support::unique_temp_json_path("project-categories-duplicate-id");

    let error = save_project_category_list_to_path(
        category_file_path
            .to_str()
            .expect("expected utf-8 project category path"),
        &[
            ProjectCategoryNode {
                id: " book ".to_string(),
                label: "Книга".to_string(),
                order: 0,
            },
            ProjectCategoryNode {
                id: "book".to_string(),
                label: "Фильм".to_string(),
                order: 1,
            },
        ],
    )
    .expect_err("expected duplicate ids to fail");

    assert!(error.contains("duplicate project category id"));
    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn saves_project_categories_with_normalized_order_and_without_temp_files() {
    let category_file_path =
        crate::test_support::unique_temp_json_path("project-categories-normalized-save");

    save_project_category_list_to_path(
        category_file_path
            .to_str()
            .expect("expected utf-8 project category path"),
        &[
            ProjectCategoryNode {
                id: "film".to_string(),
                label: " Фильм ".to_string(),
                order: 10,
            },
            ProjectCategoryNode {
                id: "book".to_string(),
                label: "Книга".to_string(),
                order: 1,
            },
        ],
    )
    .expect("expected normalized project category save");

    let categories = load_project_category_list_from_path(
        category_file_path
            .to_str()
            .expect("expected utf-8 project category path"),
    )
    .expect("expected saved categories to load");
    assert_eq!(
        categories,
        vec![
            ProjectCategoryNode {
                id: "book".to_string(),
                label: "Книга".to_string(),
                order: 0,
            },
            ProjectCategoryNode {
                id: "film".to_string(),
                label: "Фильм".to_string(),
                order: 1,
            },
        ]
    );

    let temp_file_count = category_file_path
        .parent()
        .expect("expected temp file parent")
        .read_dir()
        .expect("expected temp directory entries")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("project-categories-normalized-save")
                && entry.file_name().to_string_lossy().ends_with(".tmp")
        })
        .count();
    assert_eq!(temp_file_count, 0);

    crate::test_support::cleanup_file(&category_file_path);
}

#[test]
fn failed_project_category_save_does_not_leave_temp_files() {
    let missing_parent = crate::test_support::unique_temp_dir("missing-project-category-parent");
    crate::test_support::cleanup_dir(&missing_parent);
    let category_file_path = missing_parent.join("project_categories.json");

    let error = save_project_category_list_to_path(
        category_file_path
            .to_str()
            .expect("expected utf-8 project category path"),
        &[ProjectCategoryNode {
            id: "book".to_string(),
            label: "Книга".to_string(),
            order: 0,
        }],
    )
    .expect_err("expected save into missing parent to fail");

    assert!(error.contains("failed to write temporary project category list"));
    assert!(!missing_parent.exists());
}

#[test]
fn rejects_invalid_project_category_file_when_loading_existing_runtime_data() {
    let category_file_path =
        crate::test_support::unique_temp_json_path("project-categories-invalid-load");
    std::fs::write(
        &category_file_path,
        r#"[
  {
    "id": "book",
    "label": " Книга ",
    "order": 3
  },
  {
    "id": "film",
    "label": "книга",
    "order": 1
  }
]
"#,
    )
    .expect("expected invalid project category file");

    let error = load_project_category_list_from_path(
        category_file_path
            .to_str()
            .expect("expected utf-8 project category path"),
    )
    .expect_err("expected invalid runtime file to be rejected");

    assert!(error.contains("failed to validate project category list"));
    assert!(error.contains("duplicate project category label"));

    crate::test_support::cleanup_file(&category_file_path);
}
