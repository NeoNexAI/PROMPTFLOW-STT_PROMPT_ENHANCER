# Threat Model — PromptFlow STT

**Date:** 2026-04-22
**Status:** Approved

---

## Assets

| Asset | Value | Notes |
|---|---|---|
| API keys (OpenAI, Anthropic, etc.) | **High** | Billable credentials; leakage = unauthorized spend |
| Input / output text | **Medium** | May contain confidential drafts, medical/legal content |
| Usage log (SQLite) | **Low** | Tokens, costs, mode, provider — no text content |
| User settings (hotkeys, provider prefs) | **Low** | Non-sensitive |

---

## Threat Actors

| Actor | Capability | Goal |
|---|---|---|
| Malicious app on same machine | Read local files, inspect memory via debugging | Steal API keys or text content |
| Network attacker (MITM) | Intercept HTTPS traffic | Read API requests/responses in transit |
| Malicious npm / cargo package | Execute arbitrary code in the build process | Inject key-stealing code, backdoor the binary |
| Malicious web content in WebView | XSS / prototype pollution via the Tauri WebView | Call privileged IPC commands without user intent |

---

## Attack Surface & Mitigations

### OS Keychain access
- **Risk:** Another process on the same machine could request keychain entries for `promptflow-stt/*`.
- **Mitigation:** The OS keychain is isolated per app bundle identifier (`com.neonexai.promptflow-stt`). Other apps cannot access these entries without the user's explicit approval (macOS) or without matching the app's identity (Windows Credential Manager).

### HTTPS to AI APIs
- **Risk:** A network attacker intercepts the HTTPS connection and reads prompt text or API keys.
- **Mitigation:** All provider connections use TLS 1.2+. The `reqwest` HTTP client in Rust uses the system's native TLS stack (rustls or native-tls), which validates server certificates. Certificate pinning is deferred to post-v1.0.

### Supply chain (cargo + npm)
- **Risk:** A compromised transitive dependency injects malicious code into the build.
- **Mitigation:**
  - `cargo audit` and `npm audit` run in CI on every PR; jobs fail on high or critical vulnerabilities.
  - Dependency versions are pinned in `Cargo.lock` and `package-lock.json`.
  - Tauri's security review covers the core IPC bridge.

### Malicious web content / IPC abuse
- **Risk:** A compromised frontend script calls privileged Tauri commands (e.g., `read_clipboard`) without user action.
- **Mitigation:**
  - Tauri WebView runs with `dangerousDisableAssetCspModification: false` and a Content Security Policy that blocks inline scripts and external origins.
  - IPC capabilities in `src-tauri/capabilities/default.json` are scoped to the minimum required: only `clipboard-manager:allow-read-text`, `clipboard-manager:allow-write-text`, `global-shortcut:allow-register`, `global-shortcut:allow-unregister`, `global-shortcut:allow-is-registered` — no wildcard permissions.
  - Node.js integration is disabled in the WebView (Tauri v2 default).

### Local SQLite (usage.db)
- **Risk:** Another app reads the usage database.
- **Mitigation:** The database stores only token counts, costs, mode names, and provider names — no text content or API keys. Leakage has low impact.

---

## Explicitly Out of Scope

The following threat scenarios are **not mitigated** in this threat model:

- **Physical access to the machine:** An attacker with physical access and the user's login can access the keychain via the OS's own UI. This is an OS-level threat, not an application threat.
- **Compromised OS kernel or privileged malware:** A rootkit or kernel-level attacker bypasses all application-level protections.
- **Malicious user (intentional self-attack):** The user extracting their own keys or data is not a threat.

---

## Security Contact

Report vulnerabilities to GitHub Security Advisories (see `SECURITY.md`). We aim to respond within 72 hours. Please do not open public GitHub issues for security vulnerabilities.
