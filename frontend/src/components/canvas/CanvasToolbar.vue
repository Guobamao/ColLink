<template>
  <div class="canvas-toolbar">
    <!-- Left: Brand & Project Selector -->
    <div class="toolbar-section left-section">
      <div class="app-brand">
        <div class="brand-badge">
          <svg class="brand-svg" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M13 2L3 14H12L11 22L21 10H12L13 2Z" fill="white" stroke="white" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <span class="brand-title">ColLink</span>
      </div>

      <div class="divider"></div>

      <div class="project-selector">
        <span class="proj-label">当前项目：</span>
        <el-select
          :model-value="projectStore.currentProject?.id"
          placeholder="请选择项目"
          class="project-select"
          @change="onProjectChange"
        >
          <el-option
            v-for="p in projectStore.projects"
            :key="p.id"
            :label="p.name"
            :value="p.id"
          />
        </el-select>
        <el-button class="btn-subtle btn-manage" @click="emit('open-project-dialog')">
          <el-icon><FolderOpened /></el-icon>
          <span>项目管理</span>
        </el-button>
      </div>
    </div>

    <!-- Center: Main Actions & Canvas Controls (Compact) -->
    <div class="toolbar-section center-section">
      <!-- Quick Table / Column Search (Ctrl+K) -->
      <div class="search-box">
        <el-autocomplete
          ref="searchRef"
          v-model="searchQuery"
          :fetch-suggestions="querySearch"
          placeholder="快速找表/字段 (Ctrl+K)..."
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
                <span class="item-col-count">({{ item.columnCount }}列)</span>
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

      <!-- 1. 导入数据源 (下拉菜单聚合) -->
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
              <span class="menu-text">DDL 解析导入</span>
            </el-dropdown-item>
            <el-dropdown-item command="db">
              <el-icon><Connection /></el-icon>
              <span class="menu-text">在线逆向拉取</span>
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <div class="divider"></div>

      <!-- 2. 画布视图排版 (紧凑并排) -->
      <div class="btn-group">
        <el-button class="btn-subtle btn-compact" @click="handleAutoLayout">
          <el-icon><MagicStick /></el-icon>
          <span>自动排版</span>
        </el-button>
        <el-button class="btn-subtle btn-compact" @click="emit('fit-view')">
          <el-icon><Aim /></el-icon>
          <span>适应画布</span>
        </el-button>
      </div>
    </div>

    <!-- Right: Deliverables & System Tools (Dropdown Aggregated) -->
    <div class="toolbar-section right-section">
      <!-- Lineage Filter State (Active when clicked) -->
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

      <!-- 3. 映射清单 (独立保留带Badge) -->
      <el-badge :value="canvasStore.edges.length" :max="999" class="badge-item">
        <el-button class="btn-subtle btn-compact" @click="emit('open-mapping-drawer')">
          <el-icon><Link /></el-icon>
          <span>映射清单</span>
        </el-button>
      </el-badge>

      <div class="divider"></div>

      <!-- 4. 导出与备份 (下拉菜单聚合) -->
      <el-dropdown trigger="click" @command="handleExportCommand">
        <el-button class="btn-excel">
          <el-icon><Download /></el-icon>
          <span>导出与备份</span>
          <el-icon class="el-icon--right"><ArrowDown /></el-icon>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu class="toolbar-dropdown-menu">
            <el-dropdown-item command="excel">
              <el-icon><DocumentChecked /></el-icon>
              <span class="menu-text">导出 Excel 映射清单</span>
            </el-dropdown-item>
            <el-dropdown-item command="image">
              <el-icon><Picture /></el-icon>
              <span class="menu-text">导出高清拓扑图片 (PNG)</span>
            </el-dropdown-item>
            <el-dropdown-item command="backup" divided>
              <el-icon><TakeawayBox /></el-icon>
              <span class="menu-text">项目快照备份 / 迁移 (JSON)</span>
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

const onGlobalKeyDown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
    e.preventDefault()
    searchRef.value?.focus()
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

const handleExportExcel = () => {
  if (!projectStore.currentProject) {
    ElMessage.warning('当前无可用项目')
    return
  }
  const url = api.getExcelExportUrl(projectStore.currentProject.id)
  window.open(url, '_blank')
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
  height: 56px;
  background: #ffffff;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.03);
  z-index: 20;
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
  gap: 10px;
}

.center-section {
  gap: 8px;
}

.right-section {
  gap: 8px;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  user-select: none;
}

.brand-badge {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 6px rgba(37, 99, 235, 0.28);
  flex-shrink: 0;
}

.brand-svg {
  width: 16px;
  height: 16px;
}

.brand-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  letter-spacing: -0.4px;
}

.divider {
  width: 1px;
  height: 18px;
  background: #e2e8f0;
  flex-shrink: 0;
  margin: 0 2px;
}

.project-selector {
  display: flex;
  align-items: center;
  gap: 6px;
}

.proj-label {
  font-size: 13px;
  color: #64748b;
  font-weight: 500;
  white-space: nowrap;
  flex-shrink: 0;
}

.project-select {
  width: 168px;
}

.btn-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* Button Stylings */
.btn-primary-action {
  height: 34px;
  background: #3b82f6;
  border-color: #3b82f6;
  color: #ffffff;
  font-weight: 500;
  font-size: 13px;
  padding: 0 12px;
  border-radius: 6px;
  transition: all 0.15s ease;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn-primary-action:hover {
  background: #2563eb;
  border-color: #2563eb;
}

.btn-subtle {
  height: 34px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  color: #334155;
  font-weight: 500;
  font-size: 13px;
  border-radius: 6px;
  transition: all 0.15s ease;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn-subtle:hover {
  background: #f8fafc;
  border-color: #cbd5e1;
  color: #0f172a;
}

.btn-manage {
  padding: 0 10px;
}

.btn-compact {
  padding: 0 10px;
}

.btn-warning-action {
  height: 34px;
  font-size: 12px;
  padding: 0 10px;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

/* Fresh business green Excel & export dropdown button */
.btn-excel {
  height: 34px;
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  color: #16a34a;
  font-weight: 500;
  font-size: 13px;
  padding: 0 12px;
  border-radius: 6px;
  transition: all 0.15s ease;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn-excel:hover {
  background: #dcfce7;
  border-color: #86efac;
  color: #15803d;
}

.badge-item :deep(.el-badge__content) {
  top: 3px;
  right: 5px;
}

/* Quick Search Box */
.search-box {
  width: 220px;
}

:deep(.global-search-input .el-input__wrapper) {
  border-radius: 6px;
  background-color: #f8fafc;
  box-shadow: 0 0 0 1px #e2e8f0 inset;
  transition: all 0.2s ease;
  padding: 0 8px;
  height: 32px;
}

:deep(.global-search-input .el-input__wrapper:hover) {
  box-shadow: 0 0 0 1px #cbd5e1 inset;
  background-color: #ffffff;
}

:deep(.global-search-input .el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 1.5px #3b82f6 inset;
  background-color: #ffffff;
}

.search-prefix-icon {
  font-size: 14px;
  color: #94a3b8;
}

.search-shortcut-badge {
  font-size: 10px;
  color: #94a3b8;
  background: #f1f5f9;
  border: 1px solid #e2e8f0;
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
  font-weight: 600;
  color: #1e293b;
}

.item-tbl-comment {
  color: #64748b;
  font-size: 12px;
}

.item-col-count {
  color: #94a3b8;
  font-size: 11px;
}

.item-sub-col {
  font-size: 11px;
  color: #64748b;
  padding-left: 8px;
  border-left: 2px solid #3b82f6;
  margin-top: 2px;
}

.highlight-matched {
  color: #2563eb;
  font-weight: 600;
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
