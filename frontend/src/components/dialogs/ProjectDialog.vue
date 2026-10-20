<template>
  <el-dialog
    v-model="visible"
    title="项目空间管理"
    width="680px"
    destroy-on-close
  >
    <div class="project-dialog-body">
      <!-- Create New Project -->
      <div class="create-box">
        <el-input
          v-model="newProjectName"
          placeholder="输入新项目名称 (如：信贷割接数仓映射)"
          style="width: 280px"
        />
        <el-input
          v-model="newProjectDesc"
          placeholder="项目描述 (可选)"
          style="width: 240px"
        />
        <el-button type="primary" :loading="creating" @click="handleCreate">
          新建项目
        </el-button>
      </div>

      <!-- Project List Table -->
      <el-table :data="projectStore.projects" border size="default" max-height="360">
        <el-table-column prop="name" label="项目名称" min-width="160">
          <template #default="{ row }">
            <span class="proj-name">{{ row.name }}</span>
            <el-tag
              v-if="projectStore.currentProject?.id === row.id"
              size="small"
              type="success"
              style="margin-left: 8px"
            >
              当前使用
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="description" label="描述" min-width="140" show-overflow-tooltip />
        <el-table-column label="表数量" width="80" align="center">
          <template #default="{ row }">
            {{ row.table_count || 0 }}
          </template>
        </el-table-column>
        <el-table-column label="映射数" width="80" align="center">
          <template #default="{ row }">
            {{ row.mapping_count || 0 }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="160" align="center">
          <template #default="{ row }">
            <el-button
              v-if="projectStore.currentProject?.id !== row.id"
              type="primary"
              link
              size="small"
              @click="handleSelect(row.id)"
            >
              切换进入
            </el-button>
            <el-button
              type="danger"
              link
              size="small"
              @click="handleDelete(row)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { ElMessage, ElMessageBox } from 'element-plus'

const visible = ref(false)
const newProjectName = ref('')
const newProjectDesc = ref('')
const creating = ref(false)

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const open = () => {
  visible.value = true
}

const handleCreate = async () => {
  if (!newProjectName.value.trim()) {
    ElMessage.warning('请输入项目名称')
    return
  }
  creating.value = true
  try {
    await projectStore.createProject(newProjectName.value.trim(), newProjectDesc.value.trim())
    newProjectName.value = ''
    newProjectDesc.value = ''
    if (projectStore.currentProject) {
      canvasStore.syncFromProject(
        projectStore.currentProject.tables,
        projectStore.currentProject.mappings
      )
    }
  } finally {
    creating.value = false
  }
}

const handleSelect = async (id: string) => {
  await projectStore.selectProject(id)
  if (projectStore.currentProject) {
    canvasStore.syncFromProject(
      projectStore.currentProject.tables,
      projectStore.currentProject.mappings
    )
  }
  visible.value = false
  ElMessage.success('已切换项目')
}

const handleDelete = (row: any) => {
  ElMessageBox.confirm(
    `确定要永久删除项目「${row.name}」吗？包含的所有表与映射都会被清空。`,
    '删除警告',
    {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
    await projectStore.deleteProject(row.id)
    if (projectStore.currentProject) {
      canvasStore.syncFromProject(
        projectStore.currentProject.tables,
        projectStore.currentProject.mappings
      )
    } else {
      canvasStore.nodes = []
      canvasStore.edges = []
    }
  }).catch(() => {})
}

defineExpose({ open })
</script>

<style scoped>
.project-dialog-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.create-box {
  display: flex;
  align-items: center;
  gap: 10px;
  background: #f8fafc;
  padding: 12px;
  border-radius: 6px;
  border: 1px solid #e2e8f0;
}

.proj-name {
  font-weight: 500;
  color: #1e293b;
}
</style>
