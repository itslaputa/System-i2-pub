import {
  For,
  Show,
  createMemo,
  createResource,
  createSignal,
} from "solid-js";
import "./taskCategoryField.css";
import {
  loadTaskCategoryTree,
  type TaskCategoryNode,
} from "../../services/tauri/taskCategories";

export type TaskCategorySelection = {
  id: string;
  pathIds: string[];
  pathLabels: string[];
  displayPath: string;
};

type TaskCategoryFieldProps = {
  value: TaskCategorySelection | null;
  onChange: (selection: TaskCategorySelection) => void;
  placeholder?: string;
};

function resolveNodesByPath(pathIds: string[], tree: TaskCategoryNode[]) {
  const resolvedPath: TaskCategoryNode[] = [];
  let currentLevel = tree;

  for (const pathId of pathIds) {
    const currentNode = currentLevel.find((node) => node.id === pathId);

    if (!currentNode) break;

    resolvedPath.push(currentNode);
    currentLevel = currentNode.children ?? [];
  }

  return resolvedPath;
}

function createSelection(pathNodes: TaskCategoryNode[]): TaskCategorySelection {
  const pathIds = pathNodes.map((node) => node.id);
  const pathLabels = pathNodes.map((node) => node.label);
  const leafNode = pathNodes[pathNodes.length - 1];

  return {
    id: leafNode.id,
    pathIds,
    pathLabels,
    displayPath: pathLabels.join(" / "),
  };
}

function TaskCategoryField(props: TaskCategoryFieldProps) {
  const [isOpen, setIsOpen] = createSignal(false);
  const [categoryTree] = createResource(loadTaskCategoryTree);
  const [activePath, setActivePath] = createSignal<TaskCategoryNode[]>([]);

  const currentBranchBasePath = createMemo(() => {
    const path = activePath();

    if (!path.length) {
      return [] as TaskCategoryNode[];
    }

    const selectedNode = path[path.length - 1];
    return selectedNode.children?.length ? path : path.slice(0, -1);
  });

  const currentOptions = createMemo(() => {
    const path = currentBranchBasePath();
    const tree = categoryTree() ?? [];

    if (!path.length) {
      return tree;
    }

    return path[path.length - 1]?.children ?? tree;
  });

  function openSelector() {
    setActivePath(resolveNodesByPath(props.value?.pathIds ?? [], categoryTree() ?? []));
    setIsOpen(true);
  }

  function closeSelector() {
    setIsOpen(false);
  }

  function handleNodeClick(node: TaskCategoryNode) {
    const nextPath = [...currentBranchBasePath(), node];
    setActivePath(nextPath);
    props.onChange(createSelection(nextPath));

    if (!node.children?.length) {
      setIsOpen(false);
    }
  }

  function jumpToPath(index: number) {
    if (index < 0) {
      setActivePath([]);
      return;
    }

    setActivePath(activePath().slice(0, index + 1));
  }

  return (
    <div class="task-category-field">
      <button
        type="button"
        class="task-category-trigger"
        aria-expanded={isOpen()}
        onClick={() => (isOpen() ? closeSelector() : openSelector())}
      >
        <span
          class="task-category-trigger-label"
          classList={{ "is-placeholder": !props.value }}
        >
          {props.value?.displayPath ?? props.placeholder ?? "Select category"}
        </span>
        <span class="task-category-trigger-meta">
          {isOpen() ? "Close" : "Browse"}
        </span>
      </button>

      <Show when={isOpen()}>
        <div class="task-category-panel" role="dialog" aria-label="Category selector">
          <Show when={categoryTree.loading}>
            <p class="task-category-status">Loading categories...</p>
          </Show>

          <Show when={categoryTree.error}>
            <p class="task-category-status is-error">
              Failed to load category tree.
            </p>
          </Show>

          <Show when={activePath().length > 0}>
            <div class="task-category-breadcrumbs">
              <button
                type="button"
                class="task-category-crumb"
                onClick={() => jumpToPath(-1)}
              >
                All categories
              </button>
            </div>
          </Show>

          <Show when={!categoryTree.loading && !categoryTree.error}>
            <div class="task-category-options">
              <For each={currentOptions()}>
                {(node) => (
                  <button
                    type="button"
                    class="task-category-option"
                    onClick={() => handleNodeClick(node)}
                  >
                    <span class="task-category-option-title">{node.label}</span>
                    <span class="task-category-option-meta">
                      {node.children?.length
                        ? `${node.children.length} subcategories`
                        : "Leaf category"}
                    </span>
                  </button>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}

export default TaskCategoryField;
