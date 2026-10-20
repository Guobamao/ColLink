<template>
  <div
    class="table-card-node nowheel"
    :class="{
      'is-dimmed': isDimmed,
      'is-highlighted': isHighlighted,
    }"
    @wheel.stop
  >
    <!-- Card Header -->
    <div class="card-header">
      <div class="header-left">
        <span class="db-icon">🗄️</span>
        <div class="title-wrap">
          <div class="table-name" :title="data.tableName">{{ data.tableName }}</div>
          <div v-if="data.tableComment" class="table-comment" :title="data.tableComment">
            {{ data.tableComment }}
          </div>
        </div>
      </div>
      <div class="header-right">
        <el-tag size="small" type="info" effect="plain" round class="count-tag">
          {{ data.columns?.length || 0 }} 列
        </el-tag>
        <el-dropdown trigger="click" @command="handleCommand">
          <span class="action-btn">⋮</span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="highlight">
                🔍 高亮上下游血缘
              </el-dropdown-item>
              <el-dropdown-item command="edit">
                ✏️ 编辑表信息
              </el-dropdown-item>
              <el-dropdown-item command="delete" divided style="color: #F56C6C;">
                🗑️ 删除此表
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- Column Search Bar -->
    <div class="search-bar">
      <input
        v-model="searchKeyword"
        type="text"
        placeholder="搜索字段名/注释..."
        class="search-input"
        @mousedown.stop
      />
      <span v-if="searchKeyword" class="clear-icon" @click="searchKeyword = ''">✕</span>
    </div>

    <!-- Column List (with nowheel and @wheel.stop to prevent canvas zooming) -->
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
              class="mapping-dot"
              title="已建立血缘映射，点击高亮并显示连线"
            >🔗</span>
            <span v-else class="drag-hint-icon" title="按住右侧蓝圈锚点拖拽连线">➔</span>
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

    <!-- Bottom Helper Bar -->
    <div class="card-footer">
      <span class="footer-tip">💡 点击字段查看连线；拖动右侧蓝点建立映射</span>
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
  width: 330px;
  background: #ffffff;
  border-radius: 10px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08), 0 1px 3px rgba(0, 0, 0, 0.04);
  border: 1px solid #cbd5e1;
  display: flex;
  flex-direction: column;
  transition: all 0.25s ease;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  overflow: visible !important;
}

.table-card-node:hover {
  box-shadow: 0 8px 24px rgba(31, 78, 121, 0.16);
  border-color: #93c5fd;
}

.table-card-node.is-dimmed {
  opacity: 0.25;
  filter: grayscale(80%);
}

.table-card-node.is-highlighted {
  border: 2px solid #3b82f6;
  box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.25), 0 8px 24px rgba(59, 130, 246, 0.3);
}

.card-header {
  padding: 10px 14px;
  background: #f8fafc;
  border-top-left-radius: 9px;
  border-top-right-radius: 9px;
  border-bottom: 1px solid #e2e8f0;
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

.db-icon {
  font-size: 16px;
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
}

.table-comment {
  font-size: 11px;
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.count-tag {
  font-size: 11px;
  padding: 0 6px;
  height: 20px;
}

.action-btn {
  font-size: 16px;
  color: #94a3b8;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  font-weight: bold;
}

.action-btn:hover {
  background: #e2e8f0;
  color: #334155;
}

.search-bar {
  position: relative;
  padding: 6px 12px;
  background: #fdfdfd;
  border-bottom: 1px solid #f1f5f9;
}

.search-input {
  width: 100%;
  box-sizing: border-box;
  padding: 5px 8px;
  font-size: 11px;
  border: 1px solid #cbd5e1;
  border-radius: 5px;
  outline: none;
  background: #ffffff;
}

.search-input:focus {
  border-color: #3b82f6;
}

.clear-icon {
  position: absolute;
  right: 18px;
  top: 10px;
  font-size: 10px;
  color: #94a3b8;
  cursor: pointer;
}

.column-list {
  max-height: 280px;
  overflow-y: auto;
  padding: 4px 0;
}

.column-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 6px 26px 6px 22px;
  transition: all 0.2s ease;
  user-select: none;
  cursor: pointer;
}

.column-item:hover {
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
  border-radius: 6px;
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
  max-width: 180px;
}

.col-name {
  font-size: 12px;
  font-weight: 500;
  color: #334155;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-comment {
  font-size: 10px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-right-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.col-type {
  font-size: 10px;
  font-family: monospace;
  padding: 2px 5px;
  border-radius: 3px;
  white-space: nowrap;
}

.mapping-dot {
  font-size: 12px;
  color: #3b82f6;
}

.drag-hint-icon {
  font-size: 11px;
  color: #94a3b8;
  opacity: 0.3;
  transition: all 0.2s;
}

.type-num {
  background: #fef3c7;
  color: #b45309;
}

.type-str {
  background: #dbeafe;
  color: #1d4ed8;
}

.type-date {
  background: #dcfce7;
  color: #15803d;
}

.type-other {
  background: #f1f5f9;
  color: #475569;
}

/* ========================================================
   Field Connection Handles (Left Target, Right Source Anchor)
   ======================================================== */
.field-handle {
  position: absolute !important;
  top: 50% !important;
  transform: translateY(-50%) !important;
  z-index: 50 !important;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

/* Target Handle (Left) */
.target-handle {
  left: 6px !important;
  width: 10px !important;
  height: 10px !important;
  border-radius: 50% !important;
  background-color: #94a3b8 !important;
  border: 2px solid #ffffff !important;
  box-shadow: 0 0 0 1px #64748b !important;
  cursor: crosshair !important;
}

.target-handle:hover {
  transform: translateY(-50%) scale(1.4) !important;
  background-color: #10b981 !important;
  box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.4) !important;
}

/* Source Handle (Right) - Draggable Anchor Port */
.source-handle {
  right: 6px !important;
  width: 14px !important;
  height: 14px !important;
  border-radius: 50% !important;
  background: #ffffff !important;
  border: 2.5px solid #2563eb !important;
  box-shadow: 0 1px 4px rgba(37, 99, 235, 0.4) !important;
  cursor: crosshair !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
}

.source-handle::after {
  content: '';
  width: 4px;
  height: 4px;
  background-color: #2563eb;
  border-radius: 50%;
  pointer-events: none;
}

.source-handle:hover {
  transform: translateY(-50%) scale(1.35) !important;
  background: #2563eb !important;
  box-shadow: 0 0 0 4px rgba(37, 99, 235, 0.3) !important;
}

.source-handle:hover::after {
  background-color: #ffffff;
}

.card-footer {
  padding: 4px 10px;
  background: #f8fafc;
  border-top: 1px solid #f1f5f9;
  border-bottom-left-radius: 9px;
  border-bottom-right-radius: 9px;
  text-align: center;
}

.footer-tip {
  font-size: 10px;
  color: #94a3b8;
}

.empty-tip {
  text-align: center;
  font-size: 11px;
  color: #94a3b8;
  padding: 16px 0;
}
</style>
