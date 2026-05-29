# Contributing to PromptFlow STT

Thank you for your interest in contributing! This document explains everything you need to get started.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Prerequisites](#prerequisites)
- [Development Setup](#development-setup)
- [Branch Model](#branch-model)
- [Commit Conventions](#commit-conventions)
- [Testing Requirements](#testing-requirements)
- [Adding a New STT Engine](#adding-a-new-stt-engine)
- [Adding a New AI Provider](#adding-a-new-ai-provider)
- [Submitting a Pull Request](#submitting-a-pull-request)
- [Developer Certificate of Origin](#developer-certificate-of-origin)

---

## Code of Conduct

All contributors must follow our [Code of Conduct](CODE_OF_CONDUCT.md). We enforce it.

---

## Prerequisites

| Tool | Version | Notes |
|---|---|---|
| Rust | stable (see `rust-toolchain.toml`) | `rustup update stable` |
| Node.js | 20 (see `.nvmrc`) | Use nvm: `nvm use` |
| npm | 10+ | bundled with Node 20 |
| System deps | — | [Tauri prerequisites](https://tauri.app/start/prerequisites/) per OS |

---

## Development Setup

```bash
git clone https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER.git
cd PROMPTFLOW-STT_PROMPT_ENHANCER
nvm use            # Node 20
npm install
npm run tauri dev  # hot-reload dev window
```

Rust dependencies are compiled on first run (allow ~2 min). Subsequent builds are incremental.

---

## Branch Model

| Branch | Purpose |
|---|---|
| `main` | Always releasable. Protected. PRs only. |
| `feat/<scope>` | New features |
| `fix/<scope>` | Bug fixes |
| `docs/<scope>` | Documentation only |
| `chore/<scope>` | Tooling, deps, CI |
| `refactor/<scope>` | No behaviour change |
| `perf/<scope>` | Performance improvements |

Branch names must be lowercase, hyphen-separated. Example: `feat/deepgram-streaming`.

---

## Commit Conventions

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short description>

[optional body]

[optional footer(s)]
```

**Types**: `feat` · `fix` · `docs` · `chore` · `refactor` · `perf` · `test` · `ci` · `style`

**Scopes**: `stt` · `ai` · `overlay` · `settings` · `hotkeys` · `clipboard` · `storage` · `permissions` · `updater` · `telemetry` · `ci` · `deps`

**Examples:**
```
feat(stt): add Deepgram streaming WebSocket engine
fix(overlay): correct glass blur radius on Windows 11
docs(contributing): add DCO section
chore(deps): bump tauri to 2.1.0
```

Breaking changes: append `!` after type/scope and add `BREAKING CHANGE:` footer.

---

## Testing Requirements

All PRs must pass the full test suite locally before review.

### Rust

```bash
cargo fmt --check                    # formatting
cargo clippy -- -D warnings          # lint (zero warnings policy)
cargo test                           # unit + integration tests
cargo bench --no-run                 # verify benchmarks compile
```

### Frontend

```bash
npm run lint                         # ESLint
npm run type-check                   # tsc --noEmit
npm test                             # Vitest
```

### CI gate

CI runs the same commands on Windows, macOS, and Linux. A red CI blocks merge.

---

## Adding a New STT Engine

STT engines live in `src-tauri/src/stt/engines/`. Each engine implements the `SttEngine` trait:

```rust
// src-tauri/src/stt/engines/mod.rs
pub trait SttEngine: Send + Sync {
    async fn transcribe(&self, audio: &AudioChunk) -> Result<TranscriptResult>;
    fn name(&self) -> &'static str;
    fn is_streaming(&self) -> bool { false }
    fn requires_api_key(&self) -> bool { true }
}
```

Steps:
1. Create `src-tauri/src/stt/engines/<engine_name>.rs`
2. Implement `SttEngine`
3. Register in `src-tauri/src/stt/engines/mod.rs`
4. Add config fields in `src-tauri/src/storage/` if the engine needs settings
5. Add a Tauri command in `src-tauri/src/commands/stt.rs`
6. Wire the frontend toggle in `src/components/settings/`
7. Add the engine to `docs/specs/06_AI_INTEGRATIONS.md`
8. Add at least 2 unit tests (happy path + error path)

Open an [STT Provider Request](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/issues/new?template=stt_provider_request.yml) issue first so we can coordinate.

---

## Adding a New AI Provider

AI providers live in `src-tauri/src/providers/`. Each provider implements the `AiProvider` trait:

```rust
// src-tauri/src/providers/mod.rs
pub trait AiProvider: Send + Sync {
    async fn enhance(&self, request: &EnhanceRequest) -> Result<EnhanceResponse>;
    fn name(&self) -> &'static str;
    fn default_model(&self) -> &'static str;
    fn available_models(&self) -> Vec<ModelInfo>;
}
```

Steps:
1. Create `src-tauri/src/providers/<provider_name>.rs`
2. Implement `AiProvider`
3. Register in `src-tauri/src/providers/mod.rs`
4. Add API key storage in `src-tauri/src/storage/` (use OS keychain — never plain text)
5. Add cost tracking entries in `src-tauri/src/cost/`
6. Add the provider to `docs/specs/06_AI_INTEGRATIONS.md`
7. Add at least 2 unit tests with mocked HTTP responses

Open an [AI Provider Request](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/issues/new?template=ai_provider_request.yml) issue first.

---

## Submitting a Pull Request

1. Fork the repo and create a branch from `main`
2. Make your changes (keep commits atomic and well-described)
3. Run the full test suite locally (see [Testing Requirements](#testing-requirements))
4. Push your branch and open a PR against `main`
5. Fill out the PR template completely
6. Respond to review feedback promptly

PRs that:
- touch `src-tauri/` require at least one Rust reviewer
- change the release workflow require maintainer approval
- modify security-sensitive code (keychain, updater, permissions) trigger a CISO review

---

## Developer Certificate of Origin

By contributing, you certify that:

> I wrote this code myself **or** I have the right to contribute it under the MIT license, and I understand it will be publicly available under the same terms.

We use the [DCO](https://developercertificate.org/) (not a CLA). Add a sign-off to each commit:

```bash
git commit -s -m "feat(stt): add Deepgram streaming engine"
```

This appends `Signed-off-by: Your Name <email@example.com>` to the commit message.
