<template>
  <div class="canvas-toolbar">
    <!-- Left: Brand & Workspace / Project Context -->
    <div class="toolbar-section left-section">
      <div class="app-brand" title="ColLink - 字段级数据血缘与映射设计平台">
        <div class="brand-badge">
          <svg class="brand-svg" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <!-- Left mini table card -->
            <rect x="2" y="3.5" width="8" height="10" rx="1.5" fill="white" fill-opacity="0.95"/>
            <rect x="2" y="3.5" width="8" height="3" rx="1.5" fill="#DBEAFE"/>
            <line x1="2" y1="6.5" x2="10" y2="6.5" stroke="#93C5FD" stroke-width="0.7"/>
            <line x1="3.5" y1="8.5" x2="7" y2="8.5" stroke="#94A3B8" stroke-width="0.8" stroke-linecap="round"/>
            <line x1="3.5" y1="10.5" x2="8" y2="10.5" stroke="#94A3B8" stroke-width="0.8" stroke-linecap="round"/>

            <!-- Right mini table card -->
            <rect x="14" y="10.5" width="8" height="10" rx="1.5" fill="white" fill-opacity="0.95"/>
            <rect x="14" y="10.5" width="8" height="3" rx="1.5" fill="#DBEAFE"/>
            <line x1="14" y1="13.5" x2="22" y2="13.5" stroke="#93C5FD" stroke-width="0.7"/>
            <line x1="15.5" y1="15.5" x2="19" y2="15.5" stroke="#94A3B8" stroke-width="0.8" stroke-linecap="round"/>
            <line x1="15.5" y1="17.5" x2="20.5" y2="17.5" stroke="#94A3B8" stroke-width="0.8" stroke-linecap="round"/>

            <!-- Connecting Line -->
            <path d="M 10 9 C 12 9, 12 16, 14 16" stroke="white" stroke-width="1.6" stroke-linecap="round"/>
            <circle cx="10" cy="9" r="1.1" fill="#FFFFFF"/>
            <circle cx="14" cy="16" r="1.1" fill="#FFFFFF"/>
          </svg>
        </div>
        <span class="brand-title">ColLink</span>
      </div>

      <div class="divider"></div>

      <!-- Project Switcher & Manager -->
      <div class="project-selector">
        <el-select
          :model-value="projectStore.currentProject?.id"
          placeholder="选择项目空间..."
          class="project-select"
          size="default"
          @change="onProjectChange"
        >
          <template #prefix>
            <el-icon class="proj-prefix-icon"><FolderOpened /></el-icon>
          </template>
          <el-option
            v-for="p in projectStore.projects"
            :key="p.id"
            :label="p.name"
            :value="p.id"
          />
        </el-select>
        <el-button
          class="btn-icon-subtle"
          title="项目空间管理"
          @click="emit('open-project-dialog')"
        >
          <el-icon><Setting /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- Center: Global Quick Search & Canvas Operations -->
    <div class="toolbar-section center-section">
      <!-- Quick Table / Column Search (Ctrl+K) -->
      <div class="search-box">
        <el-autocomplete
          ref="searchRef"
          v-model="searchQuery"
          :fetch-suggestions="querySearch"
          placeholder="快速找表 / 字段..."
          clearable
          size="default"
          class="global-search-input"
          :trigger-on-focus="true"
          @select="handleSearchSelect"
        >
          <template #prefix>
            <el-icon class="search-prefix-icon"><Search /></el-icon>
          </template>
          <template #suffix>
            <span class="search-shortcut-badge">Ctrl K</span>
          </template>
          <template #default="{ item }">
            <div class="search-result-item">
              <div class="item-main">
                <span class="item-tbl-name">[{{ item.tableName }}]</span>
                <span v-if="item.tableComment" class="item-tbl-comment">{{ item.tableComment }}</span>
                <span class="item-col-count">{{ item.columnCount }} 列</span>
              </div>
              <div v-if="item.matchedColumn" class="item-sub-col">
                匹配字段: <span class="highlight-matched">{{ item.matchedColumn.column_name }}</span>
                <span v-if="item.matchedColumn.column_comment"> ({{ item.matchedColumn.column_comment }})</span>
              </div>
            </div>
          </template>
        </el-autocomplete>
      </div>

      <div class="divider"></div>

      <!-- Import Structure Dropdown -->
      <el-dropdown trigger="click" @command="handleImportCommand">
        <el-button type="primary" class="btn-primary-action">
          <el-icon><Plus /></el-icon>
          <span>导入表结构</span>
          <el-icon class="el-icon--right"><ArrowDown /></el-icon>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu class="toolbar-dropdown-menu">
            <el-dropdown-item command="ddl">
              <el-icon><DocumentAdd /></el-icon>
              <span class="menu-text">DDL 脚本解析导入</span>
            </el-dropdown-item>
            <el-dropdown-item command="db">
              <el-icon><Connection /></el-icon>
              <span class="menu-text">连接数据库直接导入</span>
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <div class="divider"></div>

      <!-- Layout & View Fit -->
      <div class="btn-group-segmented">
        <button class="btn-segment" title="DAG 有向分层拓扑智能排版 (从左至右)" @click="handleAutoLayout">
          <el-icon><MagicStick /></el-icon>
          <span>自动排版</span>
        </button>
        <button class="btn-segment" title="自动缩放平移以完整容纳所有节点" @click="emit('fit-view')">
          <el-icon><Aim /></el-icon>
          <span>适应画布</span>
        </button>
      </div>
    </div>

    <!-- Right: Lineage Automation, Review & Deliverables -->
    <div class="toolbar-section right-section">
      <!-- 智能自动连线 -->
      <el-button class="btn-subtle btn-compact" @click="emit('open-auto-mapping')">
        <el-icon><MagicStick /></el-icon>
        <span>智能连线</span>
      </el-button>

      <!-- Clear Lineage Highlight Filter -->
      <el-button
        v-if="canvasStore.highlightedNodeIds.size > 0"
        type="warning"
        plain
        class="btn-warning-action"
        @click="canvasStore.clearHighlight()"
      >
        <el-icon><CircleClose /></el-icon>
        <span>清除高亮</span>
      </el-button>

      <!-- Mapping Review Drawer with Badge -->
      <el-badge :value="canvasStore.edges.length" :max="999" class="badge-item">
        <el-button class="btn-subtle btn-compact" @click="emit('open-mapping-drawer')">
          <el-icon><Link /></el-icon>
          <span>映射清单</span>
        </el-button>
      </el-badge>

      <div class="divider"></div>

      <!-- Export & Backup Deliverables -->
      <el-dropdown trigger="click" @command="handleExportCommand">
        <el-button class="btn-excel">
          <el-icon><Download /></el-icon>
          <span>导出交付物</span>
          <el-icon class="el-icon--right"><ArrowDown /></el-icon>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu class="toolbar-dropdown-menu">
            <el-dropdown-item command="excel">
              <el-icon><DocumentChecked /></el-icon>
              <span class="menu-text">导出 Excel 映射清单 (.xlsx)</span>
            </el-dropdown-item>
            <el-dropdown-item command="image">
              <el-icon><Picture /></el-icon>
              <span class="menu-text">导出 2x 高清拓扑图 (.png)</span>
            </el-dropdown-item>
            <el-dropdown-item command="backup" divided>
              <el-icon><TakeawayBox /></el-icon>
              <span class="menu-text">项目快照备份 / 迁移 (.json)</span>
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { api } from '@/api'
import { ElMessage } from 'element-plus'
import {
  FolderOpened,
  Setting,
  Plus,
  ArrowDown,
  DocumentAdd,
  Connection,
  MagicStick,
  Aim,
  Link,
  Download,
  DocumentChecked,
  Picture,
  TakeawayBox,
  CircleClose,
  Search,
} from '@element-plus/icons-vue'

const emit = defineEmits([
  'open-ddl-dialog',
  'open-db-dialog',
  'open-project-dialog',
  'open-backup-dialog',
  'open-mapping-drawer',
  'open-auto-mapping',
  'fit-view',
  'export-image',
  'focus-table',
])

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const searchQuery = ref('')
const searchRef = ref<any>(null)

interface SearchSuggestion {
  value: string
  tableId: string
  nodeId: string
  tableName: string
  tableComment?: string
  columnCount: number
  matchedColumn?: {
    column_name: string
    column_comment?: string
    id: string
  }
}

const querySearch = (queryString: string, cb: (results: SearchSuggestion[]) => void) => {
  const query = (queryString || '').trim().toLowerCase()
  const results: SearchSuggestion[] = []

  for (const node of canvasStore.nodes) {
    const data = node.data
    if (!data) continue

    const tName = data.tableName || ''
    const tComment = data.tableComment || ''
    const cols = data.columns || []

    const nameMatches = tName.toLowerCase().includes(query)
    const commentMatches = tComment.toLowerCase().includes(query)

    let matchedCol: any = null
    if (query) {
      matchedCol = cols.find((c: any) =>
        c.column_name.toLowerCase().includes(query) ||
        (c.column_comment && c.column_comment.toLowerCase().includes(query))
      )
    }

    if (!query || nameMatches || commentMatches || matchedCol) {
      results.push({
        value: tName,
        tableId: data.tableId,
        nodeId: node.id,
        tableName: tName,
        tableComment: tComment,
        columnCount: cols.length,
        matchedColumn: matchedCol,
      })
    }
  }

  cb(results)
}

const handleSearchSelect = (item: any) => {
  if (!item) return
  emit('focus-table', {
    tableId: item.tableId,
    nodeId: item.nodeId,
    columnId: item.matchedColumn?.id,
  })
}

const isEditableElement = (el: Element | null): boolean => {
  if (!el) return false
  const tagName = el.tagName.toLowerCase()
  if (tagName === 'input' || tagName === 'textarea') return true
  if ((el as HTMLElement).isContentEditable) return true
  return false
}

const onGlobalKeyDown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
    e.preventDefault()
    searchRef.value?.focus()
    return
  }

  // Do not intercept Ctrl+Z / Ctrl+Y when typing in text fields
  if (isEditableElement(document.activeElement)) {
    return
  }

  if ((e.ctrlKey || e.metaKey) && (e.key === 'z' || e.key === 'Z') && !e.shiftKey) {
    e.preventDefault()
    if (canvasStore.canUndo) {
      canvasStore.undo()
    }
  } else if (
    ((e.ctrlKey || e.metaKey) && (e.key === 'y' || e.key === 'Y')) ||
    ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'z' || e.key === 'Z'))
  ) {
    e.preventDefault()
    if (canvasStore.canRedo) {
      canvasStore.redo()
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', onGlobalKeyDown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onGlobalKeyDown)
})

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
    ElMessage.error('请先选择或新建一个项目！')
    return
  }
  emit('open-ddl-dialog')
}

const handleOpenDb = () => {
  if (!projectStore.currentProject?.id) {
    ElMessage.error('请先选择或新建一个项目！')
    return
  }
  emit('open-db-dialog')
}

const handleExportExcel = async () => {
  if (!projectStore.currentProject) {
    ElMessage.warning('当前无可用项目')
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
      ElMessage.error('请先选择或新建一个项目！')
      return
    }
    emit('open-backup-dialog')
  }
}
</script>

<style scoped>
.canvas-toolbar {
  height: 52px;
  background: var(--cl-surface);
  border-bottom: 1px solid var(--cl-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.03);
  z-index: 50;
  white-space: nowrap;
  box-sizing: border-box;
  width: 100%;
  overflow: hidden;
  gap: 12px;
}

.toolbar-section {
  display: flex;
  align-items: center;
  white-space: nowrap;
  flex-shrink: 0;
}

.left-section {
  gap: 12px;
}

.center-section {
  gap: 10px;
}

.right-section {
  gap: 10px;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 9px;
  user-select: none;
  cursor: default;
}

.brand-badge {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: linear-gradient(135deg, #1d4ed8 0%, #2563eb 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 5px rgba(29, 78, 216, 0.25);
  flex-shrink: 0;
}

.brand-svg {
  width: 17px;
  height: 17px;
}

.brand-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--cl-text-primary);
  letter-spacing: -0.02em;
}

.divider {
  width: 1px;
  height: 20px;
  background: var(--cl-border);
  flex-shrink: 0;
  margin: 0 2px;
}

.project-selector {
  display: flex;
  align-items: center;
  gap: 6px;
}

.proj-prefix-icon {
  color: var(--cl-text-muted);
  font-size: 14px;
}

.project-select {
  width: 180px;
}

.btn-icon-subtle {
  width: 32px;
  height: 32px;
  padding: 0;
  background: #ffffff;
  border: 1px solid var(--cl-border);
  color: var(--cl-text-muted);
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-icon-subtle:hover {
  background: var(--cl-surface-hover);
  color: var(--cl-text-primary);
  border-color: var(--cl-border-strong);
}

/* Button Stylings */
.btn-primary-action {
  height: 32px;
  background: var(--cl-primary);
  border-color: var(--cl-primary);
  color: #ffffff;
  font-weight: 500;
  font-size: 13px;
  padding: 0 12px;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  box-shadow: 0 1px 2px rgba(29, 78, 216, 0.15);
}

.btn-primary-action:hover {
  background: var(--cl-primary-hover);
  border-color: var(--cl-primary-hover);
}

.btn-subtle {
  height: 32px;
  background: #ffffff;
  border: 1px solid var(--cl-border);
  color: var(--cl-text-secondary);
  font-weight: 500;
  font-size: 12px;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn-subtle:hover {
  background: var(--cl-surface-hover);
  border-color: var(--cl-border-strong);
  color: var(--cl-text-primary);
}

.btn-compact {
  padding: 0 10px;
}

.btn-warning-action {
  height: 32px;
  font-size: 12px;
  padding: 0 10px;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

/* Segmented Button Group (Auto-Layout & Fit-View) */
.btn-group-segmented {
  display: inline-flex;
  align-items: center;
  background: #f1f5f9;
  padding: 2px;
  border-radius: 6px;
  border: 1px solid var(--cl-border);
}

.btn-segment {
  height: 28px;
  background: transparent;
  border: none;
  color: var(--cl-text-secondary);
  font-size: 12px;
  font-weight: 500;
  padding: 0 10px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  cursor: pointer;
  transition: all 0.15s ease;
  user-select: none;
}

.btn-segment:hover {
  background: #ffffff;
  color: var(--cl-text-primary);
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.05);
}

/* Fresh business green Excel & export dropdown button */
.btn-excel {
  height: 32px;
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  color: #15803d;
  font-weight: 500;
  font-size: 12px;
  padding: 0 12px;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn-excel:hover {
  background: #dcfce7;
  border-color: #86efac;
  color: #166534;
}

.badge-item :deep(.el-badge__content) {
  top: 2px;
  right: 4px;
  font-family: var(--cl-font-mono);
  font-size: 10px;
  height: 16px;
  line-height: 16px;
  padding: 0 4px;
}

/* Quick Search Box */
.search-box {
  width: 230px;
}

:deep(.global-search-input .el-input__wrapper) {
  border-radius: 6px;
  background-color: var(--cl-surface-subtle);
  box-shadow: 0 0 0 1px var(--cl-border) inset;
  transition: all 0.15s ease;
  padding: 0 10px;
  height: 32px;
}

:deep(.global-search-input .el-input__wrapper:hover) {
  box-shadow: 0 0 0 1px var(--cl-border-strong) inset;
  background-color: #ffffff;
}

:deep(.global-search-input .el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 1.5px var(--cl-primary) inset;
  background-color: #ffffff;
}

.search-prefix-icon {
  font-size: 14px;
  color: var(--cl-text-muted);
}

.search-shortcut-badge {
  font-family: var(--cl-font-mono);
  font-size: 10px;
  font-weight: 500;
  color: var(--cl-text-muted);
  background: #ffffff;
  border: 1px solid var(--cl-border);
  border-radius: 3px;
  padding: 1px 4px;
  line-height: 1.2;
}

.search-result-item {
  padding: 4px 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.item-main {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

.item-tbl-name {
  font-family: var(--cl-font-mono);
  font-weight: 600;
  color: var(--cl-text-primary);
  font-size: 12px;
}

.item-tbl-comment {
  color: var(--cl-text-muted);
  font-size: 12px;
}

.item-col-count {
  color: var(--cl-text-faint);
  font-size: 11px;
  font-family: var(--cl-font-mono);
}

.item-sub-col {
  font-size: 11px;
  color: var(--cl-text-muted);
  padding-left: 8px;
  border-left: 2px solid var(--cl-accent-glow);
  margin-top: 2px;
}

.highlight-matched {
  color: var(--cl-primary);
  font-weight: 600;
  font-family: var(--cl-font-mono);
}
</style>

<style>
/* Refined Dropdown Menu Items */
.toolbar-dropdown-menu .el-dropdown-menu__item {
  font-size: 13px;
  padding: 8px 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 4px;
  color: #334155;
  transition: all 0.15s ease;
}

.toolbar-dropdown-menu .el-dropdown-menu__item:hover {
  background-color: #f1f5f9;
  color: #2563eb;
}

.toolbar-dropdown-menu .el-dropdown-menu__item .el-icon {
  font-size: 15px;
  color: #64748b;
}

.toolbar-dropdown-menu .el-dropdown-menu__item:hover .el-icon {
  color: #2563eb;
}

.toolbar-dropdown-menu .menu-text {
  font-weight: 500;
}
</style>
