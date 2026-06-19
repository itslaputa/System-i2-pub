import {
  For,
  Show,
  createEffect,
  createMemo,
  createResource,
  createSignal,
} from "solid-js";
import type { ProjectCategoryNode } from "../../services/tauri/projectCategories";
import {
  addTaskProject,
  closeTaskProject,
  deleteTaskProject,
  loadTaskProjectList,
  reopenTaskProject,
  setTaskProjectCategory,
  type TaskProjectNode,
} from "../../services/tauri/taskProjects";
import { formatHoursMinutes } from "../../utils/formatDuration";
import type { TaskProjectSelection } from "./TaskProjectField";
import {
  PROJECT_CATEGORY_FILTER_ALL,
  PROJECT_CATEGORY_FILTER_NONE,
  buildProjectCategoryOptions,
  getProjectCategoryLabel,
} from "./projectCatalogUi";
import "./projectCatalogWorkspace.css";

type ProjectCatalogWorkspaceProps = {
  projectCategories: ProjectCategoryNode[];
  selectedTaskProjectId: string | null;
  onApplyTaskProjectSelection: (selection: TaskProjectSelection | null) => void;
  onReturnToTaskMode: () => void;
};

type StatusFilter = "all" | "active" | "done";
type CategoryFilter =
  | typeof PROJECT_CATEGORY_FILTER_ALL
  | typeof PROJECT_CATEGORY_FILTER_NONE
  | string;

const STATUS_FILTER_OPTIONS: ReadonlyArray<{
  id: StatusFilter;
  label: string;
}> = [
  { id: "all", label: "All statuses" },
  { id: "active", label: "Active" },
  { id: "done", label: "Done" },
];

function getErrorMessage(error: unknown, fallback: string) {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === "string" && error.trim()) {
    return error;
  }

  return fallback;
}

function formatProjectWindow(project: TaskProjectNode) {
  if (!project.start_date && !project.end_date) {
    return "No task window yet";
  }

  if (project.start_date && project.end_date) {
    return `${project.start_date} -> ${project.end_date}`;
  }

  return project.start_date ?? project.end_date ?? "No task window yet";
}

function ProjectCatalogWorkspace(props: ProjectCatalogWorkspaceProps) {
  const [projectList, { refetch }] = createResource(loadTaskProjectList);
  const [selectedProjectId, setSelectedProjectId] = createSignal<string | null>(
    props.selectedTaskProjectId,
  );
  const [searchQuery, setSearchQuery] = createSignal("");
  const [statusFilter, setStatusFilter] = createSignal<StatusFilter>("all");
  const [categoryFilter, setCategoryFilter] =
    createSignal<CategoryFilter>(PROJECT_CATEGORY_FILTER_ALL);
  const [newProjectLabel, setNewProjectLabel] = createSignal("");
  const [newProjectCategoryId, setNewProjectCategoryId] =
    createSignal<string | null>(null);
  const [catalogError, setCatalogError] = createSignal<string | null>(null);
  const [isAddingProject, setIsAddingProject] = createSignal(false);
  const [closingProjectId, setClosingProjectId] = createSignal<string | null>(null);
  const [reopeningProjectId, setReopeningProjectId] =
    createSignal<string | null>(null);
  const [deletingProjectId, setDeletingProjectId] = createSignal<string | null>(
    null,
  );
  const [savingCategoryProjectId, setSavingCategoryProjectId] =
    createSignal<string | null>(null);
  const [pendingDeleteProjectId, setPendingDeleteProjectId] =
    createSignal<string | null>(null);

  const projectCategoryOptions = createMemo(() =>
    buildProjectCategoryOptions(props.projectCategories),
  );

  const filteredProjects = createMemo(() => {
    const normalizedQuery = searchQuery().trim().toLowerCase();

    return (projectList() ?? []).filter((project) => {
      const matchesQuery =
        normalizedQuery.length === 0 ||
        project.label.toLowerCase().includes(normalizedQuery);
      const matchesStatus =
        statusFilter() === "all" ||
        (statusFilter() === "active" && !project.is_done) ||
        (statusFilter() === "done" && project.is_done);
      const matchesCategory =
        categoryFilter() === PROJECT_CATEGORY_FILTER_ALL ||
        (categoryFilter() === PROJECT_CATEGORY_FILTER_NONE
          ? project.project_category_id === null
          : categoryFilter() === project.project_category_id);

      return matchesQuery && matchesStatus && matchesCategory;
    });
  });

  const selectedProject = createMemo(
    () =>
      filteredProjects().find((project) => project.id === selectedProjectId()) ??
      null,
  );

  createEffect(() => {
    const nextSelectedTaskProjectId = props.selectedTaskProjectId;
    const visibleProjects = filteredProjects();

    if (
      nextSelectedTaskProjectId &&
      visibleProjects.some((project) => project.id === nextSelectedTaskProjectId)
    ) {
      setSelectedProjectId(nextSelectedTaskProjectId);
      return;
    }

    const currentSelectedProjectId = selectedProjectId();
    if (
      currentSelectedProjectId &&
      visibleProjects.some((project) => project.id === currentSelectedProjectId)
    ) {
      return;
    }

    setSelectedProjectId(visibleProjects[0]?.id ?? null);
  });

  createEffect(() => {
    selectedProjectId();
    setPendingDeleteProjectId(null);
  });

  async function refreshProjectList() {
    await refetch();
  }

  async function handleAddProject() {
    const normalizedLabel = newProjectLabel().trim();

    if (!normalizedLabel) {
      setCatalogError("Enter a project name.");
      return;
    }

    setIsAddingProject(true);
    setCatalogError(null);

    try {
      const createdProject = await addTaskProject({
        label: normalizedLabel,
        projectCategoryId: newProjectCategoryId(),
      });
      setNewProjectLabel("");
      setNewProjectCategoryId(null);
      await refreshProjectList();
      setSelectedProjectId(createdProject.id);
    } catch (error) {
      setCatalogError(getErrorMessage(error, "Failed to create project."));
    } finally {
      setIsAddingProject(false);
    }
  }

  async function handleCloseProject(project: TaskProjectNode) {
    setClosingProjectId(project.id);
    setCatalogError(null);

    try {
      await closeTaskProject(project.id);

      if (props.selectedTaskProjectId === project.id) {
        props.onApplyTaskProjectSelection(null);
      }

      await refreshProjectList();
    } catch (error) {
      setCatalogError(getErrorMessage(error, "Failed to close project."));
    } finally {
      setClosingProjectId(null);
    }
  }

  async function handleReopenProject(project: TaskProjectNode) {
    setReopeningProjectId(project.id);
    setCatalogError(null);

    try {
      await reopenTaskProject(project.id);
      await refreshProjectList();
    } catch (error) {
      setCatalogError(getErrorMessage(error, "Failed to reopen project."));
    } finally {
      setReopeningProjectId(null);
    }
  }

  async function handleProjectCategoryChange(
    project: TaskProjectNode,
    projectCategoryId: string | null,
  ) {
    if (project.project_category_id === projectCategoryId) {
      return;
    }

    setSavingCategoryProjectId(project.id);
    setCatalogError(null);

    try {
      await setTaskProjectCategory(project.id, projectCategoryId);

      if (props.selectedTaskProjectId === project.id) {
        props.onApplyTaskProjectSelection({
          id: project.id,
          label: project.label,
          projectCategoryId,
        });
      }

      await refreshProjectList();
    } catch (error) {
      setCatalogError(
        getErrorMessage(error, "Failed to update project category."),
      );
    } finally {
      setSavingCategoryProjectId(null);
    }
  }

  async function handleDeleteProject(project: TaskProjectNode) {
    if (pendingDeleteProjectId() !== project.id) {
      setPendingDeleteProjectId(project.id);
      setCatalogError(null);
      return;
    }

    setDeletingProjectId(project.id);
    setCatalogError(null);

    try {
      await deleteTaskProject(project.id);

      if (props.selectedTaskProjectId === project.id) {
        props.onApplyTaskProjectSelection(null);
      }

      await refreshProjectList();
    } catch (error) {
      setCatalogError(getErrorMessage(error, "Failed to delete project."));
    } finally {
      setDeletingProjectId(null);
      setPendingDeleteProjectId(null);
    }
  }

  function applyProjectToTaskDraft(project: TaskProjectNode) {
    setPendingDeleteProjectId(null);
    props.onApplyTaskProjectSelection({
      id: project.id,
      label: project.label,
      projectCategoryId: project.project_category_id,
    });
    props.onReturnToTaskMode();
  }

  return (
    <section class="dashboard task-project-catalog-layout">
      <article class="panel task-project-catalog-panel">
        <div class="panel-copy">
          <div>
            <p class="section-label">Project Catalog</p>
            <h2>Projects</h2>
          </div>
          <span class="stat-pill">{filteredProjects().length} visible</span>
        </div>

        <div class="task-project-composer">
          <label class="field">
            <span>Project name</span>
            <input
              class="input-field"
              type="text"
              value={newProjectLabel()}
              placeholder="New project label"
              onInput={(event) => {
                setNewProjectLabel(event.currentTarget.value);
                setCatalogError(null);
              }}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  void handleAddProject();
                }
              }}
            />
          </label>

          <div class="field">
            <span>Project type</span>
            <div class="task-project-kind-grid">
              <For each={projectCategoryOptions()}>
                {(option) => (
                  <button
                    type="button"
                    class="task-project-kind-button"
                    classList={{
                      "is-active": newProjectCategoryId() === option.id,
                    }}
                    onClick={() => setNewProjectCategoryId(option.id)}
                  >
                    {option.label}
                  </button>
                )}
              </For>
            </div>
          </div>

          <button
            type="button"
            class="action-button"
            disabled={isAddingProject()}
            onClick={() => void handleAddProject()}
          >
            {isAddingProject() ? "Creating..." : "Create project"}
          </button>
        </div>

        <div class="task-project-filter-stack">
          <label class="field">
            <span>Search</span>
            <input
              class="input-field"
              type="text"
              value={searchQuery()}
              placeholder="Find a project"
              onInput={(event) => setSearchQuery(event.currentTarget.value)}
            />
          </label>

          <div class="field">
            <span>Status</span>
            <div class="task-project-filter-group">
              <For each={STATUS_FILTER_OPTIONS}>
                {(option) => (
                  <button
                    type="button"
                    class="task-project-filter-chip"
                    classList={{ "is-active": statusFilter() === option.id }}
                    onClick={() => setStatusFilter(option.id)}
                  >
                    {option.label}
                  </button>
                )}
              </For>
            </div>
          </div>

          <div class="field">
            <span>Type</span>
            <div class="task-project-filter-group">
              <button
                type="button"
                class="task-project-filter-chip"
                classList={{
                  "is-active": categoryFilter() === PROJECT_CATEGORY_FILTER_ALL,
                }}
                onClick={() => setCategoryFilter(PROJECT_CATEGORY_FILTER_ALL)}
              >
                All types
              </button>
              <For each={projectCategoryOptions()}>
                {(option) => (
                  <button
                    type="button"
                    class="task-project-filter-chip"
                    classList={{
                      "is-active":
                        categoryFilter() ===
                        (option.id ?? PROJECT_CATEGORY_FILTER_NONE),
                    }}
                    onClick={() =>
                      setCategoryFilter(
                        option.id ?? PROJECT_CATEGORY_FILTER_NONE,
                      )
                    }
                  >
                    {option.label}
                  </button>
                )}
              </For>
            </div>
          </div>
        </div>

        <Show when={catalogError()}>
          <p class="task-project-catalog-status is-error">{catalogError()}</p>
        </Show>

        <Show when={projectList.loading}>
          <p class="task-project-catalog-status">Loading projects...</p>
        </Show>

        <Show when={projectList.error}>
          <p class="task-project-catalog-status is-error">
            Failed to load project catalog.
          </p>
        </Show>

        <Show
          when={
            !projectList.loading &&
            !projectList.error &&
            filteredProjects().length === 0
          }
        >
          <p class="task-project-catalog-status">
            No projects match the current filters.
          </p>
        </Show>

        <Show
          when={
            !projectList.loading &&
            !projectList.error &&
            filteredProjects().length > 0
          }
        >
          <div class="task-project-catalog-list">
            <For each={filteredProjects()}>
              {(project) => (
                <button
                  type="button"
                  class="task-project-catalog-card"
                  classList={{ "is-selected": selectedProjectId() === project.id }}
                  onClick={() => setSelectedProjectId(project.id)}
                >
                  <div class="task-project-catalog-card-head">
                    <span class="task-project-catalog-title">{project.label}</span>
                    <span class="task-project-kind-badge">
                      {getProjectCategoryLabel(
                        project.project_category_id,
                        props.projectCategories,
                      )}
                    </span>
                  </div>
                  <p class="task-project-catalog-meta">
                    {project.is_done ? "Done" : "Active"} ·{" "}
                    {formatHoursMinutes(project.sum_time_length)} ·{" "}
                    {project.tasks.length} tasks
                  </p>
                </button>
              )}
            </For>
          </div>
        </Show>
      </article>

      <article class="panel task-project-detail-panel">
        <Show
          when={selectedProject()}
          fallback={
            <div class="task-project-detail-empty">
              <p class="section-label">Project Editor</p>
              <h2>Select a project</h2>
              <p class="panel-text">Choose one from the catalog to the left.</p>
            </div>
          }
        >
          {(project) => (
            <div class="task-project-detail-stack">
              <div class="panel-copy">
                <div>
                  <p class="section-label">Project Editor</p>
                  <h2>{project().label}</h2>
                </div>
                <span class="task-project-kind-badge">
                  {getProjectCategoryLabel(
                    project().project_category_id,
                    props.projectCategories,
                  )}
                </span>
              </div>

              <div class="task-project-detail-grid">
                <div class="task-project-detail-card">
                  <span>Status</span>
                  <strong>{project().is_done ? "Done" : "Active"}</strong>
                </div>
                <div class="task-project-detail-card">
                  <span>Total time</span>
                  <strong>{formatHoursMinutes(project().sum_time_length)}</strong>
                </div>
                <div class="task-project-detail-card">
                  <span>Task count</span>
                  <strong>{project().tasks.length}</strong>
                </div>
                <div class="task-project-detail-card">
                  <span>Window</span>
                  <strong>{formatProjectWindow(project())}</strong>
                </div>
              </div>

              <div class="field">
                <span>Project type</span>
                <div class="task-project-kind-grid">
                  <For each={projectCategoryOptions()}>
                    {(option) => (
                      <button
                        type="button"
                        class="task-project-kind-button"
                        classList={{
                          "is-active": project().project_category_id === option.id,
                        }}
                        disabled={savingCategoryProjectId() === project().id}
                        onClick={() =>
                          void handleProjectCategoryChange(project(), option.id)
                        }
                      >
                        {option.label}
                      </button>
                    )}
                  </For>
                </div>
              </div>

              <div class="task-project-detail-actions">
                <button
                  type="button"
                  class="action-button"
                  onClick={() => applyProjectToTaskDraft(project())}
                >
                  Use in task draft
                </button>
                <button
                  type="button"
                  class="analytics-refresh-button"
                  onClick={() => {
                    setPendingDeleteProjectId(null);
                    props.onReturnToTaskMode();
                  }}
                >
                  Back to task form
                </button>
              </div>

              <div class="task-project-detail-actions">
                <Show
                  when={project().is_done}
                  fallback={
                    <button
                      type="button"
                      class="analytics-refresh-button"
                      disabled={closingProjectId() === project().id}
                      onClick={() => {
                        setPendingDeleteProjectId(null);
                        void handleCloseProject(project());
                      }}
                    >
                      {closingProjectId() === project().id
                        ? "Closing..."
                        : "Close project"}
                    </button>
                  }
                >
                  <button
                    type="button"
                    class="record-reopen-button"
                    disabled={reopeningProjectId() === project().id}
                    onClick={() => {
                      setPendingDeleteProjectId(null);
                      void handleReopenProject(project());
                    }}
                  >
                    {reopeningProjectId() === project().id
                      ? "Reopening..."
                      : "Reopen project"}
                  </button>
                </Show>

                <button
                  type="button"
                  class="record-delete-button"
                  classList={{
                    "is-danger-armed": pendingDeleteProjectId() === project().id,
                  }}
                  disabled={deletingProjectId() === project().id}
                  onClick={() => void handleDeleteProject(project())}
                >
                  {deletingProjectId() === project().id
                    ? "Deleting..."
                    : pendingDeleteProjectId() === project().id
                      ? "Are you sure?"
                      : "Delete project"}
                </button>
              </div>
            </div>
          )}
        </Show>
      </article>
    </section>
  );
}

export default ProjectCatalogWorkspace;
