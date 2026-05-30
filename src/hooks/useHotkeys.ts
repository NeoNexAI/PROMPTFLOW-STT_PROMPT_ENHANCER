import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useUIStore } from '@/stores/uiStore'
import { useSessionStore } from '@/stores/sessionStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { tauriApi } from '@/lib/tauri'

/**
 * Subscribes to backend `hotkey://enhance` events emitted when the OS hotkey fires.
 *
 * The global hotkey (Ctrl+Shift+E / Cmd+Shift+E) is registered in Rust at startup
 * (src-tauri/src/lib.rs `.setup()`). When it fires, Rust reads the clipboard and
 * emits `hotkey://enhance` with the captured text as payload.
 *
 * This hook:
 *  1. Populates `inputText` with the clipboard payload
 *  2. Clears previous `outputText` and `errorMessage`
 *  3. Shows the overlay window via `overlayVisible`
 *
 * Cleanup: the Tauri listener is removed on unmount to prevent memory leaks.
 */
export function useHotkeys() {
  const setOverlayVisible = useUIStore((s) => s.setOverlayVisible)
  const clearError = useUIStore((s) => s.clearError)
  const setError = useUIStore((s) => s.setError)
  const setInputText = useSessionStore((s) => s.setInputText)
  const setOutputText = useSessionStore((s) => s.setOutputText)
  const hotkeyEnhance = useSettingsStore((s) => s.hotkeyEnhance)
  const hotkeyDictate = useSettingsStore((s) => s.hotkeyDictate)

  // Apply the user's hotkeys on mount and whenever they change. A bad combo
  // surfaces as an error and leaves the previous bindings active (the backend
  // validates before unregistering).
  useEffect(() => {
    tauriApi.setHotkeys(hotkeyEnhance, hotkeyDictate).catch((e: unknown) => {
      const msg = e && typeof e === 'object' && 'message' in e ? String((e as { message: unknown }).message) : String(e)
      setError(`Hotkey error: ${msg}`)
    })
  }, [hotkeyEnhance, hotkeyDictate, setError])

  useEffect(() => {
    const unlistenPromise = listen<string>('hotkey://enhance', (event) => {
      setInputText(event.payload)
      setOutputText('')
      clearError()
      setOverlayVisible(true)
    })

    return () => {
      unlistenPromise.then((unlisten) => unlisten()).catch(console.error)
    }
  }, [setOverlayVisible, clearError, setInputText, setOutputText])
}
