import { describe, it, expect } from 'vitest'
import {
  ALL_MODES,
  MODE_LABELS,
  PROVIDERS,
  STT_ENGINES,
  providerMeta,
  providerRequiresApiKey,
} from '@/lib/catalog'

describe('catalog — modes', () => {
  it('exposes all 12 modes, each with a label', () => {
    expect(ALL_MODES.length).toBe(12)
    for (const m of ALL_MODES) expect(MODE_LABELS[m]).toBeTruthy()
  })
})

describe('catalog — providers', () => {
  it('exposes 8 providers', () => {
    expect(PROVIDERS.length).toBe(8)
  })

  it('reports key requirements correctly', () => {
    expect(providerRequiresApiKey('openai')).toBe(true)
    expect(providerRequiresApiKey('ollama')).toBe(false)
    expect(providerRequiresApiKey('custom')).toBe(false)
  })

  it('marks the local providers as offline-capable', () => {
    expect(providerMeta('ollama')?.offlineCapable).toBe(true)
    expect(providerMeta('custom')?.offlineCapable).toBe(true)
    expect(providerMeta('openai')?.offlineCapable).toBe(false)
  })
})

describe('catalog — STT engines', () => {
  it('marks the three working engines as implemented', () => {
    const implemented = STT_ENGINES.filter((e) => e.implemented).map((e) => e.id)
    expect(implemented).toEqual(
      expect.arrayContaining(['whisper_api', 'web_speech', 'whisper_cpp']),
    )
  })

  it('keeps the offline flag consistent with Privacy Mode', () => {
    expect(STT_ENGINES.find((e) => e.id === 'whisper_cpp')?.offline).toBe(true)
    expect(STT_ENGINES.find((e) => e.id === 'whisper_api')?.offline).toBe(false)
  })
})
