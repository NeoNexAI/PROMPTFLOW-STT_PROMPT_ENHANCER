import type { AIProvider, EnhancementMode } from '@/types'

/** All 12 enhancement modes in display order. */
export const ALL_MODES = [
  'fix_grammar',
  'formalize',
  'shorten',
  'expand',
  'translate',
  'brainstorm',
  'action_items',
  'summarize',
  'code_review',
  'simplify',
  'reframe',
  'custom',
] as const satisfies readonly EnhancementMode[]

/** Short, human-friendly labels for each mode. */
export const MODE_LABELS: Record<EnhancementMode, string> = {
  fix_grammar: 'Fix grammar',
  formalize: 'Formalize',
  shorten: 'Shorten',
  expand: 'Expand',
  translate: 'Translate',
  brainstorm: 'Brainstorm',
  action_items: 'Action items',
  summarize: 'Summarize',
  code_review: 'Code review',
  simplify: 'Simplify',
  reframe: 'Reframe',
  custom: 'Custom',
}

export interface ProviderMeta {
  id: AIProvider
  label: string
  /** Whether an API key (stored in the OS keychain) is required. */
  requiresApiKey: boolean
  /** Whether the provider can run fully offline (eligible for Privacy Mode). */
  offlineCapable: boolean
}

/** All 8 AI providers with their metadata. Mirrors `providers::KNOWN_PROVIDERS`. */
export const PROVIDERS: readonly ProviderMeta[] = [
  { id: 'openai', label: 'OpenAI (gpt-4o-mini)', requiresApiKey: true, offlineCapable: false },
  { id: 'anthropic', label: 'Anthropic (claude-haiku-4-5)', requiresApiKey: true, offlineCapable: false },
  { id: 'gemini', label: 'Google Gemini (1.5-flash)', requiresApiKey: true, offlineCapable: false },
  { id: 'groq', label: 'Groq (llama-3.1-8b-instant, free)', requiresApiKey: true, offlineCapable: false },
  { id: 'mistral', label: 'Mistral (small-latest)', requiresApiKey: true, offlineCapable: false },
  { id: 'openrouter', label: 'OpenRouter (set model in settings)', requiresApiKey: true, offlineCapable: false },
  { id: 'ollama', label: 'Ollama (local, no key)', requiresApiKey: false, offlineCapable: true },
  { id: 'custom', label: 'Custom (OpenAI-compatible endpoint)', requiresApiKey: false, offlineCapable: true },
]

const PROVIDER_BY_ID = new Map(PROVIDERS.map((p) => [p.id, p]))

export function providerMeta(id: AIProvider): ProviderMeta | undefined {
  return PROVIDER_BY_ID.get(id)
}

export function providerRequiresApiKey(id: AIProvider): boolean {
  return providerMeta(id)?.requiresApiKey ?? true
}
