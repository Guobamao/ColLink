<template>
  <Dialog
    v-model:open="visible"
    title="DDL 脚本解析导入"
    description="支持达梦、MySQL、PostgreSQL 等建表 DDL 语法的高精度智能解析与增量同步"
    class="max-w-[860px]"
  >
    <div class="flex flex-col gap-3.5 py-1">
      <!-- Dialect & Input Mode Selector -->
      <div class="flex items-center justify-between p-2.5 rounded-lg border border-border bg-surface-subtle">
        <div class="flex items-center gap-2">
          <span class="text-xs font-medium text-secondary-text">数据库方言：</span>
          <Select
            v-model="dialect"
            size="sm"
            wrapper-class="w-48"
            :options="dialectOptions"
          />
        </div>

        <div class="flex items-center p-0.5 rounded-md border border-border bg-surface">
          <button
            type="button"
            class="h-6 px-3 rounded text-xs font-medium transition-colors cursor-pointer"
            :class="inputMode === 'text' ? 'bg-primary text-white shadow-2xs' : 'text-muted hover:text-primary-text'"
            @click="inputMode = 'text'"
          >
            粘贴 SQL 文本
          </button>
          <button
            type="button"
            class="h-6 px-3 rounded text-xs font-medium transition-colors cursor-pointer"
            :class="inputMode === 'file' ? 'bg-primary text-white shadow-2xs' : 'text-muted hover:text-primary-text'"
            @click="inputMode = 'file'"
          >
            上传 .sql 文件
          </button>
        </div>
      </div>

      <!-- Text Mode -->
      <div v-if="inputMode === 'text'" class="flex flex-col gap-2">
        <textarea
          v-model="sqlText"
          rows="7"
          placeholder="在此粘贴 CREATE TABLE 及 COMMENT 语句..."
          class="w-full p-2.5 rounded-md bg-surface-subtle border border-border text-primary-text placeholder:text-muted/50 font-mono text-xs leading-relaxed focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary resize-y"
        ></textarea>
        <div class="flex items-center justify-between">
          <Button variant="outline" size="sm" @click="loadSampleSql">
            加载达梦示例 SQL
          </Button>
          <Button variant="primary" size="sm" :loading="parsing" @click="handleParseText">
            解析 DDL
          </Button>
        </div>
      </div>

      <!-- File Mode -->
      <div v-else class="flex flex-col gap-2">
        <div
          class="border-2 border-dashed border-border hover:border-primary/60 rounded-xl p-8 flex flex-col items-center justify-center gap-2.5 bg-surface-subtle/50 hover:bg-surface-subtle transition-colors cursor-pointer text-center"
          @click="triggerFileInput"
          @dragover.prevent
          @drop.prevent="handleFileDrop"
        >
          <input
            ref="fileInputRef"
            type="file"
            accept=".sql,.txt"
            class="hidden"
            @change="handleFileInputChange"
          />
          <div class="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-primary">
            <Upload class="w-5 h-5" />
          </div>
          <div>
            <span class="text-xs font-medium text-primary-text">点击选择文件</span>
            <span class="text-xs text-muted"> 或将 .sql 文件拖拽至此处</span>
          </div>
          <span class="text-[11px] text-muted">
            支持达梦、MySQL、PostgreSQL 导出的建表 DDL 文件 (UTF-8 编码)
          </span>
        </div>
      </div>

      <!-- Parsed Preview -->
      <div v-if="parsedTables.length > 0" class="flex flex-col gap-2.5 mt-1">
        <div class="flex items-center justify-between">
          <div class="flex flex-col gap-0.5">
            <span class="text-xs font-semibold text-primary-text">
              解析结果预览 (共 {{ parsedTables.length }} 张表)
            </span>
            <span class="text-[11px] text-muted">
              已在画布中的表已默认勾选，取消勾选将同步删除画布表及关联映射
            </span>
          </div>
          <Button
            variant="primary"
            size="sm"
            :loading="importing"
            @click="handleImportSelected"
          >
            {{ newTablesCount > 0 ? `确认导入新表 (${newTablesCount})` : `确认增量同步 (${selectedTables.length})` }}
          </Button>
        </div>

        <div class="border border-border rounded-lg overflow-hidden max-h-[260px] overflow-y-auto">
          <table class="w-full text-left text-xs border-collapse">
            <thead class="bg-surface-subtle sticky top-0 border-b border-border text-muted font-medium select-none z-10">
              <tr>
                <th class="w-10 px-3 py-2 text-center">
                  <input
                    type="checkbox"
                    :checked="isAllSelected"
                    class="rounded border-border text-primary focus:ring-primary cursor-pointer"
                    @change="toggleSelectAll"
                  />
                </th>
                <th class="px-3 py-2 font-semibold">表英文名</th>
                <th class="px-3 py-2 font-semibold">表中文注释</th>
                <th class="w-24 px-3 py-2 text-center font-semibold">包含字段数</th>
                <th class="w-24 px-3 py-2 text-center font-semibold">当前画布状态</th>
                <th class="px-3 py-2 font-semibold">字段概览</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border/40">
              <tr
                v-for="row in parsedTables"
                :key="row.table_name"
                class="hover:bg-surface-hover transition-colors"
              >
                <td class="px-3 py-2 text-center">
                  <input
                    type="checkbox"
                    :checked="isRowSelected(row.table_name)"
                    class="rounded border-border text-primary focus:ring-primary cursor-pointer"
                    @change="toggleRowSelection(row)"
                  />
                </td>
                <td class="px-3 py-2 font-mono font-medium text-primary-text">
                  {{ row.table_name }}
                </td>
                <td class="px-3 py-2 text-muted truncate max-w-[140px]">
                  {{ row.table_comment || '无注释' }}
                </td>
                <td class="px-3 py-2 text-center">
                  <Badge variant="success">{{ row.columns?.length || 0 }}</Badge>
                </td>
                <td class="px-3 py-2 text-center">
                  <Badge v-if="isTableInCanvas(row.table_name)" variant="primary">已在画布</Badge>
                  <Badge v-else variant="secondary">未添加</Badge>
                </td>
                <td class="px-3 py-2 text-muted font-mono text-[11px] truncate max-w-[180px]">
                  {{ row.columns?.map((c: any) => c.column_name).join(', ') }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="visible = false">关闭</Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { api } from '@/api'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { toast, confirmDialog } from '@/utils/toast'
import { Dialog, Select, Button, Badge } from '@/components/ui'
import { Upload } from 'lucide-vue-next'
import type { ParsedTable } from '@/types'

const visible = ref(false)
const emit = defineEmits(['imported'])
const dialect = ref('dm')
const inputMode = ref<'text' | 'file'>('text')
const sqlText = ref('')
const parsing = ref(false)
const importing = ref(false)
const parsedTables = ref<ParsedTable[]>([])
const selectedTables = ref<ParsedTable[]>([])
const fileInputRef = ref<HTMLInputElement | null>(null)

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const dialectOptions = [
  { label: '达梦数据库 (DM8/DM7)', value: 'dm' },
  { label: 'MySQL 5.7 / 8.0', value: 'mysql' },
  { label: 'PostgreSQL', value: 'postgres' },
  { label: 'Oracle', value: 'oracle' },
]

const isTableInCanvas = (tableName: string): boolean => {
  if (!projectStore.currentProject?.tables) return false
  return projectStore.currentProject.tables.some(
    t => t.table_name.toUpperCase() === tableName.toUpperCase()
  )
}

const isRowSelected = (tableName: string): boolean => {
  return selectedTables.value.some(t => t.table_name.toUpperCase() === tableName.toUpperCase())
}

const newTablesCount = computed(() => {
  return selectedTables.value.filter(t => !isTableInCanvas(t.table_name)).length
})

const isAllSelected = computed(() => {
  if (parsedTables.value.length === 0) return false
  return parsedTables.value.every(t => isRowSelected(t.table_name))
})

const toggleSelectAll = async () => {
  if (isAllSelected.value) {
    // Uncheck all with confirmation if existing tables in canvas
    const existingTablesInList = parsedTables.value.filter(row => isTableInCanvas(row.table_name))
    if (existingTablesInList.length > 0) {
      const confirmed = await confirmDialog({
        title: '批量同步删除确认',
        message: `取消全选包含 ${existingTablesInList.length} 张已存在于画布中的数据表。\n\n取消全选将从画布中同步删除这些表及其关联连线，确定要全部删除吗？`,
        confirmText: '确定全部删除',
        variant: 'danger',
      })

      if (!confirmed) return

      for (const row of existingTablesInList) {
        const t = projectStore.currentProject?.tables?.find(
          et => et.table_name.toUpperCase() === row.table_name.toUpperCase()
        )
        if (t) {
          await api.deleteTable(t.id)
        }
      }
      toast.success(`已从画布批量删除 ${existingTablesInList.length} 张表`)

      if (projectStore.currentProject) {
        await projectStore.selectProject(projectStore.currentProject.id)
        canvasStore.syncFromProject(
          projectStore.currentProject.tables,
          projectStore.currentProject.mappings
        )
      }
    }
    selectedTables.value = []
  } else {
    selectedTables.value = [...parsedTables.value]
  }
}

const toggleRowSelection = async (row: ParsedTable) => {
  const isChecked = isRowSelected(row.table_name)
  if (isChecked) {
    // User unchecking
    const existingTable = projectStore.currentProject?.tables?.find(
      t => t.table_name.toUpperCase() === row.table_name.toUpperCase()
    )

    if (existingTable) {
      const confirmed = await confirmDialog({
        title: '同步删除确认',
        message: `数据表「${row.table_name}」已存在于当前项目画布中。\n\n取消勾选将从画布中同步删除该表及其所有的字段映射连线清单，确定要删除吗？`,
        confirmText: '确定删除',
        variant: 'danger',
      })

      if (!confirmed) return

      await api.deleteTable(existingTable.id)
      toast.success(`已从画布同步删除表「${row.table_name}」及关联映射`)

      if (projectStore.currentProject) {
        await projectStore.selectProject(projectStore.currentProject.id)
        canvasStore.syncFromProject(
          projectStore.currentProject.tables,
          projectStore.currentProject.mappings
        )
      }
    }

    selectedTables.value = selectedTables.value.filter(
      t => t.table_name.toUpperCase() !== row.table_name.toUpperCase()
    )
  } else {
    selectedTables.value.push(row)
  }
}

const open = () => {
  visible.value = true
  if (!sqlText.value) {
    loadSampleSql()
  }
}

const triggerFileInput = () => {
  if (fileInputRef.value) {
    fileInputRef.value.value = ''
    fileInputRef.value.click()
  }
}

const handleFileInputChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) parseFile(file)
}

const handleFileDrop = (e: DragEvent) => {
  const file = e.dataTransfer?.files?.[0]
  if (file) parseFile(file)
}

const parseFile = async (file: File) => {
  parsing.value = true
  try {
    const formData = new FormData()
    formData.append('file', file)
    formData.append('dialect', dialect.value)
    const list = await api.parseDdlFile(formData)
    if (list.length === 0) {
      toast.warning('未能识别出 CREATE TABLE 语句')
    } else {
      syncTableSelections(list)
      toast.success(`成功解析出 ${list.length} 张表`)
    }
  } catch (err: any) {
    toast.error('文件解析失败: ' + (err.message || '网络错误'))
  } finally {
    parsing.value = false
  }
}

const loadSampleSql = () => {
  sqlText.value = `-- 达梦数据库 (DM8) 示例建表脚本
CREATE TABLE "ODS_USER_INFO" (
    "USER_ID" NUMBER(10, 0) NOT NULL,
    "USER_NAME" VARCHAR2(50) NOT NULL,
    "PHONE" VARCHAR2(20),
    "STATUS" CHAR(1) DEFAULT '1',
    "REG_TIME" DATETIME,
    "REMARK" CLOB,
    PRIMARY KEY ("USER_ID")
);
COMMENT ON TABLE "ODS_USER_INFO" IS 'ODS用户原始信息表';
COMMENT ON COLUMN "ODS_USER_INFO"."USER_ID" IS '用户唯一主键';
COMMENT ON COLUMN "ODS_USER_INFO"."USER_NAME" IS '用户登录名';
COMMENT ON COLUMN "ODS_USER_INFO"."PHONE" IS '绑定手机号';
COMMENT ON COLUMN "ODS_USER_INFO"."STATUS" IS '账号状态(1正常 0禁用)';
COMMENT ON COLUMN "ODS_USER_INFO"."REG_TIME" IS '注册时间';
COMMENT ON COLUMN "ODS_USER_INFO"."REMARK" IS '用户描述';

CREATE TABLE "DWD_DIM_USER" (
    "DIM_USER_KEY" NUMBER(10, 0) NOT NULL,
    "SRC_USER_ID" NUMBER(10, 0) NOT NULL,
    "USER_NAME" VARCHAR2(50) NOT NULL,
    "MOBILE_PHONE" VARCHAR2(20),
    "IS_ACTIVE" CHAR(1),
    "CREATED_AT" DATETIME,
    PRIMARY KEY ("DIM_USER_KEY")
);
COMMENT ON TABLE "DWD_DIM_USER" IS 'DWD用户维度表';
COMMENT ON COLUMN "DWD_DIM_USER"."DIM_USER_KEY" IS '维度代理键';
COMMENT ON COLUMN "DWD_DIM_USER"."SRC_USER_ID" IS '源系统用户ID';
COMMENT ON COLUMN "DWD_DIM_USER"."USER_NAME" IS '用户姓名';
COMMENT ON COLUMN "DWD_DIM_USER"."MOBILE_PHONE" IS '标准手机号';
COMMENT ON COLUMN "DWD_DIM_USER"."IS_ACTIVE" IS '有效状态';
COMMENT ON COLUMN "DWD_DIM_USER"."CREATED_AT" IS '维度同步时间';`
}

const syncTableSelections = (list: ParsedTable[]) => {
  parsedTables.value = list
  const preselected: ParsedTable[] = []
  list.forEach(row => {
    if (isTableInCanvas(row.table_name)) {
      preselected.push(row)
    }
  })
  if (preselected.length === 0) {
    selectedTables.value = [...list]
  } else {
    selectedTables.value = preselected
  }
}

const handleParseText = async () => {
  if (!sqlText.value.trim()) {
    toast.warning('请输入 SQL 语句')
    return
  }
  parsing.value = true
  try {
    const list = await api.parseDdlText({
      sql_text: sqlText.value,
      dialect: dialect.value,
    })
    if (list.length === 0) {
      toast.warning('未能识别出 CREATE TABLE 语句，请检查 SQL 语法')
    } else {
      syncTableSelections(list)
      toast.success(`成功解析出 ${list.length} 张表`)
    }
  } catch (err: any) {
    toast.error('解析失败: ' + (err.message || '网络错误'))
  } finally {
    parsing.value = false
  }
}

const handleImportSelected = async () => {
  if (selectedTables.value.length === 0) {
    toast.warning('请勾选至少一张表')
    return
  }
  if (!projectStore.currentProject) {
    toast.warning('请先选择或创建一个项目')
    return
  }

  importing.value = true
  try {
    const res = await api.importDdlTables(
      projectStore.currentProject.id,
      selectedTables.value,
      canvasStore.currentCanvasId || undefined
    )

    if (res.summary?.message) {
      toast.success('表结构同步完成', { description: res.summary.message })
    } else {
      toast.success('导入完成')
    }
    visible.value = false

    await projectStore.selectProject(projectStore.currentProject.id)
    if (projectStore.currentProject) {
      if (canvasStore.currentCanvasId) {
        await canvasStore.switchCanvas(canvasStore.currentCanvasId)
        await canvasStore.fetchCanvases(projectStore.currentProject.id, false)
      } else {
        await canvasStore.fetchCanvases(projectStore.currentProject.id)
      }
    }
    emit('imported')
  } catch (err: any) {
    toast.error('导入失败: ' + (err.message || '网络错误'))
  } finally {
    importing.value = false
  }
}

defineExpose({ open })
</script>
