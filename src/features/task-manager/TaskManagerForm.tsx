import {
  For,
  Match,
  Show,
  Switch,
  createMemo,
  createResource,
  createSignal,
} from "solid-js";
import {
  buildTaskCategoryLabelMap,
  loadTaskCategoryTree,
} from "../../services/tauri/taskCategories";
import { loadProjectCategoryList } from "../../services/tauri/projectCategories";
import {
  createTaskRecord,
  loadTaskRecords,
} from "../../services/tauri/taskRecords";
import { formatHoursMinutes } from "../../utils/formatDuration";
import {
  formatTaskEntryDate,
  parseTaskDurationToMinutes,
} from "./taskManagerFormatting";
import TaskCategoryField, {
  type TaskCategorySelection,
} from "./TaskCategoryField";
import TaskProjectField, {
  type TaskProjectSelection,
} from "./TaskProjectField";
import ProjectCatalogWorkspace from "./ProjectCatalogWorkspace";
import { getProjectCategoryLabel } from "./projectCatalogUi";

type TaskDraftPayload = {
  categoryId: string;
  projectId: string | null;
  taskDate: string;
  durationMinutes: number;
  isProjectClosingTask: boolean;
  note: string | null;
};

type SubmitFeedback = {
  tone: "idle" | "success" | "error";
  title: string;
  detail: string;
};

type TaskManagerMode = "task" | "projects";

function getTodayDateString() {
  const currentDate = new Date();
  const year = currentDate.getFullYear();
  const month = String(currentDate.getMonth() + 1).padStart(2, "0");
  const day = String(currentDate.getDate()).padStart(2, "0");

  return `${year}-${month}-${day}`;
}

function TaskManagerForm() {
  const today = getTodayDateString();
  const [activeMode, setActiveMode] = createSignal<TaskManagerMode>("task");
  const [categorySelection, setCategorySelection] =
    createSignal<TaskCategorySelection | null>(null);
  const [projectSelection, setProjectSelection] =
    createSignal<TaskProjectSelection | null>(null);
  const [entryDate, setEntryDate] = createSignal(today);
  const [durationInput, setDurationInput] = createSignal("");
  const [noteInput, setNoteInput] = createSignal("");
  const [shouldEndProject, setShouldEndProject] = createSignal(false);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [recentTasks, { refetch: refetchRecentTasks }] =
    createResource(loadTaskRecords);
  const [projectCategories] = createResource(loadProjectCategoryList);
  const [categoryTree] = createResource(loadTaskCategoryTree);
  const categoryLabelMap = createMemo(() =>
    buildTaskCategoryLabelMap(categoryTree() ?? []),
  );
  const latestTasks = createMemo(() => (recentTasks() ?? []).slice(0, 8));
  const [submitFeedback, setSubmitFeedback] = createSignal<SubmitFeedback>({
    tone: "idle",
    title: "No task has been saved yet.",
    detail: "The next saved task will appear here.",
  });

  function handleCategorySelection(selection: TaskCategorySelection) {
    setCategorySelection(selection);
  }

  function handleProjectSelection(selection: TaskProjectSelection | null) {
    setProjectSelection(selection);
    setShouldEndProject(false);
  }

  const selectedCategoryLeafLabel = () => {
    const labels = categorySelection()?.pathLabels;

    if (!labels?.length) {
      return "Required";
    }

    return labels[labels.length - 1] ?? "Required";
  };

  const formatTaskCategoryLabel = (categoryId: string) =>
    categoryLabelMap().get(categoryId) ?? categoryId;
  const selectedProjectCategoryLabel = () => {
    const selectedProject = projectSelection();

    if (!selectedProject) {
      return "None";
    }

    return getProjectCategoryLabel(
      selectedProject.projectCategoryId,
      projectCategories() ?? [],
    );
  };
  const formattedEntryDate = () => formatTaskEntryDate(entryDate());
  const normalizedNote = () => {
    const trimmedNote = noteInput().trim();
    return trimmedNote.length > 0 ? trimmedNote : null;
  };
  const normalizedDurationMinutes = () =>
    parseTaskDurationToMinutes(durationInput());
  const hasDurationInput = () => durationInput().trim().length > 0;
  const durationPreviewText = () => {
    const minutes = normalizedDurationMinutes();

    if (minutes !== null) {
      return `${minutes} min`;
    }

    return hasDurationInput() ? "Invalid format" : "Not set";
  };
  const durationFieldNote = () => {
    const minutes = normalizedDurationMinutes();

    if (minutes !== null) {
      return `${minutes} min`;
    }

    return hasDurationInput() ? "Invalid format" : "";
  };
  const normalizedTaskPayload = () => {
    const selectedCategory = categorySelection();
    const minutes = normalizedDurationMinutes();

    if (!selectedCategory || minutes === null || !hasDurationInput()) {
      return null;
    }

    return {
      categoryId: selectedCategory.id,
      projectId: projectSelection()?.id ?? null,
      taskDate: entryDate(),
      durationMinutes: minutes,
      isProjectClosingTask: Boolean(projectSelection() && shouldEndProject()),
      note: normalizedNote(),
    } satisfies TaskDraftPayload;
  };
  const buildSavedTaskDetail = (payload: TaskDraftPayload) => {
    const pathLabels = categorySelection()?.pathLabels;
    const detailParts = [
      pathLabels?.[pathLabels.length - 1] ?? payload.categoryId,
      `${payload.durationMinutes} min`,
      formatTaskEntryDate(payload.taskDate),
      payload.projectId
        ? projectSelection()?.label ?? `Project #${payload.projectId}`
        : "No project",
    ];

    if (payload.isProjectClosingTask) {
      detailParts.push("Closes project");
    }

    if (payload.note) {
      detailParts.push(`Comment: ${payload.note}`);
    }

    return detailParts.join(" · ");
  };

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();

    if (isSubmitting()) {
      return;
    }

    const selectedCategory = categorySelection();
    const payload = normalizedTaskPayload();

    if (!selectedCategory) {
      setSubmitFeedback({
        tone: "error",
        title: "Category is required.",
        detail: "Choose a category before saving this task.",
      });
      return;
    }

    if (!hasDurationInput()) {
      setSubmitFeedback({
        tone: "error",
        title: "Time is required.",
        detail: "Enter minutes or hours.minutes before saving.",
      });
      return;
    }

    if (!payload) {
      setSubmitFeedback({
        tone: "error",
        title: "Time format is invalid.",
        detail: "Use plain minutes like 90 or hours.minutes like 1.30.",
      });
      return;
    }

    setIsSubmitting(true);

    try {
      const createdTask = await createTaskRecord(payload);
      await refetchRecentTasks();
      setDurationInput("");
      setNoteInput("");
      setShouldEndProject(false);
      setSubmitFeedback({
        tone: "success",
        title: `Task #${createdTask.id} saved.`,
        detail: buildSavedTaskDetail(payload),
      });
    } catch (error) {
      if (error instanceof Error && error.message) {
        setSubmitFeedback({
          tone: "error",
          title: "Failed to save task.",
          detail: error.message,
        });
      } else if (typeof error === "string" && error.trim()) {
        setSubmitFeedback({
          tone: "error",
          title: "Failed to save task.",
          detail: error,
        });
      } else {
        setSubmitFeedback({
          tone: "error",
          title: "Failed to save task.",
          detail: "SQLite rejected the save operation.",
        });
      }
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <>
      <div class="task-manager-mode-strip" aria-label="Task Manager mode switch">
        <button
          type="button"
          class="task-manager-mode-button"
          classList={{ "is-active": activeMode() === "task" }}
          onClick={() => setActiveMode("task")}
        >
          Add Task
        </button>
        <button
          type="button"
          class="task-manager-mode-button"
          classList={{ "is-active": activeMode() === "projects" }}
          onClick={() => setActiveMode("projects")}
        >
          Projects
        </button>
      </div>

      <Switch>
        <Match when={activeMode() === "task"}>
          <section class="dashboard task-manager-layout">
            <article class="panel task-form-panel">
              <div class="panel-copy">
                <div>
                  <p class="section-label">Task Draft</p>
                  <h2>Register a task input</h2>
                </div>
                <span class="stat-pill">5 fields</span>
              </div>

              <form class="task-entry-form" onSubmit={handleSubmit}>
                <div class="field">
                  <span>Category</span>
                  <TaskCategoryField
                    value={categorySelection()}
                    onChange={handleCategorySelection}
                    placeholder="Select category path"
                  />
                </div>

                <label class="field">
                  <span>Time</span>
                  <input
                    class="input-field"
                    type="text"
                    name="duration"
                    inputMode="decimal"
                    placeholder="90 or 1.30"
                    value={durationInput()}
                    onInput={(event) => setDurationInput(event.currentTarget.value)}
                  />
                  <Show when={durationFieldNote()}>
                    <p class="field-note">{durationFieldNote()}</p>
                  </Show>
                </label>

                <div class="field">
                  <div class="field-head">
                    <span>Project</span>
                    <button
                      type="button"
                      class="field-inline-button"
                      onClick={() => setActiveMode("projects")}
                    >
                      Manage projects
                    </button>
                  </div>
                  <TaskProjectField
                    value={projectSelection()}
                    onChange={handleProjectSelection}
                    placeholder="Select project"
                  />
                </div>

                <label class="field">
                  <span>Date</span>
                  <input
                    class="input-field"
                    type="date"
                    name="entry-date"
                    max={today}
                    value={entryDate()}
                    onInput={(event) => setEntryDate(event.currentTarget.value)}
                  />
                  <p class="field-note">{formattedEntryDate()}</p>
                </label>

                <label class="field">
                  <span>Comment</span>
                  <textarea
                    class="input-field textarea-field"
                    name="task-note"
                    rows="3"
                    placeholder="Optional short task comment"
                    value={noteInput()}
                    onInput={(event) => setNoteInput(event.currentTarget.value)}
                  />
                  <Show when={normalizedNote()}>
                    <p class="field-note">{normalizedNote()}</p>
                  </Show>
                </label>

                <Show when={projectSelection()}>
                  <label class="project-completion-toggle">
                    <input
                      type="checkbox"
                      checked={shouldEndProject()}
                      onChange={(event) =>
                        setShouldEndProject(event.currentTarget.checked)}
                    />
                    <span>
                      Mark this as the final task for {projectSelection()?.label}
                    </span>
                  </label>
                </Show>

                <button class="action-button" type="submit" disabled={isSubmitting()}>
                  {isSubmitting() ? "Saving..." : "Submit"}
                </button>
              </form>
            </article>

            <article class="panel task-summary-panel">
              <div>
                <p class="section-label">Current Draft State</p>
                <h2>Submission preview</h2>
              </div>

              <dl class="task-summary-list">
                <div class="task-summary-row">
                  <dt>Category</dt>
                  <dd>{categorySelection()?.displayPath ?? "Required"}</dd>
                </div>
                <div class="task-summary-row">
                  <dt>Subcategory</dt>
                  <dd>{selectedCategoryLeafLabel()}</dd>
                </div>
                <div class="task-summary-row">
                  <dt>Time</dt>
                  <dd>{hasDurationInput() ? durationPreviewText() : "Required"}</dd>
                </div>
                <div class="task-summary-row">
                  <dt>Project</dt>
                  <dd>{projectSelection()?.label ?? "None"}</dd>
                </div>
                <div class="task-summary-row">
                  <dt>Project Type</dt>
                  <dd>{selectedProjectCategoryLabel()}</dd>
                </div>
                <div class="task-summary-row">
                  <dt>Comment</dt>
                  <dd>{normalizedNote() ?? "None"}</dd>
                </div>
                <div class="task-summary-row">
                  <dt>Date</dt>
                  <dd>{formattedEntryDate()}</dd>
                </div>
                <div class="task-summary-row">
                  <dt>Project Completion</dt>
                  <dd>
                    {projectSelection()
                      ? shouldEndProject()
                        ? "This task will close the selected project"
                        : "Project remains active"
                      : "None"}
                  </dd>
                </div>
              </dl>

              <Show when={submitFeedback().tone !== "idle"}>
                <section
                  class="task-feedback"
                  classList={{
                    "is-success": submitFeedback().tone === "success",
                    "is-error": submitFeedback().tone === "error",
                  }}
                >
                  <p class="task-feedback-title">{submitFeedback().title}</p>
                  <p class="task-feedback-detail">{submitFeedback().detail}</p>
                </section>
              </Show>
            </article>
          </section>

          <article class="panel task-latest-panel">
            <div class="panel-copy">
              <div>
                <p class="section-label">Latest Tasks</p>
                <h2>Recent task entries</h2>
                <p class="panel-text">
                  The newest saved rows from SQLite, so you can quickly verify what
                  was already logged.
                </p>
              </div>
              <span class="stat-pill">{latestTasks().length} shown</span>
            </div>

            <Show when={recentTasks.loading}>
              <p class="panel-text">Loading recent tasks...</p>
            </Show>

            <Show when={recentTasks.error}>
              <p class="response">Failed to load recent tasks from SQLite.</p>
            </Show>

            <Show
              when={
                !recentTasks.loading &&
                !recentTasks.error &&
                latestTasks().length === 0
              }
            >
              <p class="panel-text">No tasks saved yet.</p>
            </Show>

            <Show
              when={
                !recentTasks.loading &&
                !recentTasks.error &&
                latestTasks().length > 0
              }
            >
              <div class="task-latest-list">
                <For each={latestTasks()}>
                  {(task) => (
                    <div class="task-latest-row">
                      <div class="task-latest-main">
                        <p class="task-latest-title">
                          <span>{formatTaskEntryDate(task.date)}</span>
                          <span>·</span>
                          <span>{formatHoursMinutes(task.time_length)}</span>
                        </p>
                        <div class="task-latest-meta-line">
                          <span class="task-latest-chip">
                            {formatTaskCategoryLabel(task.category_id)}
                          </span>
                          <span class="task-latest-chip is-muted">
                            {task.project_label ?? "No project"}
                          </span>
                          <span
                            class="task-latest-chip"
                            classList={{ "is-closing": task.is_project_closing_task }}
                          >
                            {task.is_project_closing_task
                              ? "Closes project"
                              : "Regular"}
                          </span>
                        </div>
                      </div>
                      <Show when={task.note}>
                        <p class="task-latest-note">{task.note}</p>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </article>
        </Match>

        <Match when={activeMode() === "projects"}>
          <ProjectCatalogWorkspace
            projectCategories={projectCategories() ?? []}
            selectedTaskProjectId={projectSelection()?.id ?? null}
            onApplyTaskProjectSelection={handleProjectSelection}
            onReturnToTaskMode={() => setActiveMode("task")}
          />
        </Match>
      </Switch>
    </>
  );
}

export default TaskManagerForm;
