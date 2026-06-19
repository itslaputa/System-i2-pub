import { invoke } from "@tauri-apps/api/core";

export type RuntimeStatusSource = "none" | "user-config";

export type RuntimeStatus = {
  isReady: boolean;
  bundleDir: string | null;
  source: RuntimeStatusSource;
  missing: string[];
  invalid: string[];
  details: string[];
};

export type RuntimeBundleFolderInput = {
  bundleDir: string;
};

export type RuntimeBackupResult = {
  backupPath: string;
};

export function getRuntimeStatus() {
  return invoke<RuntimeStatus>("get_runtime_status");
}

export function attachExistingRuntimeBundle(input: RuntimeBundleFolderInput) {
  return invoke<RuntimeStatus>("attach_existing_runtime_bundle", { input });
}

export function createRuntimeBundleInFolder(input: RuntimeBundleFolderInput) {
  return invoke<RuntimeStatus>("create_runtime_bundle_in_folder", { input });
}

export function createRuntimeBundleInAppDataDir() {
  return invoke<RuntimeStatus>("create_runtime_bundle_in_app_data_dir");
}

export function createRuntimeBundleBackup() {
  return invoke<RuntimeBackupResult>("create_runtime_bundle_backup");
}
