<template>
  <DialogRoot :open="modelValue" @update:open="emit('update:modelValue', $event)">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-50 bg-black/50 backdrop-blur-xs data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 duration-200" />
      <DialogContent
        class="fixed right-0 top-0 bottom-0 z-50 h-full bg-surface border-l border-border shadow-2xl duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right flex flex-col overflow-hidden select-none"
        :style="{ width: size || '540px', maxWidth: '100vw' }"
      >
        <!-- Header -->
        <div class="px-5 py-4 border-b border-border flex items-center justify-between shrink-0 bg-surface">
          <div class="flex-1 pr-4">
            <slot name="header">
              <DialogTitle class="text-sm font-semibold text-primary-text tracking-tight m-0">
                {{ title }}
              </DialogTitle>
            </slot>
          </div>
          <DialogClose class="w-7 h-7 rounded-md flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors cursor-pointer shrink-0">
            <X class="w-4 h-4" />
          </DialogClose>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-y-auto">
          <slot />
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
  size?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()
</script>
