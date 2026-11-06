import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import './style.css'
import { useThemeStore } from './stores/themeStore'

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)

// Initialize theme state
const themeStore = useThemeStore(pinia)
themeStore.applyTheme()

app.mount('#app')
