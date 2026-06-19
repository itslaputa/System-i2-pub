import { Match, Switch, createSignal } from "solid-js";
import CategoryChangeLogView from "../features/settings/CategoryChangeLogView";
import CategoryTreeEditor from "../features/settings/CategoryTreeEditor";
import ProjectCategoryEditor from "../features/settings/ProjectCategoryEditor";
import StorageInspectorView from "../features/settings/StorageInspectorView";

type SettingsView =
  | "task-categories"
  | "project-categories"
  | "change-log"
  | "storage";

function SettingsPage() {
  const [settingsView, setSettingsView] =
    createSignal<SettingsView>("task-categories");

  return (
    <section class="content-page task-page" aria-label="Settings page">
      <div class="task-manager-mode-strip" aria-label="Settings view switch">
        <button
          type="button"
          class="task-manager-mode-button"
          classList={{ "is-active": settingsView() === "task-categories" }}
          onClick={() => setSettingsView("task-categories")}
        >
          Task Categories
        </button>
        <button
          type="button"
          class="task-manager-mode-button"
          classList={{ "is-active": settingsView() === "project-categories" }}
          onClick={() => setSettingsView("project-categories")}
        >
          Project Categories
        </button>
        <button
          type="button"
          class="task-manager-mode-button"
          classList={{ "is-active": settingsView() === "change-log" }}
          onClick={() => setSettingsView("change-log")}
        >
          Change Log
        </button>
        <button
          type="button"
          class="task-manager-mode-button"
          classList={{ "is-active": settingsView() === "storage" }}
          onClick={() => setSettingsView("storage")}
        >
          Storage
        </button>
      </div>

      <Switch>
        <Match when={settingsView() === "task-categories"}>
          <CategoryTreeEditor
            onOpenChangeLog={() => setSettingsView("change-log")}
            onOpenStorageView={() => setSettingsView("storage")}
          />
        </Match>
        <Match when={settingsView() === "project-categories"}>
          <ProjectCategoryEditor />
        </Match>
        <Match when={settingsView() === "change-log"}>
          <CategoryChangeLogView
            onBack={() => setSettingsView("task-categories")}
          />
        </Match>
        <Match when={settingsView() === "storage"}>
          <StorageInspectorView onBack={() => setSettingsView("task-categories")} />
        </Match>
      </Switch>
    </section>
  );
}

export default SettingsPage;
