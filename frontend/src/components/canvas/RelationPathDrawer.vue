<template>
  <Sheet
    v-model:open="visible"
    title="关联路径探查"
    description="基于已配置的字段血缘连线，自动计算跨表多跳关联路径并生成可执行 SQL JOIN"
    side="right"
    class="w-full sm:max-w-[760px] p-0 flex flex-col"
    @close="handleDrawerClose"
  >
    <div class="flex-1 overflow-y-auto p-5 space-y-4 font-sans">
      <!-- Table Selection Form Card -->
      <div class="rounded-xl border border-border bg-surface-subtle p-4 space-y-3.5 shadow-xs">
        <!-- Canvas Picking Banner -->
        <div
          v-if="canvasStore.pickingTableRole"
          class="flex items-center justify-between p-2.5 rounded-lg bg-primary/10 border border-primary/20 text-xs text-primary animate-pulse"
        >
          <div class="flex items-center gap-2">
            <Crosshair class="w-4 h-4 animate-spin" />
            <span>
              正在画布点选 <strong>{{ canvasStore.pickingTableRole === 'source' ? '起点表 (Source)' : '终点表 (Target)' }}</strong>，请直接在画布上点击任意表卡片
            </span>
          </div>
          <Button size="xs" variant="outline" @click="cancelCanvasPick">取消点选</Button>
        </div>

        <div class="grid grid-cols-[1fr,auto,1fr] gap-2.5 items-end">
          <!-- Source Table -->
          <div class="space-y-1">
            <div class="flex items-center gap-1.5 text-xs font-medium text-primary-text">
              <span class="w-2 h-2 rounded-full bg-blue-500"></span>
              <span>起点表 (Source)</span>
            </div>
            <div class="flex items-center gap-1.5">
              <Select
                v-model="sourceTableId"
                placeholder="请选择起点表..."
                size="md"
                wrapper-class="flex-1"
                :options="tableSelectOptions"
              />
              <Tooltip content="在画布中点击选择">
                <Button
                  :variant="canvasStore.pickingTableRole === 'source' ? 'primary' : 'outline'"
                  size="icon"
                  @click="startCanvasPick('source')"
                >
                  <Crosshair class="w-3.5 h-3.5" />
                </Button>
              </Tooltip>
            </div>
          </div>

          <!-- Swap Button in Middle -->
          <div class="pb-0.5">
            <Tooltip content="交换起止表">
              <Button
                variant="outline"
                size="icon"
                class="rounded-full"
                @click="handleSwapTables"
              >
                <ArrowLeftRight class="w-3.5 h-3.5" />
              </Button>
            </Tooltip>
          </div>

          <!-- Target Table -->
          <div class="space-y-1">
            <div class="flex items-center gap-1.5 text-xs font-medium text-primary-text">
              <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
              <span>终点表 (Target)</span>
            </div>
            <div class="flex items-center gap-1.5">
              <Select
                v-model="targetTableId"
                placeholder="请选择终点表..."
                size="md"
                wrapper-class="flex-1"
                :options="tableSelectOptions"
              />
              <Tooltip content="在画布中点击选择">
                <Button
                  :variant="canvasStore.pickingTableRole === 'target' ? 'primary' : 'outline'"
                  size="icon"
                  @click="startCanvasPick('target')"
                >
                  <Crosshair class="w-3.5 h-3.5" />
                </Button>
              </Tooltip>
            </div>
          </div>
        </div>

        <!-- Controls Row -->
        <div class="flex items-center justify-between pt-2 border-t border-border">
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted">最大允许跳数:</span>
            <div class="flex items-center p-0.5 rounded-md border border-border bg-surface">
              <button
                v-for="hop in [2, 3, 4, 5]"
                :key="hop"
                type="button"
                class="h-6 px-2.5 rounded text-xs font-medium transition-colors cursor-pointer"
                :class="maxHops === hop ? 'bg-primary text-white shadow-2xs' : 'text-muted hover:text-primary-text'"
                @click="maxHops = hop"
              >
                {{ hop }} 跳 {{ hop === 4 ? '(推荐)' : '' }}
              </button>
            </div>
          </div>

          <Button
            variant="primary"
            :loading="searching"
            :disabled="!sourceTableId || !targetTableId || sourceTableId === targetTableId"
            @click="handleDiscoverPaths"
          >
            <Search class="w-3.5 h-3.5 mr-1.5" />
            <span>计算关联路径</span>
          </Button>
        </div>
      </div>

      <!-- Result Section -->
      <div>
        <!-- Initial Blank State -->
        <div v-if="!hasSearched" class="py-16 flex flex-col items-center justify-center text-center gap-2">
          <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center text-primary mb-1">
            <Route class="w-6 h-6" />
          </div>
          <h4 class="text-sm font-semibold text-primary-text m-0">选择两张表开始探查</h4>
          <p class="text-xs text-muted max-w-sm m-0 leading-relaxed">
            系统将基于画布上已建立的字段血缘映射连线，深度搜索所有可连通的多跳路径并输出可执行 SQL JOIN。
          </p>
        </div>

        <!-- No Path Found State -->
        <div v-else-if="pathResults.length === 0" class="py-12 flex flex-col items-center justify-center text-center gap-2 border border-border rounded-xl bg-surface-subtle/50">
          <span class="text-xs font-medium text-primary-text">在当前已有血缘关系中未发现连通路径</span>
          <span class="text-xs text-muted max-w-md">
            两表之间暂无直接或多跳中继的字段映射血缘，您可先在画布上建立连线或调大跳数。
          </span>
        </div>

        <!-- Path List -->
        <div v-else class="space-y-4">
          <div class="flex items-center justify-between text-xs text-muted">
            <div>
              共探查到 <strong class="text-primary font-mono">{{ pathResults.length }}</strong> 条关联路径 (已按跳数与置信度智能排序)
            </div>
            <Button
              v-if="canvasStore.activeRelationPath"
              size="xs"
              variant="outline"
              @click="canvasStore.clearRelationPathHighlight()"
            >
              清除画布高亮
            </Button>
          </div>

          <div
            v-for="(pathItem, pIdx) in pathResults"
            :key="pathItem.path_id"
            class="rounded-xl border transition-all duration-150 p-4 space-y-3.5 bg-surface"
            :class="canvasStore.activeRelationPath?.path_id === pathItem.path_id
              ? 'border-primary ring-2 ring-primary/20 shadow-md'
              : 'border-border shadow-2xs'"
          >
            <!-- Card Header -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="px-2 py-0.5 text-xs font-bold font-mono rounded bg-surface-subtle border border-border text-primary-text">
                  方案 {{ pIdx + 1 }}
                </span>
                <Badge :variant="pathItem.hops === 1 ? 'success' : pathItem.hops === 2 ? 'primary' : 'warning'">
                  {{ pathItem.hops }} 跳血缘关联
                </Badge>
                <Badge variant="success">已有血缘连线</Badge>
              </div>

              <Button
                size="xs"
                :variant="canvasStore.activeRelationPath?.path_id === pathItem.path_id ? 'primary' : 'outline'"
                @click="toggleHighlightPath(pathItem)"
              >
                <Eye class="w-3.5 h-3.5 mr-1" />
                <span>{{ canvasStore.activeRelationPath?.path_id === pathItem.path_id ? '已在画布高亮' : '在画布高亮' }}</span>
              </Button>
            </div>

            <!-- Path Breadcrumbs Title -->
            <div class="flex items-center flex-wrap gap-1.5 p-2 rounded-lg bg-surface-subtle border border-border text-xs font-mono">
              <template v-for="(tName, idx) in pathItem.summary_title.split(' -> ')" :key="idx">
                <span class="font-semibold text-primary-text">{{ tName }}</span>
                <ArrowRight v-if="idx < pathItem.summary_title.split(' -> ').length - 1" class="w-3 h-3 text-muted" />
              </template>
            </div>

            <!-- Missing Intermediate Tables Prompt -->
            <div
              v-if="getMissingIntermediateTables(pathItem).length > 0"
              class="flex items-center justify-between p-2.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-xs text-amber-600 dark:text-amber-400"
            >
              <div class="flex items-center gap-1.5">
                <Info class="w-4 h-4 shrink-0" />
                <span>中间表 <strong>{{ getMissingIntermediateTables(pathItem).map(t => t.table_name).join(', ') }}</strong> 未在当前视图中</span>
              </div>
              <Button
                size="xs"
                variant="outline"
                class="border-amber-500/40 text-amber-600 dark:text-amber-400 hover:bg-amber-500/10"
                @click="addIntermediateToCanvas(pathItem)"
              >
                一键引入到当前画布
              </Button>
            </div>

            <!-- Steps Breakdown Timeline -->
            <div class="space-y-2 border-l-2 border-primary/30 pl-3 ml-2">
              <div
                v-for="(step, sIdx) in pathItem.steps"
                :key="sIdx"
                class="relative space-y-1 text-xs"
              >
                <div class="flex items-center gap-2">
                  <span class="w-4 h-4 rounded-full bg-primary text-white text-[10px] flex items-center justify-center font-mono font-bold">
                    {{ sIdx + 1 }}
                  </span>
                  <div class="flex items-center gap-2 font-mono">
                    <span class="text-muted">{{ step.from_table.table_name }}.</span>
                    <strong class="text-primary-text">{{ step.from_column.column_name }}</strong>
                    <span class="px-1 text-muted font-bold">=</span>
                    <span class="text-muted">{{ step.to_table.table_name }}.</span>
                    <strong class="text-primary-text">{{ step.to_column.column_name }}</strong>
                  </div>
                </div>
                <div class="pl-6 text-[11px] text-muted">
                  {{ step.match_reason }}
                </div>
              </div>
            </div>

            <!-- Generated SQL JOIN Code Block -->
            <div class="rounded-lg border border-border bg-surface-subtle overflow-hidden">
              <div class="flex items-center justify-between px-3 py-1.5 bg-surface border-b border-border">
                <span class="text-xs font-semibold text-primary-text flex items-center gap-1.5">
                  <FileCode class="w-3.5 h-3.5 text-primary" />
                  多表关联 SQL (JOIN 语句)
                </span>
                <Button size="xs" variant="ghost" @click="copySql(pathItem.sql_snippet)">
                  <Copy class="w-3 h-3 mr-1" />
                  复制 SQL
                </Button>
              </div>
              <pre class="p-3 text-xs font-mono text-primary-text overflow-x-auto leading-relaxed m-0"><code>{{ pathItem.sql_snippet }}</code></pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Sheet>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { api } from '@/api'
import type { RelationPath, TableMeta } from '@/types'
import { toast } from '@/utils/toast'
import { Sheet, Select, Button, Badge, Tooltip } from '@/components/ui'
import {
  Route,
  Crosshair,
  Search,
  ArrowLeftRight,
  ArrowRight,
  Info,
  Eye,
  FileCode,
  Copy,
} from 'lucide-vue-next'

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

const tableSelectOptions = computed(() => {
  return projectTables.value.map(tbl => ({
    label: `${tbl.table_name}${tbl.table_comment ? ' (' + tbl.table_comment + ')' : ''}`,
    value: tbl.id,
  }))
})

const open = (srcId?: string, tgtId?: string) => {
  if (srcId) sourceTableId.value = srcId
  if (tgtId) targetTableId.value = tgtId
  visible.value = true
  if (sourceTableId.value && targetTableId.value && sourceTableId.value !== targetTableId.value) {
    handleDiscoverPaths()
  }
}

const startCanvasPick = (role: 'source' | 'target') => {
  canvasStore.startPickingTable(role, (pickedRole, tblId, _tblName) => {
    if (pickedRole === 'source') {
      sourceTableId.value = tblId
    } else {
      targetTableId.value = tblId
    }
    toast.success(`已选定${pickedRole === 'source' ? '起点表' : '终点表'}: ${_tblName}`)
  })
}

const cancelCanvasPick = () => {
  canvasStore.cancelPickingTable()
}

const handleSwapTables = () => {
  const temp = sourceTableId.value
  sourceTableId.value = targetTableId.value
  targetTableId.value = temp
  if (hasSearched.value && sourceTableId.value && targetTableId.value) {
    handleDiscoverPaths()
  }
}

const handleDiscoverPaths = async () => {
  if (!projectStore.currentProject) return
  if (!sourceTableId.value || !targetTableId.value) {
    toast.warning('请选择起点表与终点表')
    return
  }
  if (sourceTableId.value === targetTableId.value) {
    toast.warning('起点表与终点表不能相同')
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
      canvasStore.highlightRelationPath(pathResults.value[0])
    } else {
      canvasStore.clearRelationPathHighlight()
    }
  } catch (e: any) {
    toast.error('计算关联路径失败: ' + (e.message || '网络错误'))
  } finally {
    searching.value = false
  }
}

const toggleHighlightPath = (path: RelationPath) => {
  if (canvasStore.activeRelationPath?.path_id === path.path_id) {
    canvasStore.clearRelationPathHighlight()
  } else {
    canvasStore.highlightRelationPath(path)
  }
}

const getMissingIntermediateTables = (path: RelationPath): TableMeta[] => {
  const canvasNodeTableIds = new Set(canvasStore.nodes.map(n => n.data.tableId))
  const allTables = projectStore.currentProject?.tables || []
  return path.intermediate_table_ids
    .filter(tId => !canvasNodeTableIds.has(tId))
    .map(tId => allTables.find(t => t.id === tId))
    .filter((t): t is TableMeta => !!t)
}

const addIntermediateToCanvas = async (path: RelationPath) => {
  const missing = getMissingIntermediateTables(path)
  if (missing.length === 0) return

  const tableIds = missing.map(m => m.id)
  await canvasStore.addTablesToCurrentCanvas(tableIds)
  canvasStore.highlightRelationPath(path)
}

const copySql = async (sql: string) => {
  try {
    await navigator.clipboard.writeText(sql)
    toast.success('SQL JOIN 语句已复制到剪贴板')
  } catch {
    toast.error('复制失败，请手动选中复制')
  }
}

const handleDrawerClose = () => {
  canvasStore.cancelPickingTable()
}

defineExpose({ open })
</script>
