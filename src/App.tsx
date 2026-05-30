import { useEffect, useCallback } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useUIStore } from '@/stores/uiStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useHotkeys } from '@/hooks/useHotkeys'
import { useEnhancement } from '@/hooks/useEnhancement'
import { useDictation } from '@/hooks/useSTT'
import { OverlayWindow } from '@/components/overlay/OverlayWindow'
import { SettingsWindow } from '@/components/settings/SettingsWindow'
import { tauriApi } from '@/lib/tauri'
import { PROVIDERS } from '@/lib/catalog'

export default function App() {
  const overlayVisible = useUIStore((s) => s.overlayVisible)
  const settingsVisible = useUIStore((s) => s.settingsVisible)
  const setHasApiKey = useSettingsStore((s) => s.setHasApiKey)

  // Pre-check keychain so ApiKeyInput shows the correct initial state.
  // Only providers that actually require a key are checked.
  useEffect(() => {
    for (const { id, requiresApiKey } of PROVIDERS) {
      if (!requiresApiKey) continue
      tauriApi.hasApiKey(id)
        .then((has) => setHasApiKey(id, has))
        .catch(console.error)
    }
  }, [setHasApiKey])

  // Sync Tauri window visibility with the React store
  useEffect(() => {
    const win = getCurrentWindow()
    if (overlayVisible || settingsVisible) {
      win.show().catch(console.error)
    } else {
      win.hide().catch(console.error)
    }
  }, [overlayVisible, settingsVisible])

  // Subscribe to hotkey events emitted by the Rust backend
  useHotkeys()
  // Voice dictation: wires the dictate hotkey + stt://done listener
  const { toggle: toggleDictation } = useDictation()

  const { enhance } = useEnhancement()
  // Stable reference so OverlayWindow doesn't re-render on every parent update
  const handleEnhance = useCallback(() => { enhance() }, [enhance])
  const handleDictate = useCallback(() => { toggleDictation() }, [toggleDictation])

  return (
    <>
      {overlayVisible && !settingsVisible && (
        <OverlayWindow onEnhance={handleEnhance} onDictate={handleDictate} />
      )}
      {settingsVisible && (
        <SettingsWindow />
      )}
    </>
  )
}
