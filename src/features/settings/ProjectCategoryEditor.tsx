import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onMount,
} from "solid-js";
import {
  loadProjectCategoryList,
  saveProjectCategoryList,
  type ProjectCategoryNode,
} from "../../services/tauri/projectCategories";
import {
  NO_PROJECT_CATEGORY_LABEL,
  getProjectCategoryLabel,
} from "../task-manager/projectCatalogUi";
import "../task-manager/projectCatalogWorkspace.css";
import "./projectCategoryEditor.css";

function createProjectCategoryId() {
  return `project-category-${Date.now().toString(36)}-${Math.random()
    .toString(36)
    .slice(2, 8)}`;
}

export function normalizeProjectCategoryEditorOrder(
  categories: ProjectCategoryNode[],
) {
  return categories.map((category, index) => ({
    ...category,
    order: index,
  }));
}

export function reorderProjectCategoryEditorCategory(
  categories: ProjectCategoryNode[],
  categoryId: string,
  direction: "up" | "down",
) {
  const currentIndex = categories.findIndex((category) => category.id === categoryId);

  if (currentIndex < 0) {
    return categories;
  }

  const targetIndex =
    direction === "up" ? currentIndex - 1 : currentIndex + 1;

  if (targetIndex < 0 || targetIndex >= categories.length) {
    return categories;
  }

  const nextCategories = categories.slice();
  const [movedCategory] = nextCategories.splice(currentIndex, 1);
  nextCategories.splice(targetIndex, 0, movedCategory);

  return normalizeProjectCategoryEditorOrder(nextCategories);
}

export function removeProjectCategoryEditorCategory(
  categories: ProjectCategoryNode[],
  categoryId: string,
) {
  return normalizeProjectCategoryEditorOrder(
    categories.filter((category) => category.id !== categoryId),
  );
}

function ProjectCategoryEditor() {
  const [categories, setCategories] = createSignal<ProjectCategoryNode[]>([]);
  const [baselineCategories, setBaselineCategories] = createSignal<
    ProjectCategoryNode[]
  >([]);
  const [selectedCategoryId, setSelectedCategoryId] = createSignal<string | null>(
    null,
  );
  const [newCategoryLabel, setNewCategoryLabel] = createSignal("");
  const [statusMessage, setStatusMessage] = createSignal(
    "Project category editor ready.",
  );
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);
  const [isLoading, setIsLoading] = createSignal(true);
  const [isSaving, setIsSaving] = createSignal(false);
  const [pendingDeleteCategoryId, setPendingDeleteCategoryId] =
    createSignal<string | null>(null);

  const orderedCategories = createMemo(() =>
    categories()
      .slice()
      .sort((left, right) => left.order - right.order),
  );
  const selectedCategory = createMemo(
    () =>
      orderedCategories().find(
        (category) => category.id === selectedCategoryId(),
      ) ?? null,
  );
  const selectedCategoryIndex = createMemo(() =>
    orderedCategories().findIndex((category) => category.id === selectedCategoryId()),
  );
  const hasUnsavedChanges = createMemo(
    () =>
      JSON.stringify(orderedCategories()) !== JSON.stringify(baselineCategories()),
  );

  onMount(() => {
    void reloadCategories();
  });

  createEffect(() => {
    const currentSelectedCategoryId = selectedCategoryId();
    const currentCategories = orderedCategories();

    if (
      currentSelectedCategoryId &&
      currentCategories.some((category) => category.id === currentSelectedCategoryId)
    ) {
      return;
    }

    setSelectedCategoryId(currentCategories[0]?.id ?? null);
  });

  createEffect(() => {
    selectedCategoryId();
    setPendingDeleteCategoryId(null);
  });

  async function reloadCategories() {
    setIsLoading(true);
    setErrorMessage(null);

    try {
      const loadedCategories = await loadProjectCategoryList();
      const nextCategories = normalizeProjectCategoryEditorOrder(loadedCategories);
      setCategories(nextCategories);
      setBaselineCategories(nextCategories);
      setSelectedCategoryId(nextCategories[0]?.id ?? null);
      setNewCategoryLabel("");
      setStatusMessage("Project categories loaded from runtime storage.");
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Failed to load project categories."));
    } finally {
      setIsLoading(false);
    }
  }

  async function handleSave() {
    setIsSaving(true);
    setErrorMessage(null);

    try {
      await saveProjectCategoryList(orderedCategories());
      await reloadCategories();
      setStatusMessage("Project categories saved.");
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Failed to save project categories."));
    } finally {
      setIsSaving(false);
    }
  }

  function handleAddCategory() {
    const normalizedLabel = newCategoryLabel().trim();

    if (!normalizedLabel) {
      setErrorMessage("Enter a category label.");
      return;
    }

    const createdCategory: ProjectCategoryNode = {
      id: createProjectCategoryId(),
      label: normalizedLabel,
      order: orderedCategories().length,
    };
    const nextCategories = normalizeProjectCategoryEditorOrder([
      ...orderedCategories(),
      createdCategory,
    ]);

    setCategories(nextCategories);
    setSelectedCategoryId(createdCategory.id);
    setNewCategoryLabel("");
    setErrorMessage(null);
    setStatusMessage("Project category added locally.");
  }

  function handleSelectedCategoryLabelChange(nextLabel: string) {
    const categoryId = selectedCategoryId();

    if (!categoryId) {
      return;
    }

    setCategories((currentCategories) =>
      currentCategories.map((category) =>
        category.id === categoryId
          ? { ...category, label: nextLabel }
          : category,
      ),
    );
    setErrorMessage(null);
  }

  function handleMove(direction: "up" | "down") {
    const categoryId = selectedCategoryId();

    if (!categoryId) {
      return;
    }

    setCategories((currentCategories) =>
      reorderProjectCategoryEditorCategory(currentCategories, categoryId, direction),
    );
    setStatusMessage(
      direction === "up"
        ? "Project category moved up locally."
        : "Project category moved down locally.",
    );
  }

  function handleDelete() {
    const category = selectedCategory();

    if (!category) {
      return;
    }

    if (pendingDeleteCategoryId() !== category.id) {
      setPendingDeleteCategoryId(category.id);
      setStatusMessage(
        `Press Delete again to remove "${category.label}" and remap linked projects to ${NO_PROJECT_CATEGORY_LABEL}.`,
      );
      return;
    }

    const nextCategories = removeProjectCategoryEditorCategory(
      orderedCategories(),
      category.id,
    );
    setCategories(nextCategories);
    setSelectedCategoryId(nextCategories[0]?.id ?? null);
    setPendingDeleteCategoryId(null);
    setStatusMessage("Project category removed locally.");
  }

  return (
    <section class="dashboard task-project-catalog-layout">
      <article class="panel task-project-catalog-panel">
        <div class="panel-copy">
          <div>
            <p class="section-label">Project Categories</p>
            <h2>Project category list</h2>
          </div>
          <span class="stat-pill">{orderedCategories().length} saved</span>
        </div>

        <div class="task-project-composer">
          <label class="field">
            <span>New category</span>
            <input
              class="input-field"
              type="text"
              value={newCategoryLabel()}
              placeholder="Add a project category"
              onInput={(event) => {
                setNewCategoryLabel(event.currentTarget.value);
                setErrorMessage(null);
              }}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  handleAddCategory();
                }
              }}
            />
          </label>

          <button
            type="button"
            class="action-button"
            disabled={isLoading() || isSaving()}
            onClick={handleAddCategory}
          >
            Add category
          </button>
        </div>

        <Show when={errorMessage()}>
          <p class="task-project-catalog-status is-error">{errorMessage()}</p>
        </Show>

        <Show when={!errorMessage()}>
          <p class="task-project-catalog-status">{statusMessage()}</p>
        </Show>

        <Show when={isLoading()}>
          <p class="task-project-catalog-status">Loading project categories...</p>
        </Show>

        <Show when={!isLoading() && orderedCategories().length === 0}>
          <p class="task-project-catalog-status">
            No project categories saved yet.
          </p>
        </Show>

        <Show when={!isLoading() && orderedCategories().length > 0}>
          <div class="task-project-catalog-list">
            <For each={orderedCategories()}>
              {(category) => (
                <button
                  type="button"
                  class="task-project-catalog-card"
                  classList={{
                    "is-selected": selectedCategoryId() === category.id,
                  }}
                  onClick={() => setSelectedCategoryId(category.id)}
                >
                  <div class="task-project-catalog-card-head">
                    <span class="task-project-catalog-title">{category.label}</span>
                    <span class="task-project-kind-badge">#{category.order + 1}</span>
                  </div>
                  <p class="task-project-catalog-meta">
                    id: {category.id}
                  </p>
                </button>
              )}
            </For>
          </div>
        </Show>
      </article>

      <article class="panel task-project-detail-panel">
        <div class="task-project-detail-stack">
          <div class="project-category-system-card">
            <div class="panel-copy">
              <div>
                <p class="section-label">System Fallback</p>
                <h2>{NO_PROJECT_CATEGORY_LABEL}</h2>
              </div>
              <span class="task-project-kind-badge">
                {getProjectCategoryLabel(null, orderedCategories())}
              </span>
            </div>
            <p class="panel-text">
              Projects without an assigned category fall back here. This entry is
              built into the system and is not editable.
            </p>
          </div>

          <Show
            when={selectedCategory()}
            fallback={
              <div class="task-project-detail-empty">
                <p class="section-label">Category Editor</p>
                <h2>Select a category</h2>
                <p class="panel-text">
                  Choose one from the list to rename, move, delete, reload, or save.
                </p>
              </div>
            }
          >
            {(category) => (
              <div class="task-project-detail-stack">
                <div class="panel-copy">
                  <div>
                    <p class="section-label">Category Editor</p>
                    <h2>{category().label}</h2>
                  </div>
                  <span class="task-project-kind-badge">#{category().order + 1}</span>
                </div>

                <label class="field">
                  <span>Label</span>
                  <input
                    class="input-field"
                    type="text"
                    value={category().label}
                    onInput={(event) =>
                      handleSelectedCategoryLabelChange(event.currentTarget.value)}
                  />
                </label>

                <div class="task-project-detail-grid">
                  <div class="task-project-detail-card">
                    <span>Category id</span>
                    <strong>{category().id}</strong>
                  </div>
                  <div class="task-project-detail-card">
                    <span>Order</span>
                    <strong>{category().order + 1}</strong>
                  </div>
                </div>

                <div class="task-project-detail-actions">
                  <button
                    type="button"
                    class="analytics-refresh-button"
                    disabled={selectedCategoryIndex() <= 0}
                    onClick={() => handleMove("up")}
                  >
                    Move up
                  </button>
                  <button
                    type="button"
                    class="analytics-refresh-button"
                    disabled={
                      selectedCategoryIndex() < 0 ||
                      selectedCategoryIndex() >= orderedCategories().length - 1
                    }
                    onClick={() => handleMove("down")}
                  >
                    Move down
                  </button>
                </div>

                <div class="task-project-detail-actions">
                  <button
                    type="button"
                    class="analytics-refresh-button"
                    disabled={isLoading() || isSaving()}
                    onClick={() => void reloadCategories()}
                  >
                    Reload
                  </button>
                  <button
                    type="button"
                    class="action-button"
                    disabled={isLoading() || isSaving() || !hasUnsavedChanges()}
                    onClick={() => void handleSave()}
                  >
                    {isSaving() ? "Saving..." : "Save"}
                  </button>
                </div>

                <div class="task-project-detail-actions">
                  <button
                    type="button"
                    class="record-delete-button"
                    classList={{
                      "is-danger-armed":
                        pendingDeleteCategoryId() === category().id,
                    }}
                    disabled={isLoading() || isSaving()}
                    onClick={handleDelete}
                  >
                    {pendingDeleteCategoryId() === category().id
                      ? "Are you sure?"
                      : "Delete"}
                  </button>
                </div>
              </div>
            )}
          </Show>
        </div>
      </article>
    </section>
  );
}

function getErrorMessage(error: unknown, fallback: string) {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === "string" && error.trim()) {
    return error;
  }

  return fallback;
}

export default ProjectCategoryEditor;
