import type { CategoryTreeEditorNode } from "./categoryTreeEditorApi";

export const MAX_CATEGORY_DEPTH = 5;

export type CategoryTreeFlatEntry = {
  node: CategoryTreeEditorNode;
  path: number[];
  depth: number;
  displayPath: string;
};

type MutationResult = {
  tree: CategoryTreeEditorNode[];
  selectedPath: number[] | null;
};

export function cloneCategoryTree(
  tree: CategoryTreeEditorNode[],
): CategoryTreeEditorNode[] {
  return tree.map(cloneCategoryNode);
}

export function flattenCategoryTree(
  tree: CategoryTreeEditorNode[],
  parentPath: number[] = [],
  parentLabels: string[] = [],
): CategoryTreeFlatEntry[] {
  return tree.flatMap((node, index) => {
    const path = [...parentPath, index];
    const labels = [...parentLabels, node.label];
    const entry: CategoryTreeFlatEntry = {
      node,
      path,
      depth: path.length,
      displayPath: labels.join(" / "),
    };
    const children = node.children
      ? flattenCategoryTree(node.children, path, labels)
      : [];

    return [entry, ...children];
  });
}

export function getCategoryNodeAtPath(
  tree: CategoryTreeEditorNode[],
  path: number[],
) {
  let currentList = tree;
  let currentNode: CategoryTreeEditorNode | null = null;

  for (const index of path) {
    currentNode = currentList[index] ?? null;

    if (!currentNode) {
      return null;
    }

    currentList = currentNode.children ?? [];
  }

  return currentNode;
}

export function updateCategoryNodeAtPath(
  tree: CategoryTreeEditorNode[],
  path: number[],
  updater: (node: CategoryTreeEditorNode) => CategoryTreeEditorNode,
) {
  const nextTree = cloneCategoryTree(tree);
  const targetNode = getMutableNodeAtPath(nextTree, path);

  if (!targetNode) {
    return tree;
  }

  const updatedNode = cleanupCategoryNode(updater(targetNode));

  if (path.length === 1) {
    nextTree[path[0]] = updatedNode;
    return nextTree;
  }

  const parentNode = getMutableNodeAtPath(nextTree, path.slice(0, -1));

  if (!parentNode?.children) {
    return tree;
  }

  parentNode.children[path[path.length - 1]] = updatedNode;
  parentNode.children = normalizeChildren(parentNode.children);

  return nextTree;
}

export function addRootCategory(
  tree: CategoryTreeEditorNode[],
): MutationResult {
  const nextTree = cloneCategoryTree(tree);
  const newNode = createCategoryTreeNode(nextTree, "New category");

  nextTree.push(newNode);

  return {
    tree: nextTree,
    selectedPath: [nextTree.length - 1],
  };
}

export function addChildCategory(
  tree: CategoryTreeEditorNode[],
  path: number[],
): MutationResult {
  if (path.length >= MAX_CATEGORY_DEPTH) {
    return { tree, selectedPath: path };
  }

  const nextTree = cloneCategoryTree(tree);
  const parentNode = getMutableNodeAtPath(nextTree, path);

  if (!parentNode) {
    return { tree, selectedPath: null };
  }

  const children = parentNode.children ? [...parentNode.children] : [];
  const newNode = createCategoryTreeNode(nextTree, "New child");

  children.push(newNode);
  parentNode.children = children;

  return {
    tree: nextTree,
    selectedPath: [...path, children.length - 1],
  };
}

export function addSiblingCategory(
  tree: CategoryTreeEditorNode[],
  path: number[],
): MutationResult {
  if (path.length === 0) {
    return { tree, selectedPath: null };
  }

  if (path.length === 1) {
    const nextTree = cloneCategoryTree(tree);
    const insertionIndex = path[0] + 1;
    const newNode = createCategoryTreeNode(nextTree, "New category");

    nextTree.splice(insertionIndex, 0, newNode);

    return {
      tree: nextTree,
      selectedPath: [insertionIndex],
    };
  }

  const nextTree = cloneCategoryTree(tree);
  const parentPath = path.slice(0, -1);
  const parentNode = getMutableNodeAtPath(nextTree, parentPath);

  if (!parentNode?.children) {
    return { tree, selectedPath: path };
  }

  const insertionIndex = path[path.length - 1] + 1;
  const newNode = createCategoryTreeNode(nextTree, "New category");

  parentNode.children.splice(insertionIndex, 0, newNode);

  return {
    tree: nextTree,
    selectedPath: [...parentPath, insertionIndex],
  };
}

export function deleteCategoryAtPath(
  tree: CategoryTreeEditorNode[],
  path: number[],
): MutationResult {
  if (path.length === 0) {
    return { tree, selectedPath: null };
  }

  const nextTree = cloneCategoryTree(tree);
  const removedIndex = path[path.length - 1];

  if (path.length === 1) {
    nextTree.splice(removedIndex, 1);

    return {
      tree: nextTree,
      selectedPath: resolveSelectionAfterDelete(nextTree.length, removedIndex),
    };
  }

  const parentPath = path.slice(0, -1);
  const parentNode = getMutableNodeAtPath(nextTree, parentPath);

  if (!parentNode?.children) {
    return { tree, selectedPath: path };
  }

  parentNode.children.splice(removedIndex, 1);
  const remainingChildren = normalizeChildren(parentNode.children);
  parentNode.children = remainingChildren;

  if (!remainingChildren) {
    return {
      tree: nextTree,
      selectedPath: parentPath.length ? parentPath : null,
    };
  }

  const nextIndex = Math.min(removedIndex, remainingChildren.length - 1);

  return {
    tree: nextTree,
    selectedPath: [...parentPath, nextIndex],
  };
}

export function moveCategoryAtPath(
  tree: CategoryTreeEditorNode[],
  path: number[],
  direction: "up" | "down",
): MutationResult {
  if (path.length === 0) {
    return { tree, selectedPath: null };
  }

  const nextTree = cloneCategoryTree(tree);
  const currentIndex = path[path.length - 1];

  if (path.length === 1) {
    const targetIndex =
      direction === "up" ? currentIndex - 1 : currentIndex + 1;

    if (targetIndex < 0 || targetIndex >= nextTree.length) {
      return { tree, selectedPath: path };
    }

    swapItems(nextTree, currentIndex, targetIndex);

    return {
      tree: nextTree,
      selectedPath: [targetIndex],
    };
  }

  const parentPath = path.slice(0, -1);
  const parentNode = getMutableNodeAtPath(nextTree, parentPath);

  if (!parentNode?.children) {
    return { tree, selectedPath: path };
  }

  const siblings = parentNode.children;
  const targetIndex =
    direction === "up" ? currentIndex - 1 : currentIndex + 1;

  if (targetIndex < 0 || targetIndex >= siblings.length) {
    return { tree, selectedPath: path };
  }

  swapItems(siblings, currentIndex, targetIndex);

  return {
    tree: nextTree,
    selectedPath: [...parentPath, targetIndex],
  };
}

export function canMoveCategory(
  tree: CategoryTreeEditorNode[],
  path: number[],
  direction: "up" | "down",
) {
  if (path.length === 0) {
    return false;
  }

  const currentIndex = path[path.length - 1];

  if (path.length === 1) {
    return direction === "up"
      ? currentIndex > 0
      : currentIndex < tree.length - 1;
  }

  const parentNode = getCategoryNodeAtPath(tree, path.slice(0, -1));
  const siblings = parentNode?.children ?? [];

  return direction === "up"
    ? currentIndex > 0
    : currentIndex < siblings.length - 1;
}

function cloneCategoryNode(
  node: CategoryTreeEditorNode,
): CategoryTreeEditorNode {
  return {
    id: node.id,
    label: node.label,
    children: node.children?.map(cloneCategoryNode),
  };
}

function getMutableNodeAtPath(
  tree: CategoryTreeEditorNode[],
  path: number[],
) {
  let currentList = tree;
  let currentNode: CategoryTreeEditorNode | null = null;

  for (const index of path) {
    currentNode = currentList[index] ?? null;

    if (!currentNode) {
      return null;
    }

    currentList = currentNode.children ?? [];
  }

  return currentNode;
}

function cleanupCategoryNode(node: CategoryTreeEditorNode) {
  return {
    ...node,
    children: normalizeChildren(node.children),
  };
}

function normalizeChildren(children?: CategoryTreeEditorNode[]) {
  if (!children || children.length === 0) {
    return undefined;
  }

  return children;
}

function resolveSelectionAfterDelete(length: number, removedIndex: number) {
  if (length === 0) {
    return null;
  }

  return [Math.min(removedIndex, length - 1)];
}

function swapItems<T>(items: T[], firstIndex: number, secondIndex: number) {
  const firstItem = items[firstIndex];

  items[firstIndex] = items[secondIndex];
  items[secondIndex] = firstItem;
}

function createCategoryTreeNode(
  tree: CategoryTreeEditorNode[],
  label: string,
): CategoryTreeEditorNode {
  const existingIds = new Set(collectCategoryIds(tree));

  return {
    id: generateFiveDigitCategoryId(existingIds),
    label,
  };
}

function collectCategoryIds(tree: CategoryTreeEditorNode[]): string[] {
  return tree.flatMap((node) => {
    const childIds = node.children ? collectCategoryIds(node.children) : [];
    return [node.id, ...childIds];
  });
}

function generateFiveDigitCategoryId(existingIds: Set<string>) {
  for (let candidate = 10000; candidate <= 99999; candidate += 1) {
    const candidateId = String(candidate);

    if (!existingIds.has(candidateId)) {
      return candidateId;
    }
  }

  throw new Error("category tree id space exhausted");
}
