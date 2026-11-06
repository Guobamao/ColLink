<template>
  <DropdownMenuItem
    :disabled="disabled"
    class="relative flex cursor-pointer select-none items-center rounded px-2 py-1.5 text-xs outline-none transition-colors hover:bg-surface-hover focus:bg-surface-hover text-primary-text data-[disabled]:pointer-events-none data-[disabled]:opacity-50"
    :class="customClass"
    @select="$emit('select', $event)"
  >
    <slot />
  </DropdownMenuItem>
</template>

<script setup lang="ts">
import { DropdownMenuItem } from 'radix-vue'

defineProps<{
  disabled?: boolean
  customClass?: string
}>()

defineEmits<{
  (e: 'select', event: Event): void
}>()
</script>
