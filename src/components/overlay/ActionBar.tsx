import { isMac } from '@/lib/platform'

const SHORTCUT_LABEL = isMac ? '⌘↵' : 'Ctrl↵'

interface ActionBarProps {
  onEnhance: () => void
  onCopy: () => void
  onClear: () => void
  onDictate: () => void
  isLoading: boolean
  isRecording: boolean
  hasOutput: boolean
}

export function ActionBar({
  onEnhance,
  onCopy,
  onClear,
  onDictate,
  isLoading,
  isRecording,
  hasOutput,
}: ActionBarProps) {
  return (
    <div className="flex flex-row items-center gap-2">
      {/* Enhance button */}
      <button
        type="button"
        onClick={onEnhance}
        disabled={isLoading}
        aria-label="Enhance text"
        className="flex items-center bg-primary text-primary-foreground font-semibold px-4 py-2 rounded-[var(--radius)] text-sm disabled:opacity-50 disabled:cursor-not-allowed hover:opacity-90 transition-opacity"
      >
        {isLoading ? (
          <svg
            className="animate-spin mr-2"
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            fill="none"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
        ) : null}
        Enhance
        <span className="text-xs text-primary-foreground/60 ml-2">{SHORTCUT_LABEL}</span>
      </button>

      {/* Copy Result button */}
      <button
        type="button"
        onClick={onCopy}
        disabled={!hasOutput}
        aria-label="Copy result"
        className="bg-secondary text-secondary-foreground px-4 py-2 rounded-[var(--radius)] text-sm disabled:opacity-50 disabled:cursor-not-allowed hover:opacity-90 transition-opacity"
      >
        Copy Result
      </button>

      {/* Dictate (voice) toggle */}
      <button
        type="button"
        onClick={onDictate}
        aria-label={isRecording ? 'Stop dictation' : 'Start dictation'}
        aria-pressed={isRecording}
        title="Dictate (Ctrl/Cmd+Shift+D)"
        className={[
          'flex items-center justify-center w-9 py-2 rounded-[var(--radius)] text-sm transition-colors',
          isRecording
            ? 'bg-destructive text-destructive-foreground animate-pulse'
            : 'bg-secondary text-secondary-foreground hover:opacity-90',
        ].join(' ')}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
          <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
          <line x1="12" y1="19" x2="12" y2="23" />
          <line x1="8" y1="23" x2="16" y2="23" />
        </svg>
      </button>

      {/* Clear button */}
      <button
        type="button"
        onClick={onClear}
        aria-label="Clear"
        className="text-muted-foreground hover:text-foreground px-3 py-2 text-sm transition-colors"
      >
        Clear
      </button>
    </div>
  )
}
