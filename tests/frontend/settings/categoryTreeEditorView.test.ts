import { describe, expect, it } from "vitest";
import type { CategoryTreeEditorNode } from "../../../src/features/settings/categoryTreeEditorApi";
import {
  hasCollapsedAncestor,
  isDescendantPath,
} from "../../../src/features/settings/categoryTreeEditorView";

const tree: CategoryTreeEditorNode[] = [
  {
    id: "10000",
    label: "Деньги",
    children: [
      {
        id: "10001",
        label: "Работа",
        children: [{ id: "10002", label: "Планирование" }],
      },
    ],
  },
];

describe("categoryTreeEditorView", () => {
  it("detects descendant paths correctly", () => {
    expect(isDescendantPath([0, 0, 0], [0])).toBe(true);
    expect(isDescendantPath([0, 0], [0, 0])).toBe(false);
    expect(isDescendantPath([1], [0])).toBe(false);
  });

  it("hides rows that have collapsed ancestors", () => {
    expect(hasCollapsedAncestor(tree, [0, 0, 0], new Set(["10000"]))).toBe(true);
    expect(hasCollapsedAncestor(tree, [0, 0, 0], new Set(["10001"]))).toBe(true);
    expect(hasCollapsedAncestor(tree, [0, 0], new Set(["10002"]))).toBe(false);
  });
});
