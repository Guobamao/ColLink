<template>
  <WorkspaceView />
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import WorkspaceView from '@/views/WorkspaceView.vue'

let heartbeatTimer: any = null

onMounted(() => {
  // Initial heartbeat
  fetch('/api/heartbeat', { method: 'POST' }).catch(() => {})

  // Heartbeat ping every 4s
  heartbeatTimer = setInterval(() => {
    fetch('/api/heartbeat', { method: 'POST' }).catch(() => {})
  }, 4000)

  // Listen for window close
  window.addEventListener('beforeunload', handleBeforeUnload)
})

onUnmounted(() => {
  if (heartbeatTimer) clearInterval(heartbeatTimer)
  window.removeEventListener('beforeunload', handleBeforeUnload)
})

function handleBeforeUnload() {
  if (navigator.sendBeacon) {
    navigator.sendBeacon('/api/shutdown')
  } else {
    fetch('/api/shutdown', { method: 'POST', keepalive: true }).catch(() => {})
  }
}
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
