<template>
  <DropdownMenuRoot v-model:open="isOpen">
    <DropdownMenuTrigger as-child>
      <slot name="trigger" />
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        :side-offset="sideOffset"
        :align="align"
        class="z-50 min-w-[8rem] overflow-hidden rounded-md border border-border bg-surface p-1 text-primary-text shadow-lg data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-1 data-[side=top]:slide-in-from-bottom-1 font-sans"
        :class="contentClass"
      >
        <slot />
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
  DropdownMenuContent,
} from 'radix-vue'

withDefaults(
  defineProps<{
    align?: 'start' | 'center' | 'end'
    sideOffset?: number
    contentClass?: string
  }>(),
  {
    align: 'start',
    sideOffset: 4,
    contentClass: '',
  }
)

const isOpen = ref(false)
</script>
