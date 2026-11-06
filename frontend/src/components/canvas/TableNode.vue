<template>
  <div
    class="table-card-node nowheel w-80 rounded-lg bg-surface border border-border shadow-card select-none flex flex-col font-sans transition-[border-color,box-shadow,opacity] duration-150"
    :class="{
      'opacity-25 grayscale': isDimmed,
      'border-primary ring-2 ring-primary/30 shadow-card-hover': isHighlighted,
      'is-focused': isFocused,
      'cursor-crosshair border-dashed border-primary ring-2 ring-primary/40': !!canvasStore.pickingTableRole,
    }"
    @click="handleCardClick"
    @wheel.stop
  >
    <!-- Card Header -->
    <div class="h-9 px-2.5 bg-surface-subtle border-b border-border rounded-t-[7px] flex items-center justify-between gap-2">
      <!-- Left: Table Icon & Title -->
      <div class="flex items-center gap-2 min-w-0 flex-1">
        <div class="w-5 h-5 rounded flex items-center justify-center bg-primary/10 text-primary border border-primary/20 shrink-0">
          <Table2 class="w-3.5 h-3.5" />
        </div>
        <div class="flex flex-col min-w-0">
          <div class="flex items-center gap-1.5">
            <span class="font-mono font-semibold text-xs text-primary-text truncate" :title="data.tableName">
              {{ data.tableName }}
            </span>
          </div>
          <span v-if="data.tableComment" class="text-[11px] text-muted truncate leading-tight -mt-0.5" :title="data.tableComment">
            {{ data.tableComment }}
          </span>
        </div>
      </div>

      <!-- Right: Column Count & Dropdown Actions -->
      <div class="flex items-center gap-1 shrink-0">
        <span class="px-1.5 py-0.2 text-[11px] font-mono rounded bg-surface-hover border border-border text-muted">
          {{ data.columns?.length || 0 }}
        </span>
        <Dropdown align="end">
          <template #trigger>
            <button
              type="button"
              class="w-5 h-5 flex items-center justify-center rounded text-muted hover:text-primary-text hover:bg-surface border border-transparent hover:border-border transition-colors cursor-pointer"
              title="更多操作"
            >
              <MoreHorizontal class="w-3.5 h-3.5" />
            </button>
          </template>
          <DropdownItem @select="handleCommand('highlight')">
            <Search class="w-3.5 h-3.5 text-primary mr-2" />
            <span>高亮上下游血缘</span>
          </DropdownItem>
          <DropdownItem @select="handleCommand('relation-path')">
            <Route class="w-3.5 h-3.5 text-primary mr-2" />
            <span>探查多跳关联路径</span>
          </DropdownItem>
          <DropdownItem @select="handleCommand('edit')">
            <Edit3 class="w-3.5 h-3.5 text-muted mr-2" />
            <span>编辑表元信息</span>
          </DropdownItem>
          <DropdownSeparator />
          <DropdownItem @select="handleCommand('remove-canvas')">
            <MinusCircle class="w-3.5 h-3.5 text-amber-500 mr-2" />
            <span>从当前视图移出</span>
          </DropdownItem>
          <DropdownItem class="text-red-500" @select="handleCommand('delete')">
            <Trash2 class="w-3.5 h-3.5 mr-2" />
            <span>从项目中彻底删除</span>
          </DropdownItem>
        </Dropdown>
      </div>
    </div>

    <!-- Search & Filter Bar -->
    <div class="h-7 px-2 bg-surface border-b border-border flex items-center justify-between gap-1.5">
      <div class="flex items-center gap-1.5 flex-1 min-w-0">
        <Search class="w-3 h-3 text-muted shrink-0" />
        <input
          v-model="searchKeyword"
          type="text"
          placeholder="过滤字段..."
          class="w-full bg-transparent text-[11px] text-primary-text placeholder:text-muted/60 focus:outline-none font-mono"
          @mousedown.stop
        />
        <button
          v-if="searchKeyword"
          type="button"
          class="text-muted hover:text-primary-text text-[10px] px-1"
          @click="searchKeyword = ''"
        >
          ×
        </button>
      </div>

      <!-- Filter Toggle: Only Show Mapped Fields -->
      <button
        type="button"
        class="w-5 h-5 flex items-center justify-center rounded text-xs transition-colors shrink-0"
        :class="onlyMapped ? 'bg-primary-light text-primary font-bold' : 'text-muted hover:text-primary-text hover:bg-surface-hover'"
        :title="onlyMapped ? '当前已折叠无连线字段，点击显示全部' : '仅展示有血缘连线的字段 (折叠其余孤立字段)'"
        @click.stop="onlyMapped = !onlyMapped"
      >
        <Filter class="w-3 h-3" />
      </button>
    </div>

    <!-- Folded Hint Banner -->
    <div v-if="onlyMapped && hiddenCount > 0" class="px-2 py-0.5 bg-primary-light/60 border-b border-primary/20 text-[10px] font-mono text-primary text-center">
      已折叠 {{ hiddenCount }} 个未映射字段
    </div>

    <!-- Column List (nowheel prevents outer canvas zooming) -->
    <div
      ref="columnListRef"
      class="column-list nowheel max-h-[300px] overflow-y-auto overflow-x-hidden divide-y divide-border/20 py-0.5 rounded-b-[7px]"
      @wheel.stop
      @scroll="handleColumnListScroll"
    >
      <div
        v-for="col in filteredColumns"
        :id="`col_item_${col.id}`"
        :key="col.id"
        class="column-item group relative flex items-center justify-between h-[27px] px-4 hover:bg-surface-hover cursor-pointer transition-colors text-xs"
        :class="{
          'is-breathing bg-primary-light/40': isColumnActive(col.id),
          'border-l-2 border-l-primary bg-primary/5': hasMapping(col.id) && !isColumnActive(col.id),
        }"
        :title="col.column_comment ? `${col.column_name} (${col.column_comment})` : col.column_name"
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

        <!-- Column Left: Name, PK Badge, Comment -->
        <div class="flex items-center gap-1.5 min-w-0 pr-2 flex-1">
          <span v-if="isPrimaryKey(col)" class="px-1 py-0.2 text-[10px] font-mono font-bold rounded bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/30 shrink-0 leading-none">
            PK
          </span>
          <span class="font-mono text-xs text-primary-text truncate font-medium">
            {{ col.column_name }}
          </span>
          <span v-if="col.column_comment" class="text-[11px] text-muted truncate max-w-[100px]">
            {{ col.column_comment }}
          </span>
        </div>

        <!-- Column Right: Type Pill & Mapping Indicator -->
        <div class="flex items-center gap-1.5 shrink-0">
          <span
            class="px-1.5 py-0.2 text-[10px] font-mono font-medium rounded border uppercase tracking-tight"
            :class="getTypeBadgeClass(col.data_type)"
          >
            {{ formatDataType(col.data_type) }}
          </span>
          <span v-if="hasMapping(col.id)" class="text-primary flex items-center" title="已建立血缘连线">
            <Link2 class="w-3 h-3" />
          </span>
          <span v-else class="text-muted/40 group-hover:text-muted/80 flex items-center">
            <ChevronRight class="w-2.5 h-2.5 opacity-60" />
          </span>
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
      </div>

      <div v-if="filteredColumns.length === 0" class="py-6 text-center text-xs text-muted font-mono">
        无匹配字段
      </div>
    </div>

    <!-- Edit Dialog -->
    <Dialog
      v-model:open="editDialogVisible"
      title="修改表元信息"
      description="编辑当前数据表的英文逻辑名称与业务注释"
      class="max-w-[420px]"
    >
      <div class="space-y-3.5 py-1" @mousedown.stop>
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-primary-text">表英文名</label>
          <Input v-model="editForm.tableName" placeholder="请输入表英文名称..." />
        </div>
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-primary-text">中文注释</label>
          <Input v-model="editForm.tableComment" placeholder="请输入中文业务描述..." />
        </div>
      </div>
      <template #footer>
        <Button variant="ghost" @click="editDialogVisible = false">取消</Button>
        <Button variant="primary" :loading="saving" @click="saveTableInfo">保存</Button>
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { Handle, Position, useVueFlow } from '@vue-flow/core'
import { useCanvasStore } from '@/stores/canvasStore'
import { useProjectStore } from '@/stores/projectStore'
import { api } from '@/api'
import { toast, confirmDialog } from '@/utils/toast'
import { Dialog, Button, Input, Dropdown, DropdownItem, DropdownSeparator } from '@/components/ui'
import {
  Table2,
  MoreHorizontal,
  Search,
  Filter,
  Link2,
  Edit3,
  Trash2,
  MinusCircle,
  Route,
  ChevronRight,
} from 'lucide-vue-next'
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
const { updateNodeInternals } = useVueFlow()

const columnListRef = ref<HTMLElement | null>(null)
let scrollRafId: number | null = null

// When user scrolls column list, notify Vue Flow to update handle positions in real time
const handleColumnListScroll = () => {
  if (scrollRafId !== null) return
  scrollRafId = requestAnimationFrame(() => {
    updateNodeInternals(props.id)
    scrollRafId = null
  })
}

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

// Automatically re-measure handle positions when filtered columns change or node mounts
watch(filteredColumns, () => {
  nextTick(() => {
    updateNodeInternals(props.id)
  })
})

onMounted(() => {
  nextTick(() => {
    updateNodeInternals(props.id)
  })
})

const formatDataType = (typeStr: string = '') => {
  return typeStr.split('(')[0].trim() || typeStr
}

const getTypeBadgeClass = (typeStr: string = '') => {
  const upper = typeStr.toUpperCase()
  if (upper.includes('INT') || upper.includes('NUMBER') || upper.includes('FLOAT') || upper.includes('DECIMAL')) {
    return 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20'
  }
  if (upper.includes('CHAR') || upper.includes('TEXT') || upper.includes('CLOB') || upper.includes('STRING')) {
    return 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20'
  }
  if (upper.includes('DATE') || upper.includes('TIME') || upper.includes('TIMESTAMP')) {
    return 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20'
  }
  return 'bg-zinc-500/10 text-zinc-600 dark:text-zinc-400 border-zinc-500/20'
}

const isPrimaryKey = (col: any) => {
  const name = (col?.column_name || '').toUpperCase()
  const comment = col?.column_comment || ''
  return name === 'ID' || name === `${props.data.tableName?.toUpperCase()}_ID` || comment.includes('主键') || comment.includes('唯一标识')
}

const handleCardClick = () => {
  if (canvasStore.pickingTableRole) {
    canvasStore.triggerTablePicked(props.data.tableId, props.data.tableName)
  }
}

const handleCommand = (cmd: string) => {
  if (cmd === 'highlight') {
    if (isHighlighted.value && canvasStore.highlightedNodeIds.size > 0) {
      canvasStore.clearHighlight()
    } else {
      canvasStore.highlightLineage(props.data.tableId)
    }
  } else if (cmd === 'relation-path') {
    window.dispatchEvent(new CustomEvent('collink:open-relation-path', { detail: { sourceTableId: props.data.tableId } }))
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
    toast.warning('表英文名不能为空')
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
    toast.success('表信息更新成功')
    editDialogVisible.value = false
  } catch (e: any) {
    toast.error('更新失败: ' + (e.message || '网络错误'))
  } finally {
    saving.value = false
  }
}

const confirmDelete = async () => {
  const confirmed = await confirmDialog({
    title: '删除确认',
    message: `确定要删除表「${props.data.tableName}」吗？关联的所有字段映射都将被清除。`,
    confirmText: '删除',
    variant: 'danger',
  })

  if (!confirmed) return

  try {
    await api.deleteTable(props.data.tableId)
    toast.success('表已删除')
    if (projectStore.currentProject) {
      await projectStore.selectProject(projectStore.currentProject.id)
      canvasStore.syncFromProject(
        projectStore.currentProject.tables,
        projectStore.currentProject.mappings
      )
    }
  } catch (e: any) {
    toast.error('删除失败: ' + (e.message || '网络错误'))
  }
}
</script>

<style scoped>
.table-card-node {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: geometricPrecision;
  image-rendering: -webkit-optimize-contrast;
  transform: translateZ(0);
}

.text-primary-text {
  color: var(--cl-text-primary);
}

/* Pulsing Animation for Focused Table (from Search or Selection) */
@keyframes focus-pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(59, 130, 246, 0.6), var(--cl-shadow-card);
    border-color: var(--cl-primary);
  }
  50% {
    box-shadow: 0 0 0 5px rgba(59, 130, 246, 0.25), 0 10px 25px rgba(59, 130, 246, 0.2);
    border-color: var(--cl-primary-hover);
    transform: scale(1.015);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(59, 130, 246, 0.6), var(--cl-shadow-card);
    border-color: var(--cl-primary);
  }
}

.table-card-node.is-focused {
  animation: focus-pulse 1.4s ease-in-out infinite;
  z-index: 100 !important;
}

/* Breathing animation for active / lineage-connected column item */
@keyframes breathe-pulse {
  0% {
    box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.8), 0 0 8px rgba(59, 130, 246, 0.25);
  }
  50% {
    box-shadow: inset 0 0 0 1.5px rgba(59, 130, 246, 1), 0 0 14px rgba(59, 130, 246, 0.45);
  }
  100% {
    box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.8), 0 0 8px rgba(59, 130, 246, 0.25);
  }
}

.column-item.is-breathing {
  animation: breathe-pulse 1.6s ease-in-out infinite;
  z-index: 20;
}

/* Field Connection Handles */
.field-handle {
  position: absolute !important;
  top: 50% !important;
  transform: translateY(-50%) !important;
  z-index: 50 !important;
  width: 8px !important;
  height: 8px !important;
  border-radius: 9999px !important;
  border: 1.5px solid var(--cl-primary) !important;
  background-color: var(--cl-surface) !important;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s ease, transform 0.15s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.15s ease !important;
}

.left-handle {
  left: 4px !important;
}

.right-handle {
  right: 4px !important;
}

.column-item:hover .field-handle,
.column-item.is-breathing .field-handle {
  opacity: 0.9 !important;
  pointer-events: all !important;
  transform: translateY(-50%) scale(1.15) !important;
}

.field-handle:hover {
  opacity: 1 !important;
  background-color: var(--cl-primary) !important;
  box-shadow: 0 0 0 3px var(--cl-primary-border) !important;
  transform: translateY(-50%) scale(1.35) !important;
}

.left-source,
.right-source {
  cursor: crosshair !important;
}
</style>
