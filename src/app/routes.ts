export type AppPage = "task-manager" | "analytics" | "settings";

export const APP_NAV_ITEMS: ReadonlyArray<{ id: AppPage; label: string }> = [
  { id: "task-manager", label: "Task Manager" },
  { id: "analytics", label: "Analytics" },
  { id: "settings", label: "Settings" },
];
