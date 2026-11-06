<template>
  <el-dialog
    v-model="visible"
    width="840px"
    destroy-on-close
    :close-on-click-modal="false"
  >
    <template #header>
      <div style="display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 16px; color: var(--cl-text-primary);">
        <el-icon color="#3b82f6"><DocumentAdd /></el-icon>
        <span>DDL 脚本解析导入</span>
      </div>
    </template>
    <div class="dialog-body">
      <!-- Dialect & Input Mode Selector -->
      <div class="config-bar">
        <div class="config-item">
          <span class="label">数据库方言：</span>
          <el-select v-model="dialect" style="width: 180px">
            <el-option label="达梦数据库 (DM8/DM7)" value="dm" />
            <el-option label="MySQL 5.7 / 8.0" value="mysql" />
            <el-option label="PostgreSQL" value="postgres" />
            <el-option label="Oracle" value="oracle" />
          </el-select>
        </div>

        <el-radio-group v-model="inputMode" size="small">
          <el-radio-button label="text">粘贴 SQL 文本</el-radio-button>
          <el-radio-button label="file">上传 .sql 文件</el-radio-button>
        </el-radio-group>
      </div>

      <!-- Text Mode -->
      <div v-if="inputMode === 'text'" class="input-area">
        <el-input
          v-model="sqlText"
          type="textarea"
          :rows="8"
          placeholder="在此粘贴 CREATE TABLE 及 COMMENT 语句...
例如 (达梦 DM8)：
CREATE TABLE &quot;SYS_USER&quot; (
    &quot;USER_ID&quot; NUMBER(10, 0) NOT NULL,
    &quot;USER_NAME&quot; VARCHAR2(50) NOT NULL,
    &quot;CREATE_TIME&quot; DATETIME
);
COMMENT ON TABLE &quot;SYS_USER&quot; IS '系统用户表';
COMMENT ON COLUMN &quot;SYS_USER&quot;.&quot;USER_ID&quot; IS '用户ID';"
          class="sql-textarea"
        />
        <div class="textarea-actions">
          <el-button type="default" size="small" @click="loadSampleSql">加载达梦示例 SQL</el-button>
          <el-button type="primary" size="small" :loading="parsing" @click="handleParseText">
            解析 DDL
          </el-button>
        </div>
      </div>

      <!-- File Mode -->
      <div v-else class="upload-area">
        <el-upload
          drag
          action="#"
          :auto-upload="false"
          :show-file-list="false"
          accept=".sql,.txt"
          :on-change="handleFileChange"
        >
          <el-icon class="el-icon--upload"><UploadFilled /></el-icon>
          <div class="el-upload__text">
            将 <em>.sql 文件</em> 拖到此处，或 <em>点击上传</em>
          </div>
          <template #tip>
            <div class="el-upload__tip">
              支持达梦、MySQL、PostgreSQL 导出的建表 DDL 文件 (UTF-8 编码)
            </div>
          </template>
        </el-upload>
      </div>

      <!-- Parsed Preview -->
      <div v-if="parsedTables.length > 0" class="preview-section">
        <div class="preview-header">
          <div class="preview-title-box">
            <span class="preview-title">解析结果预览 (共 {{ parsedTables.length }} 张表)</span>
            <span class="preview-sub-tip">已在画布中的表已默认勾选，取消勾选将同步删除画布表及关联映射</span>
          </div>
          <el-button
            type="primary"
            size="small"
            :loading="importing"
            @click="handleImportSelected"
          >
            {{ newTablesCount > 0 ? `确认导入新表 (${newTablesCount})` : `确认增量同步 (${selectedTables.length})` }}
          </el-button>
        </div>

        <el-table
          ref="tableRef"
          :data="parsedTables"
          border
          size="small"
          max-height="260"
          row-key="table_name"
          @select="handleRowSelect"
          @select-all="handleSelectAll"
        >
          <el-table-column type="selection" width="45" align="center" />
          <el-table-column prop="table_name" label="表英文名" min-width="140" />
          <el-table-column prop="table_comment" label="表中文注释" min-width="150">
            <template #default="{ row }">
              <span v-if="row.table_comment">{{ row.table_comment }}</span>
              <span v-else class="text-gray">无注释</span>
            </template>
          </el-table-column>
          <el-table-column label="包含字段数" width="90" align="center">
            <template #default="{ row }">
              <el-tag size="small" type="success">{{ row.columns?.length || 0 }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="当前画布状态" width="110" align="center">
            <template #default="{ row }">
              <el-tag v-if="isTableInCanvas(row.table_name)" type="success" size="small" effect="light">
                已在画布
              </el-tag>
              <el-tag v-else type="info" size="small" effect="plain">
                未添加
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="字段概览" min-width="180" show-overflow-tooltip>
            <template #default="{ row }">
              {{ row.columns?.map((c: any) => c.column_name).join(', ') }}
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { api } from '@/api'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus'
import { DocumentAdd, UploadFilled } from '@element-plus/icons-vue'
import type { ParsedTable } from '@/types'

const visible = ref(false)
const dialect = ref('dm')
const inputMode = ref<'text' | 'file'>('text')
const sqlText = ref('')
const parsing = ref(false)
const importing = ref(false)
const parsedTables = ref<ParsedTable[]>([])
const selectedTables = ref<ParsedTable[]>([])
const tableRef = ref<any>(null)

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

// Check if table is currently on canvas
const isTableInCanvas = (tableName: string): boolean => {
  if (!projectStore.currentProject?.tables) return false
  return projectStore.currentProject.tables.some(
    t => t.table_name.toUpperCase() === tableName.toUpperCase()
  )
}

const newTablesCount = computed(() => {
  return selectedTables.value.filter(t => !isTableInCanvas(t.table_name)).length
})

const open = () => {
  visible.value = true
  if (!sqlText.value) {
    loadSampleSql()
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
  nextTick(() => {
    tableRef.value?.clearSelection()
    const preselected: ParsedTable[] = []
    list.forEach(row => {
      // If table is on canvas, pre-check it
      if (isTableInCanvas(row.table_name)) {
        tableRef.value?.toggleRowSelection(row, true)
        preselected.push(row)
      }
    })
    // If none are on canvas yet, select all by default
    if (preselected.length === 0) {
      list.forEach(row => {
        tableRef.value?.toggleRowSelection(row, true)
      })
      selectedTables.value = [...list]
    } else {
      selectedTables.value = preselected
    }
  })
}

const handleParseText = async () => {
  if (!sqlText.value.trim()) {
    ElMessage.warning('请输入 SQL 语句')
    return
  }
  parsing.value = true
  try {
    const list = await api.parseDdlText({
      sql_text: sqlText.value,
      dialect: dialect.value,
    })
    if (list.length === 0) {
      ElMessage.warning('未能识别出 CREATE TABLE 语句，请检查 SQL 语法')
    } else {
      syncTableSelections(list)
      ElMessage.success(`成功解析出 ${list.length} 张表`)
    }
  } catch (err: any) {
    ElMessage.error('解析失败: ' + (err.message || '网络错误'))
  } finally {
    parsing.value = false
  }
}

const handleFileChange = async (file: any) => {
  if (!file.raw) return
  parsing.value = true
  try {
    const formData = new FormData()
    formData.append('file', file.raw)
    formData.append('dialect', dialect.value)
    const list = await api.parseDdlFile(formData)
    if (list.length === 0) {
      ElMessage.warning('未能识别出 CREATE TABLE 语句')
    } else {
      syncTableSelections(list)
      ElMessage.success(`成功解析出 ${list.length} 张表`)
    }
  } catch (err: any) {
    ElMessage.error('文件解析失败: ' + (err.message || '网络错误'))
  } finally {
    parsing.value = false
  }
}

// Handle unchecking an individual row with confirmation
const handleRowSelect = async (selection: ParsedTable[], row: ParsedTable) => {
  const isChecked = selection.some(item => item.table_name.toUpperCase() === row.table_name.toUpperCase())

  if (!isChecked) {
    // User UNCHECKED row: Check if table is currently on canvas
    const existingTable = projectStore.currentProject?.tables?.find(
      t => t.table_name.toUpperCase() === row.table_name.toUpperCase()
    )

    if (existingTable) {
      try {
        await ElMessageBox.confirm(
          `数据表「${row.table_name}」已存在于当前项目画布中。\n\n取消勾选将从画布中同步删除该表及其所有的字段映射连线清单，确定要删除吗？`,
          '同步删除确认',
          {
            confirmButtonText: '确定删除',
            cancelButtonText: '取消',
            type: 'warning',
            distinguishCancelAndClose: true,
          }
        )

        // Confirmed deletion
        await api.deleteTable(existingTable.id)
        ElMessage.success(`已从画布同步删除表「${row.table_name}」及关联映射`)

        if (projectStore.currentProject) {
          await projectStore.selectProject(projectStore.currentProject.id)
          canvasStore.syncFromProject(
            projectStore.currentProject.tables,
            projectStore.currentProject.mappings
          )
        }
        selectedTables.value = selection
      } catch (action) {
        // Cancelled: Revert checkbox selection!
        tableRef.value?.toggleRowSelection(row, true)
        if (!selectedTables.value.some(item => item.table_name.toUpperCase() === row.table_name.toUpperCase())) {
          selectedTables.value.push(row)
        }
      }
      return
    }
  }

  selectedTables.value = selection
}

// Handle unchecking all with confirmation
const handleSelectAll = async (selection: ParsedTable[]) => {
  const isAllChecked = selection.length > 0
  if (!isAllChecked) {
    const existingTablesInList = parsedTables.value.filter(row => isTableInCanvas(row.table_name))
    if (existingTablesInList.length > 0) {
      try {
        await ElMessageBox.confirm(
          `取消全选包含 ${existingTablesInList.length} 张已存在于画布中的数据表。\n\n取消全选将从画布中同步删除这些表及其关联连线，确定要全部删除吗？`,
          '批量同步删除确认',
          {
            confirmButtonText: '确定全部删除',
            cancelButtonText: '取消',
            type: 'warning',
          }
        )

        for (const row of existingTablesInList) {
          const t = projectStore.currentProject?.tables?.find(
            et => et.table_name.toUpperCase() === row.table_name.toUpperCase()
          )
          if (t) {
            await api.deleteTable(t.id)
          }
        }
        ElMessage.success(`已从画布批量删除 ${existingTablesInList.length} 张表`)

        if (projectStore.currentProject) {
          await projectStore.selectProject(projectStore.currentProject.id)
          canvasStore.syncFromProject(
            projectStore.currentProject.tables,
            projectStore.currentProject.mappings
          )
        }
        selectedTables.value = []
      } catch {
        // Cancelled: revert
        nextTick(() => {
          existingTablesInList.forEach(row => {
            tableRef.value?.toggleRowSelection(row, true)
          })
          selectedTables.value = existingTablesInList
        })
      }
      return
    }
  }
  selectedTables.value = selection
}

const handleImportSelected = async () => {
  if (selectedTables.value.length === 0) {
    ElMessage.warning('请勾选至少一张表')
    return
  }
  if (!projectStore.currentProject) {
    ElMessage.warning('请先选择或创建一个项目')
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
      ElNotification.success({
        title: '表结构同步完成',
        message: res.summary.message,
        duration: 6000,
      })
    } else {
      ElMessage.success('导入完成')
    }
    visible.value = false

    // Refresh project details & canvas
    await projectStore.selectProject(projectStore.currentProject.id)
    if (projectStore.currentProject) {
      if (canvasStore.currentCanvasId) {
        await canvasStore.switchCanvas(canvasStore.currentCanvasId)
        await canvasStore.fetchCanvases(projectStore.currentProject.id, false)
      } else {
        await canvasStore.fetchCanvases(projectStore.currentProject.id)
      }
    }
  } catch (err: any) {
    ElMessage.error('导入失败: ' + (err.message || '网络错误'))
  } finally {
    importing.value = false
  }
}

defineExpose({ open })
</script>

<style scoped>
.dialog-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.config-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--cl-surface-subtle);
  border-radius: 6px;
  border: 1px solid var(--cl-border);
}

.config-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.label {
  font-size: 13px;
  color: var(--cl-text-secondary);
  font-weight: 500;
}

.input-area {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sql-textarea :deep(textarea) {
  font-family: var(--cl-font-mono) !important;
  font-size: 12px;
  background: var(--cl-surface-subtle);
  color: var(--cl-text-primary);
  line-height: 1.5;
  border-radius: 6px;
  border: 1px solid var(--cl-border);
}

.textarea-actions {
  display: flex;
  justify-content: space-between;
}

.preview-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 6px;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.preview-title-box {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.preview-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--cl-text-primary);
}

.preview-sub-tip {
  font-size: 11px;
  color: var(--cl-text-muted);
}

.text-gray {
  color: var(--cl-text-faint);
  font-size: 12px;
}
</style>
