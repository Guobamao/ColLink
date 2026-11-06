<template>
  <Dialog
    v-model:open="visible"
    :title="step === 'config' ? '连接数据库导入 - 数据库配置' : '连接数据库导入 - 选择导入数据表'"
    :description="step === 'config' ? '配置达梦、MySQL 或 PostgreSQL 数据库网络连接与凭据' : '在线探查数据字典并选择需要载入画布的表'"
    class="max-w-[840px]"
  >
    <div class="flex flex-col gap-3 py-1">
      <!-- Step 1: Connection Form (hidden once tables are fetched) -->
      <div v-if="step === 'config'" class="flex flex-col gap-3.5">
        <!-- Connection Profiles Header -->
        <div class="p-2.5 rounded-lg bg-surface-subtle border border-border flex items-center justify-between gap-3">
          <div class="flex items-center gap-2 flex-1 min-w-0">
            <span class="text-xs text-muted shrink-0 font-medium">常用配置预设:</span>
            <Select
              v-model="selectedProfileId"
              size="sm"
              wrapper-class="flex-1"
              :options="profileSelectOptions"
              @change="onProfileSelect"
            />
          </div>
          <div class="flex items-center gap-1.5 shrink-0">
            <Button size="xs" variant="ghost" class="text-primary hover:text-primary" @click="handleSaveProfile">
              保存当前配置
            </Button>
            <Button
              v-if="selectedProfileId && selectedProfileId !== '__new__'"
              size="xs"
              variant="ghost"
              class="text-red-500 hover:text-red-600"
              @click="handleDeleteProfile"
            >
              删除
            </Button>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3.5">
          <div class="space-y-1">
            <label class="text-xs font-medium text-secondary-text">配置名称</label>
            <Input v-model="profileName" placeholder="例如: 本地DM8开发库" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-medium text-secondary-text">数据库类型</label>
            <Select
              v-model="form.db_type"
              :options="dbTypeOptions"
              @change="onDbTypeChange"
            />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-medium text-secondary-text">主机 IP / 域名</label>
            <Input v-model="form.host" placeholder="例如 127.0.0.1" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-medium text-secondary-text">端口号</label>
            <Input v-model.number="form.port" type="number" placeholder="5236" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-medium text-secondary-text">
              {{ form.db_type === 'dm' ? '模式名/库名' : '数据库名' }}
            </label>
            <Input v-model="form.database" placeholder="例如 SYSDBA 或 DAMENG" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-medium text-secondary-text">用户名</label>
            <Input v-model="form.username" placeholder="SYSDBA / root" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-medium text-secondary-text">密码</label>
            <Input v-model="form.password" type="password" placeholder="••••••••" />
          </div>

          <div class="flex items-center pt-5">
            <Checkbox v-model="rememberPassword" label="记住密码 (保存在本地配置中)" />
          </div>
        </div>

        <div class="flex items-center justify-end gap-2.5 pt-2 border-t border-border mt-1">
          <Button variant="outline" :loading="testing" @click="handleTestConnection">
            测试连接
          </Button>
          <Button variant="primary" :loading="fetchingTables" @click="handleFetchTables">
            下一步：获取表清单
            <ArrowRight class="w-3.5 h-3.5 ml-1" />
          </Button>
        </div>
      </div>

      <!-- Step 2: Table Selection List (Only displayed when step === 'tables') -->
      <div v-else class="flex flex-col gap-3">
        <!-- Top Connected Info Banner -->
        <div class="flex items-center justify-between p-2 rounded-lg bg-surface-subtle border border-border">
          <div class="flex items-center gap-2">
            <Badge variant="success">已连接</Badge>
            <span class="text-xs text-primary-text">
              <strong>{{ getDbLabel(form.db_type) }}</strong>: {{ form.host }}:{{ form.port }} / {{ form.database }} (用户: {{ form.username }})
            </span>
          </div>
          <Button size="xs" variant="outline" @click="step = 'config'">
            <ArrowLeft class="w-3 h-3 mr-1" />
            返回修改配置
          </Button>
        </div>

        <div class="flex items-center justify-between">
          <div class="flex flex-col gap-0.5">
            <span class="text-xs font-semibold text-primary-text">
              发现数据表 (共 {{ tables.length }} 张)
            </span>
            <span class="text-[11px] text-muted">
              已在画布中的表已自动勾选，取消勾选将同步删除画布表及关联清单
            </span>
          </div>
          <div class="flex items-center gap-2">
            <Button
              v-if="tables.length > 0"
              size="xs"
              variant="ghost"
              class="text-primary hover:text-primary"
              @click="selectAllUnadded"
            >
              一键勾选新表
            </Button>
            <Input
              v-model="searchKey"
              placeholder="快速过滤表名/注释..."
              class="w-48"
            />
          </div>
        </div>

        <div class="border border-border rounded-lg overflow-hidden max-h-[340px] overflow-y-auto">
          <table class="w-full text-left text-xs border-collapse">
            <thead class="bg-surface-subtle sticky top-0 border-b border-border text-muted font-medium select-none z-10">
              <tr>
                <th class="w-8 px-2 py-2 text-center"></th>
                <th class="w-10 px-2 py-2 text-center">
                  <input
                    type="checkbox"
                    :checked="isAllSelected"
                    class="rounded border-border text-primary focus:ring-primary cursor-pointer"
                    @change="toggleSelectAll"
                  />
                </th>
                <th class="px-3 py-2 font-semibold">表英文名</th>
                <th class="px-3 py-2 font-semibold">表中文注释</th>
                <th class="w-20 px-3 py-2 text-center font-semibold">字段数</th>
                <th class="w-24 px-3 py-2 text-center font-semibold">当前画布状态</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border/40">
              <template v-for="row in filteredTables" :key="row.table_name">
                <tr class="hover:bg-surface-hover transition-colors">
                  <td class="px-2 py-2 text-center">
                    <button
                      type="button"
                      class="text-muted hover:text-primary p-0.5 rounded cursor-pointer"
                      @click="toggleRowExpand(row.table_name)"
                    >
                      <ChevronRight
                        class="w-3.5 h-3.5 transition-transform"
                        :class="{ 'rotate-90 text-primary': expandedRows.has(row.table_name) }"
                      />
                    </button>
                  </td>
                  <td class="px-2 py-2 text-center">
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
                  <td class="px-3 py-2 text-muted truncate max-w-[200px]">
                    {{ row.table_comment || '无注释' }}
                  </td>
                  <td class="px-3 py-2 font-mono text-center text-muted">
                    {{ row.column_count != null ? row.column_count : '-' }}
                  </td>
                  <td class="px-3 py-2 text-center">
                    <Badge v-if="isTableInCanvas(row.table_name)" variant="primary">已在画布</Badge>
                    <Badge v-else variant="secondary">未添加</Badge>
                  </td>
                </tr>
                <!-- Expanded Row for Column Preview -->
                <tr v-if="expandedRows.has(row.table_name)" class="bg-surface-subtle">
                  <td colspan="6" class="p-3 border-y border-border">
                    <div v-if="loadingColumns[row.table_name]" class="text-xs text-muted flex items-center gap-1.5 py-1">
                      <Loader2 class="w-3.5 h-3.5 animate-spin text-primary" />
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
                          <span
                            v-if="col.column_comment?.includes('主键') || col.column_name.toUpperCase() === 'ID'"
                            class="px-1 text-[9px] font-mono font-bold rounded bg-amber-500/20 text-amber-500 shrink-0 leading-none"
                          >
                            PK
                          </span>
                          <span class="font-mono font-medium text-primary-text truncate">{{ col.column_name }}</span>
                          <span class="font-mono text-[10px] text-muted shrink-0">{{ col.data_type }}</span>
                          <span v-if="col.column_comment" class="text-muted text-[11px] truncate">({{ col.column_comment }})</span>
                        </div>
                      </div>
                    </div>
                    <div v-else class="text-xs text-muted py-1 flex items-center gap-2">
                      <span>展开可实时预览字段清单与数据类型</span>
                      <Button size="xs" variant="ghost" class="text-primary hover:text-primary" @click="fetchTableColumns(row.table_name)">
                        点击探查
                      </Button>
                    </div>
                  </td>
                </tr>
              </template>
            </tbody>
          </table>
        </div>

        <div class="flex items-center justify-between pt-2 border-t border-border">
          <div class="text-xs text-muted">
            已勾选 <span class="font-mono font-semibold text-primary">{{ selectedTables.length }}</span> / {{ tables.length }} 张表
            <span v-if="newTablesCount > 0">
              (其中 <strong class="text-emerald-500">{{ newTablesCount }}</strong> 张新表待导入)
            </span>
          </div>
          <div class="flex items-center gap-2">
            <Button variant="ghost" @click="visible = false">关闭</Button>
            <Button
              variant="primary"
              :loading="importing"
              :disabled="selectedTables.length === 0"
              @click="handleImportTables"
            >
              {{ newTablesCount > 0 ? `确认导入新表 (${newTablesCount})` : `确认增量同步 (${selectedTables.length})` }}
            </Button>
          </div>
        </div>
      </div>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { api } from '@/api'
import { useProjectStore } from '@/stores/projectStore'
import { useCanvasStore } from '@/stores/canvasStore'
import { toast, confirmDialog } from '@/utils/toast'
import { Dialog, Select, Input, Checkbox, Button, Badge } from '@/components/ui'
import { ArrowRight, ArrowLeft, ChevronRight, Loader2 } from 'lucide-vue-next'
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
const expandedRows = ref<Set<string>>(new Set())

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

const dbTypeOptions = [
  { label: '达梦数据库 (DM8/DM7)', value: 'dm' },
  { label: 'MySQL', value: 'mysql' },
  { label: 'PostgreSQL', value: 'postgres' },
]

const profileSelectOptions = computed(() => {
  return [
    { label: '+ 新建连接配置', value: '__new__' },
    ...savedProfiles.value.map(p => ({ label: p.name, value: p.id })),
  ]
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
  if (tables.value.length === 0) return false
  return tables.value.every(t => isRowSelected(t.table_name))
})

const toggleRowExpand = (tableName: string) => {
  if (expandedRows.value.has(tableName)) {
    expandedRows.value.delete(tableName)
  } else {
    expandedRows.value.add(tableName)
    if (!columnsCache.value[tableName]) {
      fetchTableColumns(tableName)
    }
  }
}

const selectAllUnadded = () => {
  const unadded = tables.value.filter(t => !isTableInCanvas(t.table_name))
  const existingMap = new Set(selectedTables.value.map(s => s.table_name.toUpperCase()))
  const newlyAdded = unadded.filter(u => !existingMap.has(u.table_name.toUpperCase()))
  selectedTables.value = [...selectedTables.value, ...newlyAdded]
  toast.success(`已一键勾选 ${newlyAdded.length} 张未在画布中的新表`)
}

const toggleSelectAll = async () => {
  if (isAllSelected.value) {
    const existingTablesInList = tables.value.filter(row => isTableInCanvas(row.table_name))
    if (existingTablesInList.length > 0) {
      const confirmed = await confirmDialog({
        title: '批量同步删除确认',
        message: `取消全选涉及 ${existingTablesInList.length} 张已在画布中的数据表。\n\n取消全选将从画布中同步删除这些表及其关联连线，确定要全部删除吗？`,
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
    selectedTables.value = [...tables.value]
  }
}

const toggleRowSelection = async (row: DbTableSummary) => {
  const isChecked = isRowSelected(row.table_name)
  if (isChecked) {
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

const loadProfiles = () => {
  try {
    const raw = localStorage.getItem('collink_saved_db_profiles')
    if (raw) savedProfiles.value = JSON.parse(raw)
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
  toast.success(`已保存连接配置「${name}」`)
}

const handleDeleteProfile = async () => {
  if (!selectedProfileId.value || selectedProfileId.value === '__new__') return
  const confirmed = await confirmDialog({
    title: '删除确认',
    message: '确定要删除此连接配置吗？',
    variant: 'danger',
  })
  if (confirmed) {
    savedProfiles.value = savedProfiles.value.filter(p => p.id !== selectedProfileId.value)
    selectedProfileId.value = '__new__'
    profileName.value = ''
    saveProfilesToStorage()
    toast.success('已删除连接配置')
  }
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
    toast.error(`探查字段失败: ${err.message || '未知错误'}`)
  } finally {
    loadingColumns.value[tableName] = false
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
  expandedRows.value.clear()
  loadProfiles()

  const saved = localStorage.getItem(getStorageKey())
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      form.value = { ...form.value, ...parsed }
      step.value = 'tables'
      await handleFetchTables()
      return
    } catch (e) {
      console.error('解析已保存的数据库配置失败', e)
    }
  }
  step.value = 'config'
}

const handleTestConnection = async () => {
  testing.value = true
  try {
    const res = await api.testDbConnection(form.value)
    if (res.success) {
      toast.success(res.message)
    } else {
      toast.warning(res.message)
    }
  } catch (err: any) {
    toast.error('连接异常: ' + (err.message || '网络错误'))
  } finally {
    testing.value = false
  }
}

const handleFetchTables = async () => {
  fetchingTables.value = true
  try {
    const list = await api.listDbTables(form.value)
    if (!list || list.length === 0) {
      toast.warning('连接成功，但在当前模式下未查询到数据表')
    } else {
      tables.value = list
      step.value = 'tables'
      try {
        localStorage.setItem(getStorageKey(), JSON.stringify(form.value))
      } catch (e) {
        console.error('保存数据库配置失败', e)
      }

      toast.success(`成功获取到 ${list.length} 张数据表`)

      const preselected: DbTableSummary[] = []
      tables.value.forEach(row => {
        if (isTableInCanvas(row.table_name)) {
          preselected.push(row)
        }
      })
      selectedTables.value = preselected
    }
  } catch (err: any) {
    const errorMsg = err.response?.data?.detail || err.message || '获取表失败'
    toast.error(`连接数据库失败: ${errorMsg}，已自动切回配置页面`)
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

const handleImportTables = async () => {
  if (selectedTables.value.length === 0) {
    toast.warning('请至少选择一张表')
    return
  }
  if (!projectStore.currentProject) {
    toast.warning('请先选择或创建一个项目')
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
      toast.success('数据库表结构同步完成', { description: res.summary.message })
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
    toast.error(err.response?.data?.detail || err.message || '导入失败')
  } finally {
    importing.value = false
  }
}

defineExpose({ open })
</script>
