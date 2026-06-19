import { describe, expect, it } from "vitest";
import {
  normalizeProjectCategoryEditorOrder,
  removeProjectCategoryEditorCategory,
  reorderProjectCategoryEditorCategory,
} from "../../../src/features/settings/ProjectCategoryEditor";

describe("ProjectCategoryEditor model helpers", () => {
  const categories = [
    { id: "film", label: "Фильм", order: 10 },
    { id: "book", label: "Книга", order: 20 },
    { id: "course", label: "Курс", order: 30 },
  ];

  it("normalizes order without changing stable ids or labels", () => {
    expect(normalizeProjectCategoryEditorOrder(categories)).toEqual([
      { id: "film", label: "Фильм", order: 0 },
      { id: "book", label: "Книга", order: 1 },
      { id: "course", label: "Курс", order: 2 },
    ]);
  });

  it("moves a category and renumbers the result", () => {
    expect(
      reorderProjectCategoryEditorCategory(categories, "course", "up"),
    ).toEqual([
      { id: "film", label: "Фильм", order: 0 },
      { id: "course", label: "Курс", order: 1 },
      { id: "book", label: "Книга", order: 2 },
    ]);
  });

  it("keeps the list unchanged when move target is outside the list", () => {
    expect(reorderProjectCategoryEditorCategory(categories, "film", "up")).toBe(
      categories,
    );
  });

  it("removes a category and renumbers remaining entries", () => {
    expect(removeProjectCategoryEditorCategory(categories, "book")).toEqual([
      { id: "film", label: "Фильм", order: 0 },
      { id: "course", label: "Курс", order: 1 },
    ]);
  });
});
