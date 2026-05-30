import { useEffect } from 'react'
import { useUIStore } from '@/stores/uiStore'
import { useSessionStore } from '@/stores/sessionStore'
import { tauriApi } from '@/lib/tauri'
import { ModeSelector } from './ModeSelector'
import { ALL_MODES } from '@/lib/catalog'
import { TextInput } from './TextInput'
import { TextOutput } from './TextOutput'
import { ActionBar } from './ActionBar'

interface OverlayWindowProps {
  onEnhance: () => void
  onDictate: () => void
}

export function OverlayWindow({ onEnhance, onDictate }: OverlayWindowProps) {
  const isLoading = useUIStore((s) => s.isLoading)
  const errorMessage = useUIStore((s) => s.errorMessage)
  const setOverlayVisible = useUIStore((s) => s.setOverlayVisible)
  const setSettingsVisible = useUIStore((s) => s.setSettingsVisible)

  const inputText = useSessionStore((s) => s.inputText)
  const outputText = useSessionStore((s) => s.outputText)
  const isRecording = useSessionStore((s) => s.isRecording)
  const setInputText = useSessionStore((s) => s.setInputText)
  const reset = useSessionStore((s) => s.reset)

  // Close on Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setOverlayVisible(false)
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [setOverlayVisible])

  const handleCopy = async () => {
    if (!outputText) return
    await tauriApi.writeClipboard(outputText)
  }

  const handleClear = () => {
    reset()
  }

  return (
    <div
      className="w-[480px] min-h-[320px] bg-background border border-border rounded-[12px] shadow-[0_25px_50px_rgba(0,0,0,0.5)] flex flex-col"
      role="dialog"
      aria-label="PromptFlow overlay"
    >
      {/* Drag region — click-and-drag this bar to move the window */}
      {/* eslint-disable-next-line @typescript-eslint/ban-ts-comment */}
      {/* @ts-ignore — data-tauri-drag-region is a Tauri-specific data attribute */}
      <div
        data-tauri-drag-region
        className="h-8 w-full flex items-center justify-between px-3 shrink-0 cursor-move"
      >
        <span className="text-xs text-muted-foreground select-none flex items-center gap-2" data-tauri-drag-region>
          ⠿ PromptFlow
          {isRecording && (
            <span role="status" aria-label="Recording" className="flex items-center gap-1 text-destructive">
              <span className="w-2 h-2 rounded-full bg-destructive animate-pulse" />
              Recording
            </span>
          )}
        </span>
        <button
          onClick={() => { setOverlayVisible(false); setSettingsVisible(true) }}
          aria-label="Open settings"
          className="text-muted-foreground hover:text-foreground transition-colors p-0.5 rounded"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </button>
      </div>

      {/* Content */}
      <div className="flex flex-col gap-2 p-3 flex-1">
        <ModeSelector modes={ALL_MODES} />

        <TextInput
          value={inputText}
          onChange={setInputText}
          onSubmit={onEnhance}
          disabled={isLoading}
        />

        {errorMessage && (
          <div
            role="status"
            aria-live="polite"
            className="text-destructive text-xs flex items-center gap-1"
          >
            <span>⚠</span>
            <span>{errorMessage}</span>
          </div>
        )}

        <TextOutput value={outputText} />

        <ActionBar
          onEnhance={onEnhance}
          onCopy={() => { void handleCopy() }}
          onClear={handleClear}
          onDictate={onDictate}
          isLoading={isLoading}
          isRecording={isRecording}
          hasOutput={!!outputText}
        />
      </div>
    </div>
  )
}
