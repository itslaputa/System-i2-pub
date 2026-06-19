# Release Guide

[English](RELEASE.md) | [Русский](RELEASE.ru.md)

System-I2 uses GitHub Actions to build desktop bundles and upload them to GitHub Releases.

## Release Workflow

The workflow lives at `.github/workflows/release.yml`.

It runs when:

- a tag matching `v*` is pushed;
- the workflow is started manually with a `release_tag` input.

The build matrix runs on:

- `macos-latest`
- `windows-latest`
- `ubuntu-22.04`

Each runner installs Node.js 20, Rust stable, frontend dependencies with `npm ci`, and then runs `tauri-apps/tauri-action@v0.6.2` to build and upload bundles.

## Publishing v1

Create and push the release tag:

```bash
git tag v1
git push origin v1
```

GitHub Actions will create or update the `v1` release and attach the generated desktop bundles as release assets.

Release downloads:

```text
https://github.com/itslaputa/System-i2-pub/releases/tag/v1
```

## Safety Checks Before Tagging

Before pushing a release tag, run:

```bash
rg -n --hidden -e "private-path-marker" -e "local-user-path-marker" -g '!.git' .
find . -type f \( -name "*.sqlite3" -o -name "*.sqlite3-*" -o -name "runtime-bundle.json" -o -name ".env*" \) -print
git diff --check
git status --short
```

Runtime data, local config, secrets, generated build output, and private paths must not be committed.

## Signing Caveats

The public workflow does not configure:

- macOS code signing;
- macOS notarization;
- Windows code signing;
- Linux package signing.

The generated bundles are useful for private testing and internal distribution, but operating systems may show warnings for unsigned apps.

## First Release Verification

After the first `v1` workflow run:

- Check that all three matrix jobs finish.
- Confirm release assets appear on the `v1` release page.
- Download each platform artifact from GitHub.
- Smoke-test first-run setup on each platform where possible.
- Confirm no runtime bundle or user config is included in the release assets.
