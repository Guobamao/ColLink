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
      @open-command-palette="commandPaletteRef?.open()"
      @open-settings="$emit('open-settings')"
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
        <Background
          :gap="20"
          :variant="themeStore.gridPattern === 'lines' ? 'lines' : 'dots'"
          :color="themeStore.isDark ? '#333333' : '#cbd5e1'"
        />

        <!-- Controls: Zoom In/Out, Fit View, Lock -->
        <Controls position="bottom-right" />

        <!-- MiniMap Component -->
        <MiniMap
          v-if="showMiniMap"
          :node-color="getMiniMapNodeColor"
          :node-stroke-color="getMiniMapNodeStrokeColor"
          :node-border-radius="4"
          :node-stroke-width="1.5"
          :mask-color="themeStore.isDark ? 'rgba(0, 0, 0, 0.55)' : 'rgba(15, 23, 42, 0.12)'"
          :mask-stroke-color="themeStore.isDark ? '#60a5fa' : '#2563eb'"
          :mask-stroke-width="2"
          :mask-border-radius="4"
          :pannable="true"
          :zoomable="true"
          class="canvas-minimap"
          @node-click="handleMiniMapNodeClick"
        />

        <!-- MiniMap Fold/Unfold Toggle Button -->
        <div
          class="minimap-toggle-btn"
          :class="{ 'is-active': showMiniMap, 'is-collapsed': !showMiniMap }"
          :title="showMiniMap ? '收起鹰眼缩略图' : '展开鹰眼缩略图'"
          @click="showMiniMap = !showMiniMap"
        >
          <Eye v-if="showMiniMap" class="w-3.5 h-3.5" />
          <EyeOff v-else class="w-3.5 h-3.5" />
          <span>{{ showMiniMap ? '收起鹰眼' : '展开鹰眼' }}</span>
        </div>

        <!-- Floating Operational Tip Pill -->
        <Transition name="fade-slide">
          <div v-if="showFloatingTip" class="canvas-floating-tip">
            <Info class="tip-icon w-3.5 h-3.5 shrink-0" />
            <span class="tip-text">{{ statusTipText }}</span>
            <button
              type="button"
              class="tip-close-btn"
              title="关闭提示"
              @click.stop="closeFloatingTip"
            >
              <X class="w-3 h-3" />
            </button>
          </div>
        </Transition>

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
              <Button variant="primary" @click="openDdlDialog">
                <FileCode class="w-4 h-4 mr-1.5" /> DDL 解析导入
              </Button>
              <Button variant="outline" @click="openDbDialog">
                <Database class="w-4 h-4 mr-1.5" /> 连接数据库导入
              </Button>
            </div>
          </div>
        </div>
      </VueFlow>
    </div>

    <!-- Dialogs and Drawers -->
    <DdlImportDialog ref="ddlDialogRef" @imported="handleImported" />
    <DbConnectDialog ref="dbDialogRef" @imported="handleImported" />
    <AutoMappingDialog ref="autoMappingDialogRef" />
    <ProjectDialog ref="projectDialogRef" />
    <BackupDialog ref="backupDialogRef" />
    <MappingDrawer ref="mappingDrawerRef" />
    <RelationPathDrawer ref="relationPathDrawerRef" />

    <!-- Modern Command Palette (Ctrl+K) -->
    <CommandPalette
      ref="commandPaletteRef"
      @open-ddl-dialog="openDdlDialog"
      @open-db-dialog="openDbDialog"
      @open-project-dialog="openProjectDialog"
      @open-backup-dialog="openBackupDialog"
      @open-mapping-drawer="openMappingDrawer"
      @open-auto-mapping="openAutoMappingDialog"
      @open-relation-path="openRelationPathDrawer"
      @fit-view="handleFitView"
      @auto-layout="handleAutoLayout"
      @export-excel="handleExportExcel"
      @export-image="handleExportImage"
      @focus-table="handleFocusTable"
      @open-settings="$emit('open-settings')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, markRaw, nextTick } from 'vue'
import { VueFlow, useVueFlow, ConnectionMode } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import { exportCanvasAsImage } from '@/utils/exportImage'
import { toast, confirmDialog } from '@/utils/toast'
import { Eye, EyeOff, Info, FileCode, Database, X } from 'lucide-vue-next'
import { Button } from '@/components/ui'

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
import CommandPalette from '@/components/common/CommandPalette.vue'
import { api } from '@/api'
import SmartObstacleEdge from '@/components/canvas/SmartObstacleEdge.vue'

import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { useThemeStore } from '@/stores/themeStore'

const { fitView, updateNodeInternals, setCenter } = useVueFlow()

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
const showFloatingTip = ref(localStorage.getItem('collink_show_floating_tip') !== 'false')

const closeFloatingTip = () => {
  showFloatingTip.value = false
  localStorage.setItem('collink_show_floating_tip', 'false')
}
const ddlDialogRef = ref<InstanceType<typeof DdlImportDialog> | null>(null)
const dbDialogRef = ref<InstanceType<typeof DbConnectDialog> | null>(null)
const autoMappingDialogRef = ref<InstanceType<typeof AutoMappingDialog> | null>(null)
const projectDialogRef = ref<InstanceType<typeof ProjectDialog> | null>(null)
const backupDialogRef = ref<InstanceType<typeof BackupDialog> | null>(null)
const mappingDrawerRef = ref<InstanceType<typeof MappingDrawer> | null>(null)
const relationPathDrawerRef = ref<InstanceType<typeof RelationPathDrawer> | null>(null)
const commandPaletteRef = ref<InstanceType<typeof CommandPalette> | null>(null)

// Register custom node & edge components
const nodeTypes = {
  tableCard: markRaw(TableNode),
}

const edgeTypes = {
  smartObstacle: markRaw(SmartObstacleEdge),
  smoothstep: markRaw(SmartObstacleEdge),
}

const emit = defineEmits(['open-settings'])

onMounted(async () => {
  window.addEventListener('collink:open-relation-path', (e: any) => {
    openRelationPathDrawer(e.detail?.sourceTableId)
  })
  window.addEventListener('collink:open-backup-dialog', () => {
    openBackupDialog()
  })
  await projectStore.fetchProjects()
  if (projectStore.currentProject) {
    await canvasStore.fetchCanvases(projectStore.currentProject.id)
    setTimeout(() => {
      handleFitView()
    }, 200)
  }
})


const handleAutoLayout = async () => {
  await canvasStore.applyAutoLayout('LR')
  setTimeout(handleFitView, 100)
}

const handleExportExcel = async () => {
  if (!projectStore.currentProject) {
    toast.warning('当前无可用项目')
    return
  }
  await api.exportExcel(projectStore.currentProject.id, projectStore.currentProject.name)
}

const handleFitView = () => {
  fitView({ padding: 0.2, duration: 400 })
}

// MiniMap Node dynamic colors based on table colorTag & theme
const getMiniMapNodeColor = (node: any) => {
  const isDark = themeStore.isDark
  const colorTag = node.data?.colorTag || 'blue'
  const colorMap: Record<string, { light: string; dark: string }> = {
    blue: { light: '#93c5fd', dark: '#1e40af' },
    emerald: { light: '#6ee7b7', dark: '#065f46' },
    green: { light: '#86efac', dark: '#166534' },
    amber: { light: '#fcd34d', dark: '#92400e' },
    rose: { light: '#fda4af', dark: '#9f1239' },
    red: { light: '#fca5a5', dark: '#991b1c' },
    purple: { light: '#d8b4fe', dark: '#6b21a8' },
    indigo: { light: '#a5b4fc', dark: '#3730a3' },
    cyan: { light: '#67e8f9', dark: '#155e75' },
    orange: { light: '#fdba74', dark: '#9a3412' },
  }
  if (colorMap[colorTag]) {
    return isDark ? colorMap[colorTag].dark : colorMap[colorTag].light
  }
  return isDark ? '#1e40af' : '#bfdbfe'
}

const getMiniMapNodeStrokeColor = (node: any) => {
  const isDark = themeStore.isDark
  const colorTag = node.data?.colorTag || 'blue'
  const strokeMap: Record<string, { light: string; dark: string }> = {
    blue: { light: '#3b82f6', dark: '#60a5fa' },
    emerald: { light: '#10b981', dark: '#34d399' },
    green: { light: '#22c55e', dark: '#4ade80' },
    amber: { light: '#f59e0b', dark: '#fbbf24' },
    rose: { light: '#f43f5e', dark: '#fb7185' },
    red: { light: '#ef4444', dark: '#f87171' },
    purple: { light: '#a855f7', dark: '#c084fc' },
    indigo: { light: '#6366f1', dark: '#818cf8' },
    cyan: { light: '#06b6d4', dark: '#22d3ee' },
    orange: { light: '#f97316', dark: '#fb923c' },
  }
  if (strokeMap[colorTag]) {
    return isDark ? strokeMap[colorTag].dark : strokeMap[colorTag].light
  }
  return isDark ? '#60a5fa' : '#3b82f6'
}

// Click node on minimap to smoothly fly and focus
const handleMiniMapNodeClick = ({ node }: { node: any }) => {
  if (node) {
    const x = node.computedPosition?.x ?? node.position?.x ?? 0
    const y = node.computedPosition?.y ?? node.position?.y ?? 0
    const width = node.dimensions?.width || 320
    const height = node.dimensions?.height || 200
    setCenter(x + width / 2, y + height / 2, { duration: 500 })
    if (node.data?.tableId) {
      canvasStore.focusTable(node.data.tableId)
    }
  }
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

const handleEdgeClick = async (event: any) => {
  const edge = event.edge
  const label = edge.data?.label || '选中的字段映射'

  const confirmed = await confirmDialog({
    title: '字段映射操作',
    message: `当前选中的血缘关系：\n${label}\n\n是否删除此字段映射连线？`,
    confirmText: '删除映射',
    variant: 'danger',
  })

  if (confirmed) {
    await canvasStore.removeEdge(edge.id, projectStore.currentProject?.id)
  }
}

// Project selection validator
const checkProjectSelected = (): boolean => {
  if (!projectStore.currentProject?.id) {
    toast.error('请先选择或新建一个项目！')
    return false
  }
  return true
}

const handleImported = () => {
  setTimeout(() => {
    handleFitView()
  }, 200)
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
    toast.warning('画布中暂无数据表，无法导出')
    return
  }

  toast.info('正在生成全画布 2x 超清架构图，请稍候...')

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
      backgroundColor: themeStore.isDark ? '#1e1e1e' : '#fafafa',
    })

    toast.success('全画布 2x 超清架构图导出成功')
  } catch (err: any) {
    toast.error('导出图片失败: ' + (err.message || '未知错误'))
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
  height: calc(100vh - 40px - 32px);
  position: relative;
}

:deep(.vue-flow__viewport) {
  transform-style: flat;
}

:deep(.vue-flow__node) {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: geometricPrecision;
  transform: translateZ(0);
}

:deep(.vue-flow__node *) {
  text-rendering: geometricPrecision;
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
  background: var(--cl-surface);
  border: 1px solid var(--cl-border);
  border-radius: 20px;
  padding: 5px 8px 5px 12px;
  box-shadow: var(--cl-shadow-card);
  user-select: none;
  pointer-events: auto;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
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

.tip-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 9999px;
  border: none;
  background: transparent;
  color: var(--cl-text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
  margin-left: 2px;
}

.tip-close-btn:hover {
  background: var(--cl-surface-hover);
  color: var(--cl-text-primary);
}

.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(6px);
}

/* Vue Flow Controls Aesthetic Upgrade */
:deep(.vue-flow__controls) {
  bottom: 16px !important;
  right: 16px !important;
  margin: 0 !important;
  padding: 0 !important;
  background: var(--cl-surface) !important;
  border: 1px solid var(--cl-border) !important;
  border-radius: 6px !important;
  box-shadow: var(--cl-shadow-popup) !important;
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

:deep(.vue-flow__controls-button svg) {
  fill: currentColor !important;
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
  background: var(--cl-surface);
  padding: 40px 48px;
  border-radius: 12px;
  border: 1px solid var(--cl-border);
  box-shadow: var(--cl-shadow-popup);
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
  width: 210px !important;
  height: 135px !important;
  background: var(--cl-surface) !important;
  border: 1px solid var(--cl-border) !important;
  border-radius: 8px !important;
  box-shadow: var(--cl-shadow-popup) !important;
  overflow: hidden !important;
  z-index: 9 !important;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

:deep(.canvas-minimap:hover) {
  border-color: var(--cl-border-active) !important;
}

:deep(.canvas-minimap .vue-flow__minimap-mask) {
  cursor: grab;
}

:deep(.canvas-minimap.dragging .vue-flow__minimap-mask),
:deep(.canvas-minimap .vue-flow__minimap-mask:active) {
  cursor: grabbing;
}

:deep(.canvas-minimap .vue-flow__minimap-node) {
  cursor: pointer;
  transition: opacity 0.15s ease;
}

:deep(.canvas-minimap .vue-flow__minimap-node:hover) {
  opacity: 0.8;
}

/* MiniMap Toggle Button */
.minimap-toggle-btn {
  position: absolute;
  bottom: 159px;
  right: 60px;
  z-index: 10;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 10px;
  background: var(--cl-surface);
  border: 1px solid var(--cl-border);
  border-radius: 4px;
  box-shadow: var(--cl-shadow-sm);
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
  box-shadow: var(--cl-shadow-popup);
}

.minimap-toggle-btn:hover {
  background: var(--cl-surface-hover);
  color: var(--cl-primary);
  border-color: var(--cl-border-active);
  box-shadow: var(--cl-shadow-card);
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
