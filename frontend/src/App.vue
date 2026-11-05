<template>
  <WorkspaceView />
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import WorkspaceView from '@/views/WorkspaceView.vue'

let heartbeatTimer: any = null
let lastHeartbeat = Date.now()

const sendHeartbeat = () => {
  lastHeartbeat = Date.now()
  fetch('/api/heartbeat', { method: 'POST' }).catch(() => {})
}

const handleUserActivity = () => {
  // Throttle user activity pings to at most once every 5s
  if (Date.now() - lastHeartbeat > 5000) {
    sendHeartbeat()
  }
}

const handleVisibilityChange = () => {
  if (document.visibilityState === 'visible') {
    sendHeartbeat()
  }
}

onMounted(() => {
  // Immediate initial heartbeat
  sendHeartbeat()

  // Regular heartbeat ping
  heartbeatTimer = setInterval(sendHeartbeat, 5000)

  // Wakeup & window focus listeners
  document.addEventListener('visibilitychange', handleVisibilityChange)
  window.addEventListener('focus', sendHeartbeat)

  // User activity triggers (e.g. returning to window after idle)
  window.addEventListener('pointerdown', handleUserActivity, { passive: true })
  window.addEventListener('keydown', handleUserActivity, { passive: true })

  // Listen for window close
  window.addEventListener('beforeunload', handleBeforeUnload)
})

onUnmounted(() => {
  if (heartbeatTimer) clearInterval(heartbeatTimer)
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  window.removeEventListener('focus', sendHeartbeat)
  window.removeEventListener('pointerdown', handleUserActivity)
  window.removeEventListener('keydown', handleUserActivity)
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
