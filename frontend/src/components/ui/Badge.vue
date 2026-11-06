<template>
  <span
    class="inline-flex items-center gap-1 font-mono font-medium rounded transition-colors"
    :class="[variantClasses, sizeClasses]"
  >
    <slot />
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    variant?: 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'outline'
    size?: 'sm' | 'md'
  }>(),
  {
    variant: 'default',
    size: 'sm',
  }
)

const variantClasses = computed(() => {
  switch (props.variant) {
    case 'primary':
      return 'bg-primary-light text-primary border border-primary-border'
    case 'success':
      return 'bg-emerald-500/15 text-emerald-500 dark:text-emerald-400 border border-emerald-500/25'
    case 'warning':
      return 'bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/25'
    case 'danger':
      return 'bg-red-500/15 text-red-600 dark:text-red-400 border border-red-500/25'
    case 'info':
      return 'bg-surface-subtle text-muted border border-border'
    case 'outline':
      return 'bg-transparent text-primary-text border border-border'
    case 'default':
    default:
      return 'bg-surface-subtle text-primary-text border border-border'
  }
})

const sizeClasses = computed(() => {
  return props.size === 'sm' ? 'px-1.5 py-0.5 text-[10px]' : 'px-2 py-0.5 text-xs'
})
</script>
