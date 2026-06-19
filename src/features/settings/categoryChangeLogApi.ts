import { invoke } from "@tauri-apps/api/core";

export function loadCategoryChangeLog() {
  return invoke<string[]>("load_task_category_change_log");
}
