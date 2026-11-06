<template>
  <el-drawer
    v-model="visible"
    size="720px"
    destroy-on-close
    class="relation-path-drawer"
    @close="handleDrawerClose"
  >
    <template #header>
      <div class="drawer-custom-header">
        <div class="header-title-wrap">
          <div class="header-icon-wrap">
            <el-icon color="#2563eb" :size="20"><Guide /></el-icon>
          </div>
          <div>
            <div class="header-title">关联路径探查</div>
            <div class="header-subtitle">基于已配置的字段血缘连线，自动计算跨表多跳关联路径并生成可执行 SQL JOIN</div>
          </div>
        </div>
      </div>
    </template>

    <div class="drawer-body">
      <!-- Table Selection Form Card -->
      <div class="config-card">
        <!-- Canvas Picking Banner -->
        <transition name="fade">
          <div v-if="canvasStore.pickingTableRole" class="picking-alert-banner">
            <el-icon class="picking-icon is-spinning"><Aim /></el-icon>
            <span>
              正在画布点选 <strong>{{ canvasStore.pickingTableRole === 'source' ? '起点表 (Source)' : '终点表 (Target)' }}</strong>，请直接在画布上点击任意表卡片
            </span>
            <el-button size="small" plain @click="cancelCanvasPick">取消点选</el-button>
          </div>
        </transition>

        <div class="selectors-grid">
          <!-- Source Table -->
          <div class="selector-item">
            <div class="selector-label">
              <span class="label-dot dot-source"></span>
              <span>起点表 (Source)</span>
            </div>
            <div class="selector-row">
              <el-select
                v-model="sourceTableId"
                placeholder="请选择起点表..."
                filterable
                clearable
                class="table-select"
                size="default"
              >
                <el-option
                  v-for="tbl in projectTables"
                  :key="tbl.id"
                  :label="`${tbl.table_name}${tbl.table_comment ? ' (' + tbl.table_comment + ')' : ''}`"
                  :value="tbl.id"
                />
              </el-select>
              <el-tooltip content="在画布中点击选择" placement="top">
                <el-button
                  :type="canvasStore.pickingTableRole === 'source' ? 'primary' : 'default'"
                  class="btn-pick"
                  @click="startCanvasPick('source')"
                >
                  <el-icon><Aim /></el-icon>
                </el-button>
              </el-tooltip>
            </div>
          </div>

          <!-- Swap Button in Middle -->
          <div class="swap-action-wrap">
            <el-tooltip content="交换起止表" placement="top">
              <el-button circle class="btn-swap" @click="handleSwapTables">
                <el-icon><Switch /></el-icon>
              </el-button>
            </el-tooltip>
          </div>

          <!-- Target Table -->
          <div class="selector-item">
            <div class="selector-label">
              <span class="label-dot dot-target"></span>
              <span>终点表 (Target)</span>
            </div>
            <div class="selector-row">
              <el-select
                v-model="targetTableId"
                placeholder="请选择终点表..."
                filterable
                clearable
                class="table-select"
                size="default"
              >
                <el-option
                  v-for="tbl in projectTables"
                  :key="tbl.id"
                  :label="`${tbl.table_name}${tbl.table_comment ? ' (' + tbl.table_comment + ')' : ''}`"
                  :value="tbl.id"
                />
              </el-select>
              <el-tooltip content="在画布中点击选择" placement="top">
                <el-button
                  :type="canvasStore.pickingTableRole === 'target' ? 'primary' : 'default'"
                  class="btn-pick"
                  @click="startCanvasPick('target')"
                >
                  <el-icon><Aim /></el-icon>
                </el-button>
              </el-tooltip>
            </div>
          </div>
        </div>

        <!-- Controls Row -->
        <div class="controls-row">
          <div class="hops-config">
            <span class="hops-label">最大允许跳数:</span>
            <el-radio-group v-model="maxHops" size="small">
              <el-radio-button :label="2">2 跳</el-radio-button>
              <el-radio-button :label="3">3 跳</el-radio-button>
              <el-radio-button :label="4">4 跳 (推荐)</el-radio-button>
              <el-radio-button :label="5">5 跳</el-radio-button>
            </el-radio-group>
          </div>

          <el-button
            type="primary"
            :loading="searching"
            :disabled="!sourceTableId || !targetTableId || sourceTableId === targetTableId"
            class="btn-start-search"
            @click="handleDiscoverPaths"
          >
            <el-icon><Search /></el-icon>
            <span>计算关联路径</span>
          </el-button>
        </div>
      </div>

      <!-- Result Section -->
      <div class="result-container">
        <!-- Initial Blank State -->
        <div v-if="!hasSearched" class="initial-tip-state">
          <div class="tip-icon-bg">
            <el-icon :size="40" color="#94a3b8"><Guide /></el-icon>
          </div>
          <h4>选择两张表开始探查</h4>
          <p>系统将基于画布上已建立的字段血缘映射连线，深度搜索所有可连通的多跳路径并输出可执行 SQL JOIN。</p>
        </div>

        <!-- No Path Found State -->
        <div v-else-if="pathResults.length === 0" class="no-path-state">
          <el-empty description="在当前已有血缘关系中未发现连通路径">
            <template #extra>
              <span class="empty-hint">两表之间暂无直接或多跳中继的字段映射血缘，您可先在画布上建立连线或调大跳数。</span>
            </template>
          </el-empty>
        </div>

        <!-- Path List -->
        <div v-else class="path-list-wrap">
          <div class="list-summary-bar">
            <div class="summary-text">
              共探查到 <strong>{{ pathResults.length }}</strong> 条关联路径 (已按跳数与置信度智能排序)
            </div>
            <el-button
              v-if="canvasStore.activeRelationPath"
              size="small"
              plain
              type="info"
              @click="canvasStore.clearRelationPathHighlight()"
            >
              清除画布高亮
            </el-button>
          </div>

          <div
            v-for="(pathItem, pIdx) in pathResults"
            :key="pathItem.path_id"
            class="path-card"
            :class="{ 'is-active-highlight': canvasStore.activeRelationPath?.path_id === pathItem.path_id }"
          >
            <!-- Card Header -->
            <div class="path-card-header">
              <div class="path-header-meta">
                <span class="path-index-badge">方案 {{ pIdx + 1 }}</span>
                <el-tag
                  :type="pathItem.hops === 1 ? 'success' : (pathItem.hops === 2 ? 'primary' : 'warning')"
                  size="small"
                  effect="light"
                  class="hop-tag"
                >
                  {{ pathItem.hops }} 跳血缘关联
                </el-tag>
                <el-tag
                  type="success"
                  size="small"
                  effect="dark"
                  class="confidence-tag"
                >
                  已有血缘连线
                </el-tag>
              </div>

              <div class="path-header-actions">
                <el-button
                  size="small"
                  :type="canvasStore.activeRelationPath?.path_id === pathItem.path_id ? 'primary' : 'default'"
                  class="btn-highlight-action"
                  @click="toggleHighlightPath(pathItem)"
                >
                  <el-icon><View /></el-icon>
                  <span>{{ canvasStore.activeRelationPath?.path_id === pathItem.path_id ? '已在画布高亮' : '在画布高亮' }}</span>
                </el-button>
              </div>
            </div>

            <!-- Path Breadcrumbs Title -->
            <div class="path-title-flow">
              <span
                v-for="(tName, idx) in pathItem.summary_title.split(' -> ')"
                :key="idx"
                class="flow-step-item"
              >
                <span class="step-tbl-tag">{{ tName }}</span>
                <el-icon v-if="idx < pathItem.summary_title.split(' -> ').length - 1" class="flow-arrow"><Right /></el-icon>
              </span>
            </div>

            <!-- Missing Intermediate Tables Prompt -->
            <div
              v-if="getMissingIntermediateTables(pathItem).length > 0"
              class="missing-tables-bar"
            >
              <div class="missing-text">
                <el-icon color="#eab308"><InfoFilled /></el-icon>
                <span>中间表 <strong>{{ getMissingIntermediateTables(pathItem).map(t => t.table_name).join(', ') }}</strong> 未在当前视图中</span>
              </div>
              <el-button
                size="small"
                type="warning"
                plain
                @click="addIntermediateToCanvas(pathItem)"
              >
                一键引入到当前画布
              </el-button>
            </div>

            <!-- Steps Breakdown Timeline -->
            <div class="steps-breakdown">
              <div
                v-for="(step, sIdx) in pathItem.steps"
                :key="sIdx"
                class="step-row"
              >
                <div class="step-num">{{ sIdx + 1 }}</div>
                <div class="step-content">
                  <div class="step-nodes-link">
                    <div class="endpoint from-node">
                      <span class="endpoint-tbl">{{ step.from_table.table_name }}</span>
                      <span class="endpoint-col">{{ step.from_column.column_name }}</span>
                    </div>
                    <div class="link-symbol">
                      <span class="symbol-line"></span>
                      <span class="symbol-badge">=</span>
                      <span class="symbol-line"></span>
                    </div>
                    <div class="endpoint to-node">
                      <span class="endpoint-tbl">{{ step.to_table.table_name }}</span>
                      <span class="endpoint-col">{{ step.to_column.column_name }}</span>
                    </div>
                  </div>

                  <div class="step-footer-meta">
                    <span class="relation-badge badge-explicit">已建立血缘连线</span>
                    <span class="match-reason-text">{{ step.match_reason }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Generated SQL JOIN Code Block -->
            <div class="sql-preview-box">
              <div class="sql-box-header">
                <span class="sql-title">
                  <el-icon><Connection /></el-icon>
                  <span>多表关联 SQL (JOIN 语句)</span>
                </span>
                <el-button
                  size="small"
                  class="btn-copy-sql"
                  @click="copySql(pathItem.sql_snippet)"
                >
                  <el-icon><CopyDocument /></el-icon>
                  <span>复制 SQL</span>
                </el-button>
              </div>
              <pre class="sql-code"><code>{{ pathItem.sql_snippet }}</code></pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  </el-drawer>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { api } from '@/api'
import type { RelationPath, TableMeta } from '@/types'
import { ElMessage } from 'element-plus'
import {
  Guide,
  Aim,
  Search,
  Switch,
  Right,
  InfoFilled,
  View,
  Connection,
  CopyDocument,
} from '@element-plus/icons-vue'

const visible = ref(false)
const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const sourceTableId = ref('')
const targetTableId = ref('')
const maxHops = ref(4)

const searching = ref(false)
const hasSearched = ref(false)
const pathResults = ref<RelationPath[]>([])

const projectTables = computed(() => {
  return projectStore.currentProject?.tables || []
})

// Open Drawer (can optionally pass initial source or target table ID)
const open = (srcId?: string, tgtId?: string) => {
  if (srcId) sourceTableId.value = srcId
  if (tgtId) targetTableId.value = tgtId
  visible.value = true
  if (sourceTableId.value && targetTableId.value && sourceTableId.value !== targetTableId.value) {
    handleDiscoverPaths()
  }
}

// Canvas Pick mode
const startCanvasPick = (role: 'source' | 'target') => {
  canvasStore.startPickingTable(role, (pickedRole, tblId, _tblName) => {
    if (pickedRole === 'source') {
      sourceTableId.value = tblId
    } else {
      targetTableId.value = tblId
    }
    ElMessage.success(`已选定${pickedRole === 'source' ? '起点表' : '终点表'}: ${_tblName}`)
  })
}

const cancelCanvasPick = () => {
  canvasStore.cancelPickingTable()
}

// Swap source and target
const handleSwapTables = () => {
  const temp = sourceTableId.value
  sourceTableId.value = targetTableId.value
  targetTableId.value = temp
  if (hasSearched.value && sourceTableId.value && targetTableId.value) {
    handleDiscoverPaths()
  }
}

// Discover Paths API call
const handleDiscoverPaths = async () => {
  if (!projectStore.currentProject) return
  if (!sourceTableId.value || !targetTableId.value) {
    ElMessage.warning('请选择起点表与终点表')
    return
  }
  if (sourceTableId.value === targetTableId.value) {
    ElMessage.warning('起点表与终点表不能相同')
    return
  }

  searching.value = true
  try {
    const res = await api.discoverRelationPath(projectStore.currentProject.id, {
      source_table_id: sourceTableId.value,
      target_table_id: targetTableId.value,
      max_hops: maxHops.value,
    })

    pathResults.value = res.paths || []
    hasSearched.value = true

    if (pathResults.value.length > 0) {
      // Auto-highlight the first (best) path on canvas
      canvasStore.highlightRelationPath(pathResults.value[0])
    } else {
      canvasStore.clearRelationPathHighlight()
    }
  } catch (e: any) {
    ElMessage.error('计算关联路径失败: ' + (e.message || '网络错误'))
  } finally {
    searching.value = false
  }
}

// Toggle path highlight on canvas
const toggleHighlightPath = (path: RelationPath) => {
  if (canvasStore.activeRelationPath?.path_id === path.path_id) {
    canvasStore.clearRelationPathHighlight()
  } else {
    canvasStore.highlightRelationPath(path)
  }
}

// Get missing intermediate tables for a path
const getMissingIntermediateTables = (path: RelationPath): TableMeta[] => {
  const canvasNodeTableIds = new Set(canvasStore.nodes.map(n => n.data.tableId))
  const allTables = projectStore.currentProject?.tables || []
  return path.intermediate_table_ids
    .filter(tId => !canvasNodeTableIds.has(tId))
    .map(tId => allTables.find(t => t.id === tId))
    .filter((t): t is TableMeta => !!t)
}

// Add intermediate tables to active canvas
const addIntermediateToCanvas = async (path: RelationPath) => {
  const missing = getMissingIntermediateTables(path)
  if (missing.length === 0) return

  const tableIds = missing.map(m => m.id)
  await canvasStore.addTablesToCurrentCanvas(tableIds)
  // Re-highlight
  canvasStore.highlightRelationPath(path)
}

// Copy SQL to clipboard
const copySql = async (sql: string) => {
  try {
    await navigator.clipboard.writeText(sql)
    ElMessage.success('SQL JOIN 语句已复制到剪贴板')
  } catch {
    // Fallback
    const textarea = document.createElement('textarea')
    textarea.value = sql
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
    ElMessage.success('SQL JOIN 语句已复制到剪贴板')
  }
}

const handleDrawerClose = () => {
  canvasStore.cancelPickingTable()
}

defineExpose({
  open,
  setPickedTable: (role: 'source' | 'target', tableId: string) => {
    if (role === 'source') sourceTableId.value = tableId
    else targetTableId.value = tableId
  },
})
</script>

<style scoped>
.relation-path-drawer :deep(.el-drawer__header) {
  margin-bottom: 0;
  padding: 16px 20px;
  border-bottom: 1px solid var(--cl-border);
}

.drawer-custom-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-title-wrap {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon-wrap {
  width: 38px;
  height: 38px;
  border-radius: 8px;
  background: var(--cl-primary-bg, #eff6ff);
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--cl-primary-border, #bfdbfe);
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--cl-text);
}

.header-subtitle {
  font-size: 12px;
  color: var(--cl-text-secondary);
  margin-top: 2px;
}

.drawer-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* Config Card */
.config-card {
  background: var(--cl-surface, #ffffff);
  border: 1px solid var(--cl-border, #e2e8f0);
  border-radius: 10px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.picking-alert-banner {
  background: #eff6ff;
  border: 1px dashed #3b82f6;
  border-radius: 6px;
  padding: 8px 12px;
  margin-bottom: 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: #1d4ed8;
}

.picking-alert-banner .is-spinning {
  animation: spin 3s linear infinite;
}

@keyframes spin {
  100% {
    transform: rotate(360deg);
  }
}

.selectors-grid {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: flex-end;
  gap: 10px;
}

.selector-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.selector-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--cl-text);
}

.label-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.dot-source {
  background-color: #2563eb;
}

.dot-target {
  background-color: #10b981;
}

.selector-row {
  display: flex;
  gap: 6px;
}

.table-select {
  flex: 1;
}

.btn-pick {
  padding: 8px 12px;
}

.swap-action-wrap {
  padding-bottom: 2px;
}

.btn-swap {
  color: #64748b;
  border-color: #cbd5e1;
}

.btn-swap:hover {
  color: #2563eb;
  border-color: #2563eb;
  background-color: #eff6ff;
}

.controls-row {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid var(--cl-border-light, #f1f5f9);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.hops-config {
  display: flex;
  align-items: center;
  gap: 10px;
}

.hops-label {
  font-size: 12px;
  color: var(--cl-text-secondary);
}

.btn-start-search {
  padding: 8px 20px;
  font-weight: 500;
}

/* Results Section */
.result-container {
  min-height: 250px;
}

.initial-tip-state {
  text-align: center;
  padding: 60px 20px;
  color: #64748b;
}

.tip-icon-bg {
  width: 64px;
  height: 64px;
  margin: 0 auto 16px;
  background: #f1f5f9;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.initial-tip-state h4 {
  font-size: 15px;
  font-weight: 600;
  color: #334155;
  margin-bottom: 8px;
}

.initial-tip-state p {
  font-size: 13px;
  max-width: 420px;
  margin: 0 auto;
  line-height: 1.6;
}

.no-path-state {
  padding: 40px 0;
}

.empty-hint {
  font-size: 12px;
  color: #94a3b8;
}

.list-summary-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.summary-text {
  font-size: 13px;
  color: var(--cl-text-secondary);
}

.summary-text strong {
  color: var(--cl-primary, #2563eb);
}

.path-list-wrap {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Path Card */
.path-card {
  background: var(--cl-surface);
  border: 1px solid var(--cl-border, #e2e8f0);
  border-radius: 10px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  transition: all 0.2s ease;
}

.path-card.is-active-highlight {
  border-color: #2563eb;
  box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.2), 0 4px 12px rgba(37, 99, 235, 0.08);
}

.path-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.path-header-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.path-index-badge {
  font-size: 12px;
  font-weight: 600;
  color: #475569;
  background: #f1f5f9;
  padding: 2px 8px;
  border-radius: 4px;
}

.path-title-flow {
  background: #f8fafc;
  border-radius: 6px;
  padding: 10px 14px;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  font-family: var(--cl-font-mono, monospace);
  font-size: 13px;
  margin-bottom: 14px;
}

.flow-step-item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.step-tbl-tag {
  background: var(--cl-surface);
  border: 1px solid var(--cl-border);
  padding: 3px 8px;
  border-radius: 4px;
  color: var(--cl-text-primary);
  font-weight: 600;
}

.flow-arrow {
  color: #94a3b8;
  font-size: 14px;
}

.missing-tables-bar {
  background: #fefce8;
  border: 1px solid #fef08a;
  border-radius: 6px;
  padding: 8px 12px;
  margin-bottom: 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.missing-text {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #854d0e;
}

/* Steps Breakdown */
.steps-breakdown {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 14px;
}

.step-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  background: #fcfdfe;
  border: 1px solid #f1f5f9;
  border-radius: 6px;
  padding: 10px 12px;
}

.step-num {
  width: 22px;
  height: 22px;
  background: #e2e8f0;
  color: #475569;
  font-size: 12px;
  font-weight: 600;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 2px;
}

.step-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.step-nodes-link {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
}

.endpoint {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.endpoint-tbl {
  font-size: 12px;
  color: #64748b;
}

.endpoint-col {
  font-family: var(--cl-font-mono, monospace);
  font-weight: 600;
  color: #0f172a;
}

.link-symbol {
  display: flex;
  align-items: center;
  gap: 4px;
}

.symbol-line {
  width: 12px;
  height: 1px;
  background-color: #94a3b8;
}

.symbol-badge {
  font-size: 11px;
  font-weight: 700;
  color: #2563eb;
  background: #eff6ff;
  padding: 0 4px;
  border-radius: 3px;
}

.step-footer-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}

.relation-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 3px;
  font-weight: 500;
}

.badge-explicit {
  background: #dcfce7;
  color: #15803d;
}

.badge-inferred {
  background: #fef3c7;
  color: #b45309;
}

.match-reason-text {
  color: #64748b;
  font-size: 12px;
}

/* SQL Preview Box */
.sql-preview-box {
  background: #0f172a;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.4);
}

.sql-box-header {
  background: #1e293b;
  padding: 8px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #334155;
}

.sql-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
  color: #cbd5e1;
}

.btn-copy-sql {
  background: #334155;
  color: #e2e8f0;
  border-color: #475569;
  font-size: 11px;
  padding: 4px 8px;
}

.btn-copy-sql:hover {
  background: #475569;
  color: #ffffff;
}

.sql-code {
  margin: 0;
  padding: 12px 14px;
  color: #38bdf8;
  font-family: 'JetBrains Mono', 'Fira Code', Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  overflow-x: auto;
  white-space: pre;
}
</style>
