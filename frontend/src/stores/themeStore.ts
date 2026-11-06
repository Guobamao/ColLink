import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'auto'
export type GridPattern = 'dots' | 'lines' | 'cross'

export interface AccentColorConfig {
  id: string
  name: string
  hex: string
  light: {
    primary: string
    hover: string
    light: string
    border: string
    glow: string
  }
  dark: {
    primary: string
    hover: string
    light: string
    border: string
    glow: string
  }
}

export const ACCENT_COLORS: AccentColorConfig[] = [
  {
    id: 'blue',
    name: '极客蓝',
    hex: '#2563eb',
    light: {
      primary: '#2563eb',
      hover: '#1d4ed8',
      light: '#eff6ff',
      border: '#bfdbfe',
      glow: '#3b82f6',
    },
    dark: {
      primary: '#3b82f6',
      hover: '#60a5fa',
      light: 'rgba(59, 130, 246, 0.15)',
      border: 'rgba(59, 130, 246, 0.3)',
      glow: '#60a5fa',
    },
  },
  {
    id: 'emerald',
    name: '翡翠绿',
    hex: '#059669',
    light: {
      primary: '#059669',
      hover: '#047857',
      light: '#ecfdf5',
      border: '#a7f3d0',
      glow: '#10b981',
    },
    dark: {
      primary: '#10b981',
      hover: '#34d399',
      light: 'rgba(16, 185, 129, 0.15)',
      border: 'rgba(16, 185, 129, 0.3)',
      glow: '#34d399',
    },
  },
  {
    id: 'purple',
    name: '极光紫',
    hex: '#7c3aed',
    light: {
      primary: '#7c3aed',
      hover: '#6d28d9',
      light: '#f5f3ff',
      border: '#ddd6fe',
      glow: '#8b5cf6',
    },
    dark: {
      primary: '#8b5cf6',
      hover: '#a78bfa',
      light: 'rgba(139, 92, 246, 0.15)',
      border: 'rgba(139, 92, 246, 0.3)',
      glow: '#a78bfa',
    },
  },
  {
    id: 'orange',
    name: '活力橙',
    hex: '#ea580c',
    light: {
      primary: '#ea580c',
      hover: '#c2410c',
      light: '#fff7ed',
      border: '#fed7aa',
      glow: '#f97316',
    },
    dark: {
      primary: '#f97316',
      hover: '#fb923c',
      light: 'rgba(249, 115, 22, 0.15)',
      border: 'rgba(249, 115, 22, 0.3)',
      glow: '#fb923c',
    },
  },
  {
    id: 'amber',
    name: '琥珀金',
    hex: '#d97706',
    light: {
      primary: '#d97706',
      hover: '#b45309',
      light: '#fffbeb',
      border: '#fde68a',
      glow: '#f59e0b',
    },
    dark: {
      primary: '#f59e0b',
      hover: '#fbbf24',
      light: 'rgba(245, 158, 11, 0.15)',
      border: 'rgba(245, 158, 11, 0.3)',
      glow: '#fbbf24',
    },
  },
  {
    id: 'rose',
    name: '玫瑰红',
    hex: '#e11d48',
    light: {
      primary: '#e11d48',
      hover: '#be123c',
      light: '#fff1f2',
      border: '#fecdd3',
      glow: '#f43f5e',
    },
    dark: {
      primary: '#f43f5e',
      hover: '#fb7185',
      light: 'rgba(244, 63, 94, 0.15)',
      border: 'rgba(244, 63, 94, 0.3)',
      glow: '#fb7185',
    },
  },
  {
    id: 'cyan',
    name: '青空碧',
    hex: '#0891b2',
    light: {
      primary: '#0891b2',
      hover: '#0e7490',
      light: '#ecfeff',
      border: '#a5f3fc',
      glow: '#06b6d4',
    },
    dark: {
      primary: '#06b6d4',
      hover: '#22d3ee',
      light: 'rgba(6, 182, 212, 0.15)',
      border: 'rgba(6, 182, 212, 0.3)',
      glow: '#22d3ee',
    },
  },
  {
    id: 'indigo',
    name: '靛蓝调',
    hex: '#4f46e5',
    light: {
      primary: '#4f46e5',
      hover: '#4338ca',
      light: '#eef2ff',
      border: '#c7d2fe',
      glow: '#6366f1',
    },
    dark: {
      primary: '#6366f1',
      hover: '#818cf8',
      light: 'rgba(99, 102, 241, 0.15)',
      border: 'rgba(99, 102, 241, 0.3)',
      glow: '#818cf8',
    },
  },
]

export const useThemeStore = defineStore('theme', () => {
  const savedMode = (localStorage.getItem('collink-theme-mode') as ThemeMode) || 'auto'
  const mode = ref<ThemeMode>(savedMode)

  const savedAccent = localStorage.getItem('collink-accent-color') || 'blue'
  const accentColorId = ref<string>(savedAccent)

  const savedPattern = (localStorage.getItem('collink-grid-pattern') as GridPattern) || 'dots'
  const gridPattern = ref<GridPattern>(savedPattern)

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

  // Current accent config
  const currentAccent = computed(() => {
    return ACCENT_COLORS.find(c => c.id === accentColorId.value) || ACCENT_COLORS[0]
  })

  // Apply theme and accent variables to DOM
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

    // Apply accent color tokens
    const tokens = shouldDark ? currentAccent.value.dark : currentAccent.value.light
    root.style.setProperty('--cl-primary', tokens.primary)
    root.style.setProperty('--cl-primary-hover', tokens.hover)
    root.style.setProperty('--cl-primary-light', tokens.light)
    root.style.setProperty('--cl-primary-border', tokens.border)
    root.style.setProperty('--cl-accent-glow', tokens.glow)
    root.style.setProperty('--cl-border-active', tokens.primary)
  }

  const setAccentColor = (id: string) => {
    accentColorId.value = id
    localStorage.setItem('collink-accent-color', id)
    applyTheme()
  }

  const setGridPattern = (pattern: GridPattern) => {
    gridPattern.value = pattern
    localStorage.setItem('collink-grid-pattern', pattern)
  }

  let isTransitioning = false

  // Set theme mode with optional circular wave expansion transition
  const setMode = async (newMode: ThemeMode, event?: MouseEvent) => {
    if (mode.value === newMode) return

    const prevDark = isDark.value
    const nextDark = newMode === 'dark' ? true : newMode === 'light' ? false : systemPrefersDark.value

    const needsAnimation = prevDark !== nextDark
    const hasViewTransition = typeof document !== 'undefined' && 'startViewTransition' in document
    const prefersReducedMotion = typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

    if (!needsAnimation || !hasViewTransition || prefersReducedMotion || isTransitioning) {
      mode.value = newMode
      localStorage.setItem('collink-theme-mode', newMode)
      applyTheme()
      return
    }

    let x = window.innerWidth / 2
    let y = window.innerHeight / 2
    if (event && typeof event.clientX === 'number' && typeof event.clientY === 'number') {
      x = event.clientX
      y = event.clientY
    }

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
    accentColorId,
    currentAccent,
    gridPattern,
    setMode,
    setAccentColor,
    setGridPattern,
    cycleTheme,
    toggleDark,
    applyTheme,
  }
})
