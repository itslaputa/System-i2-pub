# System-I2

[English](README.md) | [Русский](README.ru.md)

Personal desktop time tracker for logging work, organizing it by projects and categories, and seeing where your time actually goes.

System-I2 is local-first: your tasks, comments, projects, categories, and analytics stay on your machine. There is no account system, cloud sync, or hosted backend.

[Download](#download) · [Privacy](#privacy-and-data) · [For developers](#for-developers)

![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-backend-000000?style=flat-square)
![SolidJS](https://img.shields.io/badge/SolidJS-frontend-2C4F7C?style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-local%20storage-003B57?style=flat-square)
![Local-first](https://img.shields.io/badge/data-local--first-3C873A?style=flat-square)

## Download

Latest public release: [System-I2 v1](https://github.com/itslaputa/System-i2-pub/releases/tag/v1)

| Platform | Recommended download | Alternative formats |
| --- | --- | --- |
| macOS Apple Silicon | [DMG](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-darwin-aarch64.dmg) | [App archive](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-darwin-aarch64.app.tar.gz) |
| Windows x64 | [MSI installer](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-windows-x64.msi) | [EXE installer](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-windows-x64.exe) |
| Linux x64 | [AppImage](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-linux-amd64.AppImage) | [DEB](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-linux-amd64.deb), [RPM](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-linux-x86_64.rpm) |

More options: [latest release](https://github.com/itslaputa/System-i2-pub/releases/latest) · [all releases](https://github.com/itslaputa/System-i2-pub/releases)

The current builds are unsigned. macOS, Windows, and some Linux environments may show a security warning on first launch. macOS is the primary tested desktop target; Windows and Linux builds are produced automatically and should be checked manually on the target system before serious use.

## First Run

When System-I2 starts for the first time, it asks where to keep its local data folder:

- create a new local data folder in the app data directory;
- create a new data folder in a folder you choose;
- attach an existing data folder.

A valid data folder contains:

- `tasks.sqlite3`
- `task_categories.json`
- `task_category_change_log.log`
- `project_categories.json`

## Privacy and data

System-I2 does not upload your task data. There is no hosted account, no sync service, and no remote analytics backend in this public app.

The repository does not include personal runtime data. Local data files such as SQLite databases, runtime config, backups, and generated build output are ignored and should not be committed.

## Project status

System-I2 is a working local-first desktop app with public source code and unsigned release builds.

- Primary tested target: macOS.
- Build targets available through Tauri: macOS, Windows, Linux.
- Signing and notarization: not configured yet.
- Data model: local SQLite/JSON files.

## For developers

Most users only need the release downloads above. The sections below are for people who want to build, inspect, or contribute to the app.

<details>
<summary>Run from source</summary>

```bash
npm install
npm run tauri dev
```

</details>

<details>
<summary>Build locally</summary>

```bash
npm run tauri build -- --bundles app
```

Build notes, platform prerequisites, and release smoke checks are in [docs/BUILD.md](docs/BUILD.md).

</details>

<details>
<summary>Development checks</summary>

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

</details>

<details>
<summary>Repository map</summary>

- `src/`: SolidJS frontend.
- `src/services/tauri/`: TypeScript bridge to Rust Tauri commands.
- `src-tauri/src/`: Rust backend.
- `src-tauri/src/storage/`: SQLite connection, schema, validation, and runtime path helpers.
- `src-tauri/src/runtime/`: first-run setup, bundle validation, attach/create flows, and backup.
- `tests/frontend/`: frontend unit tests.
- Rust tests live next to their backend domains under `src-tauri/src/**/tests/`.

For deeper implementation rules, read [agents_pub/agents.md](agents_pub/agents.md).

</details>

## Documentation

- [Runtime data guide](docs/DATA.md)
- [Build guide](docs/BUILD.md)
- [Release guide](docs/RELEASE.md)
- [Public agent guide](agents_pub/agents.md)
- [Russian documentation](README.ru.md)
