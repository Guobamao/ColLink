<template>
  <Transition
    enter-active-class="transition duration-150 ease-out"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition duration-100 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div
      v-if="isOpen"
      class="fixed inset-0 z-50 flex items-start justify-center pt-[14vh] px-4 bg-black/60 backdrop-blur-sm select-none"
      @click.self="close"
    >
      <div
        class="w-full max-w-2xl overflow-hidden rounded-xl bg-surface border border-border shadow-2xl flex flex-col max-h-[70vh] animate-scale-in"
        @keydown="handleKeyDown"
      >
        <!-- Search Input Bar -->
        <div class="flex items-center px-4 py-3.5 border-b border-border gap-3 bg-surface">
          <Search class="w-4 h-4 text-muted shrink-0" />
          <input
            ref="inputRef"
            v-model="query"
            type="text"
            placeholder="搜索数据表、字段，或输入命令快捷操作..."
            class="w-full bg-transparent text-sm text-primary-text placeholder:text-muted focus:outline-none"
            autocomplete="off"
            spellcheck="false"
          />
          <button
            v-if="query"
            class="text-xs text-muted hover:text-primary-text px-1"
            @click="query = ''"
          >
            清除
          </button>
          <kbd class="px-1.5 py-0.5 text-[10px] font-mono rounded bg-surface-subtle text-muted border border-border">
            ESC
          </kbd>
        </div>

        <!-- Suggestions / Results List -->
        <div ref="listRef" class="overflow-y-auto p-2 space-y-4 flex-1">
          <!-- Section 1: Quick Actions (shown if query matches or query is empty) -->
          <div v-if="filteredActions.length > 0" class="space-y-1">
            <div class="px-2.5 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
              快捷指令
            </div>
            <div
              v-for="(action, idx) in filteredActions"
              :key="action.id"
              class="group flex items-center justify-between px-3 py-2 rounded-lg text-xs cursor-pointer transition-all duration-100"
              :class="selectedIndex === idx ? 'bg-primary-light text-primary font-medium' : 'text-primary-text hover:bg-surface-hover'"
              @click="executeAction(action)"
              @mouseenter="selectedIndex = idx"
            >
              <div class="flex items-center gap-2.5 min-w-0">
                <component :is="action.icon" class="w-4 h-4 shrink-0" :class="selectedIndex === idx ? 'text-primary' : 'text-muted'" />
                <span class="truncate">{{ action.title }}</span>
                <span v-if="action.subtitle" class="text-[11px] text-muted truncate">
                  {{ action.subtitle }}
                </span>
              </div>
              <div class="flex items-center gap-1.5 shrink-0">
                <span v-if="action.shortcut" class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-surface-subtle border border-border text-muted">
                  {{ action.shortcut }}
                </span>
                <CornerDownLeft v-if="selectedIndex === idx" class="w-3.5 h-3.5 text-primary opacity-80" />
              </div>
            </div>
          </div>

          <!-- Section 2: Matched Tables and Fields -->
          <div v-if="filteredTables.length > 0" class="space-y-1">
            <div class="px-2.5 py-1 text-[11px] font-semibold text-muted uppercase tracking-wider">
              数据表与字段 ({{ filteredTables.length }})
            </div>
            <div
              v-for="(item, idx) in filteredTables"
              :key="item.id"
              class="group flex items-center justify-between px-3 py-2 rounded-lg text-xs cursor-pointer transition-all duration-100"
              :class="selectedIndex === (filteredActions.length + idx) ? 'bg-primary-light text-primary font-medium' : 'text-primary-text hover:bg-surface-hover'"
              @click="selectTable(item)"
              @mouseenter="selectedIndex = filteredActions.length + idx"
            >
              <div class="flex items-center gap-2.5 min-w-0">
                <Table class="w-4 h-4 shrink-0" :class="selectedIndex === (filteredActions.length + idx) ? 'text-primary' : 'text-muted'" />
                <div class="flex flex-col min-w-0">
                  <div class="flex items-center gap-1.5">
                    <span class="font-mono font-medium truncate">{{ item.tableName }}</span>
                    <span v-if="item.tableComment" class="text-muted text-[11px] truncate">
                      {{ item.tableComment }}
                    </span>
                  </div>
                  <!-- Matched Column Hint -->
                  <div v-if="item.matchedColumn" class="flex items-center gap-1 text-[11px] text-muted font-mono mt-0.5">
                    <Columns class="w-3 h-3 text-primary shrink-0" />
                    <span>匹配字段: </span>
                    <span class="text-primary font-medium">{{ item.matchedColumn.column_name }}</span>
                    <span v-if="item.matchedColumn.column_comment">({{ item.matchedColumn.column_comment }})</span>
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-2 shrink-0">
                <span class="text-[10px] font-mono text-muted px-1.5 py-0.5 rounded bg-surface-subtle border border-border">
                  {{ item.columnCount }} 列
                </span>
                <CornerDownLeft v-if="selectedIndex === (filteredActions.length + idx)" class="w-3.5 h-3.5 text-primary opacity-80" />
              </div>
            </div>
          </div>

          <!-- Empty State -->
          <div
            v-if="filteredActions.length === 0 && filteredTables.length === 0"
            class="py-10 text-center text-xs text-muted"
          >
            未找到与 “{{ query }}” 匹配的表、字段或快捷操作
          </div>
        </div>

        <!-- Footer Bar with Keyboard Navigation Hints -->
        <div class="px-4 py-2 border-t border-border bg-surface-subtle/50 flex items-center justify-between text-[11px] text-muted">
          <div class="flex items-center gap-3">
            <span class="flex items-center gap-1">
              <kbd class="px-1 py-0.5 text-[9px] font-mono rounded bg-surface border border-border">↑</kbd>
              <kbd class="px-1 py-0.5 text-[9px] font-mono rounded bg-surface border border-border">↓</kbd>
              <span>导航</span>
            </span>
            <span class="flex items-center gap-1">
              <kbd class="px-1 py-0.5 text-[9px] font-mono rounded bg-surface border border-border">↵</kbd>
              <span>确认</span>
            </span>
            <span class="flex items-center gap-1">
              <kbd class="px-1 py-0.5 text-[9px] font-mono rounded bg-surface border border-border">ESC</kbd>
              <span>关闭</span>
            </span>
          </div>
          <div class="flex items-center gap-1 text-primary">
            <span>ColLink Studio</span>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { useCanvasStore } from '@/stores/canvasStore'
import { useProjectStore } from '@/stores/projectStore'
import { useThemeStore } from '@/stores/themeStore'
import {
  Search,
  Table,
  Columns,
  Sparkles,
  Route,
  LayoutDashboard,
  Maximize2,
  FileSpreadsheet,
  Image,
  Moon,
  Sun,
  FileCode,
  Database,
  CornerDownLeft,
} from 'lucide-vue-next'

const emit = defineEmits([
  'open-ddl-dialog',
  'open-db-dialog',
  'open-project-dialog',
  'open-backup-dialog',
  'open-mapping-drawer',
  'open-auto-mapping',
  'open-relation-path',
  'fit-view',
  'auto-layout',
  'export-excel',
  'export-image',
  'focus-table',
])

const canvasStore = useCanvasStore()
const projectStore = useProjectStore()
const themeStore = useThemeStore()

const isOpen = ref(false)
const query = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLElement | null>(null)

interface PaletteAction {
  id: string
  title: string
  subtitle?: string
  icon: any
  shortcut?: string
  action: () => void
}

const actions = computed<PaletteAction[]>(() => [
  {
    id: 'import-ddl',
    title: '导入 DDL 脚本',
    subtitle: '解析建表 SQL 并增量更新表结构',
    icon: FileCode,
    action: () => emit('open-ddl-dialog'),
  },
  {
    id: 'import-db',
    title: '连接数据库导入',
    subtitle: '在线探查与同步表结构',
    icon: Database,
    action: () => emit('open-db-dialog'),
  },
  {
    id: 'auto-mapping',
    title: '智能自动连线',
    subtitle: '基于字段名称/语义推断建立血缘映射',
    icon: Sparkles,
    action: () => emit('open-auto-mapping'),
  },
  {
    id: 'relation-path',
    title: '探查关联路径',
    subtitle: '推断起点与目标表之间的多跳最短路径',
    icon: Route,
    action: () => emit('open-relation-path'),
  },
  {
    id: 'auto-layout',
    title: 'DAG 拓扑自动排版',
    subtitle: '自左向右分层整理画布全部数据表',
    icon: LayoutDashboard,
    action: () => emit('auto-layout'),
  },
  {
    id: 'fit-view',
    title: '适应画布缩放',
    subtitle: '完整展示当前视图内所有数据表',
    icon: Maximize2,
    shortcut: 'Space',
    action: () => emit('fit-view'),
  },
  {
    id: 'export-excel',
    title: '导出 Excel 映射清单',
    subtitle: '生成标准字段映射表 (.xlsx)',
    icon: FileSpreadsheet,
    action: () => emit('export-excel'),
  },
  {
    id: 'export-image',
    title: '导出 2x 高清拓扑图',
    subtitle: '将当前视图保存为超清架构图 (.png)',
    icon: Image,
    action: () => emit('export-image'),
  },
  {
    id: 'toggle-theme',
    title: themeStore.isDark ? '切换至浅色模式' : '切换至深色模式',
    subtitle: '切换界面全局暗黑与明亮色彩',
    icon: themeStore.isDark ? Sun : Moon,
    action: () => themeStore.toggleDark(),
  },
])

// Filtered Actions
const filteredActions = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return actions.value
  return actions.value.filter(
    (a) =>
      a.title.toLowerCase().includes(q) ||
      (a.subtitle && a.subtitle.toLowerCase().includes(q))
  )
})

// Filtered Tables & Fields
interface TableSearchResult {
  id: string
  tableId: string
  nodeId: string
  tableName: string
  tableComment?: string
  columnCount: number
  matchedColumn?: {
    id: string
    column_name: string
    column_comment?: string
  }
}

const filteredTables = computed<TableSearchResult[]>(() => {
  const q = query.value.trim().toLowerCase()
  const results: TableSearchResult[] = []

  for (const node of canvasStore.nodes) {
    const data = node.data
    if (!data) continue

    const tName = data.tableName || ''
    const tComment = data.tableComment || ''
    const cols = data.columns || []

    const nameMatches = tName.toLowerCase().includes(q)
    const commentMatches = tComment.toLowerCase().includes(q)

    let matchedCol: any = null
    if (q) {
      matchedCol = cols.find(
        (c: any) =>
          c.column_name.toLowerCase().includes(q) ||
          (c.column_comment && c.column_comment.toLowerCase().includes(q))
      )
    }

    if (!q || nameMatches || commentMatches || matchedCol) {
      results.push({
        id: node.id,
        tableId: data.tableId,
        nodeId: node.id,
        tableName: tName,
        tableComment: tComment,
        columnCount: cols.length,
        matchedColumn: matchedCol,
      })
    }
  }

  return results
})

const totalItemsCount = computed(
  () => filteredActions.value.length + filteredTables.value.length
)

watch([query, isOpen], () => {
  selectedIndex.value = 0
})

const open = () => {
  isOpen.value = true
  query.value = ''
  selectedIndex.value = 0
  nextTick(() => {
    inputRef.value?.focus()
  })
}

const close = () => {
  isOpen.value = false
}

const toggle = () => {
  if (isOpen.value) close()
  else open()
}

const executeAction = (actionItem: PaletteAction) => {
  close()
  actionItem.action()
}

const selectTable = (item: TableSearchResult) => {
  close()
  emit('focus-table', {
    tableId: item.tableId,
    nodeId: item.nodeId,
    columnId: item.matchedColumn?.id,
  })
}

const handleKeyDown = (e: KeyboardEvent) => {
  const count = totalItemsCount.value
  if (count === 0) return

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = (selectedIndex.value + 1) % count
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = (selectedIndex.value - 1 + count) % count
  } else if (e.key === 'Enter') {
    e.preventDefault()
    if (selectedIndex.value < filteredActions.value.length) {
      const act = filteredActions.value[selectedIndex.value]
      if (act) executeAction(act)
    } else {
      const tblIndex = selectedIndex.value - filteredActions.value.length
      const tbl = filteredTables.value[tblIndex]
      if (tbl) selectTable(tbl)
    }
  } else if (e.key === 'Escape') {
    close()
  }
}

// Global Ctrl+K / Cmd+K listener
const handleGlobalKeydown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
    e.preventDefault()
    toggle()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
})

defineExpose({
  open,
  close,
  toggle,
})
</script>

<style scoped>
/* Text Colors mapped to CSS Variables */
.text-primary-text {
  color: var(--cl-text-primary);
}
</style>
