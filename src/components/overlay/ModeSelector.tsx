import { useRef, KeyboardEvent } from 'react'
import type { EnhancementMode } from '@/types'
import { useSettingsStore } from '@/stores/settingsStore'
import { MODE_LABELS } from '@/lib/catalog'

interface ModeSelectorProps {
  modes: readonly EnhancementMode[]
}

export function ModeSelector({ modes }: ModeSelectorProps) {
  // selectedMode is persisted across overlay invocations — lives in settingsStore
  const activeMode = useSettingsStore((s) => s.selectedMode)
  const setActiveMode = useSettingsStore((s) => s.setSelectedMode)
  const pillRefs = useRef<(HTMLButtonElement | null)[]>([])

  const handleKeyDown = (e: KeyboardEvent<HTMLButtonElement>, index: number) => {
    if (e.key === 'ArrowRight') {
      e.preventDefault()
      const next = pillRefs.current[(index + 1) % modes.length]
      next?.focus()
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault()
      const prev = pillRefs.current[(index - 1 + modes.length) % modes.length]
      prev?.focus()
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      setActiveMode(modes[index])
    }
  }

  return (
    <div
      role="tablist"
      aria-label="Enhancement mode"
      className="flex flex-row gap-1.5 overflow-x-auto [&::-webkit-scrollbar]:hidden"
    >
      {modes.map((mode, index) => {
        const isActive = mode === activeMode
        const label = MODE_LABELS[mode] ?? mode
        return (
          <button
            key={mode}
            role="tab"
            aria-selected={isActive}
            tabIndex={isActive ? 0 : -1}
            ref={(el) => { pillRefs.current[index] = el }}
            onClick={() => setActiveMode(mode)}
            onKeyDown={(e) => handleKeyDown(e, index)}
            className={[
              'py-1 px-3 text-sm whitespace-nowrap shrink-0 rounded-[var(--radius)] transition-colors',
              isActive
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-secondary hover:text-secondary-foreground',
            ].join(' ')}
          >
            {label}
          </button>
        )
      })}
    </div>
  )
}
