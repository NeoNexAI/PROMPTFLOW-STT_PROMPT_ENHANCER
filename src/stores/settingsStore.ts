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
  setProvider: (p: AIProvider) => void
  setSttEngine: (e: STTEngine) => void
  setSelectedMode: (m: EnhancementMode) => void
  setPrivacyMode: (v: boolean) => void
  setHotkeyEnhance: (k: string) => void
  setHotkeyDictate: (k: string) => void
  setHasApiKey: (provider: AIProvider, hasKey: boolean) => void
  setOnboarded: (v: boolean) => void
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
    }),
    { name: 'promptflow-settings' }
  )
)
