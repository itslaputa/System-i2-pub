# System-I2 Public Agent Guide

[English](agents.md) | [Русский](agents.ru.md)

System-I2 is a local-first desktop tracker for tasks, projects, category classifiers, comments, and time analytics. It uses a SolidJS/TypeScript/Vite frontend, a Rust Tauri 2 backend, and SQLite runtime storage.

This file is for public forks and external coding agents. Do not rely on private maintainer notes, local runtime data, or machine-specific paths.

## Runtime Invariants

- The app does not ship user data.
- Runtime path comes only from user config `runtime-bundle.json`.
- Use OS path resolvers for config/app-data directories and Desktop paths. Never hand-build local user paths.
- Debug/dev runtime config uses `System-I2-Dev`.
- Release runtime config uses `System-I2`.
- A valid runtime bundle is one folder containing:
  - `tasks.sqlite3`
  - `task_categories.json`
  - `task_category_change_log.log`
  - `project_categories.json`
- Users may attach an existing bundle, create a bundle in a chosen folder, or create one in the app's OS-resolved config/app-data directory.
- The ignored repo `Data/` folder may be used for dev runtime data after setup. Do not commit its contents.

## Data Model Rules

- Task category ids are persisted analytics identity.
- Category labels are mutable display text.
- Projects are global and independent from task categories.
- A project can contain tasks from many task categories.
- Project categories are optional project type metadata from `project_categories.json`; they are not task analytics categories.
- Done projects are hidden from the task form picker but remain visible and reopenable in catalog/settings flows.
- Project totals, start/end dates, and done state are derived from tasks, triggers, and service sync. Do not treat totals as manual source of truth.

## Architecture Map

Frontend:

- `src/app/App.tsx`: runtime gate and setup/main app switch.
- `src/app/AppShell.tsx`, `src/app/routes.ts`, `src/layout/`: shell and navigation.
- `src/features/setup/`: first-run runtime setup.
- `src/features/task-manager/`: task capture, project picker, project catalog.
- `src/features/analytics/`: dashboard blocks, charts, trends, formatting.
- `src/features/settings/`: category tree editor, project category editor, storage view, change log, backup button.
- `src/services/tauri/`: thin TypeScript bridge to Rust commands.

Backend:

- `src-tauri/src/lib.rs`: Tauri command registry only.
- `src-tauri/src/storage/`: DB connection, schema, migrations, validation, runtime path helpers.
- `src-tauri/src/runtime/`: setup, runtime validation, bundle create/attach, zip backup.
- `src-tauri/src/categories/`: task category tree JSON and change log.
- `src-tauri/src/project_categories/`: flat project type list JSON.
- `src-tauri/src/projects/`: global project CRUD, status, and type logic.
- `src-tauri/src/tasks/`: task create/list/delete rules.
- `src-tauri/src/analytics/`: SQL aggregation, category tree, project summaries, trends.

## Tauri Bridge Contracts

Keep TypeScript bridge payloads aligned with Rust serde names and command signatures.

For cross-boundary changes:

- Update Rust command/types tests.
- Update frontend bridge/helper tests.
- Preserve camelCase/serde naming intentionally.
- Keep `src/services/tauri/` as a thin bridge; put domain behavior in feature or backend modules.

## Checks

Frontend:

```bash
npm run test:frontend
npx tsc --noEmit --pretty false
npm run build
```

Backend:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

macOS app bundle:

```bash
npm run tauri build -- --bundles app
```

## Public Safety Checklist

Before publishing or accepting PRs, check for:

- Private user paths.
- Runtime config files such as `runtime-bundle.json`.
- SQLite files, sidecar files, runtime JSON, and runtime logs.
- Copied app data folders.
- `node_modules/`, `dist/`, `src-tauri/target/`, and generated schemas.
- Private maintainer docs or local scratch files.

Useful scans:

```bash
rg -n "private-path-marker|local-user-path-marker" README.md AGENTS.md agents_pub docs src src-tauri tests
find . \( -name "*.sqlite3" -o -name "*.sqlite3-*" -o -name "runtime-bundle.json" \) -print
```

## Common Traps

- Do not replace OS path resolver logic with hard-coded local paths.
- Do not use labels as stable analytics identity; use category ids.
- Do not couple projects to task categories.
- Do not show done projects in the task form picker.
- Do not hide done projects from catalog/settings recovery flows.
- Do not commit runtime data just because it is useful for local testing.
- Do not document Windows/Linux releases as verified unless they were smoke-tested.

## Contribution Notes

Keep changes focused and covered by tests at the right boundary. Documentation changes do not require code tests unless they alter commands, build behavior, or published contracts.
