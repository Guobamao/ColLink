<template>
  <div
    class="relative inline-flex items-center w-full rounded-md border border-border bg-surface text-primary-text transition-colors focus-within:border-primary focus-within:ring-1 focus-within:ring-primary shadow-2xs"
    :class="[
      size === 'sm' ? 'h-7 text-xs px-2' : size === 'lg' ? 'h-9 text-sm px-3' : 'h-8 text-xs px-2.5',
      disabled ? 'opacity-60 cursor-not-allowed bg-surface-subtle' : ''
    ]"
  >
    <div v-if="$slots.prefix" class="mr-2 text-muted flex items-center shrink-0">
      <slot name="prefix" />
    </div>

    <input
      :value="modelValue"
      :type="type || 'text'"
      :placeholder="placeholder"
      :disabled="disabled"
      class="w-full h-full bg-transparent border-0 p-0 text-inherit placeholder:text-muted/60 focus:outline-none focus:ring-0 disabled:cursor-not-allowed"
      @input="onInput"
      @keydown="emit('keydown', $event)"
    />

    <button
      v-if="clearable && modelValue"
      type="button"
      class="text-muted hover:text-primary-text ml-1 shrink-0 cursor-pointer"
      @click="emit('update:modelValue', '')"
    >
      <X class="w-3 h-3" />
    </button>

    <div v-if="$slots.suffix" class="ml-2 text-muted flex items-center shrink-0">
      <slot name="suffix" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next'

const props = defineProps<{
  modelValue?: string | number
  type?: string
  placeholder?: string
  disabled?: boolean
  clearable?: boolean
  size?: 'sm' | 'md' | 'lg'
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'keydown', event: KeyboardEvent): void
}>()

const onInput = (e: Event) => {
  const target = e.target as HTMLInputElement
  emit('update:modelValue', target.value)
}
</script>
