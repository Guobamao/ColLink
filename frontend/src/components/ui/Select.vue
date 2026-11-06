<template>
  <div class="relative inline-flex items-center" :class="wrapperClass">
    <div v-if="$slots.prefix" class="absolute left-2.5 z-10 pointer-events-none flex items-center text-muted">
      <slot name="prefix" />
    </div>
    <select
      :value="modelValue"
      :disabled="disabled"
      class="w-full appearance-none rounded-md border border-border bg-surface text-primary-text transition-colors focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary disabled:cursor-not-allowed disabled:opacity-50 font-sans cursor-pointer truncate"
      :class="[
        size === 'sm' ? 'h-7 text-xs py-1 pr-6' : size === 'lg' ? 'h-10 text-sm py-2 pr-8' : 'h-8 text-xs py-1.5 pr-7',
        $slots.prefix ? 'pl-8' : 'pl-2.5',
        customClass,
      ]"
      @change="handleChange"
    >
      <option v-if="placeholder" value="" disabled :selected="!modelValue">
        {{ placeholder }}
      </option>
      <template v-if="options && options.length">
        <option
          v-for="opt in options"
          :key="opt.value"
          :value="opt.value"
          :disabled="opt.disabled"
        >
          {{ opt.label }}
        </option>
      </template>
      <slot v-else />
    </select>
    <div class="absolute right-2 pointer-events-none flex items-center text-muted">
      <ChevronDown class="w-3.5 h-3.5" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ChevronDown } from 'lucide-vue-next'

const props = withDefaults(
  defineProps<{
    modelValue?: string | number | null
    placeholder?: string
    disabled?: boolean
    size?: 'sm' | 'md' | 'lg'
    options?: Array<{ label: string; value: string | number; disabled?: boolean }>
    wrapperClass?: string
    customClass?: string
  }>(),
  {
    modelValue: '',
    placeholder: '',
    disabled: false,
    size: 'md',
    options: () => [],
    wrapperClass: 'w-full',
    customClass: '',
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: any): void
  (e: 'change', value: any): void
}>()

const handleChange = (e: Event) => {
  const target = e.target as HTMLSelectElement
  emit('update:modelValue', target.value)
  emit('change', target.value)
}
</script>
