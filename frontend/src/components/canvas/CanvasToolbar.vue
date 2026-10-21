<template>
  <div class="canvas-toolbar">
    <!-- Left: Brand & Project Selector -->
    <div class="toolbar-section left-section">
      <div class="app-brand">
        <span class="logo">⚡</span>
        <span class="brand-title">ColLink</span>
      </div>

      <div class="divider"></div>

      <div class="project-selector">
        <span class="proj-label">当前项目：</span>
        <el-select
          :model-value="projectStore.currentProject?.id"
          placeholder="请选择项目"
          class="project-select"
          @change="onProjectChange"
        >
          <el-option
            v-for="p in projectStore.projects"
            :key="p.id"
            :label="p.name"
            :value="p.id"
          />
        </el-select>
        <el-button class="btn-subtle" @click="emit('open-project-dialog')">
          📁 项目管理
        </el-button>
      </div>
    </div>

    <!-- Center: Main Actions & Canvas Controls (Strictly single horizontal line) -->
    <div class="toolbar-section center-section">
      <!-- Data Source Group -->
      <div class="btn-group">
        <el-button type="primary" class="btn-primary-action" @click="handleOpenDdl">
          📝 DDL 解析导入
        </el-button>
        <el-button class="btn-secondary-blue" @click="handleOpenDb">
          🔌 在线逆向拉取
        </el-button>
      </div>

      <div class="divider"></div>

      <!-- Canvas Layout Group -->
      <div class="btn-group">
        <el-button class="btn-subtle" @click="handleAutoLayout">
          ✨ 智能自动排版
        </el-button>
        <el-button class="btn-subtle" @click="emit('fit-view')">
          🎯 适应画布
        </el-button>
      </div>
    </div>

    <!-- Right: Deliverables & System Tools -->
    <div class="toolbar-section right-section">
      <!-- Lineage Filter State -->
      <el-button
        v-if="canvasStore.highlightedNodeIds.size > 0"
        type="warning"
        plain
        class="btn-warning-action"
        @click="canvasStore.clearHighlight()"
      >
        ✕ 清除血缘高亮
      </el-button>

      <!-- Mapping Drawer Button -->
      <el-badge :value="canvasStore.edges.length" :max="999" class="badge-item">
        <el-button class="btn-subtle" @click="emit('open-mapping-drawer')">
          🔗 映射清单
        </el-button>
      </el-badge>

      <div class="divider"></div>

      <!-- Export Actions -->
      <el-button class="btn-excel" @click="handleExportExcel">
        📊 导出 Excel
      </el-button>

      <el-button class="btn-subtle" @click="emit('export-image')">
        🖼️ 导出图片
      </el-button>

      <el-button class="btn-subtle" @click="emit('open-backup-dialog')">
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
  height: 56px;
  background: #ffffff;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.03);
  z-index: 20;
  white-space: nowrap;
  gap: 16px;
}

.toolbar-section {
  display: flex;
  align-items: center;
  white-space: nowrap;
  flex-shrink: 0;
}

.left-section {
  gap: 14px;
}

.center-section {
  gap: 12px;
}

.right-section {
  gap: 10px;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  user-select: none;
}

.logo {
  font-size: 20px;
  line-height: 1;
}

.brand-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  letter-spacing: -0.4px;
}

.divider {
  width: 1px;
  height: 20px;
  background: #e2e8f0;
  flex-shrink: 0;
}

.project-selector {
  display: flex;
  align-items: center;
  gap: 8px;
}

.proj-label {
  font-size: 13px;
  color: #64748b;
  font-weight: 500;
  white-space: nowrap;
  flex-shrink: 0;
}

.project-select {
  width: 210px;
}

.btn-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Button Stylings */
.btn-primary-action {
  height: 34px;
  background: #3b82f6;
  border-color: #3b82f6;
  color: #ffffff;
  font-weight: 500;
  font-size: 13px;
  padding: 0 14px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.btn-primary-action:hover {
  background: #2563eb;
  border-color: #2563eb;
}

.btn-secondary-blue {
  height: 34px;
  background: #eff6ff;
  border: 1px solid #bfdbfe;
  color: #1d4ed8;
  font-weight: 500;
  font-size: 13px;
  padding: 0 14px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.btn-secondary-blue:hover {
  background: #dbeafe;
  border-color: #93c5fd;
  color: #1e40af;
}

.btn-subtle {
  height: 34px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  color: #334155;
  font-weight: 500;
  font-size: 13px;
  padding: 0 13px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.btn-subtle:hover {
  background: #f8fafc;
  border-color: #cbd5e1;
  color: #0f172a;
}

.btn-warning-action {
  height: 34px;
  font-size: 12px;
  padding: 0 12px;
  border-radius: 6px;
}

/* Fresh business green Excel export button */
.btn-excel {
  height: 34px;
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  color: #16a34a;
  font-weight: 500;
  font-size: 13px;
  padding: 0 14px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.btn-excel:hover {
  background: #dcfce7;
  border-color: #86efac;
  color: #15803d;
}

.badge-item :deep(.el-badge__content) {
  top: 3px;
  right: 5px;
}
</style>
