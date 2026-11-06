<template>
  <Sheet
    v-model:open="visible"
    title="字段映射血缘清单"
    :description="`当前项目包含 ${mappingsList.length} 条已建立的字段映射连线关系`"
    side="right"
    class="w-full sm:max-w-[720px] p-0 flex flex-col"
  >
    <div class="p-4 border-b border-border flex items-center justify-between gap-3 bg-surface-subtle">
      <Input
        v-model="searchKey"
        placeholder="搜索源/目标表名、字段名或注释..."
        class="w-72"
      >
        <template #prefix>
          <Search class="w-3.5 h-3.5 text-muted" />
        </template>
      </Input>
      <Button variant="outline" size="sm" @click="downloadExcel">
        <FileSpreadsheet class="w-4 h-4 mr-1.5 text-emerald-500" />
        导出 Excel 清单
      </Button>
    </div>

    <div class="flex-1 overflow-y-auto">
      <table class="w-full text-left text-xs border-collapse">
        <thead class="bg-surface-subtle sticky top-0 border-b border-border text-muted font-medium select-none z-10">
          <tr>
            <th class="w-10 px-3 py-2 text-center">#</th>
            <th class="px-3 py-2 font-semibold">源字段 (Source)</th>
            <th class="w-8 px-1 py-2 text-center"></th>
            <th class="px-3 py-2 font-semibold">目标字段 (Target)</th>
            <th class="w-16 px-3 py-2 text-center font-semibold">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-border/40">
          <tr
            v-for="(row, idx) in filteredMappings"
            :key="row.id"
            class="hover:bg-surface-hover transition-colors"
          >
            <td class="px-3 py-2 text-center font-mono text-muted">{{ idx + 1 }}</td>
            <td class="px-3 py-2">
              <div class="flex flex-col">
                <span class="text-[11px] text-muted font-mono">{{ row.source_table_name }}</span>
                <span class="font-mono font-medium text-primary-text">{{ row.source_column_name }}</span>
                <span v-if="row.source_column_comment" class="text-[11px] text-muted/80 truncate max-w-[200px]">
                  {{ row.source_column_comment }}
                </span>
              </div>
            </td>
            <td class="px-1 py-2 text-center text-muted">
              <ArrowRight class="w-3.5 h-3.5 mx-auto" />
            </td>
            <td class="px-3 py-2">
              <div class="flex flex-col">
                <span class="text-[11px] text-muted font-mono">{{ row.target_table_name }}</span>
                <span class="font-mono font-medium text-primary-text">{{ row.target_column_name }}</span>
                <span v-if="row.target_column_comment" class="text-[11px] text-muted/80 truncate max-w-[200px]">
                  {{ row.target_column_comment }}
                </span>
              </div>
            </td>
            <td class="px-3 py-2 text-center">
              <Button
                variant="ghost"
                size="xs"
                class="text-red-500 hover:text-red-600 hover:bg-red-500/10"
                @click="handleDeleteMapping(row.id)"
              >
                解除
              </Button>
            </td>
          </tr>
          <tr v-if="filteredMappings.length === 0">
            <td colspan="5" class="py-12 text-center text-muted">
              {{ searchKey ? '无匹配映射关系' : '暂无字段映射，可在画布中拖拽蓝点建立连线' }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </Sheet>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { api } from '@/api'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { toast } from '@/utils/toast'
import { Sheet, Input, Button } from '@/components/ui'
import { Search, FileSpreadsheet, ArrowRight } from 'lucide-vue-next'
import type { FieldMapping } from '@/types'

const visible = ref(false)
const searchKey = ref('')
const mappingsList = ref<FieldMapping[]>([])
const loading = ref(false)

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const open = async () => {
  visible.value = true
  if (projectStore.currentProject) {
    loading.value = true
    try {
      mappingsList.value = await api.getMappings(projectStore.currentProject.id)
    } finally {
      loading.value = false
    }
  }
}

const filteredMappings = computed(() => {
  if (!searchKey.value.trim()) return mappingsList.value
  const kw = searchKey.value.trim().toLowerCase()
  return mappingsList.value.filter(m =>
    (m.source_table_name && m.source_table_name.toLowerCase().includes(kw)) ||
    (m.source_column_name && m.source_column_name.toLowerCase().includes(kw)) ||
    (m.target_table_name && m.target_table_name.toLowerCase().includes(kw)) ||
    (m.target_column_name && m.target_column_name.toLowerCase().includes(kw)) ||
    (m.source_column_comment && m.source_column_comment.toLowerCase().includes(kw)) ||
    (m.target_column_comment && m.target_column_comment.toLowerCase().includes(kw))
  )
})

const handleDeleteMapping = async (mappingId: string) => {
  try {
    await api.deleteMapping(mappingId)
    mappingsList.value = mappingsList.value.filter(m => m.id !== mappingId)
    const edgeId = `edge_${mappingId}`
    canvasStore.edges = canvasStore.edges.filter(e => e.id !== edgeId)
    toast.success('映射关系已删除')
  } catch (err: any) {
    toast.error('删除映射失败: ' + (err.message || '网络错误'))
  }
}

const downloadExcel = async () => {
  if (!projectStore.currentProject) return
  await api.exportExcel(projectStore.currentProject.id, projectStore.currentProject.name)
}

defineExpose({ open })
</script>
