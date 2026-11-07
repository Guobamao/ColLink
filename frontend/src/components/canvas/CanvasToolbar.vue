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
        <Dropdown align="start" content-class="w-64 p-1.5 shadow-popup">
          <template #trigger>
            <button
              type="button"
              class="group flex items-center gap-1.5 h-7 px-2.5 rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text transition-all duration-150 cursor-pointer shadow-2xs max-w-[200px] shrink-0"
              title="切换或管理项目空间"
            >
              <FolderGit2 class="w-3.5 h-3.5 text-primary shrink-0" />
              <span class="truncate text-xs font-medium font-sans flex-1 text-left whitespace-nowrap">
                {{ projectStore.currentProject?.name || '选择项目空间...' }}
              </span>
              <ChevronDown class="w-3 h-3 text-muted group-hover:text-primary-text transition-transform duration-200 shrink-0" />
            </button>
          </template>

          <!-- Dropdown Header -->
          <div class="px-2 py-1.5 flex items-center justify-between text-[11px] font-semibold text-muted border-b border-border mb-1">
            <span>项目空间 ({{ projectStore.projects.length }})</span>
            <button
              type="button"
              class="p-0.5 rounded text-muted hover:text-primary hover:bg-surface-hover transition-colors"
              title="管理所有项目"
              @click.stop="emit('open-project-dialog')"
            >
              <Settings class="w-3.5 h-3.5" />
            </button>
          </div>

          <!-- Project List -->
          <div class="max-h-60 overflow-y-auto space-y-0.5 custom-scrollbar">
            <DropdownItem
              v-for="p in projectStore.projects"
              :key="p.id"
              custom-class="flex items-center justify-between py-1.5 px-2 rounded-md group"
              :class="{ 'bg-primary-light/40 font-medium text-primary': p.id === projectStore.currentProject?.id }"
              @select="onProjectChange(p.id)"
            >
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <FolderGit2
                  class="w-3.5 h-3.5 shrink-0"
                  :class="p.id === projectStore.currentProject?.id ? 'text-primary' : 'text-muted group-hover:text-primary-text'"
                />
                <div class="flex flex-col min-w-0">
                  <span class="truncate text-xs leading-tight" :title="p.name">{{ p.name }}</span>
                  <span v-if="p.description" class="text-[10px] text-muted truncate leading-tight mt-0.5" :title="p.description">
                    {{ p.description }}
                  </span>
                </div>
              </div>
              <div class="flex items-center gap-1.5 shrink-0 ml-2">
                <span v-if="p.table_count !== undefined" class="text-[10px] font-mono px-1 py-0.2 rounded bg-surface-hover border border-border text-muted">
                  {{ p.table_count }}表
                </span>
                <Check v-if="p.id === projectStore.currentProject?.id" class="w-3.5 h-3.5 text-primary shrink-0" />
              </div>
            </DropdownItem>
            
            <div v-if="projectStore.projects.length === 0" class="py-3 text-center text-xs text-muted">
              暂无项目空间
            </div>
          </div>

          <DropdownSeparator class="my-1" />

          <!-- Bottom Actions -->
          <div class="pt-0.5 space-y-0.5">
            <DropdownItem
              custom-class="flex items-center gap-2 py-1.5 px-2 rounded-md text-primary hover:bg-primary-light/30"
              @select="emit('open-project-dialog')"
            >
              <Plus class="w-3.5 h-3.5" />
              <span class="text-xs font-medium">新建 / 管理项目空间...</span>
            </DropdownItem>
          </div>
        </Dropdown>
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
    <div class="flex items-center gap-2 shrink-0">
      <!-- Quick Command Palette Trigger (Ctrl+K) -->
      <button
        type="button"
        class="group flex items-center justify-between w-48 sm:w-56 md:w-60 h-7 px-2 rounded-md border border-border bg-surface-subtle hover:bg-surface-hover text-muted hover:text-primary-text transition-all duration-100 cursor-pointer shadow-2xs text-[11px] whitespace-nowrap shrink-0"
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

      <!-- Import Structure Dropdown (Compact Icon-Only with Tooltip) -->
      <Tooltip content="导入表结构 (DDL 脚本解析 / 数据库在线探查)" side="bottom">
        <div>
          <Dropdown align="start">
            <template #trigger>
              <button
                type="button"
                class="h-7 px-2 rounded-md bg-primary hover:bg-primary-hover text-white text-xs font-medium flex items-center gap-1 shadow-2xs transition-colors cursor-pointer shrink-0"
                aria-label="导入表结构"
              >
                <Plus class="w-3.5 h-3.5 shrink-0" />
                <ChevronDown class="w-2.5 h-2.5 opacity-80 shrink-0" />
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
        </div>
      </Tooltip>

      <!-- Layout & View Fit Segmented Control (Compact Icon-Only with Tooltips) -->
      <div class="flex items-center p-0.5 rounded-md border border-border bg-surface-subtle shrink-0">
        <Tooltip content="DAG 有向分层拓扑智能排版 (从左至右)" side="bottom">
          <button
            type="button"
            class="w-7 h-6 flex items-center justify-center rounded text-muted hover:text-primary-text hover:bg-surface transition-colors cursor-pointer"
            aria-label="自动排版"
            @click="handleAutoLayout"
          >
            <LayoutDashboard class="w-3.5 h-3.5 shrink-0" />
          </button>
        </Tooltip>
        <div class="w-px h-3 bg-border my-auto"></div>
        <Tooltip content="自动缩放平移以完整容纳所有节点 (Fit View)" side="bottom">
          <button
            type="button"
            class="w-7 h-6 flex items-center justify-center rounded text-muted hover:text-primary-text hover:bg-surface transition-colors cursor-pointer"
            aria-label="适应画布"
            @click="emit('fit-view')"
          >
            <Maximize2 class="w-3.5 h-3.5 shrink-0" />
          </button>
        </Tooltip>
      </div>
    </div>

    <!-- Right: Automation, Deliverables & Theme -->
    <div class="flex items-center gap-1.5 shrink-0">
      <!-- 智能自动连线 -->
      <Tooltip content="智能自动连线 (基于字段同名与语义)" side="bottom">
        <button
          type="button"
          class="w-7 h-7 flex items-center justify-center rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text transition-colors cursor-pointer shrink-0"
          aria-label="智能连线"
          @click="emit('open-auto-mapping')"
        >
          <Sparkles class="w-3.5 h-3.5 text-primary shrink-0" />
        </button>
      </Tooltip>

      <!-- 关联路径探查 -->
      <Tooltip content="两表多跳关联路径探查与 SQL JOIN 推导" side="bottom">
        <button
          type="button"
          class="w-7 h-7 flex items-center justify-center rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text transition-colors cursor-pointer shrink-0"
          aria-label="路径探查"
          @click="emit('open-relation-path')"
        >
          <Route class="w-3.5 h-3.5 text-primary shrink-0" />
        </button>
      </Tooltip>

      <!-- Clear Lineage Highlight Filter (if active) -->
      <Tooltip content="清除血缘高亮聚焦" side="bottom">
        <button
          v-if="canvasStore.highlightedNodeIds.size > 0"
          type="button"
          class="h-7 px-2 rounded-md bg-amber-500/10 border border-amber-500/30 text-amber-600 dark:text-amber-400 flex items-center gap-1 text-xs font-medium transition-colors cursor-pointer shrink-0"
          @click="canvasStore.clearHighlight()"
        >
          <XCircle class="w-3.5 h-3.5 shrink-0" />
          <span class="text-[11px] whitespace-nowrap">清除高亮</span>
        </button>
      </Tooltip>

      <!-- Mapping Review Drawer Button with Count Badge -->
      <Tooltip :content="`查看字段映射清单 (共 ${canvasStore.edges.length} 个映射)`" side="bottom">
        <button
          type="button"
          class="h-7 px-2 rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text flex items-center gap-1.5 transition-colors cursor-pointer shrink-0"
          aria-label="字段映射清单"
          @click="emit('open-mapping-drawer')"
        >
          <Link2 class="w-3.5 h-3.5 text-muted shrink-0" />
          <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-primary-light text-primary font-semibold">
            {{ canvasStore.edges.length }}
          </span>
        </button>
      </Tooltip>

      <div class="w-px h-3.5 bg-border mx-1"></div>

      <!-- Export Deliverables Dropdown -->
      <Tooltip content="导出交付物 (Excel / 2x 高清图 / 项目快照)" side="bottom">
        <div>
          <Dropdown align="end">
            <template #trigger>
              <button
                type="button"
                class="h-7 px-2 rounded-md border border-border bg-surface hover:bg-surface-hover text-primary-text flex items-center gap-1 transition-colors shadow-2xs cursor-pointer shrink-0"
                aria-label="导出交付物"
              >
                <Download class="w-3.5 h-3.5 text-muted shrink-0" />
                <ChevronDown class="w-2.5 h-2.5 text-muted shrink-0" />
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
        </div>
      </Tooltip>

      <div class="w-px h-3.5 bg-border mx-1"></div>

      <!-- Settings Button (Ctrl+,) -->
      <Tooltip content="系统设置 (快捷键: Ctrl+,)" side="bottom">
        <button
          type="button"
          class="w-7 h-7 flex items-center justify-center rounded-md border border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text transition-colors cursor-pointer"
          @click="emit('open-settings')"
        >
          <Settings class="w-3.5 h-3.5" />
        </button>
      </Tooltip>

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
  'open-settings',
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
