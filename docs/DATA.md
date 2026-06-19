# Runtime Data Guide

[English](DATA.md) | [Русский](DATA.ru.md)

System-I2 is local-first. User tasks, projects, category files, comments, and analytics source data live on the user's machine in a runtime bundle. The public repository should not contain real runtime data.

## Runtime Bundle

A runtime bundle is one folder with four files:

- `tasks.sqlite3`
- `task_categories.json`
- `task_category_change_log.log`
- `project_categories.json`

The SQLite database stores task and project records. The JSON files store task category and project category classifiers. The change log file records classifier changes.

## First-Run Setup

If no runtime bundle is configured, the app opens setup and offers three paths:

- Attach an existing runtime bundle folder.
- Create a runtime bundle in a selected folder.
- Create a runtime bundle in the app's OS-resolved config/app-data directory.

All runtime files must stay together in one folder.

## User Config

The selected runtime bundle path is saved in `runtime-bundle.json` inside the app's OS-resolved config/app-data directory.

Debug/dev builds use the app directory name `System-I2-Dev`. Release builds use `System-I2`. The code should always resolve these locations through OS path APIs.

> [!IMPORTANT]
> Do not hard-code private absolute paths in code, docs, tests, or config examples.

## Dev Data

The repository includes `Data/.gitkeep` so developers have an obvious place for optional local runtime data. The `Data/` contents are ignored.

To reset local dev data:

1. Close the app.
2. Remove or move the dev runtime bundle you created.
3. Remove the dev `runtime-bundle.json` from the OS-resolved config/app-data directory.
4. Start the app again and complete first-run setup.

Do not commit the resulting runtime files.

## Backups

Settings includes a runtime backup action. It creates a zip file from the active runtime bundle and writes it to the OS-resolved Desktop.

Backups include the active runtime folder contents, including SQLite sidecar files such as `tasks.sqlite3-wal` and `tasks.sqlite3-shm` when present.

## Manual Copy Or Backup

To manually copy data, copy the whole runtime bundle folder while the app is closed. Keeping all four runtime files together avoids mismatched database and classifier state.

For a safer backup, use the Settings backup button so the app writes one zip archive from the active bundle.

## What Not To Commit

Never commit:

- `tasks.sqlite3`
- `tasks.sqlite3-wal`
- `tasks.sqlite3-shm`
- `runtime-bundle.json`
- `task_categories.json`
- `task_category_change_log.log`
- `project_categories.json`
- Any copied OS-resolved config/app-data directory
- Any backup zip containing runtime data

The public repository should contain templates, source code, tests, and documentation only.
