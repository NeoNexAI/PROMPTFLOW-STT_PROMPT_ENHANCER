import { create } from 'zustand'

interface UIState {
  overlayVisible: boolean
  isLoading: boolean
  errorMessage: string | null
  settingsVisible: boolean
  onboardingVisible: boolean
  setOverlayVisible: (v: boolean) => void
  setIsLoading: (v: boolean) => void
  setError: (msg: string | null) => void
  setSettingsVisible: (v: boolean) => void
  setOnboardingVisible: (v: boolean) => void
  clearError: () => void
}

export const useUIStore = create<UIState>()((set) => ({
  overlayVisible: false,
  isLoading: false,
  errorMessage: null,
  settingsVisible: false,
  onboardingVisible: false,
  setOverlayVisible: (overlayVisible) => set({ overlayVisible }),
  setIsLoading: (isLoading) => set({ isLoading }),
  setError: (errorMessage) => set({ errorMessage }),
  setSettingsVisible: (settingsVisible) => set({ settingsVisible }),
  setOnboardingVisible: (onboardingVisible) => set({ onboardingVisible }),
  clearError: () => set({ errorMessage: null }),
}))
