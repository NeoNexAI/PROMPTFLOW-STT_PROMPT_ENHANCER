# Decision Log

Lightweight ADRs for decisions made while taking PromptFlow STT from a working
v0.1 toward production. Each entry: context → decision → rationale.

## Product direction: OSS desktop now, optional SaaS layer later

**Context.** The PRD defines a local-first, bring-your-own-key desktop app with
no backend/accounts/billing. The product owner confirmed the goal is to keep
that desktop core while leaving room for an *optional* future monetization
layer (not a backend pivot).

**Decision.** Keep the architecture local-first. Do **not** add auth, a
multi-tenant backend, or payment code now. Treat the generic "SaaS production"
concerns (user management, server backups, staging/prod environments) as **not
applicable** to the current product. Lay only *non-committal* foundations:
opt-in telemetry (already specced) and a documented forward path below.

**Rationale.** Building speculative backend/billing now would contradict the
PRD, add large attack surface and legal obligations (data processing, PCI), and
slow the actual product. The desktop app's value (privacy, BYOK, no
subscription) is the differentiator.

**Forward path for the optional layer (not built yet):** a "Pro" tier could add
managed cloud relay / hosted Ollama / a shared prompt-library sync. The cleanest
seam is an optional license-key check gating Pro-only features, validated
against a tiny stateless endpoint — additive, no change to the free local core.

## Build/CI unblock

**Decision.** Regenerate icons as RGBA; make keychain tests hermetic with the
`keyring` mock (round-trip marked `#[ignore]`); apply `cargo fmt`; fix a
`clone_on_copy` clippy error.

**Rationale.** The Rust CI job was red on three independent counts (invalid RGB
icons panicking `generate_context!`, non-hermetic keychain tests, and unformatted
code). Nothing downstream is trustworthy until the build and CI are green, so
this came first.

## Bundle identifier `com.clawd` → `com.neonexai`

**Decision.** Rename the identifier and fix stale `JonatanGhub/PromptFlow-Speech2Text`
references to the real repo `NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER`.

**Rationale.** The identifier namespaces the keychain and app-data directory.
There are no released users, so changing it now is free; doing it post-release
would orphan stored keys and settings.

## Provider architecture: shared OpenAI-compatible core + `ProviderConfig`

**Decision.** Factor the OpenAI `/chat/completions` flow into one module
(`providers/openai_compatible.rs`) that OpenAI, Groq, Ollama, Mistral,
OpenRouter and Custom delegate to; Anthropic and Gemini keep bespoke modules.
`make_provider` takes a `ProviderConfig { api_key, model, base_url }` and
validates required fields.

**Rationale.** Six of eight providers share an identical wire format —
duplicating it six times would be a maintenance hazard. A config struct cleanly
supports no-key providers (Ollama), endpoint overrides (Custom/Ollama) and
required fields (OpenRouter model) without widening every call site.

## Pricing in one place

**Decision.** All cost figures come from `cost::tracker::estimate_cost`; the
shared provider core calls it so the request path and the usage dashboard can
never disagree.

## SQLite usage log stores metadata only

**Decision.** `usage_log` records timestamp, mode, provider, tokens, cost and
input/output **lengths** — never the text. Writes are fire-and-forget on a
background thread.

**Rationale.** Satisfies the cost-tracking requirement repeated across the
feature specs while honoring the privacy promise (no prompt content at rest) and
the <300 ms latency target (logging never blocks the response).

## Privacy Mode enforced server-side

**Decision.** `enhance_text` rejects non-offline providers when Privacy Mode is
on, independent of the UI selector.

**Rationale.** The threat model treats the UI as untrusted for this guarantee;
enforcement must live in the Rust command so a bypassed selector cannot leak
data off-device.

## Auto-updater left disabled pending a signing keypair

**Decision.** Keep the updater plugin unregistered (as v0.1 chose) and ship a
**signing-ready** release workflow instead of wiring a placeholder key.

**Rationale.** Tauri's updater requires a real public key in `tauri.conf.json`;
a placeholder would either fail the build or ship an unverifiable update channel.
The release pipeline already consumes signing secrets when present, so enabling
the updater is a config-only step once a keypair is generated (see
`docs/OPERATIONS.md`).

## Release pipeline is signing-ready, not signing-required

**Decision.** `release.yml` builds all targets and publishes a **draft** release;
it signs/notarizes only when the relevant secrets exist.

**Rationale.** Lets the project produce installable artifacts immediately while
deferring the cost/logistics of EV and Apple Developer certificates, without
blocking the pipeline.
