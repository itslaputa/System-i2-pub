import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  addTaskProject,
  setTaskProjectCategory,
} from "../../../src/services/tauri/taskProjects";
import {
  loadProjectCategoryList,
  saveProjectCategoryList,
} from "../../../src/services/tauri/projectCategories";
import { createRuntimeBundleBackup } from "../../../src/services/tauri/runtime";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("Tauri service bridge", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("sends project category id when creating a project", async () => {
    vi.mocked(invoke).mockResolvedValue({
      id: "1",
      label: "Project",
      project_category_id: "book",
      sum_time_length: 0,
      start_date: null,
      end_date: null,
      is_done: false,
      tasks: [],
    });

    await addTaskProject({ label: "Project", projectCategoryId: "book" });

    expect(invoke).toHaveBeenCalledWith("add_task_project", {
      input: { label: "Project", projectCategoryId: "book" },
    });
  });

  it("bridges project category updates including no-type fallback", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setTaskProjectCategory("12", null);

    expect(invoke).toHaveBeenCalledWith("set_task_project_category", {
      projectId: "12",
      projectCategoryId: null,
    });
  });

  it("bridges project category list load and save commands", async () => {
    const categories = [{ id: "book", label: "Книга", order: 0 }];
    vi.mocked(invoke).mockResolvedValueOnce(categories).mockResolvedValueOnce(undefined);

    await expect(loadProjectCategoryList()).resolves.toEqual(categories);
    await saveProjectCategoryList(categories);

    expect(invoke).toHaveBeenNthCalledWith(1, "load_project_category_list");
    expect(invoke).toHaveBeenNthCalledWith(2, "save_project_category_list", {
      categories,
    });
  });

  it("bridges runtime backup creation command", async () => {
    vi.mocked(invoke).mockResolvedValue({
      backupPath: "System-I2-runtime-backup.zip",
    });

    await expect(createRuntimeBundleBackup()).resolves.toEqual({
      backupPath: "System-I2-runtime-backup.zip",
    });

    expect(invoke).toHaveBeenCalledWith("create_runtime_bundle_backup");
  });
});
