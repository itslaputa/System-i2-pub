import { For, Show, createSignal } from "solid-js";
import type { RuntimeStatus } from "../../services/tauri/runtime";
import { pickFolder } from "./setupFolderPicker";

type SetupPageProps = {
  runtimeStatus: RuntimeStatus;
  isWorking: boolean;
  actionError: string | null;
  onAttachExisting: (bundleDir: string) => void;
  onCreateInFolder: (bundleDir: string) => void;
  onCreateInAppDataDir: () => void;
  onRetryStatus: () => void;
};

function SetupPage(props: SetupPageProps) {
  const [existingBundleDir, setExistingBundleDir] = createSignal("");
  const [newBundleDir, setNewBundleDir] = createSignal("");

  async function browseExistingFolder() {
    const selected = await pickFolder(existingBundleDir());
    if (selected) {
      setExistingBundleDir(selected);
    }
  }

  async function browseNewFolder() {
    const selected = await pickFolder(newBundleDir());
    if (selected) {
      setNewBundleDir(selected);
    }
  }

  return (
    <div class="setup-shell">
      <main class="setup-page">
        <section class="setup-hero">
          <p class="eyebrow">Setup</p>
          <h1>Runtime bundle is not ready.</h1>
          <p class="lead">
            The app needs one folder containing `tasks.sqlite3`,
            `task_categories.json`, `task_category_change_log.log`, and
            `project_categories.json`.
          </p>
        </section>

        <section class="setup-grid">
          <article class="panel setup-panel">
            <div class="panel-copy">
              <div>
                <p class="section-label">Status</p>
                <h2>What is missing</h2>
              </div>
              <span class="stat-pill">{props.runtimeStatus.source}</span>
            </div>

            <div class="setup-status-grid">
              <div class="setup-status-card">
                <p class="section-label">Missing files</p>
                <Show
                  when={props.runtimeStatus.missing.length > 0}
                  fallback={<p class="panel-text">None.</p>}
                >
                  <ul class="setup-status-list">
                    <For each={props.runtimeStatus.missing}>
                      {(entry) => <li>{entry}</li>}
                    </For>
                  </ul>
                </Show>
              </div>

              <div class="setup-status-card">
                <p class="section-label">Invalid state</p>
                <Show
                  when={props.runtimeStatus.invalid.length > 0}
                  fallback={<p class="panel-text">None.</p>}
                >
                  <ul class="setup-status-list">
                    <For each={props.runtimeStatus.invalid}>
                      {(entry) => <li>{entry}</li>}
                    </For>
                  </ul>
                </Show>
              </div>
            </div>

            <Show when={props.runtimeStatus.details.length > 0}>
              <div class="setup-detail-block">
                <p class="section-label">Details</p>
                <ul class="setup-status-list">
                  <For each={props.runtimeStatus.details}>
                    {(entry) => <li>{entry}</li>}
                  </For>
                </ul>
              </div>
            </Show>

            <Show when={props.runtimeStatus.bundleDir}>
              <p class="panel-text">
                Last resolved folder: <span class="setup-code">{props.runtimeStatus.bundleDir}</span>
              </p>
            </Show>

            <button
              type="button"
              class="action-button setup-button is-secondary"
              disabled={props.isWorking}
              onClick={props.onRetryStatus}
            >
              Recheck runtime status
            </button>
          </article>

          <article class="panel setup-panel">
            <div class="panel-copy">
              <div>
                <p class="section-label">Option 1</p>
                <h2>Use existing folder</h2>
                <p class="panel-text">
                  Point the app at a folder that already contains the live runtime files.
                </p>
              </div>
            </div>

            <label class="field">
              <span>Existing bundle folder</span>
              <div class="setup-input-row">
                <input
                  type="text"
                  class="input-field"
                  value={existingBundleDir()}
                  onInput={(event) => setExistingBundleDir(event.currentTarget.value)}
                  placeholder="Choose a runtime bundle folder"
                  disabled={props.isWorking}
                />
                <button
                  type="button"
                  class="action-button setup-button is-secondary setup-picker-button"
                  disabled={props.isWorking}
                  onClick={() => void browseExistingFolder()}
                >
                  Browse
                </button>
              </div>
            </label>

            <button
              type="button"
              class="action-button setup-button"
              disabled={props.isWorking || !existingBundleDir().trim()}
              onClick={() => props.onAttachExisting(existingBundleDir())}
            >
              Use this folder
            </button>
          </article>

          <article class="panel setup-panel">
            <div class="panel-copy">
              <div>
                <p class="section-label">Option 2</p>
                <h2>Create in chosen folder</h2>
                <p class="panel-text">
                  The folder must be empty for the runtime bundle files. Existing bundle files are rejected.
                </p>
              </div>
            </div>

            <label class="field">
              <span>New bundle folder</span>
              <div class="setup-input-row">
                <input
                  type="text"
                  class="input-field"
                  value={newBundleDir()}
                  onInput={(event) => setNewBundleDir(event.currentTarget.value)}
                  placeholder="Choose an empty folder for runtime data"
                  disabled={props.isWorking}
                />
                <button
                  type="button"
                  class="action-button setup-button is-secondary setup-picker-button"
                  disabled={props.isWorking}
                  onClick={() => void browseNewFolder()}
                >
                  Browse
                </button>
              </div>
            </label>

            <button
              type="button"
              class="action-button setup-button"
              disabled={props.isWorking || !newBundleDir().trim()}
              onClick={() => props.onCreateInFolder(newBundleDir())}
            >
              Create bundle there
            </button>
          </article>

          <article class="panel setup-panel">
            <div class="panel-copy">
              <div>
                <p class="section-label">Option 3</p>
                <h2>Use app data directory</h2>
                <p class="panel-text">
                  Let the app create and manage the runtime bundle in its default app data location.
                </p>
              </div>
            </div>

            <button
              type="button"
              class="action-button setup-button"
              disabled={props.isWorking}
              onClick={props.onCreateInAppDataDir}
            >
              Create in app data dir
            </button>
          </article>
        </section>

        <Show when={props.actionError}>
          <section class="setup-feedback">
            <div class="panel setup-panel setup-panel-danger">
              <p class="section-label">Action error</p>
              <p class="response">{props.actionError}</p>
            </div>
          </section>
        </Show>
      </main>
    </div>
  );
}

export default SetupPage;
