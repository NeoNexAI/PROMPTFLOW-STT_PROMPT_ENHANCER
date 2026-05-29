# Release Process — PromptFlow STT

**Date:** 2026-04-22
**Status:** Approved

---

## Branch Model

```
feature branches → bootstrap/initial-setup → PR → main
```

- `main` is **always releasable** — no WIP, no red CI, no known blockers.
- Feature branches: `feat/<scope>`, `fix/<scope>`, `docs/<scope>`, `chore/<scope>`.
- Squash merge to `main` (no merge commits).
- Delete branches after merging.

---

## Versioning

Semantic Versioning (`MAJOR.MINOR.PATCH`):

- `v0.x` — pre-release. Breaking changes between minor versions are allowed. No stability guarantees.
- `v1.0` — first stable public release. After v1.0, breaking changes require a major version bump.

---

## Release Checklist

Before tagging any release, every item must be checked:

- [ ] All CI jobs green (lint + type-check + cargo check + tests)
- [ ] `cargo audit` returns zero high or critical advisories
- [ ] `npm audit` returns zero high or critical vulnerabilities
- [ ] `CHANGELOG.md` updated with this release's changes
- [ ] Version bumped consistently in three places:
  - `package.json` → `"version"` field
  - `src-tauri/Cargo.toml` → `[package] version`
  - `src-tauri/tauri.conf.json` → `"version"` field
- [ ] Git tag created: `git tag v0.x.0 && git push origin v0.x.0`
- [ ] GitHub Release created with release notes and binary artifacts attached
- [ ] Manual smoke test on Windows 10+ and macOS 13+: hotkey fires, clipboard enhancement works end-to-end

---

## Code Signing

| Platform | Method | Required secret |
|---|---|---|
| Windows | EV certificate via `tauri-action` | `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |
| macOS | Apple Developer ID + notarization via `tauri-action` | `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` |
| Linux | No signing required — AppImage + .deb distributed unsigned | — |

Unsigned releases (pre-v1.0) will trigger OS security warnings on Windows and macOS. Users must right-click → Open (macOS) or click "More info → Run anyway" (Windows). This is acceptable for pre-release versions.

---

## CI/CD Release Flow

Tag push (`v*`) triggers the release workflow (`.github/workflows/release.yml`, created in v1.0 sprint):

1. Build matrix: `windows-latest` (`.msi` + `.exe`), `macos-latest` (`.dmg` + `.app`), `ubuntu-22.04` (`.deb` + `AppImage`) — 3 runners in parallel
2. Each runner: `npm install` → `npm run tauri build` → sign artifacts
3. Artifacts uploaded to the GitHub Release created by the tag
4. Auto-updater JSON endpoint (`github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/releases/latest/download/latest.json`) updated with new version and artifact URLs

---

## Auto-Updater

Uses `tauri-plugin-updater`. On app startup:

1. App calls the updater endpoint (max once per 24 h)
2. If a newer version is available, a non-blocking notification appears in the overlay title bar
3. User clicks "Update" → app downloads installer in background → prompts to restart
4. Rollback: if the new version fails to launch, the installer restores the previous version

The updater endpoint URL is configured in `src-tauri/tauri.conf.json` under `plugins.updater.endpoints`.

---

## Changelog Maintenance

- The `CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/) format.
- Sections per release: `Added`, `Changed`, `Fixed`, `Removed`.
- Every user-visible change must have an entry. Internal refactors and CI changes are optional.
- Generated from conventional commits as a starting point, then reviewed and edited manually before release.
