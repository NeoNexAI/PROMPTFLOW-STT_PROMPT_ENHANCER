import { useCallback } from 'react'
import type { AIProvider, STTEngine } from '@/types'
import { useSettingsStore } from '@/stores/settingsStore'
import { useUIStore } from '@/stores/uiStore'
import { PROVIDERS, STT_ENGINES, providerMeta, providerRequiresApiKey } from '@/lib/catalog'
import { ApiKeyInput } from './ApiKeyInput'

const inputClass =
  'w-full bg-secondary text-secondary-foreground text-sm rounded-[var(--radius)] px-2 py-1.5 border border-border'

export function SettingsWindow() {
  const provider = useSettingsStore((s) => s.provider)
  const setProvider = useSettingsStore((s) => s.setProvider)
  const sttEngine = useSettingsStore((s) => s.sttEngine)
  const setSttEngine = useSettingsStore((s) => s.setSttEngine)
  const privacyMode = useSettingsStore((s) => s.privacyMode)
  const setPrivacyMode = useSettingsStore((s) => s.setPrivacyMode)
  const models = useSettingsStore((s) => s.models)
  const setModel = useSettingsStore((s) => s.setModel)
  const customBaseUrl = useSettingsStore((s) => s.customBaseUrl)
  const setCustomBaseUrl = useSettingsStore((s) => s.setCustomBaseUrl)
  const customPrompt = useSettingsStore((s) => s.customPrompt)
  const setCustomPrompt = useSettingsStore((s) => s.setCustomPrompt)
  const whisperCppBinary = useSettingsStore((s) => s.whisperCppBinary)
  const setWhisperCppBinary = useSettingsStore((s) => s.setWhisperCppBinary)
  const whisperCppModel = useSettingsStore((s) => s.whisperCppModel)
  const setWhisperCppModel = useSettingsStore((s) => s.setWhisperCppModel)
  const hotkeyEnhance = useSettingsStore((s) => s.hotkeyEnhance)
  const setHotkeyEnhance = useSettingsStore((s) => s.setHotkeyEnhance)
  const hotkeyDictate = useSettingsStore((s) => s.hotkeyDictate)
  const setHotkeyDictate = useSettingsStore((s) => s.setHotkeyDictate)
  const setSettingsVisible = useUIStore((s) => s.setSettingsVisible)
  const setOverlayVisible = useUIStore((s) => s.setOverlayVisible)

  const handleClose = useCallback(() => {
    setSettingsVisible(false)
    setOverlayVisible(true)
  }, [setSettingsVisible, setOverlayVisible])

  // Toggling Privacy Mode on forces an offline provider/engine if needed.
  const handlePrivacyToggle = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const on = e.target.checked
      setPrivacyMode(on)
      if (on) {
        if (!providerMeta(provider)?.offlineCapable) setProvider('ollama')
        if (!STT_ENGINES.find((x) => x.id === sttEngine)?.offline) setSttEngine('whisper_cpp')
      }
    },
    [provider, sttEngine, setPrivacyMode, setProvider, setSttEngine],
  )

  const needsKey = providerRequiresApiKey(provider)

  return (
    <div
      className="w-[480px] h-[420px] bg-background border border-border rounded-[12px] shadow-[0_25px_50px_rgba(0,0,0,0.5)] flex flex-col"
      role="dialog"
      aria-label="PromptFlow settings"
      aria-modal="true"
    >
      {/* eslint-disable-next-line @typescript-eslint/ban-ts-comment */}
      {/* @ts-ignore */}
      <div data-tauri-drag-region className="h-8 w-full flex items-center justify-between px-3 shrink-0 border-b border-border">
        <span className="text-sm font-semibold text-foreground select-none" data-tauri-drag-region>Settings</span>
        <button type="button" onClick={handleClose} aria-label="Close settings" className="text-muted-foreground hover:text-foreground transition-colors">✕</button>
      </div>

      <div className="flex flex-col gap-4 p-4 overflow-y-auto">
        {/* Privacy Mode */}
        <section>
          <label className="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" checked={privacyMode} onChange={handlePrivacyToggle} className="accent-primary" />
            <span className="text-sm font-medium text-foreground">Privacy Mode (100% on-device)</span>
          </label>
          <p className="text-xs text-muted-foreground mt-1">
            When on, only Ollama / a localhost Custom endpoint and whisper.cpp / Web Speech are allowed — no text leaves your machine.
          </p>
        </section>

        {/* Provider */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">AI Provider</h2>
          <div className="flex flex-col gap-1 max-h-32 overflow-y-auto pr-1">
            {PROVIDERS.map((p) => {
              const blocked = privacyMode && !p.offlineCapable
              return (
                <label key={p.id} className={`flex items-center gap-2 ${blocked ? 'opacity-40 cursor-not-allowed' : 'cursor-pointer'}`}>
                  <input
                    type="radio"
                    name="provider"
                    value={p.id}
                    checked={provider === p.id}
                    disabled={blocked}
                    onChange={(e) => setProvider(e.target.value as AIProvider)}
                    className="accent-primary"
                  />
                  <span className="text-sm text-foreground">{p.label}</span>
                </label>
              )
            })}
          </div>

          {/* Provider-specific config */}
          {provider === 'openrouter' && (
            <input
              className={`${inputClass} mt-2`}
              placeholder="Model (required), e.g. anthropic/claude-3-haiku"
              value={models.openrouter ?? ''}
              onChange={(e) => setModel('openrouter', e.target.value)}
              aria-label="OpenRouter model"
            />
          )}
          {provider === 'custom' && (
            <div className="flex flex-col gap-2 mt-2">
              <input
                className={inputClass}
                placeholder="Base URL, e.g. http://localhost:8000/v1"
                value={customBaseUrl}
                onChange={(e) => setCustomBaseUrl(e.target.value)}
                aria-label="Custom base URL"
              />
              <input
                className={inputClass}
                placeholder="Model (required)"
                value={models.custom ?? ''}
                onChange={(e) => setModel('custom', e.target.value)}
                aria-label="Custom model"
              />
            </div>
          )}
        </section>

        {/* API Key */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">API Key — {provider}</h2>
          {needsKey ? (
            <ApiKeyInput provider={provider} />
          ) : (
            <p className="text-xs text-muted-foreground">
              {provider === 'ollama'
                ? 'Ollama runs locally and needs no API key. Make sure the Ollama server is running on http://localhost:11434.'
                : 'The custom provider uses your own endpoint — no key is stored unless your endpoint requires one.'}
            </p>
          )}
        </section>

        {/* Custom mode prompt */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">Custom mode prompt</h2>
          <textarea
            className={`${inputClass} resize-none h-16`}
            placeholder="System prompt used when the Custom enhancement mode is selected"
            value={customPrompt}
            onChange={(e) => setCustomPrompt(e.target.value)}
            aria-label="Custom mode system prompt"
          />
        </section>

        {/* Voice dictation */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">Voice dictation</h2>
          <select
            aria-label="STT engine"
            value={sttEngine}
            onChange={(e) => setSttEngine(e.target.value as STTEngine)}
            className={inputClass}
          >
            {STT_ENGINES.map((e) => (
              <option key={e.id} value={e.id} disabled={!e.implemented || (privacyMode && !e.offline)}>
                {e.label}
              </option>
            ))}
          </select>
          {sttEngine === 'whisper_cpp' && (
            <div className="flex flex-col gap-2 mt-2">
              <input
                className={inputClass}
                placeholder="whisper-cli binary path"
                value={whisperCppBinary}
                onChange={(e) => setWhisperCppBinary(e.target.value)}
                aria-label="whisper.cpp binary path"
              />
              <input
                className={inputClass}
                placeholder="Model path, e.g. ggml-base.en.bin"
                value={whisperCppModel}
                onChange={(e) => setWhisperCppModel(e.target.value)}
                aria-label="whisper.cpp model path"
              />
            </div>
          )}
          <p className="text-xs text-muted-foreground mt-1">
            Press the dictate hotkey or the mic button to dictate.
          </p>
        </section>

        {/* Hotkeys */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">Hotkeys</h2>
          <div className="flex flex-col gap-2">
            <label className="flex flex-col gap-1">
              <span className="text-xs text-muted-foreground">Enhance</span>
              <input
                className={inputClass}
                value={hotkeyEnhance}
                onChange={(e) => setHotkeyEnhance(e.target.value)}
                placeholder="CommandOrControl+Shift+E"
                aria-label="Enhance hotkey"
              />
            </label>
            <label className="flex flex-col gap-1">
              <span className="text-xs text-muted-foreground">Dictate</span>
              <input
                className={inputClass}
                value={hotkeyDictate}
                onChange={(e) => setHotkeyDictate(e.target.value)}
                placeholder="CommandOrControl+Shift+D"
                aria-label="Dictate hotkey"
              />
            </label>
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Use names like <code>CommandOrControl</code>, <code>Shift</code>, <code>Alt</code> joined
            with <code>+</code> (e.g. <code>CommandOrControl+Shift+E</code>). Changes apply immediately.
          </p>
        </section>
      </div>
    </div>
  )
}
