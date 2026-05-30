import { useCallback } from 'react'
import type { AIProvider } from '@/types'
import { useSettingsStore } from '@/stores/settingsStore'
import { useUIStore } from '@/stores/uiStore'
import type { STTEngine } from '@/types'
import { PROVIDERS, STT_ENGINES, providerRequiresApiKey } from '@/lib/catalog'
import { ApiKeyInput } from './ApiKeyInput'

export function SettingsWindow() {
  const provider = useSettingsStore((s) => s.provider)
  const setProvider = useSettingsStore((s) => s.setProvider)
  const sttEngine = useSettingsStore((s) => s.sttEngine)
  const setSttEngine = useSettingsStore((s) => s.setSttEngine)
  const setSettingsVisible = useUIStore((s) => s.setSettingsVisible)
  const setOverlayVisible = useUIStore((s) => s.setOverlayVisible)

  const handleClose = useCallback(() => {
    setSettingsVisible(false)
    setOverlayVisible(true) // go back to overlay instead of hiding the window
  }, [setSettingsVisible, setOverlayVisible])

  const handleProviderChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setProvider(e.target.value as AIProvider)
    },
    [setProvider],
  )

  const needsKey = providerRequiresApiKey(provider)

  const handleSttChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      setSttEngine(e.target.value as STTEngine)
    },
    [setSttEngine],
  )

  return (
    <div
      className="w-[480px] min-h-[320px] bg-background border border-border rounded-[12px] shadow-[0_25px_50px_rgba(0,0,0,0.5)] flex flex-col"
      role="dialog"
      aria-label="PromptFlow settings"
      aria-modal="true"
    >
      {/* Header — drag region so the window is moveable */}
      {/* eslint-disable-next-line @typescript-eslint/ban-ts-comment */}
      {/* @ts-ignore */}
      <div data-tauri-drag-region className="h-8 w-full flex items-center justify-between px-3 shrink-0 border-b border-border">
        <span className="text-sm font-semibold text-foreground select-none" data-tauri-drag-region>Settings</span>
        <button
          type="button"
          onClick={handleClose}
          aria-label="Close settings"
          className="text-muted-foreground hover:text-foreground transition-colors"
        >
          ✕
        </button>
      </div>

      {/* Content */}
      <div className="flex flex-col gap-4 p-4">
        {/* Provider section */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">AI Provider</h2>
          <div className="flex flex-col gap-1 max-h-40 overflow-y-auto pr-1">
            {PROVIDERS.map((p) => (
              <label key={p.id} className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="provider"
                  value={p.id}
                  checked={provider === p.id}
                  onChange={handleProviderChange}
                  className="accent-primary"
                />
                <span className="text-sm text-foreground">{p.label}</span>
              </label>
            ))}
          </div>
        </section>

        {/* API Key section — only for providers that require a key */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">
            API Key — {provider}
          </h2>
          {needsKey ? (
            <ApiKeyInput provider={provider} />
          ) : (
            <p className="text-xs text-muted-foreground">
              {provider === 'ollama'
                ? 'Ollama runs locally and needs no API key. Make sure the Ollama server is running on http://localhost:11434.'
                : 'The custom provider uses your own OpenAI-compatible endpoint — no key is stored unless your endpoint requires one.'}
            </p>
          )}
        </section>

        {/* Voice dictation (STT) section */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">Voice dictation</h2>
          <select
            aria-label="STT engine"
            value={sttEngine}
            onChange={handleSttChange}
            className="w-full bg-secondary text-secondary-foreground text-sm rounded-[var(--radius)] px-2 py-1.5 border border-border"
          >
            {STT_ENGINES.map((e) => (
              <option key={e.id} value={e.id} disabled={!e.implemented}>
                {e.label}
              </option>
            ))}
          </select>
          <p className="text-xs text-muted-foreground mt-1">
            Press Ctrl/Cmd+Shift+D or the mic button to dictate. Whisper API uses your OpenAI key.
          </p>
        </section>
      </div>
    </div>
  )
}
