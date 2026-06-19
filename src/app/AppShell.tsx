import { Match, Switch, createSignal } from "solid-js";
import MainLayout from "../layout/MainLayout";
import AnalyticsPage from "../pages/AnalyticsPage";
import SettingsPage from "../pages/SettingsPage";
import TaskManagerPage from "../pages/TaskManagerPage";
import type { AppPage } from "./routes";

function AppShell() {
  const [currentPage, setCurrentPage] = createSignal<AppPage>("task-manager");

  return (
    <MainLayout activePage={currentPage()} onNavigate={setCurrentPage}>
      <Switch>
        <Match when={currentPage() === "task-manager"}>
          <TaskManagerPage />
        </Match>
        <Match when={currentPage() === "analytics"}>
          <AnalyticsPage />
        </Match>
        <Match when={currentPage() === "settings"}>
          <SettingsPage />
        </Match>
      </Switch>
    </MainLayout>
  );
}

export default AppShell;
