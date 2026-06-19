import { invoke } from "@tauri-apps/api/core";

export type ProjectCategoryNode = {
  id: string;
  label: string;
  order: number;
};

export function loadProjectCategoryList() {
  return invoke<ProjectCategoryNode[]>("load_project_category_list");
}

export function saveProjectCategoryList(categories: ProjectCategoryNode[]) {
  return invoke<void>("save_project_category_list", { categories });
}
