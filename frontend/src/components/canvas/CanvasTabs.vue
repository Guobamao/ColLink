<template>
  <div class="h-8 px-3 bg-surface-subtle/80 border-b border-border flex items-center justify-between text-xs shrink-0 select-none">
    <!-- Left: Canvas Views Tabs List -->
    <div class="flex items-center gap-1 overflow-x-auto max-w-[calc(100%-240px)] h-full scrollbar-none py-1">
      <div
        v-for="c in canvasStore.canvases"
        :key="c.id"
        class="group relative h-6 px-2.5 rounded flex items-center gap-1.5 text-xs cursor-pointer transition-colors"
        :class="canvasStore.currentCanvasId === c.id
          ? 'bg-surface text-primary font-medium shadow-2xs border border-border'
          : 'text-muted hover:text-primary-text hover:bg-surface-hover/60 border border-transparent'"
        @click="canvasStore.switchCanvas(c.id)"
        @dblclick="handleRename(c)"
      >
        <LayoutDashboard class="w-3.5 h-3.5 shrink-0 opacity-70" />
        <span class="max-w-[130px] truncate" :title="`双击重命名: ${c.name}`">{{ c.name }}</span>
        <span
          class="px-1 py-0.2 text-[10px] font-mono rounded"
          :class="canvasStore.currentCanvasId === c.id ? 'bg-primary-light text-primary font-semibold' : 'bg-surface-hover text-muted'"
          :title="`当前视图包含 ${c.table_count || 0} 张表`"
        >
          {{ c.table_count || 0 }}
        </span>

        <!-- Tab Actions: Edit / Close on hover -->
        <div class="flex items-center gap-0.5 ml-1 opacity-0 group-hover:opacity-100 transition-opacity" @click.stop>
          <button
            type="button"
            class="p-0.5 rounded hover:bg-surface-hover text-muted hover:text-primary-text"
            title="重命名画布视图"
            @click="handleRename(c)"
          >
            <Edit3 class="w-3 h-3" />
          </button>
          <button
            v-if="canvasStore.canvases.length > 1"
            type="button"
            class="p-0.5 rounded hover:bg-red-500/10 text-muted hover:text-red-500"
            title="关闭并删除此画布视图"
            @click="handleDelete(c)"
          >
            <X class="w-3 h-3" />
          </button>
        </div>
      </div>

      <!-- + New Canvas Button -->
      <button
        type="button"
        class="h-6 px-1.5 rounded flex items-center gap-1 text-[11px] text-muted hover:text-primary-text hover:bg-surface border border-dashed border-border hover:border-primary/50 transition-colors ml-1.5 shrink-0"
        title="新建业务视图画布"
        @click="handleCreateCanvas"
      >
        <Plus class="w-3 h-3" />
        <span>新建</span>
      </button>
    </div>

    <!-- Right: Pool Summary & Import Button -->
    <div class="flex items-center gap-3 shrink-0">
      <span class="text-[11px] text-muted">
        项目表库共 <strong class="text-primary-text font-mono font-medium">{{ projectStore.currentProject?.tables?.length || 0 }}</strong> 张
      </span>
      <button
        type="button"
        class="h-6 px-2 rounded-md bg-surface hover:bg-surface-hover border border-border text-primary-text flex items-center gap-1.5 text-[11px] font-medium transition-colors shadow-2xs"
        @click="openImportDialog"
      >
        <Layers class="w-3.5 h-3.5 text-primary" />
        <span>引入数据表</span>
      </button>
    </div>

    <!-- Dialog: Import Tables from Project Pool -->
    <el-dialog
      v-model="importDialogVisible"
      title="从项目表库引入数据表"
      width="680px"
      destroy-on-close
    >
      <div class="flex flex-col gap-3">
        <p class="text-xs text-muted m-0 leading-relaxed">
          将当前项目「<strong class="text-primary-text">{{ projectStore.currentProject?.name }}</strong>」中尚未加入当前视图的公共表引入画布中参与连线。
        </p>
        <div class="flex justify-start">
          <el-input
            v-model="searchPoolKeyword"
            placeholder="按英文表名或中文注释过滤..."
            clearable
            size="small"
            class="w-72"
          >
            <template #prefix>
              <Search class="w-3.5 h-3.5 text-muted" />
            </template>
          </el-input>
        </div>

        <el-table
          ref="tablePoolRef"
          :data="filteredProjectTables"
          border
          size="small"
          max-height="320"
          row-key="id"
          @selection-change="handleSelectionChange"
        >
          <el-table-column
            type="selection"
            width="42"
            align="center"
            :selectable="(row) => !isTableInCurrentCanvas(row.id)"
          />
          <el-table-column prop="table_name" label="表英文名" min-width="160">
            <template #default="{ row }">
              <span class="font-mono font-medium text-primary-text">{{ row.table_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="table_comment" label="中文业务注释" min-width="160">
            <template #default="{ row }">
              <span v-if="row.table_comment" class="text-muted">{{ row.table_comment }}</span>
              <span v-else class="text-muted/60">无注释</span>
            </template>
          </el-table-column>
          <el-table-column label="字段数" width="75" align="center">
            <template #default="{ row }">
              <span class="font-mono text-muted">{{ row.columns?.length || 0 }}</span>
            </template>
          </el-table-column>
          <el-table-column label="视图状态" width="105" align="center">
            <template #default="{ row }">
              <el-tag v-if="isTableInCurrentCanvas(row.id)" type="info" size="small" effect="plain">
                已在视图中
              </el-tag>
              <el-tag v-else type="success" size="small" effect="light">
                可引入
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <template #footer>
        <div class="flex items-center justify-between">
          <span class="text-xs text-muted">
            已勾选 <strong class="text-primary font-mono">{{ selectedTableIds.length }}</strong> 张新数据表
          </span>
          <div class="flex items-center gap-2">
            <el-button @click="importDialogVisible = false">取消</el-button>
            <el-button
              type="primary"
              :disabled="selectedTableIds.length === 0"
              @click="confirmImportTables"
            >
              引入所选数据表
            </el-button>
          </div>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useCanvasStore } from '@/stores/canvasStore'
import { useProjectStore } from '@/stores/projectStore'
import { ElMessageBox, ElMessage } from 'element-plus'
import {
  LayoutDashboard,
  Plus,
  Edit3,
  X,
  Layers,
  Search,
} from 'lucide-vue-next'
import type { TableMeta } from '@/types'

const canvasStore = useCanvasStore()
const projectStore = useProjectStore()

const importDialogVisible = ref(false)
const searchPoolKeyword = ref('')
const selectedTableIds = ref<string[]>([])

const isTableInCurrentCanvas = (tableId: string): boolean => {
  return canvasStore.nodes.some(n => n.data?.tableId === tableId)
}

const filteredProjectTables = computed(() => {
  const list: TableMeta[] = projectStore.currentProject?.tables || []
  if (!searchPoolKeyword.value.trim()) return list
  const kw = searchPoolKeyword.value.trim().toLowerCase()
  return list.filter(t =>
    t.table_name.toLowerCase().includes(kw) ||
    (t.table_comment && t.table_comment.toLowerCase().includes(kw))
  )
})

const handleCreateCanvas = () => {
  if (!projectStore.currentProject) {
    ElMessage.warning('请先选择项目')
    return
  }

  ElMessageBox.prompt('请输入新业务视图名称（如：专项排查流、核心信贷报送）：', '新建业务视图', {
    confirmButtonText: '创建',
    cancelButtonText: '取消',
    inputPattern: /\S+/,
    inputErrorMessage: '视图名称不能为空',
  }).then(async ({ value }) => {
    if (value && value.trim()) {
      await canvasStore.createCanvas(projectStore.currentProject!.id, value.trim())
    }
  }).catch(() => {})
}

const handleRename = (canvas: any) => {
  ElMessageBox.prompt('修改业务视图名称：', '重命名视图', {
    confirmButtonText: '保存',
    cancelButtonText: '取消',
    inputValue: canvas.name,
    inputPattern: /\S+/,
    inputErrorMessage: '视图名称不能为空',
  }).then(async ({ value }) => {
    if (value && value.trim() && value.trim() !== canvas.name) {
      await canvasStore.updateCanvasName(canvas.id, value.trim())
    }
  }).catch(() => {})
}

const handleDelete = (canvas: any) => {
  if (canvasStore.canvases.length <= 1) {
    ElMessage.warning('不能删除项目中最后一个画布')
    return
  }

  ElMessageBox.confirm(
    `确定删除业务视图「${canvas.name}」吗？\n注意：此操作仅删除视图布局，不会删除项目中的底层数据表结构。`,
    '删除业务视图',
    {
      confirmButtonText: '确认删除',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
    if (projectStore.currentProject) {
      await canvasStore.deleteCanvas(canvas.id, projectStore.currentProject.id)
    }
  }).catch(() => {})
}

const openImportDialog = () => {
  if (!projectStore.currentProject) {
    ElMessage.warning('请先选择项目')
    return
  }
  selectedTableIds.value = []
  searchPoolKeyword.value = ''
  importDialogVisible.value = true
}

const handleSelectionChange = (selection: TableMeta[]) => {
  selectedTableIds.value = selection
    .filter(t => !isTableInCurrentCanvas(t.id))
    .map(t => t.id)
}

const confirmImportTables = async () => {
  if (selectedTableIds.value.length === 0) return
  await canvasStore.addTablesToCurrentCanvas(selectedTableIds.value)
  importDialogVisible.value = false
}
</script>

<style scoped>
.text-primary-text {
  color: var(--cl-text-primary);
}

.scrollbar-none::-webkit-scrollbar {
  display: none;
}
.scrollbar-none {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
