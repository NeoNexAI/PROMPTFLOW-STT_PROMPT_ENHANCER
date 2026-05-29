# Privacy — PromptFlow STT

**Date:** 2026-04-22
**Status:** Approved

---

## What we collect (default: nothing)

By default, PromptFlow STT collects **zero telemetry**. No usage data, no error reports, no analytics.

## What we collect (opt-in only)

If you explicitly opt in (prompted once at first launch), we send one anonymous event per enhancement:

| Field | Value | Notes |
|---|---|---|
| `event` | `enhancement_triggered` | Fixed string |
| `mode` | e.g., `fix_grammar` | Enhancement mode used |
| `provider` | e.g., `openai` | Provider ID (not model) |
| `tokens_used` | e.g., `312` | Token count from provider response |
| `latency_ms` | e.g., `1240` | End-to-end IPC latency |

**We never transmit:** your input text, your output text, your API keys, your IP address, or any PII.

Opt-in telemetry is sent to PostHog (self-hostable). You can opt out at any time in Settings → Privacy.

---

## Where data is stored

| Data | Location | Sensitive? |
|---|---|---|
| API keys | OS keychain (macOS Keychain / Windows Credential Manager / libsecret on Linux) | Yes — never written to disk in plaintext |
| Usage log (tokens, costs, mode, provider) | SQLite at `~/.local/share/com.neonexai.promptflow-stt/usage.db` (Linux) or platform equivalent | Low — no text content stored |
| Settings (hotkeys, selected provider, UI prefs) | Non-sensitive JSON config file (Tauri default store) | No |
| Input / output text | In-memory only, for the duration of the session | Ephemeral — not persisted anywhere |

---

## Your AI provider's privacy policy

When you send text to an AI provider (OpenAI, Anthropic, etc.), their privacy policy governs that data — not ours. We don't see it. We are not a data processor; the connection is direct from your machine to their API.

- [OpenAI Privacy Policy](https://openai.com/privacy)
- [Anthropic Privacy Policy](https://www.anthropic.com/privacy)

For full offline use with zero third-party data sharing, enable Privacy Mode (v0.5+), which restricts all operations to `ollama` and `whisper.cpp`.

---

## GDPR (EU users)

All data is stored locally on your device. We have no data processor relationship with you — we never receive, store, or process your personal data on our servers. Your AI provider may be a data processor under their own terms; consult their DPA if required.

---

## Deleting your data

- **Usage log:** delete `~/.local/share/com.neonexai.promptflow-stt/usage.db` (Linux), `%APPDATA%\com.neonexai.promptflow-stt\usage.db` (Windows), or `~/Library/Application Support/com.neonexai.promptflow-stt/usage.db` (macOS).
- **API keys:** remove via your OS keychain manager (Keychain Access on macOS, Credential Manager on Windows) — search for `promptflow-stt`.
- **Settings:** delete the Tauri config directory listed above.
- **Everything:** use Settings → Reset Application, which deletes all local data and keychain entries.

---

## Security contacts

Report security issues to GitHub Security Advisories (see `SECURITY.md`). PGP key available on request.
