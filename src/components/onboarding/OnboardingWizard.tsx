import { useCallback, useMemo, useState } from 'react'
import type { AIProvider } from '@/types'
import { tauriApi } from '@/lib/tauri'
import { useSettingsStore } from '@/stores/settingsStore'
import { useUIStore } from '@/stores/uiStore'
import { PROVIDERS, providerRequiresApiKey } from '@/lib/catalog'
import { isMac } from '@/lib/platform'

const SAMPLE_TEXT = 'helo wrld, this sentance has a fewe erors to corect.'

function errMessage(e: unknown): string {
  if (e && typeof e === 'object' && 'message' in e) {
    return String((e as { message: unknown }).message)
  }
  return e instanceof Error ? e.message : String(e)
}

/**
 * First-run wizard: choose a provider + key, review the hotkeys, then run a
 * live test enhancement so the user confirms the setup before using the app.
 */
export function OnboardingWizard() {
  const provider = useSettingsStore((s) => s.provider)
  const setProvider = useSettingsStore((s) => s.setProvider)
  const setHasApiKey = useSettingsStore((s) => s.setHasApiKey)
  const setOnboarded = useSettingsStore((s) => s.setOnboarded)
  const setOnboardingVisible = useUIStore((s) => s.setOnboardingVisible)
  const setOverlayVisible = useUIStore((s) => s.setOverlayVisible)

  const [step, setStep] = useState(1)
  const [apiKey, setApiKey] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [testResult, setTestResult] = useState<string | null>(null)

  const needsKey = providerRequiresApiKey(provider)
  const enhanceShortcut = isMac ? '⌘⇧E' : 'Ctrl+Shift+E'
  const dictateShortcut = isMac ? '⌘⇧D' : 'Ctrl+Shift+D'

  const finish = useCallback(() => {
    setOnboarded(true)
    setOnboardingVisible(false)
    setOverlayVisible(false)
  }, [setOnboarded, setOnboardingVisible, setOverlayVisible])

  const saveKeyAndContinue = useCallback(async () => {
    setError(null)
    if (needsKey) {
      if (!apiKey.trim()) {
        setError('Enter your API key (or pick Ollama / Custom, which need no key).')
        return
      }
      setBusy(true)
      try {
        await tauriApi.saveApiKey(provider, apiKey.trim())
        setHasApiKey(provider, true)
      } catch (e) {
        setError(errMessage(e))
        setBusy(false)
        return
      }
      setBusy(false)
    }
    setApiKey('')
    setStep(2)
  }, [needsKey, apiKey, provider, setHasApiKey])

  const runTest = useCallback(async () => {
    setError(null)
    setTestResult(null)
    setBusy(true)
    try {
      const res = await tauriApi.enhanceText(SAMPLE_TEXT, 'fix_grammar', provider)
      setTestResult(res.result)
    } catch (e) {
      setError(errMessage(e))
    } finally {
      setBusy(false)
    }
  }, [provider])

  const progress = useMemo(() => `Step ${step} of 3`, [step])

  return (
    <div
      className="w-[480px] min-h-[320px] bg-background border border-border rounded-[12px] shadow-[0_25px_50px_rgba(0,0,0,0.5)] flex flex-col"
      role="dialog"
      aria-label="Welcome to PromptFlow"
      aria-modal="true"
    >
      {/* eslint-disable-next-line @typescript-eslint/ban-ts-comment */}
      {/* @ts-ignore — Tauri drag region */}
      <div data-tauri-drag-region className="h-8 flex items-center justify-between px-3 border-b border-border shrink-0">
        <span className="text-sm font-semibold text-foreground select-none" data-tauri-drag-region>
          Welcome to PromptFlow
        </span>
        <span className="text-xs text-muted-foreground">{progress}</span>
      </div>

      <div className="flex flex-col gap-3 p-4 flex-1 overflow-y-auto">
        {step === 1 && (
          <>
            <h2 className="text-sm font-medium text-foreground">1. Choose your AI provider</h2>
            <select
              aria-label="AI provider"
              value={provider}
              onChange={(e) => setProvider(e.target.value as AIProvider)}
              className="w-full bg-secondary text-secondary-foreground text-sm rounded-[var(--radius)] px-2 py-1.5 border border-border"
            >
              {PROVIDERS.map((p) => (
                <option key={p.id} value={p.id}>{p.label}</option>
              ))}
            </select>
            {needsKey ? (
              <input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={`Paste your ${provider} API key`}
                aria-label="API key"
                className="w-full bg-secondary text-secondary-foreground text-sm rounded-[var(--radius)] px-2 py-1.5 border border-border"
                onKeyDown={(e) => { if (e.key === 'Enter') void saveKeyAndContinue() }}
              />
            ) : (
              <p className="text-xs text-muted-foreground">
                This provider runs locally and needs no API key.
              </p>
            )}
            <p className="text-xs text-muted-foreground">
              Keys are stored in your OS keychain — never on disk or sent to us.
            </p>
          </>
        )}

        {step === 2 && (
          <>
            <h2 className="text-sm font-medium text-foreground">2. Your hotkeys</h2>
            <ul className="text-sm text-foreground flex flex-col gap-2">
              <li className="flex justify-between"><span>Enhance clipboard text</span><kbd className="bg-secondary px-2 py-0.5 rounded text-xs">{enhanceShortcut}</kbd></li>
              <li className="flex justify-between"><span>Dictate (voice)</span><kbd className="bg-secondary px-2 py-0.5 rounded text-xs">{dictateShortcut}</kbd></li>
            </ul>
            <p className="text-xs text-muted-foreground">
              Copy any text, press the enhance hotkey, pick a mode, and the result is
              copied back to your clipboard. You can change providers and the STT
              engine later in Settings.
            </p>
          </>
        )}

        {step === 3 && (
          <>
            <h2 className="text-sm font-medium text-foreground">3. Test your setup</h2>
            <p className="text-xs text-muted-foreground">
              We&apos;ll run &ldquo;Fix grammar&rdquo; on a sample sentence using <strong>{provider}</strong>.
            </p>
            <button
              type="button"
              onClick={() => void runTest()}
              disabled={busy}
              className="self-start bg-primary text-primary-foreground font-semibold px-4 py-2 rounded-[var(--radius)] text-sm disabled:opacity-50"
            >
              {busy ? 'Testing…' : 'Run test'}
            </button>
            {testResult !== null && (
              <div className="text-sm">
                <p className="text-xs text-muted-foreground mb-1">Result:</p>
                <p className="bg-secondary rounded-[var(--radius)] p-2 text-foreground">{testResult}</p>
              </div>
            )}
          </>
        )}

        {error && (
          <div role="alert" className="text-destructive text-xs flex items-center gap-1">
            <span>⚠</span><span>{error}</span>
          </div>
        )}
      </div>

      {/* Footer nav */}
      <div className="flex items-center justify-between px-4 py-3 border-t border-border shrink-0">
        <button
          type="button"
          onClick={finish}
          className="text-xs text-muted-foreground hover:text-foreground"
        >
          Skip
        </button>
        <div className="flex gap-2">
          {step > 1 && (
            <button
              type="button"
              onClick={() => { setError(null); setStep(step - 1) }}
              className="bg-secondary text-secondary-foreground px-4 py-2 rounded-[var(--radius)] text-sm"
            >
              Back
            </button>
          )}
          {step === 1 && (
            <button
              type="button"
              onClick={() => void saveKeyAndContinue()}
              disabled={busy}
              className="bg-primary text-primary-foreground font-semibold px-4 py-2 rounded-[var(--radius)] text-sm disabled:opacity-50"
            >
              {busy ? 'Saving…' : 'Next'}
            </button>
          )}
          {step === 2 && (
            <button
              type="button"
              onClick={() => setStep(3)}
              className="bg-primary text-primary-foreground font-semibold px-4 py-2 rounded-[var(--radius)] text-sm"
            >
              Next
            </button>
          )}
          {step === 3 && (
            <button
              type="button"
              onClick={finish}
              className="bg-primary text-primary-foreground font-semibold px-4 py-2 rounded-[var(--radius)] text-sm"
            >
              Finish
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
