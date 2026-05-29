# AI Integrations — PromptFlow STT

## 1. AI Provider Contract

All AI providers implement the `AIProvider` trait defined in `src-tauri/src/providers/mod.rs`:

```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError>;
    fn provider_id(&self) -> &'static str;
    fn requires_api_key(&self) -> bool;
}

pub struct ProviderResponse {
    pub text: String,
    pub tokens_used: u32,
    pub cost_usd: f64,
}
```

API keys are stored in the OS keychain via `storage/keychain.rs` using the key format `promptflow-stt/<provider_id>`. Keys are never passed through the frontend and never written to disk in plaintext.

> **Bootstrap phase note:** Only `src-tauri/src/providers/mod.rs` (the trait definition) exists in the current scaffold. Individual provider files (`src-tauri/src/providers/<name>.rs`) are created during the v0.1 sprint when provider implementations are built. The Rust struct names listed in each section below refer to the future implementation files.

---

### openai

| Field | Value |
|---|---|
| Provider ID | `openai` |
| Rust struct | `OpenAIProvider` (`src-tauri/src/providers/openai.rs`) |
| Endpoint | `https://api.openai.com/v1/chat/completions` |
| Auth header | `Authorization: Bearer <key>` |
| Default model | `gpt-4o-mini` |
| Input cost | ~$0.15 / 1M tokens |
| Output cost | ~$0.60 / 1M tokens |
| Data retention | 30 days (API tier, not used for training by default) |
| Offline capable | No |

**Notes:** `gpt-4o-mini` is selected as the default for cost efficiency. Users can override the model via Settings → Provider → Model override field. The request body uses the `messages` format with `role: "system"` and `role: "user"` entries. Token usage is read from `response.usage.prompt_tokens` and `response.usage.completion_tokens`.

---

### anthropic

| Field | Value |
|---|---|
| Provider ID | `anthropic` |
| Rust struct | `AnthropicProvider` (`src-tauri/src/providers/anthropic.rs`) |
| Endpoint | `https://api.anthropic.com/v1/messages` |
| Auth header | `x-api-key: <key>` |
| Default model | `claude-haiku-4-5` |
| Input cost | ~$0.25 / 1M tokens |
| Output cost | ~$1.25 / 1M tokens |
| Data retention | 30 days (API logs) |
| Offline capable | No |

**Notes:** The Anthropic API requires an additional header `anthropic-version: 2023-06-01`. The system prompt is passed in the top-level `system` field (not inside `messages`). Token usage is read from `response.usage.input_tokens` and `response.usage.output_tokens`. `claude-haiku-4-5` is selected for speed and cost; Sonnet/Opus are available as user overrides.

---

### gemini

| Field | Value |
|---|---|
| Provider ID | `gemini` |
| Rust struct | `GeminiProvider` (`src-tauri/src/providers/gemini.rs`) |
| Endpoint | `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent` |
| Auth header | `x-goog-api-key: <key>` |
| Default model | `gemini-1.5-flash` |
| Input cost | Free tier: 15 RPM, 1M TPM; paid tier: ~$0.075 / 1M tokens (input) |
| Output cost | Free tier: included; paid tier: ~$0.30 / 1M tokens |
| Data retention | Google API terms; data may be used to improve models unless enterprise agreement |
| Offline capable | No |

**Notes:** The model is substituted into the URL path. The system instruction is passed via the `system_instruction` top-level field. Content is wrapped in `contents[].parts[].text`. Token usage is read from `response.usageMetadata.promptTokenCount` and `response.usageMetadata.candidatesTokenCount`. Rate limits on the free tier (15 RPM) are managed by the retry policy in §3.

---

### ollama

| Field | Value |
|---|---|
| Provider ID | `ollama` |
| Rust struct | `OllamaProvider` (`src-tauri/src/providers/ollama.rs`) |
| Endpoint | `http://localhost:11434/api/chat` |
| Auth header | None required |
| Default model | `llama3.2` (user-configurable in Settings) |
| Input cost | $0 (local inference) |
| Output cost | $0 (local inference) |
| Data retention | None — all inference is local, no data leaves the device |
| Offline capable | Yes |

**Notes:** Ollama must be running as a local process on the user's machine before use. `OllamaProvider.requires_api_key()` returns `false`. The request body uses the OpenAI-compatible `/api/chat` format. `cost_usd` is always returned as `0.0`. The Settings → Provider page shows a health indicator for the Ollama endpoint (reachable / not reachable) via a lightweight GET to `http://localhost:11434/` on settings panel open. This is one of two providers available in Privacy Mode.

---

### groq

| Field | Value |
|---|---|
| Provider ID | `groq` |
| Rust struct | `GroqProvider` (`src-tauri/src/providers/groq.rs`) |
| Endpoint | `https://api.groq.com/openai/v1/chat/completions` |
| Auth header | `Authorization: Bearer <key>` |
| Default model | `llama-3.1-8b-instant` |
| Input cost | Free tier: 30 RPM, 6,000 RPD; paid tier pricing varies by model |
| Output cost | Free tier: included; paid tier pricing varies by model |
| Data retention | Groq API terms; data not used for training |
| Offline capable | No |

**Notes:** Groq's API is OpenAI-compatible — the `GroqProvider` implementation is a thin wrapper around the same request/response format as `OpenAIProvider`, differing only in the base URL and model name. `llama-3.1-8b-instant` is chosen for sub-500 ms response times characteristic of Groq's LPU inference.

---

### mistral

| Field | Value |
|---|---|
| Provider ID | `mistral` |
| Rust struct | `MistralProvider` (`src-tauri/src/providers/mistral.rs`) |
| Endpoint | `https://api.mistral.ai/v1/chat/completions` |
| Auth header | `Authorization: Bearer <key>` |
| Default model | `mistral-small-latest` |
| Input cost | ~€0.10 / 1M tokens (billed in EUR; converted to USD for display) |
| Output cost | ~€0.30 / 1M tokens |
| Data retention | 30 days per Mistral API terms |
| Offline capable | No |

**Notes:** The Mistral API is OpenAI-compatible in request/response format. Cost estimates stored in `cost/tracker.rs` use a fixed EUR→USD conversion rate applied at build time; cost display in the UI notes that rates may vary. `mistral-small-latest` resolves to the current small model alias — pinning to a specific model version is available via the model override field.

---

### openrouter

| Field | Value |
|---|---|
| Provider ID | `openrouter` |
| Rust struct | `OpenRouterProvider` (`src-tauri/src/providers/openrouter.rs`) |
| Endpoint | `https://openrouter.ai/api/v1/chat/completions` |
| Auth header | `Authorization: Bearer <key>` |
| Default model | User-configurable (no single default — user must specify on first use) |
| Input cost | Varies by routed model (read from `response.usage` + per-model pricing) |
| Output cost | Varies by routed model |
| Data retention | OpenRouter terms; varies by underlying provider |
| Offline capable | No |

**Notes:** OpenRouter proxies requests to multiple underlying providers. The model field in the request body selects both the model and the routed provider (e.g., `"anthropic/claude-3-haiku"`). `OpenRouterProvider` reads `response.usage.prompt_tokens` and `response.usage.completion_tokens`. Cost is estimated using the per-model rate available from OpenRouter's pricing API, cached at app startup. The Settings page requires the user to enter a model string before enabling OpenRouter. An additional header `HTTP-Referer: https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER` and `X-Title: PromptFlow STT` is sent per OpenRouter's attribution requirements.

---

### custom

| Field | Value |
|---|---|
| Provider ID | `custom` |
| Rust struct | `CustomProvider` (`src-tauri/src/providers/custom.rs`) |
| Endpoint | User-defined (stored in settings file, not keychain) |
| Auth header | User-defined header name and value (value stored in keychain as `promptflow-stt/custom`) |
| Default model | User-defined |
| Input cost | N/A (cost tracking disabled for custom provider) |
| Output cost | N/A |
| Data retention | Determined by the user-controlled endpoint |
| Offline capable | Depends on the user-configured endpoint (localhost endpoints qualify for Privacy Mode) |

**Notes:** `CustomProvider` targets any OpenAI-compatible API endpoint. The user provides the base URL, auth header name, and API key in Settings → Provider → Custom. This enables use of self-hosted models (e.g., vLLM, LM Studio, LocalAI) beyond Ollama. When the endpoint is `localhost` or `127.0.0.1`, the provider qualifies as offline-capable and is selectable in Privacy Mode.

---

## 2. STT Engine Contract

All STT engines (except `web_speech`) implement the `STTEngine` trait defined in `src-tauri/src/stt/mod.rs`:

```rust
#[async_trait]
pub trait STTEngine: Send + Sync {
    async fn transcribe(&self, audio: Vec<f32>, sample_rate: u32) -> Result<String, AppError>;
    fn engine_id(&self) -> &'static str;
    fn requires_api_key(&self) -> bool;
}
```

Audio input is always 16 kHz mono PCM (`Vec<f32>`) captured by `audio/capture.rs` via `cpal`. Streaming engines also receive chunks in real time via an async channel and emit `stt://chunk` Tauri events for each partial transcript.

---

### whisper_api

| Field | Value |
|---|---|
| Engine ID | `whisper_api` |
| Rust struct | `WhisperApiEngine` (`src-tauri/src/stt/engines/whisper_api.rs`) |
| Mode | Cloud (batch) |
| API endpoint | `https://api.openai.com/v1/audio/transcriptions` |
| Latency target | < 2 seconds for 30 seconds of audio |
| Languages | 99 (Whisper model supported languages) |
| Requires API key | Yes (OpenAI API key, stored as `promptflow-stt/openai`) |
| Offline capable | No |

**Notes:** Audio is encoded to MP3 or WAV before upload. The request is `multipart/form-data` with the `model` field set to `whisper-1`. Because this is a batch engine, no `stt://chunk` events are emitted — the TextInput shows "Transcribing…" while the request is in flight, then populates on `stt://done`. Shares the OpenAI API key with the OpenAI AI provider.

---

### whisper_cpp

| Field | Value |
|---|---|
| Engine ID | `whisper_cpp` |
| Rust struct | `WhisperCppEngine` (`src-tauri/src/stt/engines/whisper_cpp.rs`) |
| Mode | Local (binary execution) |
| API/endpoint | System path to `whisper-cli` binary (configurable in Settings) |
| Latency target | < 5 seconds for 30 seconds of audio (CPU); ~1 second (GPU with CUDA/Metal) |
| Languages | 99 (Whisper model supported languages) |
| Requires API key | No |
| Offline capable | Yes |

**Notes:** `WhisperCppEngine` invokes the `whisper-cli` binary via `std::process::Command`, passing audio as a WAV file written to a temp directory and reading the transcript from stdout. The model file path (e.g., `ggml-base.en.bin`) is configurable in Settings. This engine is one of two STT engines selectable in Privacy Mode. Binary path validation is performed at settings-save time to surface a clear error if the binary is not found.

---

### deepgram

| Field | Value |
|---|---|
| Engine ID | `deepgram` |
| Rust struct | `DeepgramEngine` (`src-tauri/src/stt/engines/deepgram.rs`) |
| Mode | Cloud (streaming WebSocket) |
| API endpoint | `wss://api.deepgram.com/v1/listen` |
| Latency target | < 200 ms per partial transcript chunk (streaming) |
| Languages | 36 (Deepgram Nova-2 supported languages) |
| Requires API key | Yes (stored as `promptflow-stt/deepgram`) |
| Offline capable | No |

**Notes:** Deepgram uses a persistent WebSocket connection during recording. Audio chunks are streamed as binary frames in real time. Each interim result from Deepgram is forwarded as a `stt://chunk` Tauri event to the frontend. The WebSocket is closed when `stop_recording` is called or silence is detected. Query parameters include `model=nova-2`, `language=en`, `interim_results=true`, and `punctuate=true`.

---

### assembly_ai

| Field | Value |
|---|---|
| Engine ID | `assembly_ai` |
| Rust struct | `AssemblyAiEngine` (`src-tauri/src/stt/engines/assembly_ai.rs`) |
| Mode | Cloud (batch with streaming option) |
| API endpoint | `https://api.assemblyai.com/v2/transcript` |
| Latency target | < 3 seconds for 30 seconds of audio (batch mode) |
| Languages | 99 languages + speaker diarization |
| Requires API key | Yes (stored as `promptflow-stt/assemblyai`) |
| Offline capable | No |

**Notes:** AssemblyAI's real-time streaming API (`wss://api.assemblyai.com/v2/realtime/ws`) is used when streaming mode is selected in Settings; this enables `stt://chunk` events at ~400 ms intervals. The batch endpoint is the default for simplicity. Speaker diarization is available but disabled by default (adds ~1s of latency). Auth is via `Authorization: <key>` header (no `Bearer` prefix).

---

### google_stt

| Field | Value |
|---|---|
| Engine ID | `google_stt` |
| Rust struct | `GoogleSttEngine` (`src-tauri/src/stt/engines/google_stt.rs`) |
| Mode | Cloud (batch) |
| API endpoint | `https://speech.googleapis.com/v1/speech:recognize` |
| Latency target | < 2 seconds for 30 seconds of audio |
| Languages | 125 (Google Cloud Speech-to-Text supported locales) |
| Requires API key | Yes (Google Cloud API key, stored as `promptflow-stt/google`) |
| Offline capable | No |

**Notes:** Audio is base64-encoded and sent as JSON. The request specifies `encoding: "LINEAR16"`, `sampleRateHertz: 16000`, and `languageCode` (default `"en-US"`, configurable in Settings). The streaming API (`StreamingRecognize` gRPC method) is not used in v0.2 to avoid the gRPC dependency; batch mode covers the initial implementation. Streaming support is a candidate enhancement for a future milestone.

---

### azure_stt

| Field | Value |
|---|---|
| Engine ID | `azure_stt` |
| Rust struct | `AzureSttEngine` (`src-tauri/src/stt/engines/azure_stt.rs`) |
| Mode | Cloud (batch) |
| API endpoint | `https://{region}.stt.speech.microsoft.com/speech/recognition/conversation/cognitiveservices/v1` |
| Latency target | < 2 seconds for 30 seconds of audio |
| Languages | 100+ (Azure Cognitive Services Speech supported languages) |
| Requires API key | Yes (Azure subscription key, stored as `promptflow-stt/azure`; region also required) |
| Offline capable | No |

**Notes:** The `{region}` segment of the endpoint URL is stored alongside the API key in settings (non-sensitive; stored in the settings JSON file). Auth header is `Ocp-Apim-Subscription-Key: <key>`. Audio is sent as `audio/wav` in the request body. Azure offers an enterprise SLA (99.9% uptime) making it appropriate for business deployments. The Settings panel shows a separate "Azure region" text field when Azure STT is selected.

---

### web_speech

| Field | Value |
|---|---|
| Engine ID | `web_speech` |
| Rust struct | None — frontend-only (WebView JS API) |
| Mode | Browser-native (cloud or OS-local, depending on platform) |
| API endpoint | Browser `SpeechRecognition` / `webkitSpeechRecognition` Web API |
| Latency target | < 500 ms per partial transcript (streaming; browser-controlled) |
| Languages | Varies by browser and OS (typically 50+ on modern platforms) |
| Requires API key | No |
| Offline capable | Depends on OS (macOS Sonoma+ supports offline; Windows 11 requires network for some voices) |

**Notes:** `web_speech` is implemented entirely in the frontend (`src/hooks/useSTT.ts`) using the browser's built-in `SpeechRecognition` API exposed inside the Tauri WebView. No Rust STT module is involved. `interimResults: true` enables real-time transcript updates. Because it runs in the WebView rather than the Rust process, partial transcripts are not emitted via `stt://chunk` events — they update `sessionStore.inputText` directly via React state. This engine is the lowest-friction option for users who do not want to install any external binaries or create API accounts.

---

## 3. Error Handling Contract

All provider and STT engine implementations must adhere to the following error handling rules. Deviations require explicit justification in the implementation PR.

**Error type mapping:**
- AI provider errors map to `AppError::Provider(String)`.
- STT engine errors map to `AppError::Stt(String)`.
- Both variants serialize to `{ "code": "Provider" | "Stt", "message": "..." }` over IPC.

**Retry policy:**
- HTTP 429 (rate limit exceeded): retry once after a 1-second delay (exponential backoff base; only one retry attempt before failing). The error message on final failure includes the retry-after duration if the `Retry-After` header is present in the response.
- HTTP 500, 502, 503, 504 (server error): retry once after a 2-second delay.
- All other HTTP error codes: fail immediately with no retry.

**Hard fail on authentication errors:**
- HTTP 401 (unauthorized) and HTTP 403 (forbidden): no retry. Return `AppError::Provider("Invalid API key — check Settings → API Keys")` immediately. The frontend displays this as an error banner with a direct link to the Settings panel.

**Timeouts:**
- Enhancement requests: 30-second hard timeout. On expiry: `AppError::Provider("Request timed out after 30s")`.
- STT transcription requests: 60-second hard timeout. On expiry: `AppError::Stt("Transcription timed out after 60s")`.
- Timeouts are implemented via `tokio::time::timeout` wrapping the async HTTP call.

**Network errors:**
- DNS resolution failure, TCP connection refused, TLS handshake failure: `AppError::Provider("Network error: <underlying message>")`. No retry (the error is likely persistent).

---

## 4. Privacy Implications Per Provider

| Provider | Sends text to | Data retention | GDPR compliant | Offline option |
|---|---|---|---|---|
| openai | OpenAI servers (US) | 30 days (API tier) | Yes — DPA available | No — requires openai provider |
| anthropic | Anthropic servers (US) | 30 days (API logs) | Yes — DPA available | No — requires anthropic provider |
| gemini | Google servers (global) | Per Google terms; may be used for model improvement unless enterprise agreement | Yes — standard contractual clauses | No — requires gemini provider |
| ollama | Localhost only — no data leaves device | N/A (local) | N/A | Yes — primary offline AI provider |
| groq | Groq servers (US) | Per Groq terms; not used for training | Yes — DPA available | No — requires groq provider |
| mistral | Mistral AI servers (EU) | 30 days per Mistral terms | Yes — GDPR-native (EU company) | No — requires mistral provider |
| openrouter | OpenRouter (US) then underlying provider | Varies by routed provider | Varies by routed provider | No — requires openrouter provider |
| custom | User-defined endpoint | User-controlled | User-controlled | Yes if endpoint is localhost |
| whisper_api | OpenAI servers (US) | 30 days | Yes — DPA available | No — STT via API |
| whisper_cpp | Localhost — binary runs locally | N/A (local) | N/A | Yes — primary offline STT engine |
| deepgram | Deepgram servers (US) | Per Deepgram terms; not used for training | Yes — DPA available | No |
| assembly_ai | AssemblyAI servers (US) | 72 hours after processing; configurable | Yes — DPA available | No |
| google_stt | Google servers (global) | Per Google Cloud terms | Yes — standard contractual clauses | No |
| azure_stt | Microsoft servers (user-selected region) | Per Azure terms; region choice enables EU-only processing | Yes — GDPR-native DPA | No |
| web_speech | OS/browser-controlled | OS/browser-controlled (may be device-local on modern OSes) | Varies by platform | Partially (OS-dependent) |

**Privacy Mode enforcement:** When Privacy Mode is enabled (`settingsStore.privacyMode = true`), the backend rejects any `enhance_text` or `start_recording` call that would route to a cloud provider. Only `ollama`, `custom` (localhost endpoints), `whisper_cpp`, and `web_speech` are accepted. This enforcement occurs in `commands/enhance.rs` and `commands/stt.rs` before the provider/engine is dispatched, ensuring cloud calls cannot occur even if the UI selector constraint is bypassed.
