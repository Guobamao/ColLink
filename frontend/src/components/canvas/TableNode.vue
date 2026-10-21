<template>
  <div
    class="table-card-node nowheel"
    :class="{
      'is-dimmed': isDimmed,
      'is-highlighted': isHighlighted,
    }"
    @wheel.stop
  >
    <!-- Card Header with Top Brand Accent -->
    <div class="card-header">
      <div class="header-left">
        <div class="db-icon-badge">
          <svg class="table-svg" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect x="3" y="4" width="18" height="16" rx="2" stroke="#3b82f6" stroke-width="2"/>
            <path d="M3 9H21" stroke="#3b82f6" stroke-width="2"/>
            <path d="M9 20V9" stroke="#3b82f6" stroke-width="2"/>
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
              <el-dropdown-item command="delete" divided style="color: #ef4444;">
                <el-icon><Delete /></el-icon>
                <span>删除此表</span>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- Column Search Bar -->
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
        <!-- Target Handle (Left - Input Port) -->
        <Handle
          :id="`target_${col.id}`"
          type="target"
          :position="Position.Left"
          class="field-handle target-handle"
          title="输入锚点：接收连线"
        />

        <div class="col-content">
          <div class="col-info">
            <span class="col-name">{{ col.column_name }}</span>
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
            <span v-else class="drag-hint-wrap" title="按住右侧蓝圈锚点拖拽连线">
              <el-icon class="drag-hint-icon"><Right /></el-icon>
            </span>
          </div>
        </div>

        <!-- Source Handle (Right - Prominent Draggable Anchor Port) -->
        <Handle
          :id="`source_${col.id}`"
          type="source"
          :position="Position.Right"
          class="field-handle source-handle"
          title="输出锚点：按住拖拽连线至目标表字段"
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
import { Search, Edit, Delete, CircleClose, Link, Right, MoreFilled } from '@element-plus/icons-vue'
import type { ColumnMeta } from '@/types'

const props = defineProps<{
  id: string
  data: {
    tableId: string
    tableName: string
    tableComment?: string
    columns: ColumnMeta[]
  }
}>()

const canvasStore = useCanvasStore()
const projectStore = useProjectStore()

const searchKeyword = ref('')
const editDialogVisible = ref(false)
const saving = ref(false)
const editForm = ref({
  tableName: '',
  tableComment: '',
})

// Highlight / Dim logic for table level
const isHighlighted = computed(() => {
  return canvasStore.highlightedNodeIds.has(props.id)
})

const isDimmed = computed(() => {
  if (canvasStore.highlightedNodeIds.size === 0) return false
  return !canvasStore.highlightedNodeIds.has(props.id)
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

// Filter columns
const filteredColumns = computed(() => {
  const list = props.data.columns || []
  if (!searchKeyword.value.trim()) return list
  const kw = searchKeyword.value.trim().toLowerCase()
  return list.filter(c =>
    c.column_name.toLowerCase().includes(kw) ||
    (c.column_comment && c.column_comment.toLowerCase().includes(kw))
  )
})

const getTypeClass = (typeStr: string = '') => {
  const upper = typeStr.toUpperCase()
  if (upper.includes('INT') || upper.includes('NUMBER')) return 'type-num'
  if (upper.includes('CHAR') || upper.includes('TEXT') || upper.includes('CLOB')) return 'type-str'
  if (upper.includes('DATE') || upper.includes('TIME')) return 'type-date'
  return 'type-other'
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
  background: #ffffff;
  border-radius: 10px;
  box-shadow: 0 4px 12px -2px rgba(15, 23, 42, 0.06), 0 2px 4px -1px rgba(15, 23, 42, 0.04);
  border: 1px solid #e2e8f0;
  border-top: 3px solid #3b82f6;
  display: flex;
  flex-direction: column;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  overflow: visible !important;
}

.table-card-node:hover {
  box-shadow: 0 10px 25px -3px rgba(15, 23, 42, 0.1), 0 4px 6px -2px rgba(15, 23, 42, 0.05);
  border-color: #93c5fd;
  border-top-color: #2563eb;
}

.table-card-node.is-dimmed {
  opacity: 0.25;
  filter: grayscale(80%);
}

.table-card-node.is-highlighted {
  border-color: #3b82f6;
  border-top-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.25), 0 12px 28px rgba(59, 130, 246, 0.2);
}

.card-header {
  padding: 10px 14px;
  background: #f8fafc;
  border-top-left-radius: 7px;
  border-top-right-radius: 7px;
  border-bottom: 1px solid #f1f5f9;
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
  background: #eff6ff;
  border-radius: 6px;
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
  font-weight: 600;
  font-size: 13px;
  color: #1e293b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.table-comment {
  font-size: 11px;
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.count-pill {
  font-size: 11px;
  padding: 2px 8px;
  height: 20px;
  line-height: 16px;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #dbeafe;
  border-radius: 12px;
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
  color: #94a3b8;
  cursor: pointer;
  border-radius: 5px;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: #e2e8f0;
  color: #1e293b;
}

.search-bar {
  padding: 6px 10px;
  background: #ffffff;
  border-bottom: 1px solid #f1f5f9;
}

.search-input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 8px;
  font-size: 10px;
  color: #94a3b8;
  pointer-events: none;
}

.search-input {
  width: 100%;
  box-sizing: border-box;
  padding: 5px 24px 5px 24px;
  font-size: 11px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  outline: none;
  background: #f8fafc;
  color: #334155;
  transition: all 0.2s ease;
}

.search-input:focus {
  background: #ffffff;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.clear-icon {
  position: absolute;
  right: 8px;
  font-size: 10px;
  color: #94a3b8;
  cursor: pointer;
  padding: 2px;
}

.clear-icon:hover {
  color: #64748b;
}

.column-list {
  max-height: 280px;
  overflow-y: auto;
  padding: 4px 0;
  border-bottom-left-radius: 9px;
  border-bottom-right-radius: 9px;
}

.column-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 6px 26px 6px 22px;
  transition: all 0.15s ease;
  user-select: none;
  cursor: pointer;
  border-left: 2px solid transparent;
}

.column-item:hover {
  background: #f1f5f9;
}

/* Subtle accent for columns that have mappings */
.column-item.has-mapping {
  background: #f8faff;
  border-left-color: #3b82f6;
}

.column-item.has-mapping:hover {
  background: #eff6ff;
}

.column-item:hover .drag-hint-icon {
  opacity: 1;
  color: #2563eb;
}

/* ========================================================
   Breathing / Pulsing Animation for Activated Fields
   ======================================================== */
@keyframes breathe-pulse {
  0% {
    background-color: #eff6ff;
    box-shadow: 0 0 0 2px #3b82f6, 0 0 8px rgba(59, 130, 246, 0.4);
  }
  50% {
    background-color: #dbeafe;
    box-shadow: 0 0 0 3px #2563eb, 0 0 16px rgba(37, 99, 235, 0.65);
  }
  100% {
    background-color: #eff6ff;
    box-shadow: 0 0 0 2px #3b82f6, 0 0 8px rgba(59, 130, 246, 0.4);
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

.col-name {
  font-size: 12px;
  font-weight: 500;
  color: #1e293b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.col-comment {
  font-size: 10px;
  color: #94a3b8;
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
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  padding: 1px 5px;
  border-radius: 4px;
  white-space: nowrap;
  font-weight: 500;
}

.mapping-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
}

.mapping-icon {
  font-size: 13px;
  color: #3b82f6;
}

.drag-hint-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
}

.drag-hint-icon {
  font-size: 12px;
  color: #94a3b8;
  opacity: 0.25;
  transition: all 0.2s;
}

/* Morandi Soft Type Badges */
.type-num {
  background: #fef3c7;
  color: #92400e;
}

.type-str {
  background: #eef2ff;
  color: #3730a3;
}

.type-date {
  background: #dcfce7;
  color: #166534;
}

.type-other {
  background: #f1f5f9;
  color: #475569;
}

/* ========================================================
   Field Connection Handles (Left Target, Right Source Anchor)
   Unhovered: 0.35 opacity; Hovered or Breathing: 1.0 opacity
   ======================================================== */
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

/* Target Handle (Left) */
.target-handle {
  left: 6px !important;
  width: 9px !important;
  height: 9px !important;
  border-radius: 50% !important;
  background-color: #94a3b8 !important;
  border: 2px solid #ffffff !important;
  box-shadow: 0 0 0 1px #64748b !important;
  cursor: crosshair !important;
}

.target-handle:hover,
.column-item:hover .target-handle {
  background-color: #10b981 !important;
  box-shadow: 0 0 0 2.5px rgba(16, 185, 129, 0.35) !important;
}

/* Source Handle (Right) - Draggable Anchor Port */
.source-handle {
  right: 6px !important;
  width: 13px !important;
  height: 13px !important;
  border-radius: 50% !important;
  background: #ffffff !important;
  border: 2px solid #3b82f6 !important;
  box-shadow: 0 1px 3px rgba(59, 130, 246, 0.3) !important;
  cursor: crosshair !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
}

.source-handle::after {
  content: '';
  width: 4px;
  height: 4px;
  background-color: #3b82f6;
  border-radius: 50%;
  pointer-events: none;
}

.source-handle:hover,
.column-item:hover .source-handle {
  transform: translateY(-50%) scale(1.3) !important;
  background: #2563eb !important;
  border-color: #1d4ed8 !important;
  box-shadow: 0 0 0 4px rgba(37, 99, 235, 0.25) !important;
}

.source-handle:hover::after,
.column-item:hover .source-handle::after {
  background-color: #ffffff;
}

.empty-tip {
  text-align: center;
  font-size: 11px;
  color: #94a3b8;
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
