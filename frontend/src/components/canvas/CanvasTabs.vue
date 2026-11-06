<template>
  <div class="h-8 px-3 bg-surface-subtle/80 border-b border-border flex items-center justify-between text-xs shrink-0 select-none">
    <!-- Left: Canvas Views Tabs List -->
    <div class="flex items-center gap-1.5 overflow-x-auto max-w-[calc(100%-240px)] h-full scrollbar-none py-1">
      <div
        v-for="c in canvasStore.canvases"
        :key="c.id"
        class="group relative h-6.5 px-2.5 rounded-md flex items-center gap-1.5 text-xs cursor-pointer transition-colors"
        :class="canvasStore.currentCanvasId === c.id
          ? 'bg-surface text-primary font-medium shadow-2xs border border-border'
          : 'text-muted hover:text-primary-text hover:bg-surface-hover/60 border border-transparent'"
        @click="canvasStore.switchCanvas(c.id)"
        @dblclick="handleRename(c)"
      >
        <LayoutDashboard class="w-3.5 h-3.5 shrink-0 opacity-70" />
        <span class="max-w-[130px] truncate" :title="`双击重命名: ${c.name}`">{{ c.name }}</span>
        <span
          class="px-1.5 py-0.2 text-[10px] font-mono rounded font-semibold"
          :class="canvasStore.currentCanvasId === c.id ? 'bg-primary-light text-primary' : 'bg-surface-hover text-muted'"
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
        class="w-6 h-6 rounded-md flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface border border-transparent hover:border-border transition-colors ml-0.5 shrink-0 cursor-pointer"
        title="新建业务视图画布"
        @click="handleCreateCanvas"
      >
        <Plus class="w-3.5 h-3.5" />
      </button>
    </div>

    <!-- Right: Pool Summary & Import Button -->
    <div class="flex items-center gap-3 shrink-0">
      <span class="text-[11px] text-muted">
        项目表库共 <strong class="text-primary-text font-mono font-medium">{{ projectStore.currentProject?.tables?.length || 0 }}</strong> 张
      </span>
      <button
        type="button"
        class="h-6 px-2 rounded-md bg-surface hover:bg-surface-hover border border-border text-primary-text flex items-center gap-1.5 text-[11px] font-medium transition-colors shadow-2xs cursor-pointer"
        @click="openImportDialog"
      >
        <Layers class="w-3.5 h-3.5 text-primary" />
        <span>引入数据表</span>
      </button>
    </div>

    <!-- Dialog: Import Tables from Project Pool -->
    <Dialog
      v-model:open="importDialogVisible"
      title="从项目表库引入数据表"
      :description="`将当前项目「${projectStore.currentProject?.name}」中尚未加入当前视图的公共表引入画布中参与连线。`"
      class="max-w-[700px]"
    >
      <div class="flex flex-col gap-3">
        <div class="flex justify-start">
          <Input
            v-model="searchPoolKeyword"
            placeholder="按英文表名或中文注释过滤..."
            class="w-72"
          >
            <template #prefix>
              <Search class="w-3.5 h-3.5 text-muted" />
            </template>
          </Input>
        </div>

        <div class="border border-border rounded-md overflow-hidden max-h-[340px] overflow-y-auto">
          <table class="w-full text-left text-xs border-collapse">
            <thead class="bg-surface-subtle sticky top-0 border-b border-border text-muted font-medium select-none z-10">
              <tr>
                <th class="w-10 px-3 py-2 text-center">
                  <input
                    type="checkbox"
                    :checked="isAllSelected"
                    class="rounded border-border text-primary focus:ring-primary cursor-pointer"
                    @change="toggleSelectAll"
                  />
                </th>
                <th class="px-3 py-2 font-semibold">表英文名</th>
                <th class="px-3 py-2 font-semibold">中文业务注释</th>
                <th class="w-20 px-3 py-2 text-center font-semibold">字段数</th>
                <th class="w-24 px-3 py-2 text-center font-semibold">视图状态</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border/40">
              <tr
                v-for="row in filteredProjectTables"
                :key="row.id"
                class="hover:bg-surface-hover transition-colors"
                :class="{ 'opacity-60 bg-surface-subtle/50': isTableInCurrentCanvas(row.id) }"
              >
                <td class="px-3 py-2 text-center">
                  <input
                    type="checkbox"
                    :disabled="isTableInCurrentCanvas(row.id)"
                    :checked="selectedTableIds.includes(row.id)"
                    class="rounded border-border text-primary focus:ring-primary cursor-pointer disabled:cursor-not-allowed"
                    @change="toggleRow(row.id)"
                  />
                </td>
                <td class="px-3 py-2 font-mono font-medium text-primary-text">{{ row.table_name }}</td>
                <td class="px-3 py-2 text-muted truncate max-w-[200px]">{{ row.table_comment || '无注释' }}</td>
                <td class="px-3 py-2 font-mono text-center text-muted">{{ row.columns?.length || 0 }}</td>
                <td class="px-3 py-2 text-center">
                  <Badge v-if="isTableInCurrentCanvas(row.id)" variant="secondary">已在视图中</Badge>
                  <Badge v-else variant="success">可引入</Badge>
                </td>
              </tr>
              <tr v-if="filteredProjectTables.length === 0">
                <td colspan="5" class="py-8 text-center text-muted">暂无匹配的数据表</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-between w-full">
          <span class="text-xs text-muted">
            已勾选 <strong class="text-primary font-mono">{{ selectedTableIds.length }}</strong> 张新数据表
          </span>
          <div class="flex items-center gap-2">
            <Button variant="ghost" @click="importDialogVisible = false">取消</Button>
            <Button
              variant="primary"
              :disabled="selectedTableIds.length === 0"
              @click="confirmImportTables"
            >
              引入所选数据表
            </Button>
          </div>
        </div>
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useCanvasStore } from '@/stores/canvasStore'
import { useProjectStore } from '@/stores/projectStore'
import { toast, confirmDialog, promptDialog } from '@/utils/toast'
import { Dialog, Input, Button, Badge } from '@/components/ui'
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

const availableTables = computed(() => {
  return filteredProjectTables.value.filter(t => !isTableInCurrentCanvas(t.id))
})

const isAllSelected = computed(() => {
  if (availableTables.value.length === 0) return false
  return availableTables.value.every(t => selectedTableIds.value.includes(t.id))
})

const toggleSelectAll = () => {
  if (isAllSelected.value) {
    selectedTableIds.value = []
  } else {
    selectedTableIds.value = availableTables.value.map(t => t.id)
  }
}

const toggleRow = (id: string) => {
  const idx = selectedTableIds.value.indexOf(id)
  if (idx > -1) {
    selectedTableIds.value.splice(idx, 1)
  } else {
    selectedTableIds.value.push(id)
  }
}

const handleCreateCanvas = async () => {
  if (!projectStore.currentProject) {
    toast.warning('请先选择项目')
    return
  }

  const name = await promptDialog({
    title: '新建业务视图',
    message: '请输入新业务视图名称（如：专项排查流、核心信贷报送）：',
    placeholder: '业务视图名称...',
  })

  if (name && name.trim()) {
    await canvasStore.createCanvas(projectStore.currentProject.id, name.trim())
  }
}

const handleRename = async (canvas: any) => {
  const name = await promptDialog({
    title: '重命名视图',
    message: '修改业务视图名称：',
    defaultValue: canvas.name,
  })

  if (name && name.trim() && name.trim() !== canvas.name) {
    await canvasStore.updateCanvasName(canvas.id, name.trim())
  }
}

const handleDelete = async (canvas: any) => {
  if (canvasStore.canvases.length <= 1) {
    toast.warning('不能删除项目中最后一个画布')
    return
  }

  const confirmed = await confirmDialog({
    title: '删除业务视图',
    message: `确定删除业务视图「${canvas.name}」吗？\n注意：此操作仅删除视图布局，不会删除项目中的底层数据表结构。`,
    confirmText: '确认删除',
    variant: 'danger',
  })

  if (confirmed && projectStore.currentProject) {
    await canvasStore.deleteCanvas(canvas.id, projectStore.currentProject.id)
  }
}

const openImportDialog = () => {
  if (!projectStore.currentProject) {
    toast.warning('请先选择项目')
    return
  }
  selectedTableIds.value = []
  searchPoolKeyword.value = ''
  importDialogVisible.value = true
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
