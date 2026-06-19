import { invoke } from "@tauri-apps/api/core";

export type TaskCategoryNode = {
  id: string;
  label: string;
  children?: TaskCategoryNode[];
};

export function loadTaskCategoryTree() {
  return invoke<TaskCategoryNode[]>("load_task_category_tree");
}

export function buildTaskCategoryLabelMap(
  tree: TaskCategoryNode[],
  labels = new Map<string, string>(),
) {
  for (const node of tree) {
    labels.set(node.id, node.label);

    if (node.children?.length) {
      buildTaskCategoryLabelMap(node.children, labels);
    }
  }

  return labels;
}
