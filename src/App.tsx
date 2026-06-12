import { useEffect, useCallback } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { useUIStore } from '@/stores/uiStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useHotkeys } from '@/hooks/useHotkeys'
import { useEnhancement } from '@/hooks/useEnhancement'
import { useDictation } from '@/hooks/useSTT'
import { useBackendSettingsSync } from '@/hooks/useBackendSync'
import { OverlayWindow } from '@/components/overlay/OverlayWindow'
import { SettingsWindow } from '@/components/settings/SettingsWindow'
import { OnboardingWizard } from '@/components/onboarding/OnboardingWizard'
import { tauriApi } from '@/lib/tauri'
import { PROVIDERS } from '@/lib/catalog'

export default function App() {
  const overlayVisible = useUIStore((s) => s.overlayVisible)
  const settingsVisible = useUIStore((s) => s.settingsVisible)
  const onboardingVisible = useUIStore((s) => s.onboardingVisible)
  const setOnboardingVisible = useUIStore((s) => s.setOnboardingVisible)
  const setHasApiKey = useSettingsStore((s) => s.setHasApiKey)
  const onboarded = useSettingsStore((s) => s.onboarded)

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

  // First run: show the onboarding wizard until it's completed.
  useEffect(() => {
    if (!onboarded) setOnboardingVisible(true)
  }, [onboarded, setOnboardingVisible])

  // Sync Tauri window visibility AND size with the active view. The window is
  // not user-resizable, so we size it to fit each view (Settings is taller than
  // the overlay — otherwise its content was clipped by the fixed window height).
  useEffect(() => {
    const win = getCurrentWindow()
    const visible = overlayVisible || settingsVisible || onboardingVisible
    if (visible) {
      const height = onboardingVisible ? 470 : settingsVisible ? 480 : 360
      win.setSize(new LogicalSize(480, height)).catch(console.error)
      win.center().catch(() => {})
      win.show().catch(console.error)
    } else {
      win.hide().catch(console.error)
    }
  }, [overlayVisible, settingsVisible, onboardingVisible])

  // Subscribe to hotkey events emitted by the Rust backend
  useHotkeys()
  // Voice dictation: wires the dictate hotkey + stt://done listener
  const { toggle: toggleDictation } = useDictation()
  // Mirror non-secret settings (Privacy Mode, whisper.cpp paths, hotkeys) to the backend
  useBackendSettingsSync()

  const { enhance } = useEnhancement()
  // Stable reference so OverlayWindow doesn't re-render on every parent update
  const handleEnhance = useCallback(() => { enhance() }, [enhance])
  const handleDictate = useCallback(() => { toggleDictation() }, [toggleDictation])

  // Onboarding takes over the window when active.
  if (onboardingVisible) {
    return <OnboardingWizard />
  }

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
