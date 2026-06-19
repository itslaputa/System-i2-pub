import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onMount,
} from "solid-js";
import "./categoryTreeEditor.css";
import {
  loadCategoryTreeEditorTree,
  type CategoryTreeChangeLogTimestamp,
  saveCategoryTreeEditorTree,
  type CategoryTreeEditorNode,
} from "./categoryTreeEditorApi";
import { createRuntimeBundleBackup } from "../../services/tauri/runtime";
import {
  MAX_CATEGORY_DEPTH,
  addChildCategory,
  addRootCategory,
  addSiblingCategory,
  canMoveCategory,
  cloneCategoryTree,
  deleteCategoryAtPath,
  flattenCategoryTree,
  getCategoryNodeAtPath,
  moveCategoryAtPath,
  updateCategoryNodeAtPath,
} from "./categoryTreeEditorTree";
import {
  hasCollapsedAncestor,
  isDescendantPath,
} from "./categoryTreeEditorView";

type CategoryTreeEditorProps = {
  onOpenChangeLog: () => void;
  onOpenStorageView: () => void;
};

function CategoryTreeEditor(props: CategoryTreeEditorProps) {
  const [tree, setTree] = createSignal<CategoryTreeEditorNode[]>([]);
  const [baselineTree, setBaselineTree] = createSignal<
    CategoryTreeEditorNode[]
  >([]);
  const [collapsedIds, setCollapsedIds] = createSignal<Set<string>>(new Set());
  const [selectedPath, setSelectedPath] = createSignal<number[] | null>(null);
  const [statusMessage, setStatusMessage] = createSignal("Category editor ready.");
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);
  const [isLoading, setIsLoading] = createSignal(true);
  const [isSaving, setIsSaving] = createSignal(false);
  const [isBackingUp, setIsBackingUp] = createSignal(false);
  const [pendingDeletePath, setPendingDeletePath] = createSignal<number[] | null>(null);
  const [isRenamingLabel, setIsRenamingLabel] = createSignal(false);
  const [labelDraft, setLabelDraft] = createSignal("");
  let labelInputRef: HTMLInputElement | undefined;

  const flatEntries = createMemo(() => flattenCategoryTree(tree()));
  const visibleEntries = createMemo(() =>
    flatEntries().filter((entry) => !hasCollapsedAncestor(tree(), entry.path, collapsedIds())),
  );
  const selectedEntry = createMemo(() =>
    flatEntries().find((entry) => pathsAreEqual(entry.path, selectedPath())),
  );
  const selectedNode = createMemo(() =>
    selectedPath() ? getCategoryNodeAtPath(tree(), selectedPath() ?? []) : null,
  );
  const hasUnsavedChanges = createMemo(
    () => JSON.stringify(tree()) !== JSON.stringify(baselineTree()),
  );
  const canAddChild = createMemo(
    () => (selectedPath()?.length ?? 0) > 0 && (selectedPath()?.length ?? 0) < MAX_CATEGORY_DEPTH,
  );
  const isDeleteArmed = createMemo(() =>
    pathsAreEqual(selectedPath(), pendingDeletePath()),
  );
  const isMutationLocked = createMemo(() => isLoading() || isSaving() || isBackingUp());

  onMount(() => {
    void reloadTree();
  });

  createEffect(() => {
    const node = selectedNode();
    setPendingDeletePath(null);
    setIsRenamingLabel(false);
    setLabelDraft(node?.label ?? "");
  });

  createEffect(() => {
    if (isRenamingLabel()) {
      labelInputRef?.focus();
      labelInputRef?.select();
    }
  });

  async function reloadTree() {
    setIsLoading(true);
    setErrorMessage(null);

    try {
      const loadedTree = await loadCategoryTreeEditorTree();
      const nextTree = cloneCategoryTree(loadedTree);
      setTree(nextTree);
      setBaselineTree(cloneCategoryTree(loadedTree));
      setCollapsedIds(new Set<string>());
      setSelectedPath(null);
      setPendingDeletePath(null);
      setStatusMessage("Category tree loaded from JSON.");
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsLoading(false);
    }
  }

  async function saveTree() {
    if (isRenamingLabel() && !commitLabelRename()) {
      return;
    }

    setIsSaving(true);
    setErrorMessage(null);

    try {
      const changedAt: CategoryTreeChangeLogTimestamp = new Date().toISOString();
      await saveCategoryTreeEditorTree(tree(), changedAt);
      await reloadTree();
      setStatusMessage("Category tree saved to JSON.");
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsSaving(false);
    }
  }

  async function handleCreateBackup() {
    if (isMutationLocked()) {
      return;
    }

    setIsBackingUp(true);
    setErrorMessage(null);
    setStatusMessage("Creating runtime backup...");

    try {
      const result = await createRuntimeBundleBackup();
      setStatusMessage(`Runtime backup saved: ${result.backupPath}`);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
      setStatusMessage("Runtime backup failed.");
    } finally {
      setIsBackingUp(false);
    }
  }

  function handleAddRoot() {
    if (isMutationLocked()) {
      return;
    }

    const result = addRootCategory(tree());
    setTree(result.tree);
    setSelectedPath(result.selectedPath);
    setStatusMessage("Root category added locally.");
  }

  function handleAddChild() {
    const path = selectedPath();

    if (isMutationLocked() || !path || path.length >= MAX_CATEGORY_DEPTH) {
      return;
    }

    const result = addChildCategory(tree(), path);
    setTree(result.tree);
    setSelectedPath(result.selectedPath);
    setStatusMessage("Child category added locally.");
  }

  function handleAddSibling() {
    const path = selectedPath();

    if (isMutationLocked() || !path) {
      return;
    }

    const result = addSiblingCategory(tree(), path);
    setTree(result.tree);
    setSelectedPath(result.selectedPath);
    setStatusMessage("Sibling category added locally.");
  }

  function handleDelete() {
    const entry = selectedEntry();

    if (isMutationLocked() || !entry) {
      return;
    }

    if (!isDeleteArmed()) {
      setPendingDeletePath([...entry.path]);
      setStatusMessage(`Press Delete again to remove "${entry.node.label}" and its nested children.`);
      return;
    }

    const result = deleteCategoryAtPath(tree(), entry.path);
    setTree(result.tree);
    setSelectedPath(result.selectedPath);
    setPendingDeletePath(null);
    setStatusMessage("Category removed locally.");
  }

  function handleMove(direction: "up" | "down") {
    const path = selectedPath();

    if (isMutationLocked() || !path) {
      return;
    }

    const result = moveCategoryAtPath(tree(), path, direction);
    setTree(result.tree);
    setSelectedPath(result.selectedPath);
    setStatusMessage(
      direction === "up"
        ? "Category moved up locally."
        : "Category moved down locally.",
    );
  }

  function toggleCollapsed(path: number[], categoryId: string) {
    setCollapsedIds((current) => {
      const next = new Set(current);
      const shouldCollapse = !next.has(categoryId);

      if (shouldCollapse) {
        next.add(categoryId);
      } else {
        next.delete(categoryId);
      }

      return next;
    });

    if (selectedPath() && isDescendantPath(selectedPath() ?? [], path)) {
      setSelectedPath(path);
    }
  }

  function startLabelRename() {
    if (isMutationLocked() || !selectedPath()) {
      return;
    }

    setLabelDraft(selectedNode()?.label ?? "");
    setIsRenamingLabel(true);
    setStatusMessage("Rename mode enabled for the selected category.");
  }

  function cancelLabelRename() {
    setLabelDraft(selectedNode()?.label ?? "");
    setIsRenamingLabel(false);
  }

  function commitLabelRename() {
    const path = selectedPath();
    const nextLabel = labelDraft().trim();

    if (!path) {
      return false;
    }

    if (!nextLabel) {
      setStatusMessage("Category label cannot be empty.");
      return false;
    }

    setTree((currentTree) =>
      updateCategoryNodeAtPath(currentTree, path, (node) => ({
        ...node,
        label: nextLabel,
      })),
    );
    setLabelDraft(nextLabel);
    setIsRenamingLabel(false);
    setStatusMessage("Category label updated locally.");

    return true;
  }

  return (
    <div class="settings-category-editor">
      <section class="dashboard settings-category-editor-shell">
        <article class="panel settings-category-editor-panel">
          <div class="panel-copy">
            <div>
              <p class="section-label">Category Settings</p>
              <h2>Category tree</h2>
            </div>
            <span class="stat-pill">
              {flatEntries().length} nodes
            </span>
          </div>

          <div class="settings-category-editor-toolbar">
            <button
              type="button"
              class="settings-category-editor-button"
              onClick={handleAddRoot}
              disabled={isMutationLocked()}
            >
              Add root
            </button>
            <button
              type="button"
              class="settings-category-editor-button is-primary"
              onClick={() => void saveTree()}
              disabled={isLoading() || isSaving() || !hasUnsavedChanges()}
            >
              {isSaving() ? "Saving..." : "Save changes"}
            </button>
          </div>

          <Show when={errorMessage()}>
            <p class="settings-category-editor-status is-error">
              {errorMessage()}
            </p>
          </Show>

          <Show when={!errorMessage()}>
            <p class="settings-category-editor-status">{statusMessage()}</p>
          </Show>

          <Show when={isLoading()}>
            <p class="settings-category-editor-status">Loading category tree...</p>
          </Show>

          <Show when={!isLoading() && flatEntries().length === 0}>
            <p class="settings-category-editor-empty">
              The tree is empty. Use Add root to start a new branch.
            </p>
          </Show>

          <Show when={!isLoading() && flatEntries().length > 0}>
            <div class="settings-category-editor-tree">
              <For each={visibleEntries()}>
                {(entry) => (
                  <div
                    class="settings-category-editor-row"
                    classList={{
                      "is-root": entry.depth === 1,
                      "is-nested": entry.depth > 1,
                      "is-selected": pathsAreEqual(entry.path, selectedPath()),
                    }}
                    style={{
                      "--row-offset": `${(entry.depth - 1) * 15}px`,
                    }}
                  >
                    <Show when={(entry.node.children?.length ?? 0) > 0}>
                      <button
                        type="button"
                        class="settings-category-editor-row-toggle"
                        classList={{
                          "is-collapsed": collapsedIds().has(entry.node.id),
                        }}
                        aria-label={
                          collapsedIds().has(entry.node.id)
                            ? `Expand ${entry.node.label}`
                            : `Collapse ${entry.node.label}`
                        }
                        aria-expanded={!collapsedIds().has(entry.node.id)}
                        onClick={() => toggleCollapsed(entry.path, entry.node.id)}
                      >
                        {collapsedIds().has(entry.node.id) ? "▸" : "▾"}
                      </button>
                    </Show>

                    <button
                      type="button"
                      class="settings-category-editor-row-select"
                      onClick={() => setSelectedPath(entry.path)}
                    >
                      <span class="settings-category-editor-row-title">
                        {entry.node.label}
                      </span>
                      <span class="settings-category-editor-row-meta">
                        <span>ID: {entry.node.id}</span>
                        <span>Level: {entry.depth}</span>
                      </span>
                    </button>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </article>

        <div class="settings-category-editor-inspector-rail">
          <article class="panel settings-category-editor-panel is-inspector">
            <div class="panel-copy">
              <div>
                <p class="section-label">Inspector</p>
                <h2>Selected node</h2>
              </div>
              <span class="stat-pill">
                {hasUnsavedChanges() ? "Unsaved" : "Saved"}
              </span>
            </div>

            <Show
              when={selectedEntry()}
              fallback={
                <p class="settings-category-editor-empty">
                  Select a category on the left to edit it.
                </p>
              }
            >
              <div class="settings-category-editor-inspector">
                <label class="settings-category-editor-field">
                  <span>Label</span>
                  <Show
                    when={isRenamingLabel()}
                    fallback={
                      <div class="settings-category-editor-inline-field">
                        <div class="settings-category-editor-readonly">
                          {selectedNode()?.label ?? ""}
                        </div>
                        <button
                          type="button"
                          class="settings-category-editor-button"
                          onClick={startLabelRename}
                        >
                          Rename
                        </button>
                      </div>
                    }
                  >
                    <div class="settings-category-editor-inline-field">
                      <input
                        ref={labelInputRef}
                        class="settings-category-editor-input"
                        type="text"
                        value={labelDraft()}
                        onInput={(event) => setLabelDraft(event.currentTarget.value)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter") {
                            event.preventDefault();
                            commitLabelRename();
                          }

                          if (event.key === "Escape") {
                            event.preventDefault();
                            cancelLabelRename();
                          }
                        }}
                      />
                      <button
                        type="button"
                        class="settings-category-editor-button is-primary"
                        onClick={commitLabelRename}
                        disabled={isMutationLocked()}
                      >
                        Done
                      </button>
                    </div>
                  </Show>
                </label>

                <label class="settings-category-editor-field">
                  <span>ID</span>
                  <div class="settings-category-editor-readonly">
                    {selectedNode()?.id ?? ""}
                  </div>
                </label>

                <div class="settings-category-editor-field">
                  <span>Path</span>
                  <div class="settings-category-editor-readonly">
                    {selectedEntry()?.displayPath}
                  </div>
                </div>

                <div class="settings-category-editor-field">
                  <span>Depth</span>
                  <div class="settings-category-editor-readonly">
                    {selectedEntry()?.depth} / {MAX_CATEGORY_DEPTH}
                  </div>
                </div>

                <div class="settings-category-editor-actions">
                  <button
                    type="button"
                    class="settings-category-editor-button"
                    onClick={handleAddChild}
                    disabled={isMutationLocked() || !canAddChild()}
                  >
                    Add child
                  </button>
                  <button
                    type="button"
                    class="settings-category-editor-button"
                    onClick={handleAddSibling}
                    disabled={isMutationLocked() || !selectedPath()}
                  >
                    Add sibling
                  </button>
                  <button
                    type="button"
                    class="settings-category-editor-button"
                    onClick={() => handleMove("up")}
                    disabled={
                      isMutationLocked() ||
                      !selectedPath() ||
                      !canMoveCategory(tree(), selectedPath() ?? [], "up")
                    }
                  >
                    Move up
                  </button>
                  <button
                    type="button"
                    class="settings-category-editor-button"
                    onClick={() => handleMove("down")}
                    disabled={
                      isMutationLocked() ||
                      !selectedPath() ||
                      !canMoveCategory(tree(), selectedPath() ?? [], "down")
                    }
                  >
                    Move down
                  </button>
                  <button
                    type="button"
                    class="settings-category-editor-button"
                    classList={{ "is-danger": isDeleteArmed() }}
                    onClick={handleDelete}
                    disabled={isMutationLocked() || !selectedEntry()}
                  >
                    {isDeleteArmed() ? "Confirm delete" : "Delete"}
                  </button>
                </div>
              </div>
            </Show>

            <button
              type="button"
              class="settings-category-editor-button settings-category-editor-log-button"
              onClick={props.onOpenChangeLog}
            >
              Open change log
            </button>
            <button
              type="button"
              class="settings-category-editor-button settings-category-editor-log-button"
              onClick={props.onOpenStorageView}
            >
              Open storage view
            </button>
            <button
              type="button"
              class="settings-category-editor-button settings-category-editor-log-button"
              onClick={() => void handleCreateBackup()}
              disabled={isMutationLocked()}
            >
              {isBackingUp() ? "Creating backup..." : "Create runtime backup"}
            </button>
          </article>
        </div>
      </section>
    </div>
  );
}

function pathsAreEqual(firstPath: number[] | null, secondPath: number[] | null) {
  if (!firstPath || !secondPath) {
    return false;
  }

  if (firstPath.length !== secondPath.length) {
    return false;
  }

  return firstPath.every((segment, index) => segment === secondPath[index]);
}

function getErrorMessage(error: unknown) {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === "string" && error.trim()) {
    return error;
  }

  return "Category editor failed.";
}

export default CategoryTreeEditor;
