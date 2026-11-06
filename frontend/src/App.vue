<template>
  <div class="w-full h-full relative overflow-hidden">
    <WorkspaceView
      v-show="currentView === 'workspace'"
      @open-settings="currentView = 'settings'"
    />
    <SettingsView
      v-if="currentView === 'settings'"
      @back="currentView = 'workspace'"
      @open-backup="handleOpenBackup"
    />
  </div>

  <Toaster
    position="top-right"
    :theme="themeStore.isDark ? 'dark' : 'light'"
    :rich-colors="true"
    :close-button="true"
  />
  <ConfirmDialog />
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import WorkspaceView from '@/views/WorkspaceView.vue'
import SettingsView from '@/views/SettingsView.vue'
import { Toaster } from 'vue-sonner'
import { ConfirmDialog } from '@/components/ui'
import { useThemeStore } from '@/stores/themeStore'

const themeStore = useThemeStore()
const currentView = ref<'workspace' | 'settings'>('workspace')

const handleOpenBackup = () => {
  currentView.value = 'workspace'
  setTimeout(() => {
    window.dispatchEvent(new CustomEvent('collink:open-backup-dialog'))
  }, 100)
}

const handleGlobalKeydown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.key === ',') {
    e.preventDefault()
    currentView.value = currentView.value === 'settings' ? 'workspace' : 'settings'
  } else if (e.key === 'Escape' && currentView.value === 'settings') {
    currentView.value = 'workspace'
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
})
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
