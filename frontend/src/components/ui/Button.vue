<template>
  <button
    :type="type || 'button'"
    :disabled="disabled || loading"
    class="inline-flex items-center justify-center font-medium transition-all duration-150 rounded-md cursor-pointer select-none disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none"
    :class="[variantClasses, sizeClasses]"
  >
    <svg
      v-if="loading"
      class="animate-spin -ml-0.5 mr-2 h-3.5 w-3.5 text-current"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
    <slot />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger' | 'success'
    size?: 'xs' | 'sm' | 'md' | 'lg' | 'icon'
    type?: 'button' | 'submit' | 'reset'
    loading?: boolean
    disabled?: boolean
  }>(),
  {
    variant: 'secondary',
    size: 'sm',
    type: 'button',
    loading: false,
    disabled: false,
  }
)

const variantClasses = computed(() => {
  switch (props.variant) {
    case 'primary':
      return 'bg-primary text-white hover:bg-primary-hover shadow-xs border border-transparent'
    case 'danger':
      return 'bg-red-600 text-white hover:bg-red-700 shadow-xs border border-transparent'
    case 'success':
      return 'bg-emerald-600 text-white hover:bg-emerald-700 shadow-xs border border-transparent'
    case 'outline':
      return 'bg-surface text-primary-text hover:bg-surface-hover border border-border shadow-2xs'
    case 'ghost':
      return 'bg-transparent text-muted hover:text-primary-text hover:bg-surface-hover border border-transparent'
    case 'secondary':
    default:
      return 'bg-surface-subtle text-primary-text hover:bg-surface-hover border border-border shadow-2xs'
  }
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'xs':
      return 'h-6 px-2 text-[11px] gap-1'
    case 'sm':
      return 'h-7 px-2.5 text-xs gap-1.5'
    case 'md':
      return 'h-8 px-3.5 text-xs gap-2'
    case 'lg':
      return 'h-9 px-4 text-sm gap-2'
    case 'icon':
      return 'w-7 h-7 p-0'
    default:
      return 'h-7 px-2.5 text-xs gap-1.5'
  }
})
</script>
