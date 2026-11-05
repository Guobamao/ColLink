/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        canvas: 'var(--cl-canvas-bg)',
        surface: {
          DEFAULT: 'var(--cl-surface)',
          subtle: 'var(--cl-surface-subtle)',
          hover: 'var(--cl-surface-hover)',
        },
        border: {
          DEFAULT: 'var(--cl-border)',
          strong: 'var(--cl-border-strong)',
          active: 'var(--cl-border-active)',
        },
        muted: {
          DEFAULT: 'var(--cl-text-muted)',
          foreground: 'var(--cl-text-faint)',
        },
        primary: {
          DEFAULT: 'var(--cl-primary)',
          hover: 'var(--cl-primary-hover)',
          light: 'var(--cl-primary-light)',
          border: 'var(--cl-primary-border)',
        },
        accent: {
          glow: 'var(--cl-accent-glow)',
        },
      },
      fontFamily: {
        sans: ['var(--cl-font-sans)'],
        mono: ['var(--cl-font-mono)'],
      },
      boxShadow: {
        subtle: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
        card: 'var(--cl-shadow-card)',
        'card-hover': 'var(--cl-shadow-card-hover)',
        popup: 'var(--cl-shadow-popup)',
      },
      animation: {
        'fade-in': 'fadeIn 0.15s ease-out',
        'scale-in': 'scaleIn 0.15s cubic-bezier(0.16, 1, 0.3, 1)',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        scaleIn: {
          '0%': { opacity: '0', transform: 'scale(0.96)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
      },
    },
  },
  plugins: [],
}
