<template>
  <WorkspaceView />
  <Toaster
    position="top-right"
    :theme="themeStore.isDark ? 'dark' : 'light'"
    :rich-colors="true"
    :close-button="true"
  />
  <ConfirmDialog />
</template>

<script setup lang="ts">
import WorkspaceView from '@/views/WorkspaceView.vue'
import { Toaster } from 'vue-sonner'
import { ConfirmDialog } from '@/components/ui'
import { useThemeStore } from '@/stores/themeStore'

const themeStore = useThemeStore()
</script>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
}
</style>
