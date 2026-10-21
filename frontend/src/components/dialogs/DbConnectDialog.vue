<template>
  <el-dialog
    v-model="visible"
    width="820px"
    destroy-on-close
    :close-on-click-modal="false"
  >
    <template #header>
      <div style="display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 16px; color: #1e293b;">
        <el-icon color="#3b82f6"><Connection /></el-icon>
        <span>{{ step === 'config' ? '在线数据库逆向拉取 - 数据库配置' : '在线数据库逆向拉取 - 选择导入数据表' }}</span>
      </div>
    </template>
    <div class="dialog-body">
      <!-- Step 1: Connection Form (hidden once tables are fetched) -->
      <div v-if="step === 'config'" class="config-step">
        <el-form :model="form" label-width="100px" size="default">
          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="数据库类型">
                <el-select v-model="form.db_type" style="width: 100%" @change="onDbTypeChange">
                  <el-option label="达梦数据库 (DM8/DM7)" value="dm" />
                  <el-option label="MySQL" value="mysql" />
                  <el-option label="PostgreSQL" value="postgres" />
                </el-select>
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="主机 IP / 域名">
                <el-input v-model="form.host" placeholder="例如 127.0.0.1" />
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="端口号">
                <el-input-number v-model="form.port" :min="1" :max="65535" style="width: 100%" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item :label="form.db_type === 'dm' ? '模式名/库名' : '数据库名'">
                <el-input v-model="form.database" placeholder="例如 SYSDBA 或 DAMENG" />
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="用户名">
                <el-input v-model="form.username" placeholder="SYSDBA / root" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="密码">
                <el-input v-model="form.password" type="password" show-password />
              </el-form-item>
            </el-col>
          </el-row>

          <div class="form-actions">
            <el-button :loading="testing" @click="handleTestConnection">测试连接</el-button>
            <el-button type="primary" :loading="fetchingTables" @click="handleFetchTables">
              下一步：获取表清单 <el-icon class="el-icon--right"><Right /></el-icon>
            </el-button>
          </div>
        </el-form>
      </div>

      <!-- Step 2: Table Selection List (Only displayed when step === 'tables') -->
      <div v-else class="tables-step">
        <!-- Top Connected Info Banner -->
        <div class="conn-info-banner">
          <div class="conn-info-left">
            <el-tag type="success" size="small" effect="dark">已连接</el-tag>
            <span class="conn-text">
              <strong>{{ getDbLabel(form.db_type) }}</strong>: {{ form.host }}:{{ form.port }} / {{ form.database }} (用户: {{ form.username }})
            </span>
          </div>
          <el-button size="small" plain :icon="Back" @click="step = 'config'">
            返回修改配置
          </el-button>
        </div>

        <div class="table-list-header">
          <div class="header-left-title">
            <span class="title">发现数据表 (共 {{ tables.length }} 张)</span>
            <span class="sub-tip">已在画布中的表已自动勾选，取消勾选将同步删除画布表及关联清单</span>
          </div>
          <div class="header-right">
            <el-input
              v-model="searchKey"
              placeholder="快速过滤表名/注释..."
              size="small"
              style="width: 200px"
              clearable
            />
          </div>
        </div>

        <el-table
          ref="tableRef"
          v-loading="fetchingTables"
          element-loading-text="正在连接数据库拉取最新表结构..."
          :data="filteredTables"
          border
          size="small"
          max-height="340"
          row-key="table_name"
          @select="handleRowSelect"
          @select-all="handleSelectAll"
        >
          <el-table-column type="selection" width="45" align="center" />
          <el-table-column prop="table_name" label="表英文名" min-width="160" />
          <el-table-column prop="table_comment" label="表中文注释" min-width="180">
            <template #default="{ row }">
              <span v-if="row.table_comment">{{ row.table_comment }}</span>
              <span v-else class="text-gray">无注释</span>
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
        </el-table>

        <div class="tables-bottom-bar">
          <div class="selected-summary">
            已勾选 <span class="highlight-num">{{ selectedTables.length }}</span> / {{ tables.length }} 张表
            <span v-if="newTablesCount > 0" class="new-tables-hint">
              (其中 <strong style="color: #10b981;">{{ newTablesCount }}</strong> 张新表待导入)
            </span>
          </div>
          <div class="bottom-actions">
            <el-button @click="visible = false">关闭</el-button>
            <el-button
              type="primary"
              :loading="importing"
              :disabled="selectedTables.length === 0"
              @click="handleImportTables"
            >
              {{ newTablesCount > 0 ? `确认增量导入新表 (${newTablesCount})` : '确认完成' }}
            </el-button>
          </div>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { api } from '@/api'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Connection, Right, Back } from '@element-plus/icons-vue'
import type { DbTableSummary } from '@/types'

const visible = ref(false)
const step = ref<'config' | 'tables'>('config')
const testing = ref(false)
const fetchingTables = ref(false)
const importing = ref(false)
const searchKey = ref('')

const tables = ref<DbTableSummary[]>([])
const selectedTables = ref<DbTableSummary[]>([])
const tableRef = ref<any>(null)

const projectStore = useProjectStore()
const canvasStore = useCanvasStore()

const form = ref({
  db_type: 'dm',
  host: '127.0.0.1',
  port: 5236,
  database: 'SYSDBA',
  schema_name: '',
  username: 'SYSDBA',
  password: '',
})

const getDbLabel = (type: string) => {
  if (type === 'dm') return '达梦数据库 (DM)'
  if (type === 'mysql') return 'MySQL'
  if (type === 'postgres') return 'PostgreSQL'
  return type
}

const onDbTypeChange = (val: string) => {
  if (val === 'dm') {
    form.value.port = 5236
    form.value.username = 'SYSDBA'
    form.value.database = 'SYSDBA'
  } else if (val === 'mysql') {
    form.value.port = 3306
    form.value.username = 'root'
    form.value.database = 'test'
  } else if (val === 'postgres') {
    form.value.port = 5432
    form.value.username = 'postgres'
    form.value.database = 'postgres'
  }
}

// Check if a table is currently placed on the canvas
const isTableInCanvas = (tableName: string): boolean => {
  if (!projectStore.currentProject?.tables) return false
  return projectStore.currentProject.tables.some(
    t => t.table_name.toUpperCase() === tableName.toUpperCase()
  )
}

const newTablesCount = computed(() => {
  return selectedTables.value.filter(t => !isTableInCanvas(t.table_name)).length
})

const getStorageKey = () => {
  const projId = projectStore.currentProject?.id || 'default'
  return `dfm_db_conn_${projId}`
}

const open = async () => {
  visible.value = true
  searchKey.value = ''

  const saved = localStorage.getItem(getStorageKey())
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      form.value = { ...form.value, ...parsed }
      // Auto enter tables step directly!
      step.value = 'tables'
      await handleFetchTables()
      return
    } catch (e) {
      console.error('解析已保存的数据库配置失败', e)
    }
  }

  // Fallback to config step if no saved connection
  step.value = 'config'
}

const handleTestConnection = async () => {
  testing.value = true
  try {
    const res = await api.testDbConnection(form.value)
    if (res.success) {
      ElMessage.success(res.message)
    } else {
      ElMessage.warning(res.message)
    }
  } catch (err: any) {
    ElMessage.error('连接异常: ' + (err.message || '网络错误'))
  } finally {
    testing.value = false
  }
}

const handleFetchTables = async () => {
  fetchingTables.value = true
  try {
    const list = await api.listDbTables(form.value)
    if (!list || list.length === 0) {
      ElMessage.warning('连接成功，但在当前模式下未查询到数据表')
    } else {
      tables.value = list
      step.value = 'tables'
      // Remember this successful connection configuration for this project
      try {
        localStorage.setItem(getStorageKey(), JSON.stringify(form.value))
      } catch (e) {
        console.error('保存数据库配置失败', e)
      }

      ElMessage.success(`成功获取到 ${list.length} 张数据表`)

      // Auto check tables that are already in canvas
      nextTick(() => {
        tableRef.value?.clearSelection()
        const preselected: DbTableSummary[] = []
        tables.value.forEach(row => {
          if (isTableInCanvas(row.table_name)) {
            tableRef.value?.toggleRowSelection(row, true)
            preselected.push(row)
          }
        })
        selectedTables.value = preselected
      })
    }
  } catch (err: any) {
    const errorMsg = err.response?.data?.detail || err.message || '获取表失败'
    ElMessage.error(`连接数据库失败: ${errorMsg}，已自动切回配置页面`)
    step.value = 'config'
  } finally {
    fetchingTables.value = false
  }
}

const filteredTables = computed(() => {
  if (!searchKey.value.trim()) return tables.value
  const kw = searchKey.value.trim().toLowerCase()
  return tables.value.filter(t =>
    t.table_name.toLowerCase().includes(kw) ||
    (t.table_comment && t.table_comment.toLowerCase().includes(kw))
  )
})

// Triggered when user checks or unchecks an individual row
const handleRowSelect = async (selection: DbTableSummary[], row: DbTableSummary) => {
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

        // Confirmed deletion: delete from backend & canvas
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
        // Cancelled deletion: Revert checkbox selection!
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

// Triggered when user clicks the "Select All" header checkbox
const handleSelectAll = async (selection: DbTableSummary[]) => {
  const isAllChecked = selection.length > 0
  if (!isAllChecked) {
    // User unselected all: find tables in the list that exist on canvas
    const existingTablesInList = tables.value.filter(row => isTableInCanvas(row.table_name))
    if (existingTablesInList.length > 0) {
      try {
        await ElMessageBox.confirm(
          `取消全选涉及 ${existingTablesInList.length} 张已在画布中的数据表。\n\n取消全选将从画布中同步删除这些表及其关联连线，确定要全部删除吗？`,
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
        // Cancelled: keep the canvas tables selected
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

const handleImportTables = async () => {
  if (selectedTables.value.length === 0) {
    ElMessage.warning('请至少选择一张表')
    return
  }
  if (!projectStore.currentProject) {
    ElMessage.warning('请先选择或创建一个项目')
    return
  }

  // Filter only tables that are NOT yet on canvas
  const newTables = selectedTables.value.filter(
    t => !isTableInCanvas(t.table_name)
  )

  if (newTables.length === 0) {
    ElMessage.info('所选数据表均已在画布中，无需重复导入')
    visible.value = false
    return
  }

  importing.value = true
  try {
    const tableNames = newTables.map(t => t.table_name)
    await api.importDbTables(
      projectStore.currentProject.id,
      form.value,
      tableNames,
      canvasStore.currentCanvasId || undefined
    )
    ElMessage.success(`已成功增量导入 ${tableNames.length} 张新表到画布`)
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
  } catch (err: any) {
    ElMessage.error(err.response?.data?.detail || err.message || '导入失败')
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
  gap: 16px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 6px;
}

.tables-step {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.conn-info-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  padding: 8px 14px;
  border-radius: 6px;
}

.conn-info-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.conn-text {
  font-size: 13px;
  color: #166534;
}

.table-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left-title {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.title {
  font-size: 13px;
  font-weight: 600;
  color: #1e293b;
}

.sub-tip {
  font-size: 11px;
  color: #64748b;
}

.tables-bottom-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 6px;
}

.selected-summary {
  font-size: 13px;
  color: #64748b;
}

.highlight-num {
  font-weight: 600;
  color: #3b82f6;
}

.new-tables-hint {
  margin-left: 8px;
  font-size: 12px;
}

.bottom-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.text-gray {
  color: #94a3b8;
  font-size: 12px;
}
</style>
