# Operations & Maintenance Runbook

This is the practical guide for building, testing, releasing and supporting
PromptFlow STT. It complements the design specs in `docs/specs/` and the
release policy in `docs/RELEASE.md`.

PromptFlow STT is a **local-first desktop application**. There is no server to
operate: each user runs the app on their own machine with their own API keys.
"Operations" therefore means **build, release and support**, not running
infrastructure.

## 1. Development environment

| Requirement | Version |
|-------------|---------|
| Rust | stable (see `rust-toolchain.toml`) |
| Node.js | 20 (see `.nvmrc`) |
| OS libraries (Linux) | WebKitGTK 4.1, GTK 3, libayatana-appindicator3, librsvg2, **ALSA** (`libasound2-dev`) |

On Debian/Ubuntu (including cloud dev containers):

```bash
./scripts/setup-linux.sh   # installs the system libraries above
npm install
npm run tauri dev          # run the app
```

> The `cpal` audio crate requires `libasound2-dev`; without it the Rust build
> fails with a `pkg-config ... alsa` error. WebKitGTK is required by Tauri
> itself. macOS/Windows need no extra system packages beyond the Tauri
> prerequisites.

## 2. Local verification (mirrors CI)

Run these before pushing — they are exactly what CI enforces:

```bash
# Frontend
npm run type-check
npm run lint

# Rust (from src-tauri/)
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings -A dead_code -A unused_imports
cargo test
```

`cargo test` runs 67 unit tests. One keychain round-trip test is `#[ignore]`d
because it needs a real OS secret service; run it locally with
`cargo test -- --ignored`.

## 3. CI/CD

- **`.github/workflows/ci.yml`** — on every push/PR: frontend lint + type-check,
  and Rust fmt + clippy + test (installs the Linux system deps).
- **`.github/workflows/security.yml`** — on `main` and weekly: `npm audit`
  (high+) and `cargo audit`.
- **`.github/workflows/release.yml`** — on a `v*` tag: builds installers for
  macOS (arm64 + x64), Windows and Linux via `tauri-action` and attaches them
  to a **draft** GitHub Release.

## 4. Cutting a release

1. Bump the version in `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json` (keep all three in sync).
2. Update `CHANGELOG.md` (move items from *Unreleased* into the new version).
3. Commit, then tag: `git tag v0.x.y && git push origin v0.x.y`.
4. The release workflow produces a **draft** release with `.dmg`, `.msi`,
   `.deb` and `.AppImage` artifacts. Review, then publish.

### Signing (optional but required for a public v1.0)

The release workflow is **signing-ready**: it reads the secrets below if they
exist and otherwise produces unsigned builds.

| Secret | Purpose |
|--------|---------|
| `TAURI_SIGNING_PRIVATE_KEY` / `..._PASSWORD` | Sign auto-updater artifacts |
| `APPLE_CERTIFICATE` / `APPLE_CERTIFICATE_PASSWORD` / `APPLE_SIGNING_IDENTITY` | macOS code signing |
| `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` | macOS notarization |
| (Windows) `WINDOWS_CERTIFICATE` | Authenticode signing |

Generate the updater keypair with `npm run tauri signer generate`. Until a
keypair exists and `plugins.updater` is configured in `tauri.conf.json`, the
in-app auto-updater stays disabled (see `docs/DECISIONS.md`).

## 5. Where user data lives

Per-user, on the local machine only (identifier `com.neonexai.promptflow-stt`):

| Data | Location |
|------|----------|
| API keys | OS keychain (Credential Manager / Keychain / libsecret) |
| Settings | `<app_data_dir>/settings.json` |
| Usage log | `<app_data_dir>/usage.db` (SQLite; metadata only, no text content) |

`<app_data_dir>` is `%APPDATA%\com.neonexai.promptflow-stt` (Windows),
`~/Library/Application Support/com.neonexai.promptflow-stt` (macOS), or
`~/.local/share/com.neonexai.promptflow-stt` (Linux). See `docs/PRIVACY.md` for
deletion instructions.

## 6. Support & troubleshooting

| Symptom | Likely cause / fix |
|---------|--------------------|
| "No API key configured for provider: X" | Add the key in Settings → API Keys. |
| "Invalid API key — check Settings → API Keys" | The provider returned 401/403; the stored key is wrong or revoked. |
| "Privacy Mode is on — X would send data off-device" | Disable Privacy Mode, or switch to `ollama` / a localhost `custom` endpoint. |
| "OpenRouter requires a model" / "Custom provider requires a base URL" | These providers need extra configuration before use. |
| Linux build fails on `alsa` | Run `./scripts/setup-linux.sh`. |
| Rust build panics on `generate_context!` (`icon ... is not RGBA`) | Icons must be RGBA PNG/ICO/ICNS; regenerate from `public/assets/brand/logo.svg`. |

Security issues: **do not** open a public issue — use GitHub Security
Advisories (see `SECURITY.md`).
