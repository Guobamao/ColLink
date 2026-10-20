<template>
  <el-drawer
    v-model="visible"
    title="字段映射血缘清单"
    size="680px"
    destroy-on-close
  >
    <div class="drawer-body">
      <div class="drawer-header-bar">
        <el-input
          v-model="searchKey"
          placeholder="搜索源/目标表名、字段名..."
          size="small"
          style="width: 240px"
          clearable
        />
        <el-button type="primary" size="small" @click="downloadExcel">
          📊 导出 Excel 清单
        </el-button>
      </div>

      <el-table :data="filteredMappings" border size="small" max-height="calc(100vh - 180px)">
        <el-table-column type="index" label="#" width="45" align="center" />
        <el-table-column label="源字段 (Source)" min-width="160">
          <template #default="{ row }">
            <div class="field-cell">
              <span class="tbl-name">{{ row.source_table_name }}</span>
              <span class="col-name">{{ row.source_column_name }}</span>
              <span v-if="row.source_column_comment" class="comment">({{ row.source_column_comment }})</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="➔" width="40" align="center">
          <template #default>➔</template>
        </el-table-column>

        <el-table-column label="目标字段 (Target)" min-width="160">
          <template #default="{ row }">
            <div class="field-cell">
              <span class="tbl-name">{{ row.target_table_name }}</span>
              <span class="col-name">{{ row.target_column_name }}</span>
              <span v-if="row.target_column_comment" class="comment">({{ row.target_column_comment }})</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="70" align="center">
          <template #default="{ row }">
            <el-button
              type="danger"
              link
              size="small"
              @click="handleDeleteMapping(row.id)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </el-drawer>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { api } from '@/api'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { ElMessage } from 'element-plus'
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
    // Also remove edge from canvas
    const edgeId = `edge_${mappingId}`
    canvasStore.edges = canvasStore.edges.filter(e => e.id !== edgeId)
    ElMessage.success('映射关系已删除')
  } catch (err: any) {
    ElMessage.error('删除映射失败: ' + (err.message || '网络错误'))
  }
}

const downloadExcel = () => {
  if (!projectStore.currentProject) return
  const url = api.getExcelExportUrl(projectStore.currentProject.id)
  window.open(url, '_blank')
}

defineExpose({ open })
</script>

<style scoped>
.drawer-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.drawer-header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.field-cell {
  display: flex;
  flex-direction: column;
  font-size: 12px;
}

.tbl-name {
  color: #64748b;
  font-size: 11px;
}

.col-name {
  font-weight: 600;
  color: #1e293b;
}

.comment {
  font-size: 10px;
  color: #94a3b8;
}
</style>
