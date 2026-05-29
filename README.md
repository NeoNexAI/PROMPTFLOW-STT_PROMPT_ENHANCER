<div align="center">
  <img src="public/assets/brand/logo.svg" alt="PromptFlow STT" width="120" height="120" />
  <h1>PromptFlow STT</h1>
  <p><strong>AI-powered prompt enhancement + intelligent voice dictation — open source, privacy-first.</strong></p>

  [![CI](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/actions/workflows/ci.yml/badge.svg)](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/actions/workflows/ci.yml)
  [![Security](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/actions/workflows/security.yml/badge.svg)](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/actions/workflows/security.yml)
  [![License: MIT](https://img.shields.io/badge/License-MIT-7c6af7.svg)](LICENSE)
  [![GitHub release](https://img.shields.io/github/v/release/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER)](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/releases)
  [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
</div>

---

## What is PromptFlow STT?

PromptFlow STT is a **desktop overlay** (Windows · macOS · Linux) that:

1. **Captures text** — from your clipboard or via voice dictation (local or cloud STT)
2. **Enhances it with AI** — applies one of 12 enhancement modes (Fix Grammar, Formalize, Shorten, Brainstorm…)
3. **Returns the result** — back to your clipboard or directly in the focused field, in under 300 ms

Built with **Tauri v2** (Rust + React/TypeScript): ~15 MB binary, ~60 MB RAM, no Electron overhead.

---

## Why PromptFlow STT?

| | PromptFlow STT | Raycast AI | TextSoap | Whisper App |
|---|:---:|:---:|:---:|:---:|
| Open source | ✅ | ❌ | ❌ | ✅ |
| Privacy mode (100% local) | ✅ | ❌ | ❌ | ✅ |
| Voice dictation | ✅ | ❌ | ❌ | ✅ |
| AI prompt enhancement | ✅ | ✅ | ❌ | ❌ |
| Cross-platform | ✅ | macOS only | macOS only | ✅ |
| Binary size | ~15 MB | ~200 MB | ~50 MB | ~25 MB |
| RAM at idle | ~60 MB | ~250 MB | ~80 MB | ~40 MB |
| Custom AI providers | ✅ | ❌ | ❌ | ❌ |

---

## Features

### Enhancement Modes
`Fix Grammar` · `Formalize` · `Shorten` · `Expand` · `Translate` · `Brainstorm`
`Action Items` · `Summarize` · `Code Review` · `Simplify` · `Reframe` · `Custom`

### Voice Dictation (STT Engines)
| Engine | Mode | Notes |
|---|---|---|
| Whisper API | Cloud | OpenAI, high accuracy |
| whisper.cpp | Local | Privacy mode, GPU-accelerated |
| Deepgram | Cloud streaming | Sub-200 ms latency |
| AssemblyAI | Cloud | Speaker diarization |
| Google Speech-to-Text | Cloud | 120+ languages |
| Azure Cognitive Speech | Cloud | Enterprise SLA |
| Web Speech API | Browser | No API key needed |

### AI Providers
OpenAI · Anthropic (Claude) · Google Gemini · Ollama (local) · Groq · Mistral · OpenRouter · Custom API

### Privacy & Security
- **Privacy Mode**: all processing stays on-device (whisper.cpp + Ollama)
- API keys stored in OS keychain (never in plain text)
- No telemetry by default — opt-in, anonymized, self-hostable
- Full audit trail: [THREAT_MODEL.md](docs/THREAT_MODEL.md) · [PRIVACY.md](docs/PRIVACY.md)

---

## Installation

### Download (recommended)
Grab the latest release from [Releases](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/releases):

| Platform | File |
|---|---|
| Windows | `PromptFlow-STT_x.y.z_x64-setup.exe` |
| macOS (Apple Silicon) | `PromptFlow-STT_x.y.z_aarch64.dmg` |
| macOS (Intel) | `PromptFlow-STT_x.y.z_x64.dmg` |
| Linux (Debian) | `PromptFlow-STT_x.y.z_amd64.deb` |
| Linux (AppImage) | `PromptFlow-STT_x.y.z_amd64.AppImage` |

### Build from source

**Prerequisites**: Rust stable · Node.js 20 · system deps per [Tauri prerequisites](https://tauri.app/start/prerequisites/)

```bash
git clone https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER.git
cd PROMPTFLOW-STT_PROMPT_ENHANCER
npm install
npm run tauri build
```

**Dev mode:**
```bash
npm run tauri dev
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  React/TypeScript Frontend (Tauri WebView)              │
│  Overlay · Settings · Onboarding                        │
└──────────────────────┬──────────────────────────────────┘
                       │ Tauri IPC (invoke / events)
┌──────────────────────▼──────────────────────────────────┐
│  Rust Backend (src-tauri/)                               │
│  commands/ · audio/ · stt/engines/ · enhancement/       │
│  providers/ · hotkeys/ · clipboard/ · storage/          │
│  permissions/ · updater/ · telemetry/ · cost/           │
└──────────────────────┬──────────────────────────────────┘
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
    OS Keychain    SQLite DB    AI/STT APIs
```

See [docs/specs/03_ARCHITECTURE.md](docs/specs/03_ARCHITECTURE.md) for the full technical architecture.

---

## Roadmap

| Version | Theme | ETA |
|---|---|---|
| v0.1 | Core overlay — clipboard capture, 3 modes, OpenAI | Week 6 |
| v0.2 | Voice dictation — Whisper API + whisper.cpp | Week 10 |
| v0.3 | More AI providers (Anthropic, Ollama) | Week 14 |
| v0.4 | Advanced STT (Deepgram streaming, AssemblyAI) | Week 18 |
| v0.5 | Privacy Mode (100% local) | Week 22 |
| v0.6 | OCR + screenshot capture | Week 26 |
| v0.7 | Cost tracking + usage analytics | Week 30 |
| v0.8 | Community prompt library | Week 34 |
| v1.0 | Production-ready + auto-updater | Week 38 |

Full roadmap: [docs/specs/07_ROADMAP.md](docs/specs/07_ROADMAP.md)

### Current status

Text enhancement is functional end-to-end: global hotkey → clipboard → AI →
clipboard, with **all 12 enhancement modes**, **8 AI providers** (OpenAI,
Anthropic, Gemini, Groq, Mistral, OpenRouter, Ollama, Custom), keychain-stored
keys, SQLite usage logging, and server-side Privacy Mode enforcement. Voice
dictation (STT engines), OCR, the onboarding wizard, the usage dashboard UI and
the in-app auto-updater are scaffolded but not yet wired — see the roadmap and
[docs/DECISIONS.md](docs/DECISIONS.md).

---

## Documentation

| Doc | Description |
|---|---|
| [docs/specs/01_PRD.md](docs/specs/01_PRD.md) | Product Requirements Document |
| [docs/specs/02_COMPETITIVE_ANALYSIS.md](docs/specs/02_COMPETITIVE_ANALYSIS.md) | Market analysis |
| [docs/specs/03_ARCHITECTURE.md](docs/specs/03_ARCHITECTURE.md) | Technical architecture |
| [docs/specs/04_FEATURES.md](docs/specs/04_FEATURES.md) | Feature specifications |
| [docs/specs/05_UI_UX.md](docs/specs/05_UI_UX.md) | Design system |
| [docs/specs/06_AI_INTEGRATIONS.md](docs/specs/06_AI_INTEGRATIONS.md) | AI & STT integrations |
| [docs/specs/08_PROJECT_STRUCTURE.md](docs/specs/08_PROJECT_STRUCTURE.md) | Codebase structure |
| [docs/specs/09_CODING_GUIDELINES.md](docs/specs/09_CODING_GUIDELINES.md) | Coding standards |
| [docs/PRIVACY.md](docs/PRIVACY.md) | Privacy policy |
| [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) | Security threat model |
| [docs/PERFORMANCE.md](docs/PERFORMANCE.md) | Performance targets |
| [docs/RELEASE.md](docs/RELEASE.md) | Release process |
| [docs/OPERATIONS.md](docs/OPERATIONS.md) | Build, test, release & support runbook |
| [docs/DECISIONS.md](docs/DECISIONS.md) | Architecture decision log |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |
| [SECURITY.md](SECURITY.md) | Security policy |

---

## Contributing

We welcome contributions of all sizes! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- How to add a new STT engine
- How to add a new AI provider
- Branch naming, commit conventions, and test requirements

First-time contributors: look for issues labelled [`good first issue`](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/issues?q=label%3A%22good+first+issue%22).

---

## License

MIT — see [LICENSE](LICENSE).

PromptFlow STT is **not** affiliated with any AI provider mentioned in this README.
