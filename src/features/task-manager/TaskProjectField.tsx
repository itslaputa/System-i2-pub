import { For, Show, createResource, createSignal } from "solid-js";
import "./taskProjectField.css";
import {
  addTaskProject,
  closeTaskProject,
  loadActiveTaskProjectList,
  type TaskProjectNode,
} from "../../services/tauri/taskProjects";
import { formatHoursMinutes } from "../../utils/formatDuration";

export type TaskProjectSelection = {
  id: string;
  label: string;
  projectCategoryId: string | null;
};

type TaskProjectFieldProps = {
  value: TaskProjectSelection | null;
  onChange: (selection: TaskProjectSelection | null) => void;
  placeholder?: string;
};

function TaskProjectField(props: TaskProjectFieldProps) {
  const [isOpen, setIsOpen] = createSignal(false);
  const [newProjectLabel, setNewProjectLabel] = createSignal("");
  const [addProjectError, setAddProjectError] = createSignal<string | null>(
    null,
  );
  const [isAddingProject, setIsAddingProject] = createSignal(false);
  const [closingProjectId, setClosingProjectId] = createSignal<string | null>(null);
  const [projectList, { refetch }] = createResource(loadActiveTaskProjectList);

  function openSelector() {
    setAddProjectError(null);
    setIsOpen(true);
  }

  function closeSelector() {
    setAddProjectError(null);
    setIsOpen(false);
  }

  function handleProjectClick(project: TaskProjectNode) {
    props.onChange({
      id: project.id,
      label: project.label,
      projectCategoryId: project.project_category_id,
    });
    setAddProjectError(null);
    setIsOpen(false);
  }

  function handleClearProject() {
    props.onChange(null);
    setAddProjectError(null);
    setNewProjectLabel("");
    setIsOpen(false);
  }

  function getErrorMessage(error: unknown) {
    if (error instanceof Error && error.message) {
      return error.message;
    }

    if (typeof error === "string" && error.trim()) {
      return error;
    }

    return "Failed to add project.";
  }

  async function handleAddProject() {
    const normalizedLabel = newProjectLabel().trim();

    if (!normalizedLabel) {
      setAddProjectError("Enter a project name.");
      return;
    }

    setIsAddingProject(true);
    setAddProjectError(null);

    try {
      const createdProject = await addTaskProject({
        label: normalizedLabel,
      });
      setNewProjectLabel("");
      props.onChange({
        id: createdProject.id,
        label: createdProject.label,
        projectCategoryId: createdProject.project_category_id,
      });
      await refetch();
    } catch (error) {
      setAddProjectError(getErrorMessage(error));
    } finally {
      setIsAddingProject(false);
    }
  }

  async function handleCloseProject(project: TaskProjectNode) {
    setClosingProjectId(project.id);
    setAddProjectError(null);

    try {
      await closeTaskProject(project.id);

      if (props.value?.id === project.id) {
        props.onChange(null);
      }

      await refetch();
    } catch (error) {
      setAddProjectError(getErrorMessage(error));
    } finally {
      setClosingProjectId(null);
    }
  }

  return (
    <div class="task-project-field">
      <button
        type="button"
        class="task-project-trigger"
        aria-expanded={isOpen()}
        onClick={() => (isOpen() ? closeSelector() : openSelector())}
      >
        <span
          class="task-project-trigger-label"
          classList={{ "is-placeholder": !props.value }}
        >
          {props.value?.label ?? props.placeholder ?? "Select project"}
        </span>
        <span class="task-project-trigger-meta">
          {isOpen() ? "Close" : "Browse"}
        </span>
      </button>

      <Show when={isOpen()}>
        <div class="task-project-panel" role="dialog" aria-label="Project selector">
          <Show when={projectList.loading}>
            <p class="task-project-status">Loading projects...</p>
          </Show>

          <Show when={projectList.error}>
            <p class="task-project-status is-error">Failed to load project list.</p>
          </Show>

          <Show when={!projectList.loading && !projectList.error}>
            <>
              <Show when={props.value}>
                <button
                  type="button"
                  class="task-project-clear-button"
                  onClick={handleClearProject}
                >
                  Clear project
                </button>
              </Show>

              <div class="task-project-options">
                <For each={projectList() ?? []}>
                  {(project) => (
                    <div
                      class="task-project-option-row"
                      classList={{ "is-selected": props.value?.id === project.id }}
                    >
                      <button
                        type="button"
                        class="task-project-option"
                        onClick={() => handleProjectClick(project)}
                      >
                        <span class="task-project-option-title">{project.label}</span>
                        <span class="task-project-option-meta">
                          {formatHoursMinutes(project.sum_time_length)} ·{" "}
                          {project.is_done ? "Done" : "Active"}
                        </span>
                      </button>
                      <button
                        type="button"
                        class="task-project-close-button"
                        disabled={closingProjectId() === project.id}
                        onClick={() => void handleCloseProject(project)}
                      >
                        {closingProjectId() === project.id ? "Closing..." : "Close"}
                      </button>
                    </div>
                  )}
                </For>
              </div>

              <div class="task-project-add">
                <input
                  class="task-project-add-input"
                  type="text"
                  value={newProjectLabel()}
                  placeholder="Add project"
                  onInput={(event) => {
                    setNewProjectLabel(event.currentTarget.value);
                    setAddProjectError(null);
                  }}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void handleAddProject();
                    }
                  }}
                />
                <button
                  type="button"
                  class="task-project-add-button"
                  disabled={isAddingProject()}
                  onClick={() => void handleAddProject()}
                >
                  {isAddingProject() ? "Adding..." : "Add"}
                </button>
              </div>

              <Show when={addProjectError()}>
                <p class="task-project-status is-error">{addProjectError()}</p>
              </Show>
            </>
          </Show>
        </div>
      </Show>
    </div>
  );
}

export default TaskProjectField;
