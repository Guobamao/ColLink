<template>
  <label class="inline-flex items-center gap-2 cursor-pointer select-none text-xs text-primary-text" :class="{ 'opacity-50 cursor-not-allowed': disabled }">
    <CheckboxRoot
      :checked="modelValue"
      :disabled="disabled"
      class="w-4 h-4 rounded border border-border bg-surface flex items-center justify-center transition-colors focus:outline-none focus:ring-1 focus:ring-primary data-[state=checked]:bg-primary data-[state=checked]:border-primary data-[state=indeterminate]:bg-primary data-[state=indeterminate]:border-primary cursor-pointer disabled:cursor-not-allowed"
      @update:checked="emit('update:modelValue', $event)"
    >
      <CheckboxIndicator class="text-white flex items-center justify-center">
        <Minus v-if="indeterminate" class="w-3 h-3 stroke-[3]" />
        <Check v-else class="w-3 h-3 stroke-[3]" />
      </CheckboxIndicator>
    </CheckboxRoot>
    <span v-if="$slots.default" class="text-xs">
      <slot />
    </span>
  </label>
</template>

<script setup lang="ts">
import { CheckboxRoot, CheckboxIndicator } from 'radix-vue'
import { Check, Minus } from 'lucide-vue-next'

defineProps<{
  modelValue?: boolean | 'indeterminate'
  disabled?: boolean
  indeterminate?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean | 'indeterminate'): void
}>()
</script>
