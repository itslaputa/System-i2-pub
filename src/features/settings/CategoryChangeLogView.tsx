import { For, Show, createResource } from "solid-js";
import "./categoryChangeLogView.css";
import { loadCategoryChangeLog } from "./categoryChangeLogApi";

type CategoryChangeLogViewProps = {
  onBack: () => void;
};

function CategoryChangeLogView(props: CategoryChangeLogViewProps) {
  const [logEntries, { refetch }] = createResource(loadCategoryChangeLog);

  return (
    <div class="settings-category-log">
      <section class="hero">
        <p class="eyebrow">Settings</p>
        <h1>Category change log.</h1>
      </section>

      <section class="dashboard">
        <article class="panel settings-category-log-panel">
          <div class="panel-copy">
            <div>
              <p class="section-label">Category Settings</p>
              <h2>Classifier change history</h2>
            </div>
            <span class="stat-pill">{logEntries()?.length ?? 0} lines</span>
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
              onClick={() => void refetch()}
            >
              Refresh
            </button>
          </div>

          <Show when={logEntries.loading}>
            <p class="settings-category-log-status">Loading change log...</p>
          </Show>

          <Show when={logEntries.error}>
            <p class="settings-category-log-status is-error">
              Failed to load category change log.
            </p>
          </Show>

          <Show
            when={
              !logEntries.loading && !logEntries.error && (logEntries()?.length ?? 0) === 0
            }
          >
            <p class="settings-category-log-empty">
              No classifier changes have been logged yet.
            </p>
          </Show>

          <Show
            when={
              !logEntries.loading && !logEntries.error && (logEntries()?.length ?? 0) > 0
            }
          >
            <div class="settings-category-log-list">
              <For each={logEntries() ?? []}>
                {(entry) => <p class="settings-category-log-line">{entry}</p>}
              </For>
            </div>
          </Show>
        </article>
      </section>
    </div>
  );
}

export default CategoryChangeLogView;
