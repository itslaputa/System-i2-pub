import { For, Show, createMemo, createResource, createSignal } from "solid-js";
import "./categoryChangeLogView.css";
import {
  buildTaskCategoryLabelMap,
  loadTaskCategoryTree,
} from "../../services/tauri/taskCategories";
import {
  deleteTaskProject,
  loadTaskProjectList,
  reopenTaskProject,
} from "../../services/tauri/taskProjects";
import {
  deleteTaskRecord,
  loadTaskRecords,
} from "../../services/tauri/taskRecords";
import { formatHoursMinutes } from "../../utils/formatDuration";

type StorageInspectorViewProps = {
  onBack: () => void;
};

function formatProjectSummary(sumTimeLength: number, isDone: boolean) {
  return `${formatHoursMinutes(sumTimeLength)}${isDone ? " · Done" : " · Active"}`;
}

function formatTaskType(isClosingTask: boolean) {
  return isClosingTask ? "Closes project" : "Regular";
}

function formatTaskCount(taskIds: string[]) {
  const count = taskIds.length;

  if (count === 0) {
    return "None";
  }

  return count === 1 ? "1 task" : `${count} tasks`;
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

function StorageInspectorView(props: StorageInspectorViewProps) {
  const [projectActionError, setProjectActionError] = createSignal<string | null>(
    null,
  );
  const [taskDeleteError, setTaskDeleteError] = createSignal<string | null>(null);
  const [deletingProjectId, setDeletingProjectId] = createSignal<string | null>(
    null,
  );
  const [reopeningProjectId, setReopeningProjectId] = createSignal<string | null>(
    null,
  );
  const [deletingTaskId, setDeletingTaskId] = createSignal<string | null>(null);
  const [projects, { refetch: refetchProjects }] = createResource(
    () => true,
    () => loadTaskProjectList(),
  );
  const [taskRecords, { refetch: refetchTaskRecords }] = createResource(
    loadTaskRecords,
  );
  const [categoryTree] = createResource(loadTaskCategoryTree);
  const categoryLabelMap = createMemo(() =>
    buildTaskCategoryLabelMap(categoryTree() ?? []),
  );

  function formatCategoryLabel(categoryId: string) {
    return categoryLabelMap().get(categoryId) ?? categoryId;
  }

  async function refreshStorageLists() {
    await Promise.all([refetchProjects(), refetchTaskRecords()]);
  }

  async function handleDeleteProject(projectId: string) {
    setDeletingProjectId(projectId);
    setProjectActionError(null);

    try {
      await deleteTaskProject(projectId);
      await refreshStorageLists();
    } catch (error) {
      setProjectActionError(getErrorMessage(error, "Failed to delete project."));
    } finally {
      setDeletingProjectId(null);
    }
  }

  async function handleReopenProject(projectId: string) {
    setReopeningProjectId(projectId);
    setProjectActionError(null);

    try {
      await reopenTaskProject(projectId);
      await refreshStorageLists();
    } catch (error) {
      setProjectActionError(getErrorMessage(error, "Failed to reopen project."));
    } finally {
      setReopeningProjectId(null);
    }
  }

  async function handleDeleteTask(taskId: string) {
    setDeletingTaskId(taskId);
    setTaskDeleteError(null);

    try {
      await deleteTaskRecord(taskId);
      await refreshStorageLists();
    } catch (error) {
      setTaskDeleteError(getErrorMessage(error, "Failed to delete task."));
    } finally {
      setDeletingTaskId(null);
    }
  }

  return (
    <div class="settings-category-log">
      <section class="hero">
        <p class="eyebrow">Settings</p>
        <h1>Live SQLite storage view.</h1>
      </section>

      <section class="dashboard">
        <article class="panel settings-category-log-panel">
          <div class="panel-copy">
            <div>
              <p class="section-label">Storage Tools</p>
              <h2>Runtime SQLite inspector</h2>
            </div>
            <span class="stat-pill">
              {(projects()?.length ?? 0) + (taskRecords()?.length ?? 0)} rows
            </span>
          </div>

          <div class="settings-category-log-toolbar">
            <button
              type="button"
              class="settings-category-log-button"
              onClick={props.onBack}
            >
              Back to editor
            </button>
            <button
              type="button"
              class="settings-category-log-button is-primary"
              onClick={() => {
                setProjectActionError(null);
                setTaskDeleteError(null);
                void refreshStorageLists();
              }}
            >
              Refresh both
            </button>
          </div>
        </article>
      </section>

      <section class="dashboard analytics-records-layout">
        <article class="panel analytics-record-panel">
          <div class="panel-copy">
            <div>
              <p class="section-label">SQLite Projects</p>
              <h2>Projects in storage</h2>
              <p class="panel-text analytics-panel-text">
                Live project rows with derived totals, date window, and linked task counts.
              </p>
            </div>
            <span class="stat-pill">{projects()?.length ?? 0} projects</span>
          </div>

          <div class="analytics-record-toolbar">
            <button
              class="analytics-refresh-button"
              type="button"
              onClick={() => {
                setProjectActionError(null);
                void refetchProjects();
              }}
            >
              Refresh projects
            </button>
          </div>

          <Show when={projectActionError()}>
            <p class="response">{projectActionError()}</p>
          </Show>

          <Show when={projects.loading}>
            <p class="panel-text">Loading projects from SQLite...</p>
          </Show>

          <Show when={projects.error}>
            <p class="response">Failed to load projects from SQLite.</p>
          </Show>

          <Show
            when={
              !projects.loading && !projects.error && (projects()?.length ?? 0) === 0
            }
          >
            <p class="panel-text">No projects stored yet.</p>
          </Show>

          <Show
            when={!projects.loading && !projects.error && (projects()?.length ?? 0) > 0}
          >
            <div class="analytics-record-list">
              <For each={projects() ?? []}>
                {(project) => (
                  <div class="analytics-record-row">
                    <div class="analytics-record-copy">
                      <div class="analytics-record-head">
                        <p class="analytics-record-title">{project.label}</p>
                        <span
                          class="analytics-record-tag"
                          classList={{
                            "is-done": project.is_done,
                            "is-active": !project.is_done,
                          }}
                        >
                          {project.is_done ? "Done" : "Active"}
                        </span>
                      </div>

                      <div class="analytics-record-grid">
                        <p class="analytics-record-meta">
                          <span class="analytics-record-key">Total</span>
                          <span class="analytics-record-value">
                            {formatProjectSummary(project.sum_time_length, project.is_done)}
                          </span>
                        </p>
                        <p class="analytics-record-meta">
                          <span class="analytics-record-key">Window</span>
                          <span class="analytics-record-value">
                            {project.start_date ?? "None"} {"->"} {project.end_date ?? "None"}
                          </span>
                        </p>
                        <p class="analytics-record-meta">
                          <span class="analytics-record-key">Task count</span>
                          <span class="analytics-record-value">
                            {formatTaskCount(project.tasks)}
                          </span>
                        </p>
                      </div>
                    </div>
                    <div class="analytics-record-actions">
                      <Show when={project.is_done}>
                        <button
                          class="record-reopen-button"
                          type="button"
                          disabled={
                            reopeningProjectId() === project.id ||
                            deletingProjectId() === project.id
                          }
                          onClick={() => void handleReopenProject(project.id)}
                        >
                          {reopeningProjectId() === project.id
                            ? "Reopening..."
                            : "Reopen"}
                        </button>
                      </Show>
                      <button
                        class="record-delete-button"
                        type="button"
                        disabled={
                          deletingProjectId() === project.id ||
                          reopeningProjectId() === project.id
                        }
                        onClick={() => void handleDeleteProject(project.id)}
                      >
                        {deletingProjectId() === project.id ? "Deleting..." : "Delete"}
                      </button>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </article>

        <article class="panel analytics-record-panel">
          <div class="panel-copy">
            <div>
              <p class="section-label">SQLite Tasks</p>
              <h2>Tasks in storage</h2>
              <p class="panel-text analytics-panel-text">
                Live task rows with category, project link, and closing-task state.
              </p>
            </div>
            <span class="stat-pill">{taskRecords()?.length ?? 0} tasks</span>
          </div>

          <div class="analytics-record-toolbar">
            <button
              class="analytics-refresh-button"
              type="button"
              onClick={() => {
                setTaskDeleteError(null);
                void refetchTaskRecords();
              }}
            >
              Refresh tasks
            </button>
          </div>

          <Show when={taskDeleteError()}>
            <p class="response">{taskDeleteError()}</p>
          </Show>

          <Show when={taskRecords.loading}>
            <p class="panel-text">Loading tasks from SQLite...</p>
          </Show>

          <Show when={taskRecords.error}>
            <p class="response">Failed to load tasks from SQLite.</p>
          </Show>

          <Show
            when={
              !taskRecords.loading &&
              !taskRecords.error &&
              (taskRecords()?.length ?? 0) === 0
            }
          >
            <p class="panel-text">No tasks stored yet.</p>
          </Show>

          <Show
            when={
              !taskRecords.loading &&
              !taskRecords.error &&
              (taskRecords()?.length ?? 0) > 0
            }
          >
            <div class="storage-task-list">
              <For each={taskRecords() ?? []}>
                {(task) => (
                  <div class="storage-task-row">
                    <div class="storage-task-main">
                      <p class="storage-task-title">
                        <span>{task.date}</span>
                        <span>·</span>
                        <span>{task.time_length} min</span>
                      </p>
                      <div class="storage-task-meta-line">
                        <span class="storage-task-chip">
                          {formatCategoryLabel(task.category_id)}
                        </span>
                        <span class="storage-task-chip is-muted">
                          {task.project_label ?? "No project"}
                        </span>
                        <span
                          class="storage-task-chip"
                          classList={{
                            "is-closing": task.is_project_closing_task,
                          }}
                        >
                          {formatTaskType(task.is_project_closing_task)}
                        </span>
                      </div>
                      <Show when={task.note}>
                        <p class="storage-task-note">{task.note}</p>
                      </Show>
                    </div>
                    <button
                      class="record-delete-button storage-task-delete"
                      type="button"
                      disabled={deletingTaskId() === task.id}
                      onClick={() => void handleDeleteTask(task.id)}
                    >
                      {deletingTaskId() === task.id ? "Deleting..." : "Delete"}
                    </button>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </article>
      </section>
    </div>
  );
}

export default StorageInspectorView;
