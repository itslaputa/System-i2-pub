import { open } from "@tauri-apps/plugin-dialog";

export function normalizePickedFolderPath(
  value: string | string[] | null,
): string | null {
  if (Array.isArray(value)) {
    return value[0] ?? null;
  }

  if (typeof value === "string" && value.trim()) {
    return value;
  }

  return null;
}

export async function pickFolder(defaultPath?: string) {
  const selection = await open({
    directory: true,
    multiple: false,
    defaultPath: defaultPath?.trim() || undefined,
  });

  return normalizePickedFolderPath(selection);
}
