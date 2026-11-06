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

  let isTransitioning = false

  // Set theme mode with optional circular wave expansion transition
  const setMode = async (newMode: ThemeMode, event?: MouseEvent) => {
    if (mode.value === newMode) return

    const prevDark = isDark.value
    const nextDark = newMode === 'dark' ? true : newMode === 'light' ? false : systemPrefersDark.value

    // If active visual theme doesn't change, apply directly without full screen ripple
    const needsAnimation = prevDark !== nextDark

    const hasViewTransition = typeof document !== 'undefined' && 'startViewTransition' in document
    const prefersReducedMotion = typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

    if (!needsAnimation || !hasViewTransition || prefersReducedMotion || isTransitioning) {
      mode.value = newMode
      localStorage.setItem('collink-theme-mode', newMode)
      applyTheme()
      return
    }

    // Determine circular expansion epicenter coordinates
    let x = window.innerWidth / 2
    let y = window.innerHeight / 2
    if (event && typeof event.clientX === 'number' && typeof event.clientY === 'number') {
      x = event.clientX
      y = event.clientY
    }

    // Calculate maximum distance to the furthest screen corner
    const endRadius = Math.hypot(
      Math.max(x, window.innerWidth - x),
      Math.max(y, window.innerHeight - y)
    )

    isTransitioning = true

    try {
      const transition = (document as any).startViewTransition(async () => {
        mode.value = newMode
        localStorage.setItem('collink-theme-mode', newMode)
        applyTheme()
      })

      await transition.ready

      const animation = document.documentElement.animate(
        {
          clipPath: [
            `circle(0px at ${x}px ${y}px)`,
            `circle(${endRadius}px at ${x}px ${y}px)`
          ]
        },
        {
          duration: 450,
          easing: 'cubic-bezier(0.4, 0, 0.2, 1)',
          pseudoElement: '::view-transition-new(root)'
        }
      )

      await animation.finished
    } catch (e) {
      console.warn('View transition animation fallback:', e)
      mode.value = newMode
      localStorage.setItem('collink-theme-mode', newMode)
      applyTheme()
    } finally {
      isTransitioning = false
    }
  }

  // Cycle directly between light -> dark -> auto -> light
  const cycleTheme = (event?: MouseEvent) => {
    let next: ThemeMode = 'light'
    if (mode.value === 'light') {
      next = 'dark'
    } else if (mode.value === 'dark') {
      next = 'auto'
    } else {
      next = 'light'
    }
    return setMode(next, event)
  }

  // Toggle directly between light and dark
  const toggleDark = (event?: MouseEvent) => {
    setMode(isDark.value ? 'light' : 'dark', event)
  }

  // Initial application
  applyTheme()

  return {
    mode,
    isDark,
    setMode,
    cycleTheme,
    toggleDark,
    applyTheme,
  }
})
