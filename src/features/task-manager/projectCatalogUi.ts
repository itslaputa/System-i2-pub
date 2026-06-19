import type { ProjectCategoryNode } from "../../services/tauri/projectCategories";

export const NO_PROJECT_CATEGORY_LABEL = "No type";
export const PROJECT_CATEGORY_FILTER_ALL = "__all__";
export const PROJECT_CATEGORY_FILTER_NONE = "__none__";

export type ProjectCategoryOption = {
  id: string | null;
  label: string;
};

export function buildProjectCategoryOptions(
  categories: ProjectCategoryNode[],
): ProjectCategoryOption[] {
  return [
    { id: null, label: NO_PROJECT_CATEGORY_LABEL },
    ...categories
      .slice()
      .sort((left, right) => left.order - right.order)
      .map((category) => ({
        id: category.id,
        label: category.label,
      })),
  ];
}

export function buildProjectCategoryLabelMap(categories: ProjectCategoryNode[]) {
  const labels = new Map<string, string>();

  for (const category of categories) {
    labels.set(category.id, category.label);
  }

  return labels;
}

export function getProjectCategoryLabel(
  projectCategoryId: string | null | undefined,
  categories: ProjectCategoryNode[],
) {
  if (!projectCategoryId) {
    return NO_PROJECT_CATEGORY_LABEL;
  }

  return (
    categories.find((category) => category.id === projectCategoryId)?.label ??
    NO_PROJECT_CATEGORY_LABEL
  );
}
