import { invoke } from "@tauri-apps/api/core";
import {
  loadTaskCategoryTree,
  type TaskCategoryNode,
} from "../../services/tauri/taskCategories";

export type CategoryTreeEditorNode = TaskCategoryNode;
export type CategoryTreeChangeLogTimestamp = string;

export function loadCategoryTreeEditorTree() {
  return loadTaskCategoryTree();
}

export function saveCategoryTreeEditorTree(
  tree: CategoryTreeEditorNode[],
  changedAt: CategoryTreeChangeLogTimestamp,
) {
  return invoke<void>("save_task_category_tree", { tree, changedAt });
}
