import { describe, expect, it } from "vitest";
import type { CategoryTreeEditorNode } from "../../../src/features/settings/categoryTreeEditorApi";
import {
  addRootCategory,
  canMoveCategory,
  deleteCategoryAtPath,
} from "../../../src/features/settings/categoryTreeEditorTree";

describe("categoryTreeEditorTree", () => {
  it("generates the first free five-digit id for new roots", () => {
    const tree: CategoryTreeEditorNode[] = [
      { id: "10000", label: "A" },
      { id: "10002", label: "B" },
    ];

    const result = addRootCategory(tree);

    expect(result.tree[2]?.id).toBe("10001");
    expect(result.selectedPath).toEqual([2]);
  });

  it("selects the parent when deleting the last child", () => {
    const tree: CategoryTreeEditorNode[] = [
      {
        id: "10000",
        label: "Root",
        children: [{ id: "10001", label: "Only child" }],
      },
    ];

    const result = deleteCategoryAtPath(tree, [0, 0]);

    expect(result.tree[0]?.children).toBeUndefined();
    expect(result.selectedPath).toEqual([0]);
  });

  it("reports move availability for sibling positions", () => {
    const tree: CategoryTreeEditorNode[] = [
      { id: "10000", label: "A" },
      { id: "10001", label: "B" },
    ];

    expect(canMoveCategory(tree, [0], "up")).toBe(false);
    expect(canMoveCategory(tree, [0], "down")).toBe(true);
    expect(canMoveCategory(tree, [1], "down")).toBe(false);
  });
});
