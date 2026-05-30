# Changelog

All notable changes to PromptFlow STT are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — Settings UX (closing v0.2)
- **Privacy Mode toggle** in Settings; turning it on restricts the provider/STT
  selectors to offline options (and auto-switches to Ollama / whisper.cpp if
  needed). Backend enforcement was already in place.
- **OpenRouter / Custom configuration**: model field (OpenRouter) and base-URL +
  model fields (Custom), so those providers are now usable from the UI.
- **Custom-mode system prompt** textarea (threaded through `enhance_text`).
- **whisper.cpp paths** (binary + model) surfaced in Settings.
- **Frontend→backend settings sync** (`useBackendSettingsSync`): mirrors
  Privacy Mode, the whisper.cpp paths and hotkeys into the Rust `settings.json`
  so the backend actually sees them (the store otherwise persists to
  localStorage only).

### Added — STT depth (closing v0.2)
- **VAD auto-stop**: recording stops automatically after ~1.5 s of silence
  following speech; the backend emits `stt://autostop` and the frontend
  finalizes the transcript just like a manual stop. (`audio/vad.rs` +
  `audio/capture.rs`, unit-tested.)
- **whisper.cpp (local STT)**: offline transcription by invoking the
  `whisper-cli` binary; audio is resampled to 16 kHz (new unit-tested
  `resample_linear`) and written to a temp WAV. Binary/model paths come from
  settings. `make_engine` now takes an `EngineConfig`.

### Added — First-run onboarding
- A 3-step onboarding wizard (choose provider + save API key → review hotkeys →
  run a live test enhancement) shown automatically on first launch and skippable.
  Completion is persisted (`onboarded`), so the window opens straight into the
  wizard until the user is set up.

### Added — Voice dictation (v0.2)
- **Whisper API STT engine**: captured audio is encoded to WAV (new
  `audio/wav.rs`, unit-tested) and uploaded as multipart/form-data; shares the
  OpenAI key.
- **Web Speech engine**: browser `SpeechRecognition` streamed directly into the
  input field (`src/lib/speech.ts`), no key or backend required.
- **Microphone capture** via `cpal` on a dedicated thread (`audio/capture.rs`),
  with mono downmixing for F32/I16/U16 inputs.
- **Recording commands** (`start_recording`/`stop_recording`) backed by managed
  Tauri state; `stop_recording` transcribes and emits `stt://done`.
- **STT engine factory** + real `check_stt_status`; `Ctrl/Cmd+Shift+D` dictate
  hotkey; a mic toggle button and a "Recording" badge in the overlay; an STT
  engine selector in Settings.

### Added
- **Six new AI providers**: Anthropic, Google Gemini, Ollama (local), Mistral,
  OpenRouter and a Custom OpenAI-compatible endpoint — alongside OpenAI and Groq
  (8 total). OpenAI-compatible providers share one HTTP core
  (`providers/openai_compatible.rs`).
- **All 12 enhancement modes** are now selectable in the overlay (was 3), with a
  shared frontend catalog (`src/lib/catalog.ts`) of modes and providers.
- **SQLite usage logging** (`usage_log`): per-request metadata (mode, provider,
  tokens, estimated cost, lengths — never text), plus monthly-total and
  per-provider breakdown queries. Logging is fire-and-forget.
- **Server-side Privacy Mode enforcement**: `enhance_text` refuses cloud
  providers while Privacy Mode is on.
- `Settings::validate()` rejects unknown provider/STT/mode values and empty
  hotkeys at save time.
- Custom system prompt and model/base-URL overrides are plumbed through
  `enhance_text` and `tauriApi.enhanceText`.
- Multi-OS **release pipeline** (`.github/workflows/release.yml`, signing-ready),
  issue templates, a Linux dev-setup script (`scripts/setup-linux.sh`), and
  `docs/OPERATIONS.md` + `docs/DECISIONS.md`.

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
