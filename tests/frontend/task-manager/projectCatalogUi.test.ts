import { describe, expect, it } from "vitest";
import {
  NO_PROJECT_CATEGORY_LABEL,
  PROJECT_CATEGORY_FILTER_ALL,
  PROJECT_CATEGORY_FILTER_NONE,
  buildProjectCategoryLabelMap,
  buildProjectCategoryOptions,
  getProjectCategoryLabel,
} from "../../../src/features/task-manager/projectCatalogUi";

describe("projectCatalogUi", () => {
  const categories = [
    { id: "film", label: "Фильм", order: 1 },
    { id: "book", label: "Книга", order: 0 },
  ];

  it("builds sorted options with the system no-type fallback first", () => {
    expect(buildProjectCategoryOptions(categories)).toEqual([
      { id: null, label: NO_PROJECT_CATEGORY_LABEL },
      { id: "book", label: "Книга" },
      { id: "film", label: "Фильм" },
    ]);
  });

  it("resolves category labels and falls back to no type", () => {
    expect(getProjectCategoryLabel("book", categories)).toBe("Книга");
    expect(getProjectCategoryLabel(null, categories)).toBe(NO_PROJECT_CATEGORY_LABEL);
    expect(getProjectCategoryLabel("missing", categories)).toBe(
      NO_PROJECT_CATEGORY_LABEL,
    );
  });

  it("builds a label map and keeps filter sentinels stable", () => {
    expect(Array.from(buildProjectCategoryLabelMap(categories).entries())).toEqual([
      ["film", "Фильм"],
      ["book", "Книга"],
    ]);
    expect(PROJECT_CATEGORY_FILTER_ALL).toBe("__all__");
    expect(PROJECT_CATEGORY_FILTER_NONE).toBe("__none__");
  });
});
