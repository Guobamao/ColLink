<template>
  <div>
    <!-- Confirmation Dialog -->
    <DialogRoot :open="confirmState.open" @update:open="onConfirmOpenChange">
      <DialogPortal>
        <DialogOverlay class="fixed inset-0 z-[100] bg-black/60 backdrop-blur-xs transition-opacity duration-200" />
        <DialogContent class="fixed left-1/2 top-1/2 z-[101] w-full max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl bg-surface border border-border p-6 shadow-2xl transition-all duration-200 select-none font-sans">
          <div class="flex items-start gap-3.5">
            <div
              class="w-9 h-9 rounded-full flex items-center justify-center shrink-0"
              :class="confirmState.type === 'danger'
                ? 'bg-red-500/15 text-red-500'
                : confirmState.type === 'warning'
                  ? 'bg-amber-500/15 text-amber-500'
                  : 'bg-primary/15 text-primary'"
            >
              <AlertTriangle v-if="confirmState.type === 'danger' || confirmState.type === 'warning'" class="w-5 h-5" />
              <HelpCircle v-else class="w-5 h-5" />
            </div>
            <div class="flex-1">
              <DialogTitle class="text-sm font-semibold text-primary-text tracking-tight m-0">
                {{ confirmState.title }}
              </DialogTitle>
              <p class="text-xs text-muted mt-2 mb-0 leading-relaxed whitespace-pre-line">
                {{ confirmState.message }}
              </p>
            </div>
          </div>

          <div class="flex items-center justify-end gap-2.5 mt-6">
            <button
              type="button"
              class="h-8 px-3.5 rounded-md text-xs font-medium text-muted hover:text-primary-text bg-surface-subtle hover:bg-surface-hover border border-border transition-colors cursor-pointer"
              @click="handleConfirmCancel"
            >
              {{ confirmState.cancelText }}
            </button>
            <button
              type="button"
              class="h-8 px-4 rounded-md text-xs font-medium text-white transition-colors cursor-pointer shadow-xs"
              :class="confirmState.type === 'danger'
                ? 'bg-red-600 hover:bg-red-700'
                : confirmState.type === 'warning'
                  ? 'bg-amber-600 hover:bg-amber-700'
                  : 'bg-primary hover:bg-primary-hover'"
              @click="handleConfirmOk"
            >
              {{ confirmState.confirmText }}
            </button>
          </div>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>

    <!-- Prompt / Input Dialog -->
    <DialogRoot :open="promptState.open" @update:open="onPromptOpenChange">
      <DialogPortal>
        <DialogOverlay class="fixed inset-0 z-[100] bg-black/60 backdrop-blur-xs transition-opacity duration-200" />
        <DialogContent class="fixed left-1/2 top-1/2 z-[101] w-full max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl bg-surface border border-border p-6 shadow-2xl transition-all duration-200 select-none font-sans">
          <div class="flex items-start gap-3.5">
            <div class="w-9 h-9 rounded-full flex items-center justify-center shrink-0 bg-primary/15 text-primary">
              <HelpCircle class="w-5 h-5" />
            </div>
            <div class="flex-1">
              <DialogTitle class="text-sm font-semibold text-primary-text tracking-tight m-0">
                {{ promptState.title }}
              </DialogTitle>
              <p class="text-xs text-muted mt-1.5 mb-3 leading-relaxed">
                {{ promptState.message }}
              </p>
              <input
                ref="promptInputRef"
                v-model="promptState.inputValue"
                type="text"
                :placeholder="promptState.placeholder"
                class="w-full h-8 px-3 rounded-md bg-surface border border-border text-xs text-primary-text placeholder:text-muted/60 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary font-mono transition-colors"
                @keydown.enter="handlePromptOk"
              />
            </div>
          </div>

          <div class="flex items-center justify-end gap-2.5 mt-6">
            <button
              type="button"
              class="h-8 px-3.5 rounded-md text-xs font-medium text-muted hover:text-primary-text bg-surface-subtle hover:bg-surface-hover border border-border transition-colors cursor-pointer"
              @click="handlePromptCancel"
            >
              {{ promptState.cancelText }}
            </button>
            <button
              type="button"
              class="h-8 px-4 rounded-md text-xs font-medium text-white bg-primary hover:bg-primary-hover transition-colors cursor-pointer shadow-xs"
              @click="handlePromptOk"
            >
              {{ promptState.confirmText }}
            </button>
          </div>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import {
  DialogRoot,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
} from 'radix-vue'
import { AlertTriangle, HelpCircle } from 'lucide-vue-next'
import { confirmState, promptState } from '@/utils/toast'

const promptInputRef = ref<HTMLInputElement | null>(null)

watch(() => promptState.open, (open) => {
  if (open) {
    nextTick(() => {
      promptInputRef.value?.focus()
      promptInputRef.value?.select()
    })
  }
})

const onConfirmOpenChange = (open: boolean) => {
  if (!open) handleConfirmCancel()
}

const handleConfirmOk = () => {
  confirmState.open = false
  confirmState.resolve?.(true)
}

const handleConfirmCancel = () => {
  confirmState.open = false
  confirmState.resolve?.(false)
}

const onPromptOpenChange = (open: boolean) => {
  if (!open) handlePromptCancel()
}

const handlePromptOk = () => {
  promptState.open = false
  promptState.resolve?.(promptState.inputValue)
}

const handlePromptCancel = () => {
  promptState.open = false
  promptState.resolve?.(null)
}
</script>
