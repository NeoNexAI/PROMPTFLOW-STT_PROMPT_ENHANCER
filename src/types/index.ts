export type EnhancementMode =
  | 'fix_grammar'
  | 'formalize'
  | 'shorten'
  | 'expand'
  | 'translate'
  | 'brainstorm'
  | 'action_items'
  | 'summarize'
  | 'code_review'
  | 'simplify'
  | 'reframe'
  | 'custom'

export type STTEngine =
  | 'whisper_api'
  | 'whisper_cpp'
  | 'deepgram'
  | 'assembly_ai'
  | 'google_stt'
  | 'azure_stt'
  | 'web_speech'

export type AIProvider =
  | 'openai'
  | 'anthropic'
  | 'gemini'
  | 'ollama'
  | 'groq'
  | 'mistral'
  | 'openrouter'
  | 'custom'

export interface EnhanceRequest {
  text: string
  mode: EnhancementMode
  provider: AIProvider
}

export interface EnhanceResponse {
  result: string
  tokens_used: number
  cost_usd: number
}

export interface AppError {
  code: string
  message: string
}

export interface STTStatus {
  available: boolean
  reason?: string
}

export interface Settings {
  provider: AIProvider
  stt_engine: STTEngine
  selected_mode: EnhancementMode
  privacy_mode: boolean
  hotkey_enhance: string
  hotkey_dictate: string
  whisper_cpp_binary: string
  whisper_cpp_model: string
}
