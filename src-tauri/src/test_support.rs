#![cfg(test)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static UNIQUE_TEST_PATH_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn unique_temp_database_path(scope: &str) -> PathBuf {
    unique_temp_file_path(&format!("system-i2-{scope}-db"), "sqlite3")
}

pub fn unique_temp_dir(scope: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("system-i2-{scope}-{timestamp}"));
    fs::create_dir_all(&path).expect("expected temp dir to be creatable");
    path
}

pub fn unique_temp_json_path(scope: &str) -> PathBuf {
    unique_temp_file_path(&format!("system-i2-{scope}"), "json")
}

pub fn unique_temp_log_path(scope: &str) -> PathBuf {
    unique_temp_file_path(&format!("system-i2-{scope}"), "log")
}

pub fn cleanup_database_artifacts(database_path: &Path) {
    let wal_path = PathBuf::from(format!("{}-wal", database_path.display()));
    let shm_path = PathBuf::from(format!("{}-shm", database_path.display()));

    let _ = fs::remove_file(database_path);
    let _ = fs::remove_file(wal_path);
    let _ = fs::remove_file(shm_path);
}

pub fn cleanup_file(path: &Path) {
    let _ = fs::remove_file(path);
}

pub fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

pub fn seed_minimal_category_file(path: &Path) {
    fs::write(
        path,
        r#"[
  {
    "id": "logic",
    "label": "Логика",
    "children": [
      {
        "id": "logic-math",
        "label": "Математика"
      }
    ]
  },
  {
    "id": "mind",
    "label": "Психика",
    "children": [
      {
        "id": "mind-reading",
        "label": "Читал"
      }
    ]
  },
  {
    "id": "10026",
    "label": "Работа"
  }
]
"#,
    )
    .expect("expected temp category file to be writable");
}

pub fn seed_minimal_project_category_file(path: &Path) {
    fs::write(
        path,
        r#"[
  {
    "id": "book",
    "label": "Книга",
    "order": 0
  },
  {
    "id": "film",
    "label": "Фильм",
    "order": 1
  }
]
"#,
    )
    .expect("expected temp project category file to be writable");
}

fn unique_temp_file_path(prefix: &str, extension: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let counter = UNIQUE_TEST_PATH_COUNTER.fetch_add(1, Ordering::Relaxed);

    env::temp_dir().join(format!("{prefix}-{timestamp}-{counter}.{extension}"))
}
