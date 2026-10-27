<template>
  <el-drawer
    v-model="visible"
    size="700px"
    destroy-on-close
  >
    <template #header>
      <div class="drawer-custom-header">
        <div class="header-title-wrap">
          <el-icon color="#1d4ed8"><Link /></el-icon>
          <span>字段映射血缘清单</span>
          <span class="count-pill">{{ mappingsList.length }} 条映射</span>
        </div>
      </div>
    </template>
    <div class="drawer-body">
      <div class="drawer-header-bar">
        <el-input
          v-model="searchKey"
          placeholder="搜索源/目标表名、字段名或注释..."
          size="default"
          style="width: 280px"
          clearable
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-button class="btn-drawer-excel" :icon="DocumentChecked" size="default" @click="downloadExcel">
          导出 Excel 清单
        </el-button>
      </div>

      <el-table
        :data="filteredMappings"
        border
        size="small"
        max-height="calc(100vh - 170px)"
        class="mappings-table"
      >
        <el-table-column type="index" label="#" width="45" align="center" />
        <el-table-column label="源字段 (Source)" min-width="190">
          <template #default="{ row }">
            <div class="field-cell">
              <span class="tbl-name">{{ row.source_table_name }}</span>
              <span class="col-name">{{ row.source_column_name }}</span>
              <span v-if="row.source_column_comment" class="comment">{{ row.source_column_comment }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column width="42" align="center">
          <template #header>
            <el-icon color="#94a3b8"><Right /></el-icon>
          </template>
          <template #default>
            <el-icon color="#94a3b8"><Right /></el-icon>
          </template>
        </el-table-column>

        <el-table-column label="目标字段 (Target)" min-width="190">
          <template #default="{ row }">
            <div class="field-cell">
              <span class="tbl-name">{{ row.target_table_name }}</span>
              <span class="col-name">{{ row.target_column_name }}</span>
              <span v-if="row.target_column_comment" class="comment">{{ row.target_column_comment }}</span>
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
              解除
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
import { Link, DocumentChecked, Right, Search } from '@element-plus/icons-vue'
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

const downloadExcel = async () => {
  if (!projectStore.currentProject) return
  await api.exportExcel(projectStore.currentProject.id, projectStore.currentProject.name)
}

defineExpose({ open })
</script>

<style scoped>
.drawer-custom-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.header-title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 15px;
  color: var(--cl-text-primary);
}

.count-pill {
  font-family: var(--cl-font-mono);
  font-size: 11px;
  font-weight: 500;
  padding: 1px 7px;
  border-radius: 10px;
  background: var(--cl-primary-light);
  color: var(--cl-primary);
  border: 1px solid var(--cl-primary-border);
}

.drawer-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.drawer-header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.btn-drawer-excel {
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  color: #15803d;
  font-weight: 500;
  border-radius: 6px;
}

.btn-drawer-excel:hover {
  background: #dcfce7;
  border-color: #86efac;
  color: #166534;
}

.mappings-table {
  border-radius: 6px;
  overflow: hidden;
}

.field-cell {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 2px 0;
}

.tbl-name {
  font-family: var(--cl-font-mono);
  color: var(--cl-text-muted);
  font-size: 11px;
  line-height: 1.3;
}

.col-name {
  font-family: var(--cl-font-mono);
  font-weight: 600;
  color: var(--cl-text-primary);
  font-size: 12px;
  line-height: 1.3;
}

.comment {
  font-size: 11px;
  color: var(--cl-text-muted);
  line-height: 1.3;
}
</style>
