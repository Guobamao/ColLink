<template>
  <div
    class="table-card-node nowheel"
    :class="{
      'is-dimmed': isDimmed,
      'is-highlighted': isHighlighted,
      'is-focused': isFocused,
    }"
    @wheel.stop
  >
    <!-- Card Header with Top Brand Accent -->
    <div class="card-header">
      <div class="header-left">
        <div class="db-icon-badge">
          <svg class="table-svg" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect x="3" y="4" width="18" height="16" rx="2" stroke="#3B82F6" stroke-width="2"/>
            <path d="M3 9H21" stroke="#3B82F6" stroke-width="2"/>
            <path d="M9 20V9" stroke="#3B82F6" stroke-width="2"/>
          </svg>
        </div>
        <div class="title-wrap">
          <div class="table-name" :title="data.tableName">{{ data.tableName }}</div>
          <div v-if="data.tableComment" class="table-comment" :title="data.tableComment">
            {{ data.tableComment }}
          </div>
        </div>
      </div>
      <div class="header-right">
        <span class="count-pill">
          {{ data.columns?.length || 0 }} 列
        </span>
        <el-dropdown trigger="click" @command="handleCommand">
          <span class="action-btn" title="更多操作">
            <el-icon><MoreFilled /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu class="table-dropdown-menu">
              <el-dropdown-item command="highlight">
                <el-icon><Search /></el-icon>
                <span>高亮上下游血缘</span>
              </el-dropdown-item>
              <el-dropdown-item command="edit">
                <el-icon><Edit /></el-icon>
                <span>编辑表信息</span>
              </el-dropdown-item>

              <!-- Canvas / Project separation -->
              <el-dropdown-item command="remove-canvas" divided>
                <el-icon><Remove /></el-icon>
                <span>从当前画布移出</span>
              </el-dropdown-item>
              <el-dropdown-item command="delete" style="color: #ef4444;">
                <el-icon><Delete /></el-icon>
                <span>从项目中彻底删除</span>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- Column Search Bar with Filter Toggle -->
    <div class="search-bar">
      <div class="search-input-wrap">
        <el-icon class="search-icon"><Search /></el-icon>
        <input
          v-model="searchKeyword"
          type="text"
          placeholder="搜索字段名/注释..."
          class="search-input"
          @mousedown.stop
        />
        <el-icon v-if="searchKeyword" class="clear-icon" @click="searchKeyword = ''"><CircleClose /></el-icon>
      </div>

      <!-- Filter Toggle: Only Show Mapped Fields -->
      <el-tooltip
        :content="onlyMapped ? '当前已折叠未连线字段，点击显示全部' : '仅显示已连线映射字段 (折叠无连线字段)'"
        placement="top"
      >
        <button
          class="btn-filter-mapped"
          :class="{ 'is-active': onlyMapped }"
          @click.stop="onlyMapped = !onlyMapped"
        >
          <el-icon><Filter /></el-icon>
        </button>
      </el-tooltip>
    </div>

    <!-- Folded Hint Banner -->
    <div v-if="onlyMapped && hiddenCount > 0" class="filter-folded-banner">
      已折叠 {{ hiddenCount }} 个未映射字段
    </div>

    <!-- Column List (nowheel + @wheel.stop prevents canvas zooming) -->
    <div class="column-list nowheel" @wheel.stop>
      <div
        v-for="col in filteredColumns"
        :id="`col_item_${col.id}`"
        :key="col.id"
        class="column-item"
        :class="{
          'is-breathing': isColumnActive(col.id),
          'has-mapping': hasMapping(col.id),
        }"
        :title="col.column_comment ? `${col.column_name} (${col.column_comment}) - 点击高亮血缘` : `${col.column_name} - 点击高亮血缘`"
        @click.stop="handleFieldClick(col.id)"
      >
        <!-- Left Handles (Dual-directional: Target & Source) -->
        <Handle
          :id="`target_${col.id}_left`"
          type="target"
          :position="Position.Left"
          class="field-handle left-handle left-target"
          title="左侧输入锚点"
        />
        <Handle
          :id="`source_${col.id}_left`"
          type="source"
          :position="Position.Left"
          class="field-handle left-handle left-source"
          title="左侧拖拽/输出锚点"
        />
        <!-- Legacy left fallback -->
        <Handle
          :id="`target_${col.id}`"
          type="target"
          :position="Position.Left"
          class="field-handle left-handle"
          style="display: none;"
        />

        <div class="col-content">
          <div class="col-info">
            <div class="col-name-row">
              <span class="col-name">{{ col.column_name }}</span>
              <span v-if="isPrimaryKey(col)" class="pk-badge" title="主键字段">PK</span>
            </div>
            <span v-if="col.column_comment" class="col-comment">
              {{ col.column_comment }}
            </span>
          </div>

          <div class="col-right-group">
            <span class="col-type" :class="getTypeClass(col.data_type)">
              {{ col.data_type }}
            </span>
            <span
              v-if="hasMapping(col.id)"
              class="mapping-icon-wrap"
              title="已建立血缘映射，点击高亮并显示连线"
            >
              <el-icon class="mapping-icon"><Link /></el-icon>
            </span>
            <span v-else class="drag-hint-wrap" title="拖拽两侧蓝圈锚点建立映射">
              <el-icon class="drag-hint-icon"><Right /></el-icon>
            </span>
          </div>
        </div>

        <!-- Right Handles (Dual-directional: Target & Source) -->
        <Handle
          :id="`target_${col.id}_right`"
          type="target"
          :position="Position.Right"
          class="field-handle right-handle right-target"
          title="右侧输入锚点"
        />
        <Handle
          :id="`source_${col.id}_right`"
          type="source"
          :position="Position.Right"
          class="field-handle right-handle right-source"
          title="右侧拖拽/输出锚点"
        />
        <!-- Legacy right fallback -->
        <Handle
          :id="`source_${col.id}`"
          type="source"
          :position="Position.Right"
          class="field-handle right-handle"
          style="display: none;"
        />
      </div>

      <div v-if="filteredColumns.length === 0" class="empty-tip">
        无匹配字段
      </div>
    </div>

    <!-- Edit Dialog -->
    <el-dialog
      v-model="editDialogVisible"
      title="修改表信息"
      width="400px"
      append-to-body
      @mousedown.stop
    >
      <el-form :model="editForm" label-width="80px">
        <el-form-item label="表英文名">
          <el-input v-model="editForm.tableName" />
        </el-form-item>
        <el-form-item label="中文注释">
          <el-input v-model="editForm.tableComment" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveTableInfo">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Handle, Position } from '@vue-flow/core'
import { useCanvasStore } from '@/stores/canvasStore'
import { useProjectStore } from '@/stores/projectStore'
import { api } from '@/api'
import { ElMessageBox, ElMessage } from 'element-plus'
import { Search, Edit, Delete, CircleClose, Link, Right, MoreFilled, Filter, Remove } from '@element-plus/icons-vue'
import type { ColumnMeta } from '@/types'

const props = defineProps<{
  id: string
  data: {
    tableId: string
    tableName: string
    tableComment?: string
    columns: ColumnMeta[]
    colorTag?: string
  }
}>()

const canvasStore = useCanvasStore()
const projectStore = useProjectStore()

const searchKeyword = ref('')
const onlyMapped = ref(false)
const editDialogVisible = ref(false)
const saving = ref(false)
const editForm = ref({
  tableName: '',
  tableComment: '',
})

// Highlight / Dim / Focus logic for table level
const isHighlighted = computed(() => {
  return canvasStore.highlightedNodeIds.has(props.id)
})

const isDimmed = computed(() => {
  if (canvasStore.highlightedNodeIds.size === 0) return false
  return !canvasStore.highlightedNodeIds.has(props.id)
})

const isFocused = computed(() => {
  return canvasStore.focusedTableId === props.data.tableId
})


// Column level active & breathing logic
const isColumnActive = (colId: string) => {
  return (
    canvasStore.activeColumnId === colId ||
    canvasStore.associatedColumnIds.has(colId)
  )
}

const hasMapping = (colId: string) => {
  return canvasStore.edges.some(
    e => e.data?.sourceColumnId === colId || e.data?.targetColumnId === colId
  )
}

const handleFieldClick = (colId: string) => {
  canvasStore.selectColumn(colId)
}

// Filter columns with onlyMapped and searchKeyword
const filteredColumns = computed(() => {
  let list = props.data.columns || []
  if (onlyMapped.value) {
    list = list.filter(c => hasMapping(c.id))
  }
  if (!searchKeyword.value.trim()) return list
  const kw = searchKeyword.value.trim().toLowerCase()
  return list.filter(c =>
    c.column_name.toLowerCase().includes(kw) ||
    (c.column_comment && c.column_comment.toLowerCase().includes(kw))
  )
})

const hiddenCount = computed(() => {
  if (!onlyMapped.value) return 0
  return (props.data.columns || []).length - filteredColumns.value.length
})

const getTypeClass = (typeStr: string = '') => {
  const upper = typeStr.toUpperCase()
  if (upper.includes('INT') || upper.includes('NUMBER')) return 'type-num'
  if (upper.includes('CHAR') || upper.includes('TEXT') || upper.includes('CLOB')) return 'type-str'
  if (upper.includes('DATE') || upper.includes('TIME')) return 'type-date'
  return 'type-other'
}

const isPrimaryKey = (col: any) => {
  const name = (col?.column_name || '').toUpperCase()
  const comment = col?.column_comment || ''
  return name === 'ID' || name === `${props.data.tableName?.toUpperCase()}_ID` || comment.includes('主键') || comment.includes('唯一标识')
}

const handleCommand = (cmd: string) => {
  if (cmd === 'highlight') {
    if (isHighlighted.value && canvasStore.highlightedNodeIds.size > 0) {
      canvasStore.clearHighlight()
    } else {
      canvasStore.highlightLineage(props.data.tableId)
    }
  } else if (cmd === 'edit') {
    editForm.value = {
      tableName: props.data.tableName,
      tableComment: props.data.tableComment || '',
    }
    editDialogVisible.value = true
  } else if (cmd === 'remove-canvas') {
    canvasStore.removeTableFromCurrentCanvas(props.data.tableId)
  } else if (cmd === 'delete') {
    confirmDelete()
  }
}

const saveTableInfo = async () => {
  if (!editForm.value.tableName.trim()) {
    ElMessage.warning('表英文名不能为空')
    return
  }
  saving.value = true
  try {
    await api.updateTable(props.data.tableId, {
      table_name: editForm.value.tableName.trim(),
      table_comment: editForm.value.tableComment.trim(),
    })
    props.data.tableName = editForm.value.tableName.trim()
    props.data.tableComment = editForm.value.tableComment.trim()
    ElMessage.success('表信息更新成功')
    editDialogVisible.value = false
  } catch (e: any) {
    ElMessage.error('更新失败: ' + (e.message || '网络错误'))
  } finally {
    saving.value = false
  }
}

const confirmDelete = () => {
  ElMessageBox.confirm(
    `确定要删除表「${props.data.tableName}」吗？关联的所有字段映射都将被清除。`,
    '删除确认',
    {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
    try {
      await api.deleteTable(props.data.tableId)
      ElMessage.success('表已删除')
      if (projectStore.currentProject) {
        await projectStore.selectProject(projectStore.currentProject.id)
        canvasStore.syncFromProject(
          projectStore.currentProject.tables,
          projectStore.currentProject.mappings
        )
      }
    } catch (e: any) {
      ElMessage.error('删除失败: ' + (e.message || '网络错误'))
    }
  }).catch(() => {})
}
</script>

<style scoped>
.table-card-node {
  width: 320px;
  background: var(--cl-surface);
  border-radius: 8px;
  box-shadow: var(--cl-shadow-card);
  border: 1px solid var(--cl-border);
  display: flex;
  flex-direction: column;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  font-family: var(--cl-font-sans);
  overflow: visible !important;
}

.table-card-node:hover {
  box-shadow: var(--cl-shadow-card-hover);
  border-color: var(--cl-border-strong);
}

.table-card-node.is-dimmed {
  opacity: 0.22;
  filter: grayscale(85%);
}

.table-card-node.is-highlighted {
  border-color: var(--cl-primary);
  box-shadow: 0 0 0 2.5px var(--cl-primary-border), var(--cl-shadow-card-hover);
}

@keyframes focus-pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(29, 78, 216, 0.6), var(--cl-shadow-card);
    border-color: var(--cl-primary);
  }
  50% {
    box-shadow: 0 0 0 5px rgba(29, 78, 216, 0.25), 0 12px 30px rgba(29, 78, 216, 0.2);
    border-color: var(--cl-primary-hover);
    transform: scale(1.015);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(29, 78, 216, 0.6), var(--cl-shadow-card);
    border-color: var(--cl-primary);
  }
}

.table-card-node.is-focused {
  animation: focus-pulse 1.4s ease-in-out infinite;
  z-index: 100 !important;
}

.card-header {
  padding: 10px 12px;
  background: #f8fafc;
  border-top-left-radius: 7px;
  border-top-right-radius: 7px;
  border-bottom: 1px solid var(--cl-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  max-width: 220px;
}

.db-icon-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: var(--cl-primary-light);
  border: 1px solid var(--cl-primary-border);
  border-radius: 5px;
  flex-shrink: 0;
}

.table-svg {
  width: 14px;
  height: 14px;
}

.title-wrap {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.table-name {
  font-family: var(--cl-font-mono);
  font-weight: 600;
  font-size: 13px;
  color: var(--cl-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
  letter-spacing: -0.01em;
}

.table-comment {
  font-size: 11px;
  color: var(--cl-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
  margin-top: 1px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.count-pill {
  font-family: var(--cl-font-mono);
  font-size: 10px;
  padding: 1px 6px;
  height: 18px;
  line-height: 16px;
  background: #f1f5f9;
  color: var(--cl-text-secondary);
  border: 1px solid var(--cl-border);
  border-radius: 4px;
  font-weight: 500;
  user-select: none;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  font-size: 14px;
  color: var(--cl-text-muted);
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: #e2e8f0;
  color: var(--cl-text-primary);
}

.search-bar {
  padding: 6px 10px;
  background: #ffffff;
  border-bottom: 1px solid var(--cl-border);
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-input-wrap {
  position: relative;
  display: flex;
  align-items: center;
  flex: 1;
}

.btn-filter-mapped {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f8fafc;
  border: 1px solid var(--cl-border);
  border-radius: 4px;
  color: var(--cl-text-muted);
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.btn-filter-mapped:hover {
  background: var(--cl-surface-hover);
  color: var(--cl-text-primary);
  border-color: var(--cl-border-strong);
}

.btn-filter-mapped.is-active {
  background: var(--cl-primary-light);
  color: var(--cl-primary);
  border-color: var(--cl-primary-border);
}

.filter-folded-banner {
  font-size: 11px;
  color: var(--cl-primary);
  background: var(--cl-primary-light);
  border-bottom: 1px solid var(--cl-primary-border);
  padding: 3px 10px;
  text-align: center;
  font-weight: 500;
  font-family: var(--cl-font-mono);
}

.search-icon {
  position: absolute;
  left: 8px;
  font-size: 11px;
  color: var(--cl-text-faint);
  pointer-events: none;
}

.search-input {
  width: 100%;
  box-sizing: border-box;
  padding: 4px 24px;
  font-size: 11px;
  border: 1px solid var(--cl-border);
  border-radius: 4px;
  outline: none;
  background: var(--cl-surface-subtle);
  color: var(--cl-text-primary);
  transition: all 0.15s ease;
}

.search-input:focus {
  background: #ffffff;
  border-color: var(--cl-primary);
  box-shadow: 0 0 0 1px var(--cl-primary);
}

.clear-icon {
  position: absolute;
  right: 8px;
  font-size: 11px;
  color: var(--cl-text-faint);
  cursor: pointer;
  padding: 2px;
}

.clear-icon:hover {
  color: var(--cl-text-muted);
}

.column-list {
  max-height: 280px;
  overflow-y: auto;
  padding: 2px 0;
  border-bottom-left-radius: 7px;
  border-bottom-right-radius: 7px;
}

.column-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 6px 24px 6px 20px;
  transition: all 0.15s ease;
  user-select: none;
  cursor: pointer;
  border-left: 2px solid transparent;
}

.column-item:hover {
  background: #f8fafc;
}

/* Subtle accent for columns that have mappings */
.column-item.has-mapping {
  background: #f8faff;
  border-left-color: var(--cl-primary);
}

.column-item.has-mapping:hover {
  background: var(--cl-primary-light);
}

.column-item:hover .drag-hint-icon {
  opacity: 1;
  color: var(--cl-primary);
}

/* Breathing / Pulsing Animation for Activated Fields */
@keyframes breathe-pulse {
  0% {
    background-color: var(--cl-primary-light);
    box-shadow: 0 0 0 1.5px var(--cl-primary), 0 0 8px rgba(29, 78, 216, 0.35);
  }
  50% {
    background-color: #dbeafe;
    box-shadow: 0 0 0 2px var(--cl-primary-hover), 0 0 14px rgba(29, 78, 216, 0.55);
  }
  100% {
    background-color: var(--cl-primary-light);
    box-shadow: 0 0 0 1.5px var(--cl-primary), 0 0 8px rgba(29, 78, 216, 0.35);
  }
}

.column-item.is-breathing {
  animation: breathe-pulse 1.6s ease-in-out infinite;
  border-radius: 4px;
  z-index: 20;
}

.col-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: 8px;
}

.col-info {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  max-width: 175px;
}

.col-name-row {
  display: flex;
  align-items: center;
  gap: 5px;
}

.pk-badge {
  font-family: var(--cl-font-mono);
  font-size: 9px;
  font-weight: 700;
  color: #b45309;
  background: #fef3c7;
  border: 1px solid #fde68a;
  padding: 0 3px;
  border-radius: 2px;
  line-height: 1.1;
  flex-shrink: 0;
}

.col-name {
  font-family: var(--cl-font-mono);
  font-size: 12px;
  font-weight: 500;
  color: var(--cl-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.col-comment {
  font-size: 10px;
  color: var(--cl-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.col-right-group {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.col-type {
  font-size: 10px;
  font-family: var(--cl-font-mono);
  padding: 1px 5px;
  border-radius: 3px;
  white-space: nowrap;
  font-weight: 500;
  background: #f1f5f9;
  color: var(--cl-text-secondary);
  border: 1px solid var(--cl-border);
}

.mapping-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
}

.mapping-icon {
  font-size: 12px;
  color: var(--cl-primary);
}

.drag-hint-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
}

.drag-hint-icon {
  font-size: 11px;
  color: var(--cl-text-faint);
  opacity: 0.25;
  transition: all 0.15s ease;
}

/* Subtle, Muted Type Badges */
.type-num {
  background: #fafaf9;
  color: #78350f;
  border-color: #fef3c7;
}

.type-str {
  background: #f8fafc;
  color: #1e3a8a;
  border-color: #dbeafe;
}

.type-date {
  background: #f0fdf4;
  color: #166534;
  border-color: #dcfce7;
}

.type-other {
  background: #f8fafc;
  color: #475569;
  border-color: #e2e8f0;
}

/* Field Connection Handles */
.field-handle {
  position: absolute !important;
  top: 50% !important;
  transform: translateY(-50%) !important;
  z-index: 50 !important;
  opacity: 0.35;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.column-item:hover .field-handle,
.column-item.is-breathing .field-handle {
  opacity: 1 !important;
}

/* Left Port Handles */
.left-handle {
  left: 6px !important;
  width: 10px !important;
  height: 10px !important;
  border-radius: 50% !important;
  background-color: #ffffff !important;
  border: 2px solid var(--cl-text-faint) !important;
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.08) !important;
  cursor: crosshair !important;
}

.left-source {
  z-index: 52 !important;
}

.left-handle:hover,
.column-item:hover .left-handle {
  border-color: var(--cl-primary) !important;
  background-color: var(--cl-primary-light) !important;
  box-shadow: 0 0 0 3px var(--cl-primary-border) !important;
}

/* Right Port Handles */
.right-handle {
  right: 6px !important;
  width: 11px !important;
  height: 11px !important;
  border-radius: 50% !important;
  background: #ffffff !important;
  border: 2px solid var(--cl-primary) !important;
  box-shadow: 0 1px 3px rgba(29, 78, 216, 0.25) !important;
  cursor: crosshair !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
}

.right-source {
  z-index: 52 !important;
}

.right-handle::after {
  content: '';
  width: 3.5px;
  height: 3.5px;
  background-color: var(--cl-primary);
  border-radius: 50%;
  pointer-events: none;
}

.right-handle:hover,
.column-item:hover .right-handle {
  transform: translateY(-50%) scale(1.2) !important;
  background: var(--cl-primary) !important;
  border-color: var(--cl-primary-hover) !important;
  box-shadow: 0 0 0 3.5px var(--cl-primary-border) !important;
}

.right-handle:hover::after,
.column-item:hover .right-handle::after {
  background-color: #ffffff;
}

.empty-tip {
  text-align: center;
  font-size: 11px;
  color: var(--cl-text-muted);
  padding: 16px 0;
}

:deep(.table-dropdown-menu .el-dropdown-menu__item) {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  padding: 8px 14px;
}
</style>
