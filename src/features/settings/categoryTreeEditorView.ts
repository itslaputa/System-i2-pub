import type { CategoryTreeEditorNode } from "./categoryTreeEditorApi";
import { getCategoryNodeAtPath } from "./categoryTreeEditorTree";

export function isDescendantPath(path: number[], ancestorPath: number[]) {
  if (path.length <= ancestorPath.length) {
    return false;
  }

  return ancestorPath.every((segment, index) => path[index] === segment);
}

export function hasCollapsedAncestor(
  tree: CategoryTreeEditorNode[],
  path: number[],
  collapsedIds: Set<string>,
) {
  for (let depth = 1; depth < path.length; depth += 1) {
    const ancestorNode = getCategoryNodeAtPath(tree, path.slice(0, depth));
    if (ancestorNode && collapsedIds.has(ancestorNode.id)) {
      return true;
    }
  }

  return false;
}
