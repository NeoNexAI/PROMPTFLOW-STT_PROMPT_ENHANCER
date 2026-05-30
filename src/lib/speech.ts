//! Thin, typed wrapper around the browser Web Speech API
//! (`SpeechRecognition` / `webkitSpeechRecognition`) used by the `web_speech`
//! STT engine. This runs entirely in the WebView — no Rust involvement.

export interface SpeechController {
  stop: () => void
}

export type SpeechResultHandler = (text: string, isFinal: boolean) => void

interface SpeechAlternative {
  transcript: string
}
interface SpeechResult {
  readonly length: number
  isFinal: boolean
  [index: number]: SpeechAlternative
}
interface SpeechResultList {
  readonly length: number
  [index: number]: SpeechResult
}
interface SpeechRecognitionEventLike {
  resultIndex: number
  results: SpeechResultList
}
interface MinimalSpeechRecognition {
  lang: string
  continuous: boolean
  interimResults: boolean
  onresult: ((e: SpeechRecognitionEventLike) => void) | null
  onerror: ((e: { error?: string }) => void) | null
  onend: (() => void) | null
  start: () => void
  stop: () => void
}
type SpeechRecognitionCtor = new () => MinimalSpeechRecognition

function getCtor(): SpeechRecognitionCtor | undefined {
  const w = window as unknown as {
    SpeechRecognition?: SpeechRecognitionCtor
    webkitSpeechRecognition?: SpeechRecognitionCtor
  }
  return w.SpeechRecognition ?? w.webkitSpeechRecognition
}

/** True if this WebView exposes the Web Speech API. */
export function isWebSpeechSupported(): boolean {
  return getCtor() !== undefined
}

/**
 * Starts continuous recognition with interim results. `onResult` receives the
 * accumulated final text (with `isFinal=true`) or the current interim text
 * (`isFinal=false`). Returns a controller whose `stop()` ends recognition.
 */
export function startWebSpeech(
  onResult: SpeechResultHandler,
  onError: (message: string) => void,
): SpeechController {
  const Ctor = getCtor()
  if (!Ctor) {
    onError('Web Speech API is not supported in this environment')
    return { stop: () => {} }
  }
  const rec = new Ctor()
  rec.lang = navigator.language || 'en-US'
  rec.continuous = true
  rec.interimResults = true

  rec.onresult = (e) => {
    let finalText = ''
    let interim = ''
    for (let i = e.resultIndex; i < e.results.length; i++) {
      const res = e.results[i]
      const transcript = res[0]?.transcript ?? ''
      if (res.isFinal) finalText += transcript
      else interim += transcript
    }
    onResult(finalText || interim, finalText.length > 0)
  }
  rec.onerror = (ev) => onError(ev.error ?? 'speech recognition error')

  rec.start()
  return { stop: () => rec.stop() }
}
