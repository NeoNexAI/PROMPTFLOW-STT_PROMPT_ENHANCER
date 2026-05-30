import { useCallback, useEffect, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { tauriApi } from '@/lib/tauri'
import { useSessionStore } from '@/stores/sessionStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useUIStore } from '@/stores/uiStore'
import { isWebSpeechSupported, startWebSpeech, type SpeechController } from '@/lib/speech'

/** Best-effort message extraction from a Tauri `AppError` or any thrown value. */
function errMessage(e: unknown): string {
  if (e && typeof e === 'object' && 'message' in e) {
    return String((e as { message: unknown }).message)
  }
  return String(e)
}

/**
 * Voice dictation. Supports two engine families:
 *  - `web_speech`: uses the browser SpeechRecognition API directly in the
 *    WebView, streaming interim results into `inputText`.
 *  - backend engines (e.g. `whisper_api`): `start_recording` captures audio in
 *    Rust; `stop_recording` returns the transcript (also emitted as
 *    `stt://done`).
 *
 * Mount this once (in `App`); it wires the `hotkey://dictate` toggle and the
 * `stt://done` listener. Returns `{ start, stop, toggle }`.
 */
export function useDictation() {
  const setIsRecording = useSessionStore((s) => s.setIsRecording)
  const setInputText = useSessionStore((s) => s.setInputText)
  const sttEngine = useSettingsStore((s) => s.sttEngine)
  const setError = useUIStore((s) => s.setError)
  const setOverlayVisible = useUIStore((s) => s.setOverlayVisible)

  const recordingRef = useRef(false)
  const speechRef = useRef<SpeechController | null>(null)
  const baseTextRef = useRef('')
  // Keep the latest engine without re-binding the hotkey listener.
  const engineRef = useRef(sttEngine)
  useEffect(() => {
    engineRef.current = sttEngine
  }, [sttEngine])

  const finish = useCallback(() => {
    recordingRef.current = false
    setIsRecording(false)
  }, [setIsRecording])

  const start = useCallback(async () => {
    if (recordingRef.current) return
    recordingRef.current = true
    setIsRecording(true)
    setError(null)
    const engine = engineRef.current

    if (engine === 'web_speech') {
      if (!isWebSpeechSupported()) {
        setError('Web Speech is not available here — pick another STT engine in Settings.')
        finish()
        return
      }
      baseTextRef.current = useSessionStore.getState().inputText
      speechRef.current = startWebSpeech(
        (text, isFinal) => {
          const base = baseTextRef.current
          const joined = base && text ? `${base} ${text}` : base + text
          setInputText(joined)
          if (isFinal) baseTextRef.current = useSessionStore.getState().inputText
        },
        (msg) => {
          setError(msg)
          finish()
        },
      )
      return
    }

    try {
      await tauriApi.startRecording(engine)
    } catch (e) {
      setError(errMessage(e))
      finish()
    }
  }, [setIsRecording, setInputText, setError, finish])

  const stop = useCallback(async () => {
    if (!recordingRef.current) return
    const engine = engineRef.current
    finish()

    if (engine === 'web_speech') {
      speechRef.current?.stop()
      speechRef.current = null
      return
    }

    try {
      const transcript = await tauriApi.stopRecording()
      if (transcript) setInputText(transcript)
    } catch (e) {
      setError(errMessage(e))
    }
  }, [finish, setInputText, setError])

  const toggle = useCallback(() => {
    if (recordingRef.current) void stop()
    else void start()
  }, [start, stop])

  // Dictate hotkey + backend events (VAD auto-stop, final transcript).
  useEffect(() => {
    const unDictate = listen('hotkey://dictate', () => {
      setOverlayVisible(true)
      toggle()
    })
    // VAD detected end-of-speech: finalize exactly like a manual stop.
    const unAutoStop = listen('stt://autostop', () => {
      void stop()
    })
    const unDone = listen<string>('stt://done', (event) => {
      if (event.payload) setInputText(event.payload)
    })
    return () => {
      unDictate.then((u) => u()).catch(() => {})
      unAutoStop.then((u) => u()).catch(() => {})
      unDone.then((u) => u()).catch(() => {})
    }
  }, [toggle, stop, setInputText, setOverlayVisible])

  return { start, stop, toggle }
}
