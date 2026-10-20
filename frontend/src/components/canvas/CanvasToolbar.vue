<template>
  <div class="canvas-toolbar">
    <!-- Left: Project Selector -->
    <div class="toolbar-section left-section">
      <div class="app-brand">
        <span class="logo">⚡</span>
        <span class="brand-title">DataField Mapper</span>
      </div>

      <div class="divider"></div>

      <div class="project-selector">
        <span class="proj-label">当前项目：</span>
        <el-select
          :model-value="projectStore.currentProject?.id"
          placeholder="选择项目"
          style="width: 200px"
          @change="onProjectChange"
        >
          <el-option
            v-for="p in projectStore.projects"
            :key="p.id"
            :label="p.name"
            :value="p.id"
          />
        </el-select>
        <el-button type="default" plain size="small" @click="emit('open-project-dialog')">
          📁 项目管理
        </el-button>
      </div>
    </div>

    <!-- Right: Action Buttons -->
    <div class="toolbar-section right-section">
      <!-- Import Buttons -->
      <el-button-group>
        <el-button type="primary" size="small" @click="handleOpenDdl">
          📝 DDL 解析导入
        </el-button>
        <el-button type="primary" plain size="small" @click="handleOpenDb">
          🔌 在线逆向拉取
        </el-button>
      </el-button-group>

      <div class="divider"></div>

      <!-- Canvas Layout & View -->
      <el-button-group>
        <el-button size="small" @click="handleAutoLayout">
          ✨ 智能自动排版
        </el-button>
        <el-button size="small" @click="emit('fit-view')">
          🎯 适应画布
        </el-button>
      </el-button-group>

      <div class="divider"></div>

      <!-- Lineage Filter State -->
      <el-button
        v-if="canvasStore.highlightedNodeIds.size > 0"
        type="warning"
        size="small"
        plain
        @click="canvasStore.clearHighlight()"
      >
        ✕ 清除血缘高亮
      </el-button>

      <!-- Mapping Drawer Button -->
      <el-badge :value="canvasStore.edges.length" :max="999" class="badge-item">
        <el-button size="small" @click="emit('open-mapping-drawer')">
          🔗 映射清单
        </el-button>
      </el-badge>

      <div class="divider"></div>

      <!-- Export Actions -->
      <el-button type="success" size="small" plain @click="handleExportExcel">
        📊 导出 Excel
      </el-button>

      <el-button size="small" plain @click="emit('export-image')">
        🖼️ 导出图片
      </el-button>

      <el-button size="small" plain @click="emit('open-backup-dialog')">
        ⚙️ 备份/迁移
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { api } from '@/api'
import { ElMessage } from 'element-plus'

const emit = defineEmits([
  'open-ddl-dialog',
  'open-db-dialog',
  'open-project-dialog',
  'open-backup-dialog',
  'open-mapping-drawer',
  'fit-view',
  'export-image',
])

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const onProjectChange = async (projectId: string) => {
  await projectStore.selectProject(projectId)
  if (projectStore.currentProject) {
    canvasStore.syncFromProject(
      projectStore.currentProject.tables,
      projectStore.currentProject.mappings
    )
    emit('fit-view')
  }
}

const handleAutoLayout = async () => {
  await canvasStore.applyAutoLayout('LR')
  setTimeout(() => {
    emit('fit-view')
  }, 100)
}

const handleOpenDdl = () => {
  if (!projectStore.currentProject?.id) {
    ElMessage.error('请先选择或新建一个项目！')
    return
  }
  emit('open-ddl-dialog')
}

const handleOpenDb = () => {
  if (!projectStore.currentProject?.id) {
    ElMessage.error('请先选择或新建一个项目！')
    return
  }
  emit('open-db-dialog')
}

const handleExportExcel = () => {
  if (!projectStore.currentProject) {
    ElMessage.warning('当前无可用项目')
    return
  }
  const url = api.getExcelExportUrl(projectStore.currentProject.id)
  window.open(url, '_blank')
}
</script>

<style scoped>
.canvas-toolbar {
  height: 52px;
  background: #ffffff;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  z-index: 20;
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 8px;
}

.logo {
  font-size: 20px;
}

.brand-title {
  font-size: 15px;
  font-weight: 700;
  color: #1e3a8a;
  letter-spacing: -0.3px;
}

.divider {
  width: 1px;
  height: 22px;
  background: #e2e8f0;
}

.project-selector {
  display: flex;
  align-items: center;
  gap: 8px;
}

.proj-label {
  font-size: 12px;
  color: #64748b;
}

.badge-item :deep(.el-badge__content) {
  top: 4px;
  right: 6px;
}
</style>
