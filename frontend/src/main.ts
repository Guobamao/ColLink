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

// 禁用原生浏览器右键菜单，保障桌面端原生交互体验
document.addEventListener('contextmenu', (e) => {
  e.preventDefault()
})

app.mount('#app')
