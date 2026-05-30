import { useEffect } from 'react'
import { tauriApi } from '@/lib/tauri'
import { useSettingsStore } from '@/stores/settingsStore'

/**
 * Mirrors the non-secret settings the **backend** needs (Privacy Mode, the
 * whisper.cpp paths, hotkeys) from the Zustand store into the Rust
 * `settings.json` whenever they change. Without this, `enhance_text`'s
 * Privacy-Mode check and the whisper.cpp engine would never see the user's
 * choices (the frontend store persists to localStorage only).
 */
export function useBackendSettingsSync() {
  const provider = useSettingsStore((s) => s.provider)
  const sttEngine = useSettingsStore((s) => s.sttEngine)
  const selectedMode = useSettingsStore((s) => s.selectedMode)
  const privacyMode = useSettingsStore((s) => s.privacyMode)
  const hotkeyEnhance = useSettingsStore((s) => s.hotkeyEnhance)
  const hotkeyDictate = useSettingsStore((s) => s.hotkeyDictate)
  const whisperCppBinary = useSettingsStore((s) => s.whisperCppBinary)
  const whisperCppModel = useSettingsStore((s) => s.whisperCppModel)

  useEffect(() => {
    tauriApi
      .setSettings({
        provider,
        stt_engine: sttEngine,
        selected_mode: selectedMode,
        privacy_mode: privacyMode,
        hotkey_enhance: hotkeyEnhance,
        hotkey_dictate: hotkeyDictate,
        whisper_cpp_binary: whisperCppBinary,
        whisper_cpp_model: whisperCppModel,
      })
      .catch(console.error)
  }, [
    provider,
    sttEngine,
    selectedMode,
    privacyMode,
    hotkeyEnhance,
    hotkeyDictate,
    whisperCppBinary,
    whisperCppModel,
  ])
}
