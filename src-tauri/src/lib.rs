mod analytics;
mod categories;
mod project_categories;
mod projects;
mod runtime;
pub mod storage;
mod tasks;
#[cfg(test)]
mod test_support;

#[tauri::command]
fn load_task_category_tree() -> Result<Vec<categories::TaskCategoryNode>, String> {
    categories::load_task_category_tree()
}

#[tauri::command]
fn save_task_category_tree(
    tree: Vec<categories::TaskCategoryNode>,
    changed_at: String,
) -> Result<(), String> {
    categories::save_task_category_tree(tree, changed_at)
}

#[tauri::command]
fn load_task_category_change_log() -> Result<Vec<String>, String> {
    categories::load_task_category_change_log()
}

#[tauri::command]
fn load_project_category_list() -> Result<Vec<project_categories::ProjectCategoryNode>, String> {
    project_categories::load_project_category_list()
}

#[tauri::command]
fn save_project_category_list(
    categories: Vec<project_categories::ProjectCategoryNode>,
) -> Result<(), String> {
    project_categories::save_project_category_list(categories)
}

#[tauri::command]
fn load_task_project_list() -> Result<Vec<projects::TaskProjectNode>, String> {
    projects::load_task_project_list()
}

#[tauri::command]
fn load_active_task_project_list() -> Result<Vec<projects::TaskProjectNode>, String> {
    projects::load_active_task_project_list()
}

#[tauri::command]
fn add_task_project(
    input: projects::CreateTaskProjectInput,
) -> Result<projects::TaskProjectNode, String> {
    projects::add_task_project(input)
}

#[tauri::command]
fn set_task_project_category(
    project_id: String,
    project_category_id: Option<String>,
) -> Result<(), String> {
    projects::set_task_project_category(project_id, project_category_id)
}

#[tauri::command]
fn delete_task_project(project_id: String) -> Result<(), String> {
    projects::delete_task_project(project_id)
}

#[tauri::command]
fn close_task_project(project_id: String) -> Result<(), String> {
    projects::close_task_project(project_id)
}

#[tauri::command]
fn reopen_task_project(project_id: String) -> Result<(), String> {
    projects::reopen_task_project(project_id)
}

#[tauri::command]
fn create_task_record(input: tasks::CreateTaskInput) -> Result<tasks::TaskRecord, String> {
    tasks::create_task_record(input)
}

#[tauri::command]
fn load_task_records() -> Result<Vec<tasks::TaskRecord>, String> {
    tasks::load_task_records()
}

#[tauri::command]
fn delete_task_record(task_id: String) -> Result<(), String> {
    tasks::delete_task_record(task_id)
}

#[tauri::command]
fn load_analytics_dashboard(
    input: analytics::AnalyticsDashboardInput,
) -> Result<analytics::AnalyticsDashboardData, String> {
    analytics::load_dashboard(input)
}

#[tauri::command]
fn get_runtime_status() -> runtime::RuntimeStatus {
    runtime::load_runtime_status()
}

#[tauri::command]
fn attach_existing_runtime_bundle(
    input: runtime::RuntimeBundleFolderInput,
) -> Result<runtime::RuntimeStatus, String> {
    runtime::attach_existing_runtime_bundle(input)
}

#[tauri::command]
fn create_runtime_bundle_in_folder(
    input: runtime::RuntimeBundleFolderInput,
) -> Result<runtime::RuntimeStatus, String> {
    runtime::create_runtime_bundle_in_folder(input)
}

#[tauri::command]
fn create_runtime_bundle_in_app_data_dir() -> Result<runtime::RuntimeStatus, String> {
    runtime::create_runtime_bundle_in_app_data_dir()
}

#[tauri::command]
fn create_runtime_bundle_backup() -> Result<runtime::RuntimeBackupResult, String> {
    runtime::create_runtime_bundle_backup()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime_status = runtime::load_runtime_status();

    if runtime_status.is_ready {
        if let Err(error) = storage::init_database() {
            eprintln!("failed to initialize sqlite database: {error}");
        }
    } else {
        eprintln!(
            "runtime bundle not ready; source={:?}; missing={:?}; invalid={:?}",
            runtime_status.source, runtime_status.missing, runtime_status.invalid
        );
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_runtime_status,
            attach_existing_runtime_bundle,
            create_runtime_bundle_in_folder,
            create_runtime_bundle_in_app_data_dir,
            create_runtime_bundle_backup,
            load_task_category_tree,
            save_task_category_tree,
            load_task_category_change_log,
            load_project_category_list,
            save_project_category_list,
            load_task_project_list,
            load_active_task_project_list,
            add_task_project,
            set_task_project_category,
            delete_task_project,
            close_task_project,
            reopen_task_project,
            create_task_record,
            load_task_records,
            delete_task_record,
            load_analytics_dashboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
