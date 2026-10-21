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
        <el-button class="btn-subtle btn-manage" @click="emit('open-project-dialog')">
          📁 项目管理
        </el-button>
      </div>
    </div>

    <!-- Center: Main Actions & Canvas Controls (Compact) -->
    <div class="toolbar-section center-section">
      <!-- 1. 导入数据源 (下拉菜单聚合) -->
      <el-dropdown trigger="click" @command="handleImportCommand">
        <el-button type="primary" class="btn-primary-action">
          <span>➕ 导入表结构</span>
          <span class="dropdown-arrow">▾</span>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu class="toolbar-dropdown-menu">
            <el-dropdown-item command="ddl">
              <span class="menu-icon">📝</span>
              <span class="menu-text">DDL 解析导入</span>
            </el-dropdown-item>
            <el-dropdown-item command="db">
              <span class="menu-icon">🔌</span>
              <span class="menu-text">在线逆向拉取</span>
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <div class="divider"></div>

      <!-- 2. 画布视图排版 (紧凑并排) -->
      <div class="btn-group">
        <el-button class="btn-subtle btn-compact" @click="handleAutoLayout">
          ✨ 自动排版
        </el-button>
        <el-button class="btn-subtle btn-compact" @click="emit('fit-view')">
          🎯 适应画布
        </el-button>
      </div>
    </div>

    <!-- Right: Deliverables & System Tools (Dropdown Aggregated) -->
    <div class="toolbar-section right-section">
      <!-- Lineage Filter State (Active when clicked) -->
      <el-button
        v-if="canvasStore.highlightedNodeIds.size > 0"
        type="warning"
        plain
        class="btn-warning-action"
        @click="canvasStore.clearHighlight()"
      >
        ✕ 清除血缘高亮
      </el-button>

      <!-- 3. 映射清单 (独立保留带Badge) -->
      <el-badge :value="canvasStore.edges.length" :max="999" class="badge-item">
        <el-button class="btn-subtle btn-compact" @click="emit('open-mapping-drawer')">
          🔗 映射清单
        </el-button>
      </el-badge>

      <div class="divider"></div>

      <!-- 4. 导出与备份 (下拉菜单聚合) -->
      <el-dropdown trigger="click" @command="handleExportCommand">
        <el-button class="btn-excel">
          <span>📤 导出与备份</span>
          <span class="dropdown-arrow">▾</span>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu class="toolbar-dropdown-menu">
            <el-dropdown-item command="excel">
              <span class="menu-icon">📊</span>
              <span class="menu-text">导出 Excel 映射清单</span>
            </el-dropdown-item>
            <el-dropdown-item command="image">
              <span class="menu-icon">🖼️</span>
              <span class="menu-text">导出高清拓扑图片 (PNG)</span>
            </el-dropdown-item>
            <el-dropdown-item command="backup" divided>
              <span class="menu-icon">⚙️</span>
              <span class="menu-text">项目快照备份 / 迁移 (JSON)</span>
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
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

const handleImportCommand = (cmd: string) => {
  if (cmd === 'ddl') {
    handleOpenDdl()
  } else if (cmd === 'db') {
    handleOpenDb()
  }
}

const handleExportCommand = (cmd: string) => {
  if (cmd === 'excel') {
    handleExportExcel()
  } else if (cmd === 'image') {
    emit('export-image')
  } else if (cmd === 'backup') {
    if (!projectStore.currentProject?.id) {
      ElMessage.error('请先选择或新建一个项目！')
      return
    }
    emit('open-backup-dialog')
  }
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
  padding: 0 16px;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.03);
  z-index: 20;
  white-space: nowrap;
  box-sizing: border-box;
  width: 100%;
  overflow: hidden;
  gap: 12px;
}

.toolbar-section {
  display: flex;
  align-items: center;
  white-space: nowrap;
  flex-shrink: 0;
}

.left-section {
  gap: 10px;
}

.center-section {
  gap: 8px;
}

.right-section {
  gap: 8px;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 6px;
  user-select: none;
}

.logo {
  font-size: 19px;
  line-height: 1;
}

.brand-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  letter-spacing: -0.4px;
}

.divider {
  width: 1px;
  height: 18px;
  background: #e2e8f0;
  flex-shrink: 0;
  margin: 0 2px;
}

.project-selector {
  display: flex;
  align-items: center;
  gap: 6px;
}

.proj-label {
  font-size: 13px;
  color: #64748b;
  font-weight: 500;
  white-space: nowrap;
  flex-shrink: 0;
}

.project-select {
  width: 170px;
}

.btn-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.dropdown-arrow {
  margin-left: 4px;
  font-size: 11px;
  opacity: 0.8;
  display: inline-block;
  line-height: 1;
}

/* Button Stylings */
.btn-primary-action {
  height: 34px;
  background: #3b82f6;
  border-color: #3b82f6;
  color: #ffffff;
  font-weight: 500;
  font-size: 13px;
  padding: 0 12px;
  border-radius: 6px;
  transition: all 0.15s ease;
  display: inline-flex;
  align-items: center;
}

.btn-primary-action:hover {
  background: #2563eb;
  border-color: #2563eb;
}

.btn-subtle {
  height: 34px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  color: #334155;
  font-weight: 500;
  font-size: 13px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.btn-subtle:hover {
  background: #f8fafc;
  border-color: #cbd5e1;
  color: #0f172a;
}

.btn-manage {
  padding: 0 10px;
}

.btn-compact {
  padding: 0 10px;
}

.btn-warning-action {
  height: 34px;
  font-size: 12px;
  padding: 0 10px;
  border-radius: 6px;
}

/* Fresh business green Excel & export dropdown button */
.btn-excel {
  height: 34px;
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  color: #16a34a;
  font-weight: 500;
  font-size: 13px;
  padding: 0 12px;
  border-radius: 6px;
  transition: all 0.15s ease;
  display: inline-flex;
  align-items: center;
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

<style>
/* Refined Dropdown Menu Items */
.toolbar-dropdown-menu .el-dropdown-menu__item {
  font-size: 13px;
  padding: 8px 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 4px;
  color: #334155;
  transition: all 0.15s ease;
}

.toolbar-dropdown-menu .el-dropdown-menu__item:hover {
  background-color: #f1f5f9;
  color: #2563eb;
}

.toolbar-dropdown-menu .menu-icon {
  font-size: 14px;
  line-height: 1;
}

.toolbar-dropdown-menu .menu-text {
  font-weight: 500;
}
</style>
