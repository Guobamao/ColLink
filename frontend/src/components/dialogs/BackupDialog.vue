<template>
  <Dialog
    v-model:open="visible"
    title="项目备份与数据迁移"
    description="导出或恢复项目全量数据快照 (.json)"
    class="max-w-[540px]"
  >
    <div class="flex flex-col gap-4 py-1">
      <!-- Section 1: Backup Download -->
      <div class="rounded-lg border border-border bg-surface-subtle p-4 flex flex-col gap-2">
        <div class="flex items-center gap-2 text-sm font-semibold text-primary-text">
          <Download class="w-4 h-4 text-primary" />
          <span>项目快照导出 (.json)</span>
        </div>
        <p class="text-xs text-muted leading-relaxed m-0">
          将当前项目「{{ projectStore.currentProject?.name }}」包含的所有表结构、字段、节点坐标及连线血缘导出为标准 JSON 备份包。
        </p>
        <div class="pt-1">
          <Button variant="outline" @click="handleDownloadJson">
            <Download class="w-4 h-4 mr-1.5 text-primary" />
            下载项目 JSON 备份包
          </Button>
        </div>
      </div>

      <!-- Section 2: Restore / Import -->
      <div class="rounded-lg border border-border bg-surface-subtle p-4 flex flex-col gap-2">
        <div class="flex items-center gap-2 text-sm font-semibold text-primary-text">
          <Upload class="w-4 h-4 text-emerald-500" />
          <span>备份文件导入恢复</span>
        </div>
        <p class="text-xs text-muted leading-relaxed m-0">
          从本地上传之前导出的 JSON 备份文件，系统将完整还原所有表卡片及其映射关系并创建新项目。
        </p>
        <div class="pt-1">
          <input
            ref="fileInputRef"
            type="file"
            accept=".json"
            class="hidden"
            @change="handleFileInputChange"
          />
          <Button
            variant="success"
            :loading="restoring"
            @click="triggerFileInput"
          >
            <Upload class="w-4 h-4 mr-1.5" />
            选择 JSON 备份文件恢复
          </Button>
        </div>
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="visible = false">关闭</Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { api } from '@/api'
import { toast } from '@/utils/toast'
import { Dialog, Button } from '@/components/ui'
import { Download, Upload } from 'lucide-vue-next'

const visible = ref(false)
const restoring = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const open = () => {
  visible.value = true
}

const triggerFileInput = () => {
  if (fileInputRef.value) {
    fileInputRef.value.value = ''
    fileInputRef.value.click()
  }
}

const handleDownloadJson = async () => {
  if (!projectStore.currentProject) {
    toast.warning('当前无可用项目')
    return
  }
  await api.exportProjectJson(projectStore.currentProject.id, projectStore.currentProject.name)
}

const handleFileInputChange = async (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  restoring.value = true
  try {
    const formData = new FormData()
    formData.append('file', file)
    const newProj = await api.restoreProjectJson(formData)
    toast.success(`项目「${newProj.name}」恢复成功`)
    visible.value = false

    await projectStore.fetchProjects()
    await projectStore.selectProject(newProj.id)
    if (projectStore.currentProject) {
      await canvasStore.fetchCanvases(projectStore.currentProject.id)
    }
  } catch (err: any) {
    toast.error(err.response?.data?.detail || err.message || '恢复项目失败')
  } finally {
    restoring.value = false
  }
}

defineExpose({ open })
</script>
