import { invoke } from "@tauri-apps/api/core";

export type TaskProjectNode = {
  id: string;
  label: string;
  project_category_id: string | null;
  sum_time_length: number;
  start_date: string | null;
  end_date: string | null;
  is_done: boolean;
  tasks: string[];
};

export type CreateTaskProjectInput = {
  label: string;
  projectCategoryId?: string | null;
};

export function loadTaskProjectList() {
  return invoke<TaskProjectNode[]>("load_task_project_list");
}

export function loadActiveTaskProjectList() {
  return invoke<TaskProjectNode[]>("load_active_task_project_list");
}

export function addTaskProject(input: CreateTaskProjectInput) {
  return invoke<TaskProjectNode>("add_task_project", { input });
}

export function setTaskProjectCategory(
  projectId: string,
  projectCategoryId: string | null,
) {
  return invoke<void>("set_task_project_category", {
    projectId,
    projectCategoryId,
  });
}

export function deleteTaskProject(projectId: string) {
  return invoke<void>("delete_task_project", { projectId });
}

export function closeTaskProject(projectId: string) {
  return invoke<void>("close_task_project", { projectId });
}

export function reopenTaskProject(projectId: string) {
  return invoke<void>("reopen_task_project", { projectId });
}
