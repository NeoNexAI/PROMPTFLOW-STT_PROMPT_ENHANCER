# Changelog

All notable changes to PromptFlow STT are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Bundled app icons are now valid **RGBA** PNG/ICO/ICNS. The previous RGB icons
  caused `tauri::generate_context!` to panic at compile time, breaking every
  Rust build (and the `clippy` + `test` CI job).
- Keychain unit tests are now **hermetic** — they use the in-memory `keyring`
  mock store, so they pass in headless CI and sandboxes. The full set/get/delete
  round-trip is marked `#[ignore]` (run locally with `cargo test -- --ignored`)
  because it requires a real OS secret service.

### Changed
- Bundle identifier `com.clawd.promptflow-stt` → `com.neonexai.promptflow-stt`
  (no released users affected; this changes the keychain namespace and app-data
  directory).
- Corrected stale repository references (`JonatanGhub/PromptFlow-Speech2Text` →
  `NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER`), the README logo path, badge URLs,
  the updater endpoint, and security contact (now GitHub Security Advisories).

### Added
- `SECURITY.md` (coordinated disclosure policy) and this `CHANGELOG.md`.

## [0.1.0] — Core Overlay

The first functional milestone: a working clipboard-to-clipboard text
enhancement loop.

### Added
- Frameless, always-on-top, transparent overlay window (480×320).
- Global hotkey (`Ctrl/Cmd+Shift+E`) that reads the clipboard and shows the overlay.
- Text enhancement via the OpenAI (`gpt-4o-mini`) and Groq providers.
- Three enhancement modes: Fix Grammar, Formalize, Shorten.
- API key storage in the OS keychain with a masked settings UI.
- Settings persistence (non-secret fields) to the app data directory.
- Automatic write-back of the enhanced result to the clipboard.
- Typed IPC error surface (`AppError`) and a Rust unit-test suite.
