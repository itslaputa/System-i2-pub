import { Match, Show, Switch, createResource, createSignal } from "solid-js";
import SetupPage from "../features/setup/SetupPage";
import {
  attachExistingRuntimeBundle,
  createRuntimeBundleInAppDataDir,
  createRuntimeBundleInFolder,
  getRuntimeStatus,
  type RuntimeStatus,
} from "../services/tauri/runtime";
import AppShell from "./AppShell";

function getErrorMessage(error: unknown, fallback: string) {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === "string" && error.trim()) {
    return error;
  }

  return fallback;
}

function App() {
  const [actionError, setActionError] = createSignal<string | null>(null);
  const [isWorking, setIsWorking] = createSignal(false);
  const [runtimeStatus, { refetch: refetchRuntimeStatus }] = createResource(
    getRuntimeStatus,
  );

  async function runRuntimeAction(
    action: () => Promise<RuntimeStatus>,
    fallbackError: string,
  ) {
    setIsWorking(true);
    setActionError(null);

    try {
      await action();
      await refetchRuntimeStatus();
    } catch (error) {
      setActionError(getErrorMessage(error, fallbackError));
    } finally {
      setIsWorking(false);
    }
  }

  return (
    <Switch>
      <Match when={runtimeStatus.loading}>
        <div class="setup-shell">
          <main class="setup-page">
            <div class="panel setup-panel">
              <p class="eyebrow">Setup</p>
              <h1>Checking runtime bundle.</h1>
            </div>
          </main>
        </div>
      </Match>

      <Match when={runtimeStatus.error}>
        <div class="setup-shell">
          <main class="setup-page">
            <div class="panel setup-panel setup-panel-danger">
              <p class="eyebrow">Setup</p>
              <h1>Failed to load runtime status.</h1>
              <p class="response">
                {getErrorMessage(
                  runtimeStatus.error,
                  "The app could not query runtime readiness.",
                )}
              </p>
              <button
                type="button"
                class="action-button setup-button"
                onClick={() => void refetchRuntimeStatus()}
              >
                Retry runtime check
              </button>
            </div>
          </main>
        </div>
      </Match>

      <Match when={runtimeStatus()}>
        <Show
          when={runtimeStatus()!.isReady}
          fallback={
            <SetupPage
              runtimeStatus={runtimeStatus()!}
              isWorking={isWorking()}
              actionError={actionError()}
              onRetryStatus={() => {
                setActionError(null);
                void refetchRuntimeStatus();
              }}
              onAttachExisting={(bundleDir) => {
                void runRuntimeAction(
                  () => attachExistingRuntimeBundle({ bundleDir }),
                  "Failed to attach existing runtime bundle.",
                );
              }}
              onCreateInFolder={(bundleDir) => {
                void runRuntimeAction(
                  () => createRuntimeBundleInFolder({ bundleDir }),
                  "Failed to create runtime bundle in folder.",
                );
              }}
              onCreateInAppDataDir={() => {
                void runRuntimeAction(
                  createRuntimeBundleInAppDataDir,
                  "Failed to create runtime bundle in app data dir.",
                );
              }}
            />
          }
        >
          <AppShell />
        </Show>
      </Match>
    </Switch>
  );
}

export default App;
