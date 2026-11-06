<template>
  <Dialog
    v-model:open="visible"
    title="智能同名/同义字段自动连线向导"
    description="自动分析源表与目标表的字段同名规则与中文业务同义词，一键批量建立血缘映射"
    class="max-w-[860px]"
  >
    <div class="flex flex-col gap-4 py-1">
      <!-- Strategy Settings -->
      <div class="rounded-xl border border-border bg-surface-subtle p-4 space-y-3.5">
        <div class="grid grid-cols-[1fr,auto,1fr] gap-3 items-center">
          <!-- Source Card -->
          <div class="space-y-1.5">
            <div class="flex items-center gap-1.5 text-xs font-semibold text-primary-text">
              <span class="w-2 h-2 rounded-full bg-blue-500"></span>
              <span>源数据表 (Source)</span>
            </div>
            <Select
              v-model="sourceTableId"
              placeholder="选择源数据表"
              :options="sourceTableOptions"
              @change="onTableSelectionChange"
            />
          </div>

          <!-- Arrow -->
          <div class="pt-4 text-muted">
            <ArrowRight class="w-4 h-4 mx-auto" />
          </div>

          <!-- Target Card -->
          <div class="space-y-1.5">
            <div class="flex items-center gap-1.5 text-xs font-semibold text-primary-text">
              <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
              <span>目标数据表 (Target)</span>
            </div>
            <Select
              v-model="targetTableId"
              placeholder="选择目标数据表"
              :options="targetTableOptions"
              @change="onTableSelectionChange"
            />
          </div>
        </div>

        <div class="flex items-center flex-wrap gap-4 pt-1 border-t border-border">
          <span class="text-xs font-medium text-secondary-text">匹配策略：</span>
          <Checkbox
            v-model="matchExactName"
            label="同名字段精确匹配 (忽略大小写及下划线风格)"
          />
          <Checkbox
            v-model="matchCommentSynonym"
            label="中文注释同义匹配 (忽略主键/ID/编号后缀)"
          />
          <Checkbox
            v-model="skipExisting"
            label="排除已连线字段"
          />
        </div>

        <div class="flex justify-end pt-1">
          <Button
            variant="primary"
            :disabled="!sourceTableId || !targetTableId || sourceTableId === targetTableId"
            @click="runMatch"
          >
            <Search class="w-3.5 h-3.5 mr-1.5" />
            开始智能匹配
          </Button>
        </div>
      </div>

      <!-- Match Results Preview -->
      <div v-if="hasRunMatch" class="space-y-2.5">
        <div class="flex items-center justify-between text-xs">
          <div class="text-primary-text font-medium">
            匹配候选结果 (共发现 <strong class="text-primary font-mono">{{ candidates.length }}</strong> 对可用映射)
          </div>
          <div class="flex items-center gap-1.5">
            <Button size="xs" variant="outline" @click="toggleSelectAll(true)">全选</Button>
            <Button size="xs" variant="outline" @click="toggleSelectAll(false)">全不选</Button>
          </div>
        </div>

        <div v-if="candidates.length === 0" class="py-10 text-center text-xs text-muted border border-border rounded-lg bg-surface-subtle">
          未匹配到符合规则的候选字段对，请尝试更换匹配策略或选择其他数据表
        </div>

        <div v-else class="border border-border rounded-lg overflow-hidden max-h-[300px] overflow-y-auto">
          <table class="w-full text-left text-xs border-collapse">
            <thead class="bg-surface-subtle sticky top-0 border-b border-border text-muted font-medium select-none z-10">
              <tr>
                <th class="w-10 px-3 py-2 text-center">
                  <input
                    type="checkbox"
                    :checked="isAllSelected"
                    class="rounded border-border text-primary focus:ring-primary cursor-pointer"
                    @change="toggleSelectAll(!isAllSelected)"
                  />
                </th>
                <th class="px-3 py-2 font-semibold">源字段 (Source)</th>
                <th class="w-8 px-1 py-2 text-center"></th>
                <th class="px-3 py-2 font-semibold">目标字段 (Target)</th>
                <th class="w-32 px-3 py-2 text-center font-semibold">匹配依据</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border/40">
              <tr
                v-for="row in candidates"
                :key="row.id"
                class="hover:bg-surface-hover transition-colors"
              >
                <td class="px-3 py-2 text-center">
                  <input
                    type="checkbox"
                    :checked="selectedCandidates.some(c => c.id === row.id)"
                    class="rounded border-border text-primary focus:ring-primary cursor-pointer"
                    @change="toggleRow(row)"
                  />
                </td>
                <td class="px-3 py-2">
                  <div class="flex items-center gap-1.5">
                    <span class="font-mono font-medium text-primary-text">{{ row.sourceCol.column_name }}</span>
                    <span v-if="row.sourceCol.column_comment" class="text-muted text-[11px] truncate max-w-[120px]">
                      ({{ row.sourceCol.column_comment }})
                    </span>
                    <span class="font-mono text-[10px] text-muted">{{ row.sourceCol.data_type }}</span>
                  </div>
                </td>
                <td class="px-1 py-2 text-center text-muted">
                  <ArrowRight class="w-3.5 h-3.5 mx-auto" />
                </td>
                <td class="px-3 py-2">
                  <div class="flex items-center gap-1.5">
                    <span class="font-mono font-medium text-primary-text">{{ row.targetCol.column_name }}</span>
                    <span v-if="row.targetCol.column_comment" class="text-muted text-[11px] truncate max-w-[120px]">
                      ({{ row.targetCol.column_comment }})
                    </span>
                    <span class="font-mono text-[10px] text-muted">{{ row.targetCol.data_type }}</span>
                  </div>
                </td>
                <td class="px-3 py-2 text-center">
                  <Badge :variant="row.matchType === 'name' ? 'success' : 'warning'">
                    {{ row.reason }}
                  </Badge>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between w-full">
        <span class="text-xs text-muted">
          <span v-if="hasRunMatch && candidates.length > 0">
            已勾选 <strong class="text-primary font-mono">{{ selectedCandidates.length }}</strong> / {{ candidates.length }} 对映射
          </span>
        </span>
        <div class="flex items-center gap-2">
          <Button variant="ghost" @click="visible = false">取消</Button>
          <Button
            variant="primary"
            :disabled="selectedCandidates.length === 0"
            :loading="submitting"
            @click="handleSubmit"
          >
            确认创建所选映射 ({{ selectedCandidates.length }})
          </Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { toast } from '@/utils/toast'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { Dialog, Select, Checkbox, Button, Badge } from '@/components/ui'
import { Search, ArrowRight } from 'lucide-vue-next'
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

const availableTables = computed<TableMeta[]>(() => {
  return projectStore.currentProject?.tables || []
})

const sourceTableOptions = computed(() => {
  return availableTables.value.map(t => ({
    label: `${t.table_name}${t.table_comment ? ` (${t.table_comment})` : ''}`,
    value: t.id,
    disabled: t.id === targetTableId.value,
  }))
})

const targetTableOptions = computed(() => {
  return availableTables.value.map(t => ({
    label: `${t.table_name}${t.table_comment ? ` (${t.table_comment})` : ''}`,
    value: t.id,
    disabled: t.id === sourceTableId.value,
  }))
})

const isAllSelected = computed(() => {
  if (candidates.value.length === 0) return false
  return candidates.value.every(c => selectedCandidates.value.some(s => s.id === c.id))
})

const toggleSelectAll = (select: boolean) => {
  selectedCandidates.value = select ? [...candidates.value] : []
}

const toggleRow = (row: MatchCandidate) => {
  const idx = selectedCandidates.value.findIndex(c => c.id === row.id)
  if (idx > -1) {
    selectedCandidates.value.splice(idx, 1)
  } else {
    selectedCandidates.value.push(row)
  }
}

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

const normalizeName = (name: string): string => {
  return (name || '').toLowerCase().replace(/[-_]/g, '')
}

const cleanComment = (comm: string): string => {
  if (!comm) return ''
  return comm
    .trim()
    .toLowerCase()
    .replace(/(主键|id|编号|标识|代码|序号)$/gi, '')
    .trim()
}

const isAlreadyMapped = (srcColId: string, tgtColId: string): boolean => {
  return canvasStore.edges.some(
    e =>
      (e.data?.sourceColumnId === srcColId && e.data?.targetColumnId === tgtColId) ||
      (e.data?.sourceColumnId === tgtColId && e.data?.targetColumnId === srcColId)
  )
}

const runMatch = () => {
  if (!sourceTableId.value || !targetTableId.value) {
    toast.warning('请选择源数据表与目标数据表')
    return
  }

  const srcTable = availableTables.value.find(t => t.id === sourceTableId.value)
  const tgtTable = availableTables.value.find(t => t.id === targetTableId.value)

  if (!srcTable || !tgtTable) {
    toast.warning('所选表不存在')
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

      if (matchExactName.value) {
        if (normalizeName(sCol.column_name) === normalizeName(tCol.column_name)) {
          matched = true
          matchType = 'name'
          reason = '同名字段匹配'
          confidence = 100
        }
      }

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
  selectedCandidates.value = [...candidates.value]
}

const handleSubmit = async () => {
  if (selectedCandidates.value.length === 0) {
    toast.warning('请勾选至少一条映射连线')
    return
  }
  if (!projectStore.currentProject) {
    toast.warning('当前无可用项目')
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
    toast.success(`成功智能创建 ${count} 条字段映射连线！`)
    visible.value = false
  } catch (e: any) {
    toast.error('创建映射失败: ' + (e.message || '网络错误'))
  } finally {
    submitting.value = false
  }
}

defineExpose({
  open,
})
</script>
