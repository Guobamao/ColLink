<template>
  <div class="workspace-container">
    <!-- Top Toolbar -->
    <CanvasToolbar
      @open-ddl-dialog="openDdlDialog"
      @open-db-dialog="openDbDialog"
      @open-project-dialog="openProjectDialog"
      @open-backup-dialog="openBackupDialog"
      @open-mapping-drawer="openMappingDrawer"
      @open-auto-mapping="openAutoMappingDialog"
      @open-relation-path="openRelationPathDrawer"
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
        :edge-types="edgeTypes"
        :default-edge-options="{ type: 'smartObstacle' }"
        :connection-mode="ConnectionMode.Loose"
        :connection-line-style="{ stroke: '#3b82f6', strokeWidth: 2 }"
        :connection-line-type="'smoothstep'"
        @node-drag-start="canvasStore.onNodeDragStart"
        @node-drag="canvasStore.onNodeDrag"
        @node-drag-stop="canvasStore.onNodeDragStop"
        @pane-click="canvasStore.selectColumn(null)"
        @connect="handleConnect"
        @edge-click="handleEdgeClick"
      >
        <!-- Background Grid Pattern -->
        <Background :gap="20" :color="themeStore.isDark ? '#1e293b' : '#cbd5e1'" />

        <!-- Controls: Zoom In/Out, Fit View, Lock -->
        <Controls position="bottom-right" />

        <!-- MiniMap Component -->
        <MiniMap
          v-if="showMiniMap"
          :node-stroke-color="themeStore.isDark ? '#60a5fa' : '#3b82f6'"
          :node-color="themeStore.isDark ? '#141e30' : '#dbeafe'"
          :node-border-radius="3"
          :mask-color="themeStore.isDark ? 'rgba(11, 17, 32, 0.75)' : 'rgba(241, 245, 249, 0.6)'"
          class="canvas-minimap"
        />

        <!-- MiniMap Fold/Unfold Toggle Button -->
        <div
          class="minimap-toggle-btn"
          :class="{ 'is-active': showMiniMap, 'is-collapsed': !showMiniMap }"
          :title="showMiniMap ? '收起鹰眼缩略图' : '展开鹰眼缩略图'"
          @click="showMiniMap = !showMiniMap"
        >
          <el-icon><View v-if="showMiniMap" /><Hide v-else /></el-icon>
          <span>{{ showMiniMap ? '收起鹰眼' : '展开鹰眼' }}</span>
        </div>

        <!-- Floating Operational Tip Pill -->
        <div class="canvas-floating-tip">
          <el-icon class="tip-icon"><InfoFilled /></el-icon>
          <span class="tip-text">{{ statusTipText }}</span>
        </div>

        <!-- Custom Empty State overlay if no tables -->
        <div v-if="canvasStore.nodes.length === 0" class="canvas-empty-state">
          <div class="empty-card">
            <div class="empty-icon-wrap">
              <svg class="blueprint-svg" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
                <rect x="6" y="8" width="15" height="22" rx="3" stroke="#1D4ED8" stroke-width="2" stroke-dasharray="3 3"/>
                <rect x="27" y="18" width="15" height="22" rx="3" stroke="#1D4ED8" stroke-width="2"/>
                <path d="M 21 16 C 24 16, 24 26, 27 26" stroke="#3B82F6" stroke-width="2" stroke-linecap="round"/>
                <circle cx="21" cy="16" r="2" fill="#1D4ED8"/>
                <circle cx="27" cy="26" r="2" fill="#3B82F6"/>
              </svg>
            </div>
            <h3>当前业务视图暂无数据表</h3>
            <p>通过 DDL 建表脚本解析或直连数据库，一键载入数据表模型并开启可视化字段映射。</p>
            <div class="empty-actions">
              <el-button type="primary" :icon="DocumentAdd" @click="openDdlDialog">DDL 解析导入</el-button>
              <el-button plain :icon="Connection" @click="openDbDialog">连接数据库导入</el-button>
            </div>
          </div>
        </div>
      </VueFlow>
    </div>

    <!-- Dialogs and Drawers -->
    <DdlImportDialog ref="ddlDialogRef" />
    <DbConnectDialog ref="dbDialogRef" />
    <AutoMappingDialog ref="autoMappingDialogRef" />
    <ProjectDialog ref="projectDialogRef" />
    <BackupDialog ref="backupDialogRef" />
    <MappingDrawer ref="mappingDrawerRef" />
    <RelationPathDrawer ref="relationPathDrawerRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, markRaw, nextTick } from 'vue'
import { VueFlow, useVueFlow, ConnectionMode } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import { exportCanvasAsImage } from '@/utils/exportImage'
import { ElMessageBox, ElMessage } from 'element-plus'
import { InfoFilled, DataAnalysis, DocumentAdd, Connection, View, Hide } from '@element-plus/icons-vue'

// Styles for Vue Flow
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'
import '@vue-flow/minimap/dist/style.css'

import TableNode from '@/components/canvas/TableNode.vue'
import CanvasToolbar from '@/components/canvas/CanvasToolbar.vue'
import CanvasTabs from '@/components/canvas/CanvasTabs.vue'
import DdlImportDialog from '@/components/dialogs/DdlImportDialog.vue'
import DbConnectDialog from '@/components/dialogs/DbConnectDialog.vue'
import AutoMappingDialog from '@/components/canvas/AutoMappingDialog.vue'
import ProjectDialog from '@/components/dialogs/ProjectDialog.vue'
import BackupDialog from '@/components/dialogs/BackupDialog.vue'
import MappingDrawer from '@/components/canvas/MappingDrawer.vue'
import RelationPathDrawer from '@/components/canvas/RelationPathDrawer.vue'
import SmartObstacleEdge from '@/components/canvas/SmartObstacleEdge.vue'

import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { useThemeStore } from '@/stores/themeStore'

const { fitView, updateNodeInternals } = useVueFlow()

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()
const themeStore = useThemeStore()
canvasStore.registerUpdateNodeInternals(updateNodeInternals)

const statusTipText = computed(() => {
  if (canvasStore.highlightedNodeIds.size > 0) {
    return '已高亮表上下游血缘 · 单击空白画布退出高亮'
  }
  if (canvasStore.selectedColumnId) {
    return '已激活字段血缘脉冲连线 · 单击空白画布恢复全景'
  }
  return '点击字段聚焦血缘 · 拖拽两端蓝点建立连线 · Ctrl+Z 撤销'
})

const canvasWrapperRef = ref<HTMLElement | null>(null)
const showMiniMap = ref(true)
const ddlDialogRef = ref<InstanceType<typeof DdlImportDialog> | null>(null)
const dbDialogRef = ref<InstanceType<typeof DbConnectDialog> | null>(null)
const autoMappingDialogRef = ref<InstanceType<typeof AutoMappingDialog> | null>(null)
const projectDialogRef = ref<InstanceType<typeof ProjectDialog> | null>(null)
const backupDialogRef = ref<InstanceType<typeof BackupDialog> | null>(null)
const mappingDrawerRef = ref<InstanceType<typeof MappingDrawer> | null>(null)
const relationPathDrawerRef = ref<InstanceType<typeof RelationPathDrawer> | null>(null)

// Register custom node & edge components
const nodeTypes = {
  tableCard: markRaw(TableNode),
}

const edgeTypes = {
  smartObstacle: markRaw(SmartObstacleEdge),
  smoothstep: markRaw(SmartObstacleEdge),
}

onMounted(async () => {
  window.addEventListener('collink:open-relation-path', (e: any) => {
    openRelationPathDrawer(e.detail?.sourceTableId)
  })
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
    await canvasStore.removeEdge(edge.id, projectStore.currentProject?.id)
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

const openAutoMappingDialog = () => {
  if (!checkProjectSelected()) return
  autoMappingDialogRef.value?.open()
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
const openRelationPathDrawer = (srcId?: string, tgtId?: string) => {
  if (!checkProjectSelected()) return
  relationPathDrawerRef.value?.open(srcId, tgtId)
}

// Full Canvas High-Res 2x PNG Export (SVG Serialization & Offscreen Canvas)
const handleExportImage = async () => {
  if (!canvasWrapperRef.value) return
  if (canvasStore.nodes.length === 0) {
    ElMessage.warning('画布中暂无数据表，无法导出')
    return
  }

  ElMessage.info('正在生成全画布 2x 超清架构图，请稍候...')

  // Temporarily make all edges visible for full architecture overview
  const prevEdgesState = canvasStore.edges.map(e => ({
    id: e.id,
    hidden: e.hidden,
    animated: e.animated,
    style: { ...(e.style || {}) },
  }))

  canvasStore.edges.forEach(e => {
    e.hidden = false
    e.animated = false
    e.style = {
      stroke: '#3b82f6',
      strokeWidth: 2,
    }
  })

  // Wait for nextTick and layout to stabilize
  await nextTick()
  await new Promise(resolve => setTimeout(resolve, 150))

  try {
    const projName = projectStore.currentProject?.name || '数据架构'
    const canvasName = canvasStore.currentCanvas?.name || '全景视图'
    const dateStr = new Date().toISOString().slice(0, 10)
    const filename = `${projName}_${canvasName}_${dateStr}.png`

    await exportCanvasAsImage({
      canvasWrapper: canvasWrapperRef.value,
      nodes: canvasStore.nodes,
      filename,
      padding: 70,
      scale: 2,
      backgroundColor: '#F8FAFC',
    })

    ElMessage.success('全画布 2x 超清架构图导出成功')
  } catch (err: any) {
    ElMessage.error('导出图片失败: ' + (err.message || '未知错误'))
  } finally {
    // Restore original edge visibility and animations
    prevEdgesState.forEach(st => {
      const e = canvasStore.edges.find(edge => edge.id === st.id)
      if (e) {
        e.hidden = st.hidden
        e.animated = st.animated
        e.style = st.style
      }
    })
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
  background-color: var(--cl-canvas-bg);
}

.canvas-wrapper {
  flex: 1;
  width: 100%;
  height: calc(100vh - 52px - 38px);
  position: relative;
}

:deep(.vue-flow__viewport) {
  will-change: transform;
}

:deep(.vue-flow__node) {
  will-change: transform;
}

/* Floating Operational Tip Pill (Bottom Left) */
.canvas-floating-tip {
  position: absolute;
  bottom: 20px;
  left: 20px;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 7px;
  background: #ffffff;
  border: 1px solid var(--cl-border);
  border-radius: 20px;
  padding: 5px 14px;
  box-shadow: 0 4px 14px rgba(15, 23, 42, 0.08);
  user-select: none;
  pointer-events: none;
}

.tip-icon {
  font-size: 14px;
  color: var(--cl-primary);
  display: flex;
  align-items: center;
  justify-content: center;
}

.tip-text {
  font-size: 11px;
  color: var(--cl-text-secondary);
  font-weight: 500;
  letter-spacing: 0.01em;
}

/* Vue Flow Controls Aesthetic Upgrade */
:deep(.vue-flow__controls) {
  bottom: 16px !important;
  right: 16px !important;
  margin: 0 !important;
  padding: 0 !important;
  background: #ffffff !important;
  border: 1px solid var(--cl-border) !important;
  border-radius: 6px !important;
  box-shadow: 0 4px 14px rgba(15, 23, 42, 0.08) !important;
  overflow: hidden !important;
  display: flex !important;
  flex-direction: column !important;
  z-index: 10 !important;
}

:deep(.vue-flow__controls-button) {
  width: 32px !important;
  height: 32px !important;
  box-sizing: border-box !important;
  padding: 0 !important;
  margin: 0 !important;
  border: none !important;
  border-bottom: 1px solid var(--cl-border) !important;
  background: transparent !important;
  color: var(--cl-text-secondary) !important;
  transition: all 0.15s ease !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
}

:deep(.vue-flow__controls-button:last-child) {
  border-bottom: none !important;
}

:deep(.vue-flow__controls-button:hover) {
  background: var(--cl-surface-hover) !important;
  color: var(--cl-text-primary) !important;
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
  background: #ffffff;
  padding: 40px 48px;
  border-radius: 12px;
  border: 1px solid var(--cl-border);
  box-shadow: 0 12px 32px -4px rgba(15, 23, 42, 0.08);
  text-align: center;
  max-width: 440px;
}

.empty-icon-wrap {
  width: 60px;
  height: 60px;
  background: var(--cl-primary-light);
  border: 1px solid var(--cl-primary-border);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 18px auto;
}

.blueprint-svg {
  width: 36px;
  height: 36px;
}

.empty-card h3 {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--cl-text-primary);
  letter-spacing: -0.01em;
}

.empty-card p {
  font-size: 13px;
  color: var(--cl-text-muted);
  line-height: 1.6;
  margin: 0 0 24px 0;
}

.empty-actions {
  display: flex;
  justify-content: center;
  gap: 12px;
}

/* MiniMap Container */
:deep(.canvas-minimap) {
  bottom: 16px !important;
  right: 60px !important;
  margin: 0 !important;
  width: 200px !important;
  height: 130px !important;
  background: #ffffff !important;
  border: 1px solid var(--cl-border) !important;
  border-radius: 6px !important;
  box-shadow: 0 4px 14px rgba(15, 23, 42, 0.08) !important;
  overflow: hidden !important;
  z-index: 9 !important;
}

/* MiniMap Toggle Button */
.minimap-toggle-btn {
  position: absolute;
  bottom: 154px;
  right: 60px;
  z-index: 10;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 10px;
  background: #ffffff;
  border: 1px solid var(--cl-border);
  border-radius: 4px;
  box-shadow: 0 2px 6px rgba(15, 23, 42, 0.06);
  font-size: 11px;
  font-weight: 500;
  color: var(--cl-text-secondary);
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
}

.minimap-toggle-btn.is-collapsed {
  bottom: 16px;
  right: 60px;
  height: 32px;
  padding: 0 12px;
  border-radius: 6px;
  box-shadow: 0 4px 14px rgba(15, 23, 42, 0.08);
}

.minimap-toggle-btn:hover {
  background: #ffffff;
  color: var(--cl-primary);
  border-color: var(--cl-border-active);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.12);
}

.minimap-toggle-btn.is-active {
  color: var(--cl-primary);
  font-weight: 600;
}

/* Edge and Node Layering */
:deep(.vue-flow__edges) {
  z-index: 10 !important;
}

:deep(.vue-flow__node) {
  z-index: 20 !important;
}

:deep(.vue-flow__node.selected),
:deep(.vue-flow__node:hover) {
  z-index: 30 !important;
}
</style>
