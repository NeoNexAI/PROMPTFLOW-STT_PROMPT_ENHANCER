import { invoke } from '@tauri-apps/api/core'
import type { EnhanceResponse, Settings, STTStatus } from '@/types'

/** Optional per-request provider/mode overrides for {@link tauriApi.enhanceText}. */
export interface EnhanceOptions {
  /** System prompt used when `mode === 'custom'`. */
  customPrompt?: string
  /** Model override (e.g. `gpt-4o`, or the required model for OpenRouter). */
  model?: string
  /** Base URL override for the Ollama or custom provider. */
  baseUrl?: string
}

export const tauriApi = {
  enhanceText: (text: string, mode: string, provider: string, opts: EnhanceOptions = {}) =>
    invoke<EnhanceResponse>('enhance_text', {
      text,
      mode,
      provider,
      customPrompt: opts.customPrompt ?? null,
      model: opts.model ?? null,
      baseUrl: opts.baseUrl ?? null,
    }),

  startRecording: (engine: string) =>
    invoke<void>('start_recording', { engine }),

  stopRecording: () =>
    invoke<string>('stop_recording'),

  checkSttStatus: (engine: string) =>
    invoke<STTStatus>('check_stt_status', { engine }),

  getSettings: () =>
    invoke<Settings>('get_settings'),

  setSettings: (settings: Settings) =>
    invoke<void>('set_settings', { settings }),

  registerHotkey: (id: string, shortcut: string) =>
    invoke<void>('register_hotkey', { id, shortcut }),

  unregisterHotkey: (id: string) =>
    invoke<void>('unregister_hotkey', { id }),

  readClipboard: () =>
    invoke<string>('read_clipboard'),

  writeClipboard: (text: string) =>
    invoke<void>('write_clipboard', { text }),

  saveApiKey: (provider: string, key: string) =>
    invoke<void>('save_api_key', { provider, key }),

  hasApiKey: (provider: string) =>
    invoke<boolean>('has_api_key', { provider }),

  deleteApiKey: (provider: string) =>
    invoke<void>('delete_api_key', { provider }),
}
