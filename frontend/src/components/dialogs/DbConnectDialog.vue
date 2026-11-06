<template>
  <el-dialog
    v-model="visible"
    width="820px"
    destroy-on-close
    :close-on-click-modal="false"
  >
    <template #header>
      <div style="display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 16px; color: var(--cl-text-primary);">
        <el-icon color="#3b82f6"><Connection /></el-icon>
        <span>{{ step === 'config' ? '连接数据库导入 - 数据库配置' : '连接数据库导入 - 选择导入数据表' }}</span>
      </div>
    </template>
    <div class="dialog-body">
      <!-- Step 1: Connection Form (hidden once tables are fetched) -->
      <div v-if="step === 'config'" class="config-step">
        <!-- Connection Profiles Header -->
        <div class="mb-4 p-2.5 rounded-lg bg-surface-subtle border border-border flex items-center justify-between gap-3">
          <div class="flex items-center gap-2 flex-1 min-w-0">
            <span class="text-xs text-muted shrink-0 font-medium">常用配置预设:</span>
            <el-select
              v-model="selectedProfileId"
              placeholder="选择已保存的连接配置或新建..."
              size="small"
              class="flex-1"
              @change="onProfileSelect"
            >
              <el-option label="+ 新建连接配置" value="__new__" />
              <el-option
                v-for="p in savedProfiles"
                :key="p.id"
                :label="p.name"
                :value="p.id"
              />
            </el-select>
          </div>
          <div class="flex items-center gap-1.5 shrink-0">
            <el-button size="small" text type="primary" @click="handleSaveProfile">
              保存当前配置
            </el-button>
            <el-button
              v-if="selectedProfileId && selectedProfileId !== '__new__'"
              size="small"
              text
              type="danger"
              @click="handleDeleteProfile"
            >
              删除
            </el-button>
          </div>
        </div>

        <el-form :model="form" label-width="100px" size="default">
          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="配置名称">
                <el-input v-model="profileName" placeholder="例如: 本地DM8开发库" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="数据库类型">
                <el-select v-model="form.db_type" style="width: 100%" @change="onDbTypeChange">
                  <el-option label="达梦数据库 (DM8/DM7)" value="dm" />
                  <el-option label="MySQL" value="mysql" />
                  <el-option label="PostgreSQL" value="postgres" />
                </el-select>
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="主机 IP / 域名">
                <el-input v-model="form.host" placeholder="例如 127.0.0.1" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="端口号">
                <el-input-number v-model="form.port" :min="1" :max="65535" style="width: 100%" />
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item :label="form.db_type === 'dm' ? '模式名/库名' : '数据库名'">
                <el-input v-model="form.database" placeholder="例如 SYSDBA 或 DAMENG" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="用户名">
                <el-input v-model="form.username" placeholder="SYSDBA / root" />
              </el-form-item>
            </el-col>
          </el-row>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="密码">
                <el-input v-model="form.password" type="password" show-password />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="密码保存">
                <el-checkbox v-model="rememberPassword">记住密码 (保存在本地配置中)</el-checkbox>
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
          <div class="header-right" style="display: flex; align-items: center; gap: 8px;">
            <el-button
              v-if="tables.length > 0"
              size="small"
              text
              type="primary"
              @click="selectAllUnadded"
            >
              一键勾选新表
            </el-button>
            <el-input
              v-model="searchKey"
              placeholder="快速过滤表名/注释..."
              size="small"
              style="width: 180px"
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
          @expand-change="handleExpandChange"
        >
          <el-table-column type="expand" width="35">
            <template #default="{ row }">
              <div class="p-3 bg-surface-subtle border-y border-border">
                <div v-if="loadingColumns[row.table_name]" class="text-xs text-muted flex items-center gap-1.5 py-1">
                  <el-icon class="is-loading text-primary"><Loading /></el-icon>
                  <span>正在探查数据表「{{ row.table_name }}」字段元数据...</span>
                </div>
                <div v-else-if="columnsCache[row.table_name]">
                  <div class="text-[11px] font-semibold text-muted mb-2 flex items-center justify-between">
                    <span>字段定义清单 (共 {{ columnsCache[row.table_name].length }} 个字段)</span>
                  </div>
                  <div class="grid grid-cols-2 md:grid-cols-3 gap-1.5 max-h-48 overflow-y-auto pr-1">
                    <div
                      v-for="col in columnsCache[row.table_name]"
                      :key="col.column_name"
                      class="flex items-center gap-1.5 px-2 py-1 rounded bg-surface border border-border text-xs truncate"
                      :title="col.column_comment ? `${col.column_name} (${col.column_comment}) - ${col.data_type}` : `${col.column_name} - ${col.data_type}`"
                    >
                      <span v-if="col.column_comment?.includes('主键') || col.column_name.toUpperCase() === 'ID'" class="px-1 text-[9px] font-mono font-bold rounded bg-amber-500/20 text-amber-500 shrink-0 leading-none">PK</span>
                      <span class="font-mono font-medium text-primary-text truncate">{{ col.column_name }}</span>
                      <span class="font-mono text-[10px] text-muted shrink-0">{{ col.data_type }}</span>
                      <span v-if="col.column_comment" class="text-muted text-[11px] truncate">({{ col.column_comment }})</span>
                    </div>
                  </div>
                </div>
                <div v-else class="text-xs text-muted py-1 flex items-center gap-2">
                  <span>展开可实时预览字段清单与数据类型</span>
                  <el-button size="small" text type="primary" @click="fetchTableColumns(row.table_name)">
                    点击探查
                  </el-button>
                </div>
              </div>
            </template>
          </el-table-column>
          <el-table-column type="selection" width="45" align="center" />
          <el-table-column prop="table_name" label="表英文名" min-width="160" />
          <el-table-column prop="table_comment" label="表中文注释" min-width="180">
            <template #default="{ row }">
              <span v-if="row.table_comment">{{ row.table_comment }}</span>
              <span v-else class="text-gray">无注释</span>
            </template>
          </el-table-column>
          <el-table-column prop="column_count" label="字段数" width="80" align="center">
            <template #default="{ row }">
              <span v-if="row.column_count != null" class="font-mono">{{ row.column_count }}</span>
              <span v-else class="text-gray">-</span>
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
              {{ newTablesCount > 0 ? `确认导入新表 (${newTablesCount})` : `确认增量同步 (${selectedTables.length})` }}
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
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus'
import { Connection, Right, Back, Loading } from '@element-plus/icons-vue'
import type { DbTableSummary } from '@/types'

const visible = ref(false)
const emit = defineEmits(['imported'])
const step = ref<'config' | 'tables'>('config')
const testing = ref(false)
const fetchingTables = ref(false)
const importing = ref(false)
const searchKey = ref('')

const tables = ref<DbTableSummary[]>([])
const selectedTables = ref<DbTableSummary[]>([])
const tableRef = ref<any>(null)

interface DbProfile {
  id: string
  name: string
  db_type: string
  host: string
  port: number
  database: string
  username: string
  password?: string
  rememberPassword?: boolean
}

const savedProfiles = ref<DbProfile[]>([])
const selectedProfileId = ref<string>('__new__')
const profileName = ref('')
const rememberPassword = ref(true)

const loadingColumns = ref<Record<string, boolean>>({})
const columnsCache = ref<Record<string, any[]>>({})

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

const selectAllUnadded = () => {
  const unadded = tables.value.filter(t => !isTableInCanvas(t.table_name))
  unadded.forEach(row => {
    tableRef.value?.toggleRowSelection(row, true)
  })
  const existingMap = new Set(selectedTables.value.map(s => s.table_name.toUpperCase()))
  const newlyAdded = unadded.filter(u => !existingMap.has(u.table_name.toUpperCase()))
  selectedTables.value = [...selectedTables.value, ...newlyAdded]
  ElMessage.success(`已一键勾选 ${newlyAdded.length} 张未在画布中的新表`)
}

const loadProfiles = () => {
  try {
    const raw = localStorage.getItem('collink_saved_db_profiles')
    if (raw) {
      savedProfiles.value = JSON.parse(raw)
    }
  } catch (e) {
    console.error('加载连接预设失败', e)
  }
}

const saveProfilesToStorage = () => {
  try {
    localStorage.setItem('collink_saved_db_profiles', JSON.stringify(savedProfiles.value))
  } catch (e) {
    console.error('保存连接预设失败', e)
  }
}

const onProfileSelect = (val: string) => {
  if (val === '__new__') {
    profileName.value = ''
    onDbTypeChange(form.value.db_type)
    return
  }
  const found = savedProfiles.value.find(p => p.id === val)
  if (found) {
    profileName.value = found.name
    form.value.db_type = found.db_type
    form.value.host = found.host
    form.value.port = found.port
    form.value.database = found.database
    form.value.username = found.username
    form.value.password = found.password || ''
    rememberPassword.value = found.rememberPassword ?? true
  }
}

const handleSaveProfile = () => {
  const name =
    profileName.value.trim() ||
    `${getDbLabel(form.value.db_type)} ${form.value.host}:${form.value.port}/${form.value.database}`
  profileName.value = name

  const profileData: DbProfile = {
    id: selectedProfileId.value !== '__new__' ? selectedProfileId.value : `profile_${Date.now()}`,
    name,
    db_type: form.value.db_type,
    host: form.value.host,
    port: form.value.port,
    database: form.value.database,
    username: form.value.username,
    password: rememberPassword.value ? form.value.password : '',
    rememberPassword: rememberPassword.value,
  }

  const existingIdx = savedProfiles.value.findIndex(p => p.id === profileData.id)
  if (existingIdx >= 0) {
    savedProfiles.value[existingIdx] = profileData
  } else {
    savedProfiles.value.push(profileData)
  }
  selectedProfileId.value = profileData.id
  saveProfilesToStorage()
  ElMessage.success(`已保存连接配置「${name}」`)
}

const handleDeleteProfile = async () => {
  if (!selectedProfileId.value || selectedProfileId.value === '__new__') return
  try {
    await ElMessageBox.confirm('确定要删除此连接配置吗？', '删除确认', {
      type: 'warning',
    })
    savedProfiles.value = savedProfiles.value.filter(p => p.id !== selectedProfileId.value)
    selectedProfileId.value = '__new__'
    profileName.value = ''
    saveProfilesToStorage()
    ElMessage.success('已删除连接配置')
  } catch {}
}

const fetchTableColumns = async (tableName: string) => {
  if (columnsCache.value[tableName]) return
  loadingColumns.value[tableName] = true
  try {
    const res = await api.inspectSingleTable(form.value, tableName)
    if (res && res.columns) {
      columnsCache.value[tableName] = res.columns
    }
  } catch (err: any) {
    ElMessage.error(`探查字段失败: ${err.message || '未知错误'}`)
  } finally {
    loadingColumns.value[tableName] = false
  }
}

const handleExpandChange = (row: any, expandedRows: any[]) => {
  const isExpanded = expandedRows.some((r: any) => r.table_name === row.table_name)
  if (isExpanded && !columnsCache.value[row.table_name]) {
    fetchTableColumns(row.table_name)
  }
}

const getStorageKey = () => {
  const projId = projectStore.currentProject?.id || 'default'
  return `dfm_db_conn_${projId}`
}

const open = async () => {
  visible.value = true
  searchKey.value = ''
  columnsCache.value = {}
  loadProfiles()

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

  importing.value = true
  try {
    const tableNames = selectedTables.value.map(t => t.table_name)
    const res = await api.importDbTables(
      projectStore.currentProject.id,
      form.value,
      tableNames,
      canvasStore.currentCanvasId || undefined
    )

    if (res.summary?.message) {
      ElNotification.success({
        title: '数据库表结构同步完成',
        message: res.summary.message,
        duration: 6000,
      })
    } else {
      ElMessage.success('导入完成')
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

:deep(html.dark) .conn-info-banner,
html.dark .conn-info-banner {
  background: rgba(16, 185, 129, 0.12) !important;
  border-color: rgba(16, 185, 129, 0.3) !important;
}

:deep(html.dark) .conn-text,
html.dark .conn-text {
  color: #34d399 !important;
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
  color: var(--cl-text-primary);
}

.sub-tip {
  font-size: 11px;
  color: var(--cl-text-muted);
}

.tables-bottom-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 6px;
}

.selected-summary {
  font-size: 13px;
  color: var(--cl-text-secondary);
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
