import { invoke } from "@tauri-apps/api/core";

export type TaskRecord = {
  id: string;
  category_id: string;
  time_length: number;
  date: string;
  note: string | null;
  project_id: string | null;
  project_label: string | null;
  is_project_closing_task: boolean;
};

export type CreateTaskRecordInput = {
  categoryId: string;
  projectId: string | null;
  taskDate: string;
  durationMinutes: number;
  isProjectClosingTask: boolean;
  note: string | null;
};

export function createTaskRecord(input: CreateTaskRecordInput) {
  return invoke<TaskRecord>("create_task_record", { input });
}

export function loadTaskRecords() {
  return invoke<TaskRecord[]>("load_task_records");
}

export function deleteTaskRecord(taskId: string) {
  return invoke<void>("delete_task_record", { taskId });
}
