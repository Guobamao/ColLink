<template>
  <TooltipProvider :delay-duration="delay || 200">
    <TooltipRoot>
      <TooltipTrigger as-child>
        <slot />
      </TooltipTrigger>
      <TooltipPortal>
        <TooltipContent
          :side="side || 'bottom'"
          :side-offset="sideOffset || 4"
          class="z-50 overflow-hidden rounded-md bg-surface-hover px-2.5 py-1 text-xs text-primary-text border border-border shadow-md animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 select-none"
        >
          <slot name="content">
            {{ content }}
          </slot>
        </TooltipContent>
      </TooltipPortal>
    </TooltipRoot>
  </TooltipProvider>
</template>

<script setup lang="ts">
import {
  TooltipProvider,
  TooltipRoot,
  TooltipTrigger,
  TooltipPortal,
  TooltipContent,
} from 'radix-vue'

defineProps<{
  content?: string
  side?: 'top' | 'right' | 'bottom' | 'left'
  sideOffset?: number
  delay?: number
}>()
</script>
