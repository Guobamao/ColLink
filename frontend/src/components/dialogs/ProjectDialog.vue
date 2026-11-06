<template>
  <Dialog
    v-model:open="visible"
    title="项目空间管理"
    description="管理多个业务项目空间，支持项目新建、快速切换与清理"
    class="max-w-[720px]"
  >
    <div class="flex flex-col gap-4 py-1">
      <!-- Create New Project -->
      <div class="flex items-center gap-2.5">
        <Input
          v-model="newProjectName"
          placeholder="输入新项目名称 (如：信贷割接数仓映射)"
          class="flex-1"
        />
        <Input
          v-model="newProjectDesc"
          placeholder="项目描述 (可选)"
          class="flex-1"
        />
        <Button
          variant="primary"
          :loading="creating"
          @click="handleCreate"
        >
          <Plus class="w-4 h-4 mr-1.5" />
          新建项目
        </Button>
      </div>

      <!-- Project List Table -->
      <div class="border border-border rounded-lg overflow-hidden max-h-[380px] overflow-y-auto">
        <table class="w-full text-left text-xs border-collapse">
          <thead class="bg-surface-subtle sticky top-0 border-b border-border text-muted font-medium select-none z-10">
            <tr>
              <th class="px-3.5 py-2.5 font-semibold">项目名称</th>
              <th class="px-3.5 py-2.5 font-semibold">描述</th>
              <th class="w-20 px-3 py-2.5 text-center font-semibold">表数量</th>
              <th class="w-20 px-3 py-2.5 text-center font-semibold">映射数</th>
              <th class="w-36 px-3 py-2.5 text-center font-semibold">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border/40">
            <tr
              v-for="row in projectStore.projects"
              :key="row.id"
              class="hover:bg-surface-hover transition-colors"
            >
              <td class="px-3.5 py-2.5 font-medium text-primary-text">
                <div class="flex items-center gap-2">
                  <span>{{ row.name }}</span>
                  <Badge
                    v-if="projectStore.currentProject?.id === row.id"
                    variant="success"
                  >
                    当前使用
                  </Badge>
                </div>
              </td>
              <td class="px-3.5 py-2.5 text-muted truncate max-w-[180px]">
                {{ row.description || '无描述' }}
              </td>
              <td class="px-3 py-2.5 font-mono text-center text-muted">
                {{ row.table_count || 0 }}
              </td>
              <td class="px-3 py-2.5 font-mono text-center text-muted">
                {{ row.mapping_count || 0 }}
              </td>
              <td class="px-3 py-2.5 text-center">
                <div class="flex items-center justify-center gap-2">
                  <Button
                    v-if="projectStore.currentProject?.id !== row.id"
                    variant="ghost"
                    size="xs"
                    class="text-primary hover:text-primary hover:bg-primary/10"
                    @click="handleSelect(row.id)"
                  >
                    切换进入
                  </Button>
                  <Button
                    variant="ghost"
                    size="xs"
                    class="text-red-500 hover:text-red-600 hover:bg-red-500/10"
                    @click="handleDelete(row)"
                  >
                    删除
                  </Button>
                </div>
              </td>
            </tr>
            <tr v-if="projectStore.projects.length === 0">
              <td colspan="5" class="py-10 text-center text-muted">
                暂无项目，请在上方输入名称创建新项目
              </td>
            </tr>
          </tbody>
        </table>
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
import { toast, confirmDialog } from '@/utils/toast'
import { Dialog, Input, Button, Badge } from '@/components/ui'
import { Plus } from 'lucide-vue-next'

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
    toast.warning('请输入项目名称')
    return
  }
  creating.value = true
  try {
    await projectStore.createProject(newProjectName.value.trim(), newProjectDesc.value.trim())
    newProjectName.value = ''
    newProjectDesc.value = ''
    if (projectStore.currentProject) {
      await canvasStore.fetchCanvases(projectStore.currentProject.id)
    }
  } finally {
    creating.value = false
  }
}

const handleSelect = async (id: string) => {
  await projectStore.selectProject(id)
  if (projectStore.currentProject) {
    await canvasStore.fetchCanvases(projectStore.currentProject.id)
  }
  visible.value = false
  toast.success('已切换项目')
}

const handleDelete = async (row: any) => {
  const confirmed = await confirmDialog({
    title: '删除警告',
    message: `确定要永久删除项目「${row.name}」吗？包含的所有表与映射都会被清空。`,
    confirmText: '删除',
    variant: 'danger',
  })

  if (!confirmed) return

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
}

defineExpose({ open })
</script>
