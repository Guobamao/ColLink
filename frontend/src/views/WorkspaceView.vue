<template>
  <div class="workspace-container">
    <!-- Top Toolbar -->
    <CanvasToolbar
      @open-ddl-dialog="openDdlDialog"
      @open-db-dialog="openDbDialog"
      @open-project-dialog="openProjectDialog"
      @open-backup-dialog="openBackupDialog"
      @open-mapping-drawer="openMappingDrawer"
      @fit-view="handleFitView"
      @export-image="handleExportImage"
      @focus-table="handleFocusTable"
    />

    <!-- Multi-Canvas / Subject View Tabs -->
    <CanvasTabs />

    <!-- Vue Flow Infinite Canvas Area -->
    <div ref="canvasWrapperRef" class="canvas-wrapper">
      <VueFlow
        v-model:nodes="canvasStore.nodes"
        v-model:edges="canvasStore.edges"
        :default-viewport="{ zoom: 0.9, x: 50, y: 50 }"
        :min-zoom="0.1"
        :max-zoom="2.5"
        :fit-view-on-init="true"
        :node-types="nodeTypes"
        :default-edge-options="{ type: 'smoothstep' }"
        :connection-mode="ConnectionMode.Loose"
        :connection-line-style="{ stroke: '#2563eb', strokeWidth: 2.5 }"
        :connection-line-type="'smoothstep'"
        @node-drag="canvasStore.onNodeDrag"
        @node-drag-stop="canvasStore.onNodeDragStop"
        @pane-click="canvasStore.selectColumn(null)"
        @connect="handleConnect"
        @edge-click="handleEdgeClick"
      >
        <!-- Background Grid Pattern -->
        <Background :gap="20" color="#e2e8f0" />

        <!-- Controls: Zoom In/Out, Fit View, Lock -->
        <Controls position="bottom-right" />

        <!-- Floating Operational Tip Pill -->
        <div class="canvas-floating-tip">
          <el-icon class="tip-icon" color="#3b82f6"><InfoFilled /></el-icon>
          <span class="tip-text">点击字段查看连线 · 拖动右侧蓝点建立映射</span>
        </div>

        <!-- Custom Empty State overlay if no tables -->
        <div v-if="canvasStore.nodes.length === 0" class="canvas-empty-state">
          <div class="empty-card">
            <div class="empty-icon-wrap">
              <el-icon class="empty-icon" color="#3b82f6"><DataAnalysis /></el-icon>
            </div>
            <h3>当前项目画布暂无数据表</h3>
            <p>请通过 <strong>DDL 脚本导入</strong> 或 <strong>在线数据库逆向拉取</strong> 快速载入数据表模型。</p>
            <div class="empty-actions">
              <el-button type="primary" :icon="DocumentAdd" @click="openDdlDialog">DDL 解析导入 (推荐)</el-button>
              <el-button plain :icon="Connection" @click="openDbDialog">在线数据库拉取</el-button>
            </div>
          </div>
        </div>
      </VueFlow>
    </div>

    <!-- Dialogs and Drawers -->
    <DdlImportDialog ref="ddlDialogRef" />
    <DbConnectDialog ref="dbDialogRef" />
    <ProjectDialog ref="projectDialogRef" />
    <BackupDialog ref="backupDialogRef" />
    <MappingDrawer ref="mappingDrawerRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, markRaw } from 'vue'
import { VueFlow, useVueFlow, ConnectionMode } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import html2canvas from 'html2canvas'
import { ElMessageBox, ElMessage } from 'element-plus'
import { InfoFilled, DataAnalysis, DocumentAdd, Connection } from '@element-plus/icons-vue'

// Styles for Vue Flow
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'

import TableNode from '@/components/canvas/TableNode.vue'
import CanvasToolbar from '@/components/canvas/CanvasToolbar.vue'
import CanvasTabs from '@/components/canvas/CanvasTabs.vue'
import DdlImportDialog from '@/components/dialogs/DdlImportDialog.vue'
import DbConnectDialog from '@/components/dialogs/DbConnectDialog.vue'
import ProjectDialog from '@/components/dialogs/ProjectDialog.vue'
import BackupDialog from '@/components/dialogs/BackupDialog.vue'
import MappingDrawer from '@/components/canvas/MappingDrawer.vue'

import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'

const { fitView } = useVueFlow()

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const canvasWrapperRef = ref<HTMLElement | null>(null)
const ddlDialogRef = ref<InstanceType<typeof DdlImportDialog> | null>(null)
const dbDialogRef = ref<InstanceType<typeof DbConnectDialog> | null>(null)
const projectDialogRef = ref<InstanceType<typeof ProjectDialog> | null>(null)
const backupDialogRef = ref<InstanceType<typeof BackupDialog> | null>(null)
const mappingDrawerRef = ref<InstanceType<typeof MappingDrawer> | null>(null)

// Register custom node components
const nodeTypes = {
  tableCard: markRaw(TableNode),
}

onMounted(async () => {
  await projectStore.fetchProjects()
  if (projectStore.currentProject) {
    await canvasStore.fetchCanvases(projectStore.currentProject.id)
    setTimeout(() => {
      handleFitView()
    }, 200)
  }
})

const handleFitView = () => {
  fitView({ padding: 0.2, duration: 400 })
}

const handleFocusTable = ({ tableId, nodeId, columnId }: { tableId: string; nodeId: string; columnId?: string }) => {
  canvasStore.focusTable(tableId)
  fitView({ nodes: [nodeId], duration: 600, maxZoom: 1.1, padding: 0.4 })
  if (columnId) {
    canvasStore.selectColumn(columnId)
    setTimeout(() => {
      const el = document.getElementById(`col_item_${columnId}`)
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
      }
    }, 100)
  }
}

const handleConnect = (params: any) => {
  if (projectStore.currentProject) {
    canvasStore.onConnect(params, projectStore.currentProject.id)
  }
}

const handleEdgeClick = (event: any) => {
  const edge = event.edge
  const label = edge.data?.label || '选中的字段映射'

  ElMessageBox.confirm(
    `当前选中的血缘关系：\n${label}\n\n是否删除此字段映射连线？`,
    '字段映射操作',
    {
      confirmButtonText: '删除映射',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
    await canvasStore.removeEdge(edge.id)
  }).catch(() => {})
}

// Project selection validator
const checkProjectSelected = (): boolean => {
  if (!projectStore.currentProject?.id) {
    ElMessage.error('请先选择或新建一个项目！')
    return false
  }
  return true
}

// Dialog openers
const openDdlDialog = () => {
  if (!checkProjectSelected()) return
  ddlDialogRef.value?.open()
}

const openDbDialog = () => {
  if (!checkProjectSelected()) return
  dbDialogRef.value?.open()
}

const openProjectDialog = () => projectDialogRef.value?.open()
const openBackupDialog = () => {
  if (!checkProjectSelected()) return
  backupDialogRef.value?.open()
}
const openMappingDrawer = () => {
  if (!checkProjectSelected()) return
  mappingDrawerRef.value?.open()
}

// Export High-Res PNG Image
const handleExportImage = async () => {
  if (!canvasWrapperRef.value) return
  ElMessage.info('正在生成高清画布图片，请稍候...')

  try {
    const canvasElement = canvasWrapperRef.value.querySelector('.vue-flow__viewport') as HTMLElement
    if (!canvasElement) {
      ElMessage.warning('未找到画布视口')
      return
    }

    const canvas = await html2canvas(canvasElement, {
      backgroundColor: '#f8fafc',
      scale: 2, // High resolution
      useCORS: true,
      logging: false,
    })

    const dataUrl = canvas.toDataURL('image/png')
    const link = document.createElement('a')
    const projName = projectStore.currentProject?.name || '数据架构'
    link.download = `${projName}_字段映射架构图.png`
    link.href = dataUrl
    link.click()

    ElMessage.success('画布架构图导出成功')
  } catch (err: any) {
    ElMessage.error('导出图片失败: ' + (err.message || '未知错误'))
  }
}
</script>

<style scoped>
.workspace-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: #f8fafc;
}

.canvas-wrapper {
  flex: 1;
  width: 100%;
  height: calc(100vh - 56px - 38px);
  position: relative;
}

/* Floating Operational Tip Pill (Bottom Left) */
.canvas-floating-tip {
  position: absolute;
  bottom: 20px;
  left: 20px;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.88);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid #e2e8f0;
  border-radius: 24px;
  padding: 6px 14px;
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.05);
  user-select: none;
  pointer-events: none;
}

.tip-icon {
  font-size: 15px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tip-text {
  font-size: 12px;
  color: #64748b;
  font-weight: 500;
  letter-spacing: 0.1px;
}

/* Vue Flow Controls Aesthetic Upgrade */
:deep(.vue-flow__controls) {
  bottom: 20px !important;
  right: 20px !important;
  background: rgba(255, 255, 255, 0.88) !important;
  backdrop-filter: blur(8px) !important;
  -webkit-backdrop-filter: blur(8px) !important;
  border: 1px solid #e2e8f0 !important;
  border-radius: 8px !important;
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.06) !important;
  overflow: hidden !important;
  display: flex !important;
  flex-direction: column !important;
}

:deep(.vue-flow__controls-button) {
  width: 32px !important;
  height: 32px !important;
  border: none !important;
  border-bottom: 1px solid #f1f5f9 !important;
  background: transparent !important;
  color: #475569 !important;
  transition: all 0.15s ease !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
}

:deep(.vue-flow__controls-button:last-child) {
  border-bottom: none !important;
}

:deep(.vue-flow__controls-button:hover) {
  background: #f1f5f9 !important;
  color: #0f172a !important;
}

.canvas-empty-state {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  pointer-events: all;
  z-index: 5;
}

.empty-card {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(8px);
  padding: 36px 48px;
  border-radius: 12px;
  border: 1px dashed #cbd5e1;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.06);
  text-align: center;
  max-width: 460px;
}

.empty-icon-wrap {
  width: 56px;
  height: 56px;
  background: #eff6ff;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 16px auto;
}

.empty-icon {
  font-size: 28px;
}

.empty-card h3 {
  margin: 0 0 8px 0;
  font-size: 16px;
  color: #1e293b;
}

.empty-card p {
  font-size: 13px;
  color: #64748b;
  line-height: 1.6;
  margin: 0 0 20px 0;
}

.empty-actions {
  display: flex;
  justify-content: center;
  gap: 12px;
}
</style>
