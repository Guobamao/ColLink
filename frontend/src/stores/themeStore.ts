import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'auto'

export const useThemeStore = defineStore('theme', () => {
  const savedMode = (localStorage.getItem('collink-theme-mode') as ThemeMode) || 'auto'
  const mode = ref<ThemeMode>(savedMode)

  // System prefers dark query
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  const systemPrefersDark = ref(mediaQuery.matches)

  // Listen to system theme changes
  mediaQuery.addEventListener('change', (e) => {
    systemPrefersDark.value = e.matches
    if (mode.value === 'auto') {
      applyTheme()
    }
  })

  // Computed whether current active theme is dark
  const isDark = computed(() => {
    if (mode.value === 'dark') return true
    if (mode.value === 'light') return false
    return systemPrefersDark.value
  })

  // Apply theme to DOM
  const applyTheme = () => {
    const shouldDark = isDark.value
    const root = document.documentElement
    if (shouldDark) {
      root.classList.add('dark')
      root.setAttribute('data-theme', 'dark')
    } else {
      root.classList.remove('dark')
      root.setAttribute('data-theme', 'light')
    }
  }

  // Set theme mode
  const setMode = (newMode: ThemeMode) => {
    mode.value = newMode
    localStorage.setItem('collink-theme-mode', newMode)
    applyTheme()
  }

  // Toggle directly between light and dark
  const toggleDark = () => {
    setMode(isDark.value ? 'light' : 'dark')
  }

  // Initial application
  applyTheme()

  return {
    mode,
    isDark,
    setMode,
    toggleDark,
    applyTheme,
  }
})
