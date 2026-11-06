<template>
  <DialogRoot :open="modelValue" @update:open="emit('update:modelValue', $event)">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
      <DialogContent
        class="fixed left-1/2 top-1/2 z-50 -translate-x-1/2 -translate-y-1/2 rounded-xl bg-surface border border-border shadow-2xl duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 flex flex-col max-h-[90vh] overflow-hidden select-none"
        :style="{ width: width || '560px', maxWidth: 'calc(100vw - 32px)' }"
      >
        <!-- Header -->
        <div class="px-5 py-4 border-b border-border flex items-center justify-between shrink-0 bg-surface">
          <slot name="header">
            <DialogTitle class="text-sm font-semibold text-primary-text tracking-tight m-0 flex items-center gap-2">
              {{ title }}
            </DialogTitle>
          </slot>
          <DialogClose class="w-7 h-7 rounded-md flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors cursor-pointer">
            <X class="w-4 h-4" />
          </DialogClose>
        </div>

        <!-- Body / Content -->
        <div class="p-5 overflow-y-auto flex-1">
          <slot />
        </div>

        <!-- Footer (if provided) -->
        <div v-if="$slots.footer" class="px-5 py-3.5 border-t border-border bg-surface-subtle flex items-center justify-end gap-2.5 shrink-0">
          <slot name="footer" />
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<script setup lang="ts">
import {
  DialogRoot,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogClose,
} from 'radix-vue'
import { X } from 'lucide-vue-next'

defineProps<{
  modelValue: boolean
  title?: string
  width?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()
</script>
