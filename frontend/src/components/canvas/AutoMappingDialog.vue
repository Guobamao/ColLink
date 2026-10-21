<template>
  <el-dialog
    v-model="visible"
    title="智能同名/同义字段自动连线向导"
    width="860px"
    destroy-on-close
    :close-on-click-modal="false"
    class="auto-mapping-dialog"
  >
    <template #header>
      <div class="dialog-header-title">
        <el-icon class="header-icon" color="#3B82F6"><MagicStick /></el-icon>
        <span>智能同名/同义字段自动连线向导</span>
      </div>
    </template>

    <div class="dialog-body">
      <!-- Strategy Settings -->
      <div class="config-panel">
        <div class="tables-selector-row">
          <div class="selector-item">
            <span class="label">源数据表 (Source Table)</span>
            <el-select
              v-model="sourceTableId"
              placeholder="选择源数据表"
              filterable
              style="width: 100%"
              @change="onTableSelectionChange"
            >
              <el-option
                v-for="table in availableTables"
                :key="table.id"
                :label="`${table.table_name}${table.table_comment ? ` (${table.table_comment})` : ''}`"
                :value="table.id"
                :disabled="table.id === targetTableId"
              />
            </el-select>
          </div>

          <div class="arrow-indicator">
            <el-icon><Right /></el-icon>
          </div>

          <div class="selector-item">
            <span class="label">目标数据表 (Target Table)</span>
            <el-select
              v-model="targetTableId"
              placeholder="选择目标数据表"
              filterable
              style="width: 100%"
              @change="onTableSelectionChange"
            >
              <el-option
                v-for="table in availableTables"
                :key="table.id"
                :label="`${table.table_name}${table.table_comment ? ` (${table.table_comment})` : ''}`"
                :value="table.id"
                :disabled="table.id === sourceTableId"
              />
            </el-select>
          </div>
        </div>

        <div class="strategies-row">
          <span class="strategy-label">匹配策略：</span>
          <el-checkbox v-model="matchExactName">
            同名字段精确匹配 <span class="strategy-hint">(忽略大小写及下划线，如 WORK_PLAN_ID 与 work_plan_id)</span>
          </el-checkbox>
          <el-checkbox v-model="matchCommentSynonym">
            中文注释同义匹配 <span class="strategy-hint">(忽略“主键/ID/编号”后缀或注释完全一致)</span>
          </el-checkbox>
          <el-checkbox v-model="skipExisting">
            排除已连线字段
          </el-checkbox>
        </div>

        <div class="action-btn-row">
          <el-button
            type="primary"
            :icon="Search"
            :disabled="!sourceTableId || !targetTableId"
            @click="runMatch"
          >
            开始智能匹配
          </el-button>
        </div>
      </div>

      <!-- Match Results Preview -->
      <div v-if="hasRunMatch" class="results-panel">
        <div class="results-header">
          <div class="results-title">
            匹配候选结果 (共发现 <strong>{{ candidates.length }}</strong> 对可用映射)
          </div>
          <div class="results-actions">
            <el-button size="small" @click="toggleSelectAll(true)">全选</el-button>
            <el-button size="small" @click="toggleSelectAll(false)">全不选</el-button>
          </div>
        </div>

        <div v-if="candidates.length === 0" class="empty-candidate">
          <el-empty description="未匹配到符合规则的候选字段对，请尝试更换匹配策略或选择其他数据表" :image-size="64" />
        </div>

        <el-table
          v-else
          ref="tableRef"
          :data="candidates"
          size="small"
          border
          max-height="300"
          row-key="id"
          @selection-change="handleSelectionChange"
        >
          <el-table-column type="selection" width="45" align="center" />

          <el-table-column label="源字段 (Source)" min-width="190">
            <template #default="{ row }">
              <div class="col-preview">
                <span class="col-name">{{ row.sourceCol.column_name }}</span>
                <span v-if="row.sourceCol.column_comment" class="col-comment">
                  {{ row.sourceCol.column_comment }}
                </span>
                <span class="col-type">{{ row.sourceCol.data_type }}</span>
              </div>
            </template>
          </el-table-column>

          <el-table-column width="40" align="center">
            <template #default>
              <el-icon color="#94A3B8"><Right /></el-icon>
            </template>
          </el-table-column>

          <el-table-column label="目标字段 (Target)" min-width="190">
            <template #default="{ row }">
              <div class="col-preview">
                <span class="col-name">{{ row.targetCol.column_name }}</span>
                <span v-if="row.targetCol.column_comment" class="col-comment">
                  {{ row.targetCol.column_comment }}
                </span>
                <span class="col-type">{{ row.targetCol.data_type }}</span>
              </div>
            </template>
          </el-table-column>

          <el-table-column label="匹配依据" width="160" align="center">
            <template #default="{ row }">
              <el-tag
                :type="row.matchType === 'name' ? 'success' : 'warning'"
                size="small"
                effect="light"
              >
                {{ row.reason }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <div class="footer-left">
          <span v-if="hasRunMatch && candidates.length > 0">
            已勾选 <strong style="color: #3b82f6;">{{ selectedCandidates.length }}</strong> / {{ candidates.length }} 对映射
          </span>
        </div>
        <div class="footer-right">
          <el-button @click="visible = false">取消</el-button>
          <el-button
            type="primary"
            :disabled="selectedCandidates.length === 0"
            :loading="submitting"
            @click="handleSubmit"
          >
            确认创建所选映射 ({{ selectedCandidates.length }})
          </el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { MagicStick, Right, Search } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import type { TableMeta, ColumnMeta } from '@/types'

const visible = ref(false)
const sourceTableId = ref('')
const targetTableId = ref('')
const matchExactName = ref(true)
const matchCommentSynonym = ref(true)
const skipExisting = ref(true)
const submitting = ref(false)
const hasRunMatch = ref(false)

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

interface MatchCandidate {
  id: string
  sourceTableId: string
  sourceCol: ColumnMeta
  targetTableId: string
  targetCol: ColumnMeta
  matchType: 'name' | 'comment'
  reason: string
  confidence: number
}

const candidates = ref<MatchCandidate[]>([])
const selectedCandidates = ref<MatchCandidate[]>([])
const tableRef = ref<any>(null)

// Available tables from current canvas/project
const availableTables = computed<TableMeta[]>(() => {
  return projectStore.currentProject?.tables || []
})

const open = (preSelectSourceId?: string, preSelectTargetId?: string) => {
  visible.value = true
  hasRunMatch.value = false
  candidates.value = []
  selectedCandidates.value = []

  if (preSelectSourceId) {
    sourceTableId.value = preSelectSourceId
  } else if (!sourceTableId.value && availableTables.value.length > 0) {
    sourceTableId.value = availableTables.value[0].id
  }

  if (preSelectTargetId) {
    targetTableId.value = preSelectTargetId
  } else if (!targetTableId.value && availableTables.value.length > 1) {
    targetTableId.value = availableTables.value[1].id
  }
}

const onTableSelectionChange = () => {
  hasRunMatch.value = false
  candidates.value = []
  selectedCandidates.value = []
}

// Name normalization: ignore case, underscore, hyphen
const normalizeName = (name: string): string => {
  return (name || '').toLowerCase().replace(/[-_]/g, '')
}

// Clean comment for synonym comparison
const cleanComment = (comm: string): string => {
  if (!comm) return ''
  return comm
    .trim()
    .toLowerCase()
    .replace(/(主键|id|编号|标识|代码|序号)$/gi, '')
    .trim()
}

// Check if edge already exists on canvas
const isAlreadyMapped = (srcColId: string, tgtColId: string): boolean => {
  return canvasStore.edges.some(
    e =>
      (e.data?.sourceColumnId === srcColId && e.data?.targetColumnId === tgtColId) ||
      (e.data?.sourceColumnId === tgtColId && e.data?.targetColumnId === srcColId)
  )
}

const runMatch = () => {
  if (!sourceTableId.value || !targetTableId.value) {
    ElMessage.warning('请选择源数据表与目标数据表')
    return
  }

  const srcTable = availableTables.value.find(t => t.id === sourceTableId.value)
  const tgtTable = availableTables.value.find(t => t.id === targetTableId.value)

  if (!srcTable || !tgtTable) {
    ElMessage.warning('所选表不存在')
    return
  }

  const srcCols = srcTable.columns || []
  const tgtCols = tgtTable.columns || []
  const results: MatchCandidate[] = []

  for (const sCol of srcCols) {
    for (const tCol of tgtCols) {
      if (skipExisting.value && isAlreadyMapped(sCol.id, tCol.id)) {
        continue
      }

      let matched = false
      let matchType: 'name' | 'comment' = 'name'
      let reason = ''
      let confidence = 0

      // Rule 1: Exact Name (case & separator insensitive)
      if (matchExactName.value) {
        if (normalizeName(sCol.column_name) === normalizeName(tCol.column_name)) {
          matched = true
          matchType = 'name'
          reason = '同名字段匹配'
          confidence = 100
        }
      }

      // Rule 2: Chinese comment synonym
      if (!matched && matchCommentSynonym.value) {
        if (sCol.column_comment && tCol.column_comment) {
          const sComm = sCol.column_comment.trim()
          const tComm = tCol.column_comment.trim()

          if (sComm === tComm) {
            matched = true
            matchType = 'comment'
            reason = '中文注释完全一致'
            confidence = 95
          } else {
            const cleanS = cleanComment(sComm)
            const cleanT = cleanComment(tComm)
            if (cleanS.length >= 2 && cleanS === cleanT) {
              matched = true
              matchType = 'comment'
              reason = '中文注释核心同义'
              confidence = 90
            } else if (
              cleanS.length >= 2 &&
              cleanT.length >= 2 &&
              (cleanS.includes(cleanT) || cleanT.includes(cleanS))
            ) {
              matched = true
              matchType = 'comment'
              reason = '中文注释相似'
              confidence = 80
            }
          }
        }
      }

      if (matched) {
        results.push({
          id: `${sCol.id}_to_${tCol.id}`,
          sourceTableId: sourceTableId.value,
          sourceCol: sCol,
          targetTableId: targetTableId.value,
          targetCol: tCol,
          matchType,
          reason,
          confidence,
        })
      }
    }
  }

  candidates.value = results
  hasRunMatch.value = true

  // Select all matched candidates by default
  nextTick(() => {
    candidates.value.forEach(row => {
      tableRef.value?.toggleRowSelection(row, true)
    })
    selectedCandidates.value = [...candidates.value]
  })
}

const handleSelectionChange = (selection: MatchCandidate[]) => {
  selectedCandidates.value = selection
}

const toggleSelectAll = (select: boolean) => {
  candidates.value.forEach(row => {
    tableRef.value?.toggleRowSelection(row, select)
  })
  selectedCandidates.value = select ? [...candidates.value] : []
}

const handleSubmit = async () => {
  if (selectedCandidates.value.length === 0) {
    ElMessage.warning('请勾选至少一条映射连线')
    return
  }
  if (!projectStore.currentProject) {
    ElMessage.warning('当前无可用项目')
    return
  }

  submitting.value = true
  try {
    const pairs = selectedCandidates.value.map(c => ({
      sourceTableId: c.sourceTableId,
      sourceColumnId: c.sourceCol.id,
      targetTableId: c.targetTableId,
      targetColumnId: c.targetCol.id,
    }))

    const count = await canvasStore.createBatchMappings(projectStore.currentProject.id, pairs)
    ElMessage.success(`成功智能创建 ${count} 条字段映射连线！`)
    visible.value = false
  } catch (e: any) {
    ElMessage.error('创建映射失败: ' + (e.message || '网络错误'))
  } finally {
    submitting.value = false
  }
}

defineExpose({
  open,
})
</script>

<style scoped>
.dialog-header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 16px;
  color: #1e293b;
}

.header-icon {
  font-size: 18px;
}

.dialog-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-panel {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.tables-selector-row {
  display: flex;
  align-items: center;
  gap: 14px;
}

.selector-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.selector-item .label {
  font-size: 12px;
  font-weight: 600;
  color: #475569;
}

.arrow-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: #94a3b8;
  margin-top: 18px;
}

.strategies-row {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
  font-size: 13px;
  padding-top: 4px;
}

.strategy-label {
  font-weight: 600;
  color: #475569;
  font-size: 12px;
}

.strategy-hint {
  font-size: 11px;
  color: #94a3b8;
  margin-left: 2px;
}

.action-btn-row {
  display: flex;
  justify-content: flex-end;
  padding-top: 4px;
}

.results-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.results-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.results-title {
  font-size: 13px;
  color: #334155;
}

.col-preview {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.col-name {
  font-weight: 600;
  font-size: 12px;
  color: #1e293b;
}

.col-comment {
  font-size: 11px;
  color: #64748b;
  background: #f1f5f9;
  padding: 1px 6px;
  border-radius: 4px;
}

.col-type {
  font-size: 10px;
  color: #94a3b8;
  font-family: monospace;
}

.empty-candidate {
  padding: 20px 0;
  background: #f8fafc;
  border-radius: 8px;
  border: 1px dashed #e2e8f0;
}

.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.footer-left {
  font-size: 12px;
  color: #64748b;
}

.footer-right {
  display: flex;
  gap: 8px;
}
</style>
