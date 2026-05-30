import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { AIProvider, EnhancementMode, STTEngine } from '@/types'

interface SettingsState {
  provider: AIProvider
  sttEngine: STTEngine
  selectedMode: EnhancementMode
  privacyMode: boolean
  // camelCase in TS store; maps to hotkey_enhance/hotkey_dictate in Rust Settings struct
  // Tauri serde handles the snake_case <-> camelCase mapping via #[serde(rename_all)]
  hotkeyEnhance: string
  hotkeyDictate: string
  // hasApiKey tracks whether a key exists in the OS keychain — never stores the key itself
  hasApiKey: Record<AIProvider, boolean>
  /** True once the first-run onboarding wizard has been completed. Persisted. */
  onboarded: boolean
  /** Per-provider model override ('' / missing = provider default). */
  models: Partial<Record<AIProvider, string>>
  /** Base URL for the Custom provider (OpenAI-compatible endpoint root). */
  customBaseUrl: string
  /** System prompt used by the Custom enhancement mode. */
  customPrompt: string
  /** Local whisper.cpp binary + model paths (mirrored to the backend). */
  whisperCppBinary: string
  whisperCppModel: string
  setProvider: (p: AIProvider) => void
  setSttEngine: (e: STTEngine) => void
  setSelectedMode: (m: EnhancementMode) => void
  setPrivacyMode: (v: boolean) => void
  setHotkeyEnhance: (k: string) => void
  setHotkeyDictate: (k: string) => void
  setHasApiKey: (provider: AIProvider, hasKey: boolean) => void
  setOnboarded: (v: boolean) => void
  setModel: (provider: AIProvider, model: string) => void
  setCustomBaseUrl: (url: string) => void
  setCustomPrompt: (p: string) => void
  setWhisperCppBinary: (p: string) => void
  setWhisperCppModel: (p: string) => void
}

const defaultHasApiKey: Record<AIProvider, boolean> = {
  openai: false,
  anthropic: false,
  gemini: false,
  ollama: false,
  groq: false,
  mistral: false,
  openrouter: false,
  custom: false,
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      provider: 'openai',
      sttEngine: 'whisper_api',
      selectedMode: 'fix_grammar',
      privacyMode: false,
      hotkeyEnhance: 'CommandOrControl+Shift+E',
      hotkeyDictate: 'CommandOrControl+Shift+D',
      hasApiKey: defaultHasApiKey,
      onboarded: false,
      models: {},
      customBaseUrl: '',
      customPrompt: '',
      whisperCppBinary: '',
      whisperCppModel: '',
      setProvider: (provider) => set({ provider }),
      setSttEngine: (sttEngine) => set({ sttEngine }),
      setSelectedMode: (selectedMode) => set({ selectedMode }),
      setPrivacyMode: (privacyMode) => set({ privacyMode }),
      setHotkeyEnhance: (hotkeyEnhance) => set({ hotkeyEnhance }),
      setHotkeyDictate: (hotkeyDictate) => set({ hotkeyDictate }),
      setHasApiKey: (provider, hasKey) =>
        set((state) => ({
          hasApiKey: { ...state.hasApiKey, [provider]: hasKey },
        })),
      setOnboarded: (onboarded) => set({ onboarded }),
      setModel: (provider, model) =>
        set((state) => ({ models: { ...state.models, [provider]: model } })),
      setCustomBaseUrl: (customBaseUrl) => set({ customBaseUrl }),
      setCustomPrompt: (customPrompt) => set({ customPrompt }),
      setWhisperCppBinary: (whisperCppBinary) => set({ whisperCppBinary }),
      setWhisperCppModel: (whisperCppModel) => set({ whisperCppModel }),
    }),
    { name: 'promptflow-settings' }
  )
)
