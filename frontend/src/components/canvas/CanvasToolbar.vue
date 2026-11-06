<template>
  <div
    data-tauri-drag-region
    class="h-10 pl-3 pr-0 bg-surface border-b border-border flex items-center justify-between z-50 text-xs shrink-0 select-none shadow-sm gap-2"
    @dblclick="handleHeaderDblClick"
  >
    <!-- Left: Brand & Workspace / Project Context -->
    <div class="flex items-center gap-2 shrink-0">
      <!-- App Brand -->
      <div class="flex items-center gap-1.5 cursor-default group" title="ColLink - 字段级数据血缘与映射设计平台">
        <div class="w-6 h-6 rounded-md bg-gradient-to-br from-blue-600 to-indigo-600 flex items-center justify-center shadow-xs">
          <svg class="w-3.5 h-3.5 text-white" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect x="2" y="3.5" width="8" height="10" rx="1.5" fill="white" fill-opacity="0.95"/>
            <rect x="14" y="10.5" width="8" height="10" rx="1.5" fill="white" fill-opacity="0.95"/>
            <path d="M 10 9 C 12 9, 12 16, 14 16" stroke="white" stroke-width="1.8" stroke-linecap="round"/>
          </svg>
        </div>
        <span class="font-bold text-sm text-primary-text tracking-tight font-sans">ColLink</span>
      </div>

      <div class="w-px h-3.5 bg-border mx-1"></div>

      <!-- Project Switcher & Manager -->
      <div class="flex items-center gap-1">
        <Select
          :model-value="projectStore.currentProject?.id"
          placeholder="选择项目空间..."
          size="sm"
          wrapper-class="w-36"
          :options="projectStore.projects.map(p => ({ label: p.name, value: p.id }))"
          @change="onProjectChange"
        >
          <template #prefix>
            <FolderGit2 class="w-3.5 h-3.5 text-muted shrink-0" />
          </template>
        </Select>
        <button
          type="button"
          class="w-7 h-7 flex items-center justify-center rounded-md text-muted hover:text-primary-text hover:bg-surface-hover border border-transparent hover:border-border transition-colors duration-100"
          title="项目空间管理"
          @click="emit('open-project-dialog')"
        >
          <Settings class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Center: Global Quick Command Search & Canvas Layout Operations -->
    <div class="flex items-center gap-2">
      <!-- Quick Command Palette Trigger (Ctrl+K) -->
      <button
        type="button"
        class="group flex items-center justify-between w-60 h-7 px-2 rounded-md border border-border bg-surface-subtle hover:bg-surface-hover text-muted hover:text-primary-text transition-all duration-100 cursor-pointer shadow-2xs text-[11px]"
        title="唤起全局命令中心 (快捷键: Ctrl+K)"
        @click="emit('open-command-palette')"
      >
        <div class="flex items-center gap-1.5 truncate">
          <Search class="w-3.5 h-3.5 text-muted group-hover:text-primary shrink-0 transition-colors" />
          <span class="truncate">快速找表、字段或指令...</span>
        </div>
        <kbd class="px-1 py-0.2 text-[9px] font-mono rounded bg-surface border border-border text-muted shrink-0">
          ⌘K
        </kbd>
      </button>

      <div class="w-px h-3.5 bg-border mx-1"></div>

      <!-- Import Structure Dropdown -->
      <Dropdown align="start">
        <template #trigger>
          <button
            type="button"
            class="h-7 px-2.5 rounded-md bg-primary hover:bg-primary-hover text-white text-xs font-medium flex items-center gap-1.5 shadow-2xs transition-colors cursor-pointer"
          >
            <Plus class="w-3.5 h-3.5 shrink-0" />
            <span>导入表结构</span>
            <ChevronDown class="w-3 h-3 opacity-80 shrink-0" />
          </button>
        </template>
        <DropdownItem @select="handleImportCommand('ddl')">
          <FileCode class="w-4 h-4 text-primary mr-2" />
          <span class="text-xs">DDL 脚本解析导入</span>
        </DropdownItem>
        <DropdownItem @select="handleImportCommand('db')">
          <Database class="w-4 h-4 text-primary mr-2" />
          <span class="text-xs">连接数据库在线探查</span>
        </DropdownItem>
      </Dropdown>

      <!-- Layout & View Fit Segmented Control -->
      <div class="flex items-center p-0.5 rounded-md border border-border bg-surface-subtle">
        <button
          type="button"
          class="h-6 px-2 rounded flex items-center gap-1 text-[11px] text-muted hover:text-primary-text hover:bg-surface transition-colors"
          title="DAG 有向分层拓扑智能排版 (从左至右)"
          @click="handleAutoLayout"
        >
          <LayoutDashboard class="w-3.5 h-3.5 shrink-0" />
          <span>自动排版</span>
        </button>
        <div class="w-px h-3 bg-border my-auto"></div>
        <button
          type="button"
          class="h-6 px-2 rounded flex items-center gap-1 text-[11px] text-muted hover:text-primary-text hover:bg-surface transition-colors"
          title="自动缩放平移以完整容纳所有节点"
          @click="emit('fit-view')"
        >
          <Maximize2 class="w-3.5 h-3.5 shrink-0" />
          <span>适应画布</span>
        </button>
      </div>
    </div>

    <!-- Right: Automation, Deliverables & Theme -->
    <div class="flex items-center gap-1.5 shrink-0">
      <!-- 智能自动连线 -->
      <button
        type="button"
        class="h-7 px-2 rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text flex items-center gap-1.5 text-xs font-medium transition-colors"
        title="基于字段同名与中文语义自动匹配连线"
        @click="emit('open-auto-mapping')"
      >
        <Sparkles class="w-3.5 h-3.5 text-primary shrink-0" />
        <span>智能连线</span>
      </button>

      <!-- 关联路径探查 -->
      <button
        type="button"
        class="h-7 px-2 rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text flex items-center gap-1.5 text-xs font-medium transition-colors"
        title="计算两表间多跳关联路径与生成 SQL JOIN"
        @click="emit('open-relation-path')"
      >
        <Route class="w-3.5 h-3.5 text-primary shrink-0" />
        <span>路径探查</span>
      </button>

      <!-- Clear Lineage Highlight Filter (if active) -->
      <button
        v-if="canvasStore.highlightedNodeIds.size > 0"
        type="button"
        class="h-7 px-2 rounded-md bg-amber-500/10 border border-amber-500/30 text-amber-600 dark:text-amber-400 flex items-center gap-1 text-xs font-medium transition-colors"
        @click="canvasStore.clearHighlight()"
      >
        <XCircle class="w-3.5 h-3.5 shrink-0" />
        <span>清除高亮</span>
      </button>

      <!-- Mapping Review Drawer Button with Count Badge -->
      <button
        type="button"
        class="h-7 px-2 rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text flex items-center gap-1.5 text-xs font-medium transition-colors"
        title="查看当前项目所有已建立的字段映射连线"
        @click="emit('open-mapping-drawer')"
      >
        <Link2 class="w-3.5 h-3.5 text-muted shrink-0" />
        <span>映射</span>
        <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-primary-light text-primary font-semibold">
          {{ canvasStore.edges.length }}
        </span>
      </button>

      <div class="w-px h-3.5 bg-border mx-1"></div>

      <!-- Export Deliverables Dropdown -->
      <Dropdown align="end">
        <template #trigger>
          <button
            type="button"
            class="h-7 px-2.5 rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text flex items-center gap-1.5 text-xs font-medium transition-colors shadow-2xs cursor-pointer"
          >
            <Download class="w-3.5 h-3.5 text-muted shrink-0" />
            <span>导出</span>
            <ChevronDown class="w-3 h-3 text-muted shrink-0" />
          </button>
        </template>
        <DropdownItem @select="handleExportCommand('excel')">
          <FileSpreadsheet class="w-4 h-4 text-emerald-500 mr-2" />
          <span class="text-xs">导出 Excel 映射清单 (.xlsx)</span>
        </DropdownItem>
        <DropdownItem @select="handleExportCommand('image')">
          <Image class="w-4 h-4 text-blue-500 mr-2" />
          <span class="text-xs">导出 2x 高清拓扑长图 (.png)</span>
        </DropdownItem>
        <DropdownSeparator />
        <DropdownItem @select="handleExportCommand('backup')">
          <Archive class="w-4 h-4 text-muted mr-2" />
          <span class="text-xs">项目快照备份 / 迁移 (.json)</span>
        </DropdownItem>
      </Dropdown>

      <div class="w-px h-3.5 bg-border mx-1"></div>

      <!-- Theme Switcher (Cycle: Light -> Dark -> Auto) with Wave Transition -->
      <Tooltip :content="themeTooltipText" side="bottom">
        <button
          type="button"
          class="w-7 h-7 flex items-center justify-center rounded-md border border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text transition-colors relative cursor-pointer"
          @click="handleThemeClick"
        >
          <Sun v-if="themeStore.mode === 'light'" class="w-3.5 h-3.5 text-amber-500" />
          <Moon v-else-if="themeStore.mode === 'dark'" class="w-3.5 h-3.5 text-sky-400" />
          <div v-else class="relative flex items-center justify-center">
            <Monitor class="w-3.5 h-3.5 text-primary" />
            <span
              class="absolute -bottom-0.5 -right-0.5 w-1.5 h-1.5 rounded-full ring-1 ring-surface"
              :class="themeStore.isDark ? 'bg-sky-400' : 'bg-amber-500'"
            ></span>
          </div>
        </button>
      </Tooltip>

      <!-- Window Controls (Tauri Custom Titlebar) -->
      <div v-if="isTauri" class="flex items-center h-10 ml-1 select-none">
        <!-- Minimize -->
        <button
          type="button"
          class="w-11 h-10 flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors"
          title="最小化"
          @click="handleMinimize"
        >
          <Minus class="w-3.5 h-3.5" />
        </button>

        <!-- Maximize / Restore -->
        <button
          type="button"
          class="w-11 h-10 flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors"
          :title="isMaximized ? '向下还原' : '最大化'"
          @click="handleToggleMaximize"
        >
          <Copy v-if="isMaximized" class="w-3 h-3" />
          <Square v-else class="w-3 h-3" />
        </button>

        <!-- Close -->
        <button
          type="button"
          class="w-11 h-10 flex items-center justify-center text-muted hover:text-white hover:bg-[#e81123] transition-colors"
          title="关闭"
          @click="handleClose"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { useThemeStore } from '@/stores/themeStore'
import { api } from '@/api'
import { toast } from '@/utils/toast'
import { Select, Dropdown, DropdownItem, DropdownSeparator, Tooltip } from '@/components/ui'
import {
  Search,
  Plus,
  Sparkles,
  Route,
  LayoutDashboard,
  Maximize2,
  Download,
  FileSpreadsheet,
  Image,
  Archive,
  Moon,
  Sun,
  Settings,
  FolderGit2,
  ChevronDown,
  Check,
  Monitor,
  XCircle,
  Link2,
  FileCode,
  Database,
  Minus,
  Square,
  Copy,
  X,
} from 'lucide-vue-next'

const emit = defineEmits([
  'open-ddl-dialog',
  'open-db-dialog',
  'open-project-dialog',
  'open-backup-dialog',
  'open-mapping-drawer',
  'open-auto-mapping',
  'open-relation-path',
  'open-command-palette',
  'fit-view',
  'export-image',
  'focus-table',
])

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()
const themeStore = useThemeStore()

// Tauri Frameless Window Controls
const isTauri = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
const isMaximized = ref(false)

onMounted(async () => {
  if (isTauri) {
    try {
      isMaximized.value = await invoke<boolean>('window_is_maximized')
      window.addEventListener('resize', async () => {
        try {
          isMaximized.value = await invoke<boolean>('window_is_maximized')
        } catch (_) {}
      })
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      const appWindow = getCurrentWindow()
      await appWindow.onResized(async () => {
        try {
          isMaximized.value = await invoke<boolean>('window_is_maximized')
        } catch (_) {}
      })
    } catch (e) {
      console.warn('Tauri window API init:', e)
    }
  }
})

const handleMinimize = async () => {
  try {
    await invoke('window_minimize')
  } catch (e) {
    console.error('Failed to minimize window:', e)
  }
}

const handleToggleMaximize = async () => {
  try {
    const res = await invoke<boolean>('window_toggle_maximize')
    isMaximized.value = res
  } catch (e) {
    console.error('Failed to toggle maximize window:', e)
  }
}

const handleClose = async () => {
  try {
    await invoke('window_close')
  } catch (e) {
    console.error('Failed to close window:', e)
  }
}

const handleHeaderDblClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  if (target && (target.hasAttribute('data-tauri-drag-region') || target.classList.contains('drag-spacer'))) {
    handleToggleMaximize()
  }
}

const themeTooltipText = computed(() => {
  if (themeStore.mode === 'light') {
    return '当前：浅色模式（点击切换为深色模式）'
  }
  if (themeStore.mode === 'dark') {
    return '当前：深色模式（点击切换为跟随系统）'
  }
  const sysState = themeStore.isDark ? '深色' : '浅色'
  return `当前：跟随系统 - ${sysState}（点击切换为浅色模式）`
})

const handleThemeClick = (e: MouseEvent) => {
  themeStore.cycleTheme(e)
}

const onProjectChange = async (projectId: string) => {
  await projectStore.selectProject(projectId)
  if (projectStore.currentProject) {
    await canvasStore.fetchCanvases(projectStore.currentProject.id)
    emit('fit-view')
  }
}

const handleAutoLayout = async () => {
  await canvasStore.applyAutoLayout('LR')
  setTimeout(() => {
    emit('fit-view')
  }, 100)
}

const handleOpenDdl = () => {
  if (!projectStore.currentProject?.id) {
    toast.error('请先选择或新建一个项目！')
    return
  }
  emit('open-ddl-dialog')
}

const handleOpenDb = () => {
  if (!projectStore.currentProject?.id) {
    toast.error('请先选择或新建一个项目！')
    return
  }
  emit('open-db-dialog')
}

const handleExportExcel = async () => {
  if (!projectStore.currentProject) {
    toast.warning('当前无可用项目')
    return
  }
  await api.exportExcel(projectStore.currentProject.id, projectStore.currentProject.name)
}

const handleImportCommand = (cmd: string) => {
  if (cmd === 'ddl') {
    handleOpenDdl()
  } else if (cmd === 'db') {
    handleOpenDb()
  }
}

const handleExportCommand = (cmd: string) => {
  if (cmd === 'excel') {
    handleExportExcel()
  } else if (cmd === 'image') {
    emit('export-image')
  } else if (cmd === 'backup') {
    if (!projectStore.currentProject?.id) {
      toast.error('请先选择或新建一个项目！')
      return
    }
    emit('open-backup-dialog')
  }
}
</script>

<style scoped>
.text-primary-text {
  color: var(--cl-text-primary);
}
</style>
