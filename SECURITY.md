# Security Policy

## Supported Versions

PromptFlow STT is pre-1.0. Security fixes are applied to the latest released
version and `main`. Until v1.0, only the most recent minor release receives
security updates.

| Version | Supported |
|---------|-----------|
| latest `0.x` | ✅ |
| older `0.x` | ❌ |

## Reporting a Vulnerability

**Please do not open public GitHub issues for security vulnerabilities.**

Report privately through GitHub's coordinated disclosure flow:

1. Go to the repository **Security** tab →
   [**Report a vulnerability**](https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER/security/advisories/new).
2. Include:
   - A description of the vulnerability and its impact.
   - Steps to reproduce (proof-of-concept if possible).
   - Affected version(s) and platform(s).
   - Any suggested remediation.

We aim to acknowledge reports within **72 hours** and to ship a fix or
mitigation for confirmed high/critical issues within **30 days**. We will
credit reporters in the release notes unless anonymity is requested.

## Scope

PromptFlow STT is a **local-first, bring-your-own-key desktop application**.
It has no first-party backend: API keys are stored in the OS keychain and all
provider/STT calls originate from the user's machine. The security boundary and
trust model are documented in [`docs/THREAT_MODEL.md`](docs/THREAT_MODEL.md).

In scope:

- Leakage of API keys from the keychain, logs, or IPC surface.
- Bypass of Privacy Mode (any outbound network request while it is enabled).
- Local privilege escalation via the bundled binary, updater, or installer.
- Tampering with the auto-update channel (unsigned/forged updates).
- Injection via clipboard, STT transcript, or settings persisted to disk.

Out of scope:

- Vulnerabilities in third-party AI/STT providers' own services.
- Issues requiring a pre-compromised host (malware already running as the user).
- The contents users voluntarily send to a cloud provider they configured.

## Disclosure

We follow coordinated disclosure. Once a fix is released, we publish a GitHub
Security Advisory describing the issue, affected versions, and remediation.
