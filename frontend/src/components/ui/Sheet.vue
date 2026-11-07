<template>
  <DialogRoot :open="isOpen" @update:open="isOpen = $event">
    <DialogPortal>
      <DialogOverlay class="sheet-overlay fixed inset-0 z-50 bg-black/60 backdrop-blur-xs" />
      <DialogContent
        class="sheet-content fixed right-0 top-0 bottom-0 z-50 h-full bg-surface border-l border-border shadow-2xl flex flex-col overflow-hidden select-none"
        :style="{ width: size || '560px', maxWidth: '100vw' }"
      >
        <!-- Header -->
        <div class="px-5 py-4 border-b border-border flex items-start justify-between shrink-0 bg-surface">
          <div class="flex-1 pr-4">
            <slot name="header">
              <DialogTitle class="text-base font-semibold text-primary-text tracking-tight m-0">
                {{ title }}
              </DialogTitle>
              <DialogDescription v-if="description" class="text-xs text-muted mt-1 leading-normal m-0">
                {{ description }}
              </DialogDescription>
            </slot>
          </div>
          <DialogClose class="w-7 h-7 rounded-md flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors cursor-pointer shrink-0 mt-0.5">
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
import { computed } from 'vue'
import {
  DialogRoot,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogClose,
} from 'radix-vue'
import { X } from 'lucide-vue-next'

const props = withDefaults(
  defineProps<{
    modelValue?: boolean
    open?: boolean
    title?: string
    description?: string
    size?: string
    side?: 'left' | 'right' | 'top' | 'bottom'
  }>(),
  {
    modelValue: undefined,
    open: undefined,
    size: '560px',
    side: 'right',
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'update:open', value: boolean): void
  (e: 'close'): void
}>()

const isOpen = computed({
  get: () => (props.open !== undefined ? props.open : (props.modelValue ?? false)),
  set: (val: boolean) => {
    emit('update:open', val)
    emit('update:modelValue', val)
    if (!val) {
      emit('close')
    }
  },
})
</script>

<style scoped>
@keyframes sheetSlideInRight {
  from {
    transform: translate3d(100%, 0, 0);
  }
  to {
    transform: translate3d(0, 0, 0);
  }
}

@keyframes sheetSlideOutRight {
  from {
    transform: translate3d(0, 0, 0);
  }
  to {
    transform: translate3d(100%, 0, 0);
  }
}

@keyframes sheetFadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes sheetFadeOut {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}

.sheet-overlay[data-state='open'] {
  animation: sheetFadeIn 0.28s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.sheet-overlay[data-state='closed'] {
  animation: sheetFadeOut 0.2s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.sheet-content[data-state='open'] {
  animation: sheetSlideInRight 0.32s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.sheet-content[data-state='closed'] {
  animation: sheetSlideOutRight 0.22s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}
</style>
