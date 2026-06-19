# Build Guide

[English](BUILD.md) | [Русский](BUILD.ru.md)

This guide covers local development, desktop builds, and the minimum smoke checks before publishing a release artifact.

## Requirements

- Node.js 20+
- npm
- Rust toolchain
- [Tauri 2 prerequisites](https://tauri.app/start/prerequisites/) for the target OS

Install frontend dependencies first:

```bash
npm install
```

For platform-specific setup details, use the official Tauri prerequisites guide for your OS.

## macOS

Run the dev app:

```bash
npm run tauri dev
```

Build a macOS `.app` bundle:

```bash
npm run tauri build -- --bundles app
```

The app bundle is produced under:

```text
src-tauri/target/release/bundle/macos/System-I2.app
```

Signing and notarization are not documented as configured in this public copy. Treat unsigned local bundles as development artifacts unless release signing is added and verified.

## Windows

Install the Windows Tauri prerequisites, including the Rust toolchain and the native WebView requirements for Tauri 2.

Development command:

```bash
npm run tauri dev
```

Build command:

```bash
npm run tauri build
```

Windows release packaging should be smoke-tested on Windows before publishing. Do not assume a production-ready release from an unverified cross-platform build.

## Linux

Install the Linux Tauri prerequisites for your distribution, including the Rust toolchain and required WebKit/system libraries.

Development command:

```bash
npm run tauri dev
```

Build command:

```bash
npm run tauri build
```

Linux release packaging should be smoke-tested on the target distribution before publishing.

## Release Smoke Checklist

Run checks:

```bash
npm run test:frontend
npx tsc --noEmit --pretty false
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Then verify app behavior:

- First-run setup appears when no runtime bundle is configured.
- Creating a bundle in the app's OS-resolved config/app-data directory works.
- Creating a bundle in a chosen folder works.
- Attaching an existing bundle works.
- Task entry, project selection, analytics, and settings screens open without errors.
- Backup from Settings writes a zip file to the OS-resolved Desktop.
- No runtime data, local config, or generated output is included in the repository.
