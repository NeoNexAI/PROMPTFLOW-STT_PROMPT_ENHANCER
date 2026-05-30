import { useEnhanceMutation } from '@/queries/enhancementQueries'
import { useSessionStore } from '@/stores/sessionStore'
import { useSettingsStore } from '@/stores/settingsStore'

export function useEnhancement() {
  const mutation = useEnhanceMutation()
  const inputText = useSessionStore((s) => s.inputText)
  const provider = useSettingsStore((s) => s.provider)
  const selectedMode = useSettingsStore((s) => s.selectedMode)
  const models = useSettingsStore((s) => s.models)
  const customBaseUrl = useSettingsStore((s) => s.customBaseUrl)
  const customPrompt = useSettingsStore((s) => s.customPrompt)

  const enhance = () => {
    if (!inputText.trim()) return
    mutation.mutate({
      text: inputText,
      mode: selectedMode,
      provider,
      opts: {
        model: models[provider] || undefined,
        baseUrl: provider === 'custom' ? customBaseUrl || undefined : undefined,
        customPrompt: selectedMode === 'custom' ? customPrompt || undefined : undefined,
      },
    })
  }

  return {
    enhance,
    isLoading: mutation.isPending,
    error: mutation.error,
    result: mutation.data,
  }
}
