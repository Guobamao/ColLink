<template>
  <el-dialog
    v-model="visible"
    title="项目备份与数据迁移"
    width="540px"
    destroy-on-close
  >
    <div class="backup-dialog-body">
      <!-- Section 1: Backup Download -->
      <div class="card-box">
        <div class="box-title">📦 项目快照导出 (.json)</div>
        <p class="box-desc">
          将当前项目「{{ projectStore.currentProject?.name }}」包含的所有表结构、字段、节点坐标及连线血缘导出为标准 JSON 备份包。
        </p>
        <el-button type="primary" plain @click="handleDownloadJson">
          下载项目 JSON 备份包
        </el-button>
      </div>

      <!-- Section 2: Restore / Import -->
      <div class="card-box">
        <div class="box-title">📥 备份文件导入恢复</div>
        <p class="box-desc">
          从本地上传之前导出的 JSON 备份文件，系统将完整还原所有表卡片及其映射关系并创建新项目。
        </p>
        <el-upload
          action="#"
          :auto-upload="false"
          :show-file-list="false"
          accept=".json"
          :on-change="handleRestoreFile"
        >
          <el-button type="success" :loading="restoring">
            选择 JSON 备份文件恢复
          </el-button>
        </el-upload>
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { api } from '@/api'
import { ElMessage } from 'element-plus'

const visible = ref(false)
const restoring = ref(false)

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const open = () => {
  visible.value = true
}

const handleDownloadJson = () => {
  if (!projectStore.currentProject) {
    ElMessage.warning('当前无可用项目')
    return
  }
  const url = api.getJsonExportUrl(projectStore.currentProject.id)
  window.open(url, '_blank')
}

const handleRestoreFile = async (file: any) => {
  if (!file.raw) return
  restoring.value = true
  try {
    const formData = new FormData()
    formData.append('file', file.raw)
    const newProj = await api.restoreProjectJson(formData)
    ElMessage.success(`项目「${newProj.name}」恢复成功`)
    visible.value = false

    await projectStore.fetchProjects()
    await projectStore.selectProject(newProj.id)
    if (projectStore.currentProject) {
      canvasStore.syncFromProject(
        projectStore.currentProject.tables,
        projectStore.currentProject.mappings
      )
    }
  } catch (err: any) {
    ElMessage.error(err.response?.data?.detail || err.message || '恢复项目失败')
  } finally {
    restoring.value = false
  }
}

defineExpose({ open })
</script>

<style scoped>
.backup-dialog-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-box {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.box-title {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
}

.box-desc {
  font-size: 12px;
  color: #64748b;
  margin: 0;
  line-height: 1.5;
}
</style>
