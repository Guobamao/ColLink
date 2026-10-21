<template>
  <div class="canvas-tabs-bar">
    <!-- Left: Canvas Views Tabs List -->
    <div class="tabs-scroll-container">
      <div
        v-for="c in canvasStore.canvases"
        :key="c.id"
        class="canvas-tab"
        :class="{ 'is-active': canvasStore.currentCanvasId === c.id }"
        @click="canvasStore.switchCanvas(c.id)"
        @dblclick="handleRename(c)"
      >
        <el-icon class="tab-icon"><DataBoard /></el-icon>
        <span class="tab-title" :title="`双击重命名: ${c.name}`">{{ c.name }}</span>
        <span class="tab-badge" :title="`包含 ${c.table_count || 0} 张表`">
          {{ c.table_count || 0 }}
        </span>

        <!-- Tab Actions: Edit / Close -->
        <span class="tab-actions" @click.stop>
          <el-icon
            class="action-icon edit-icon"
            title="重命名画布"
            @click="handleRename(c)"
          >
            <Edit />
          </el-icon>
          <el-icon
            v-if="canvasStore.canvases.length > 1"
            class="action-icon close-icon"
            title="删除此画布视图"
            @click="handleDelete(c)"
          >
            <Close />
          </el-icon>
        </span>
      </div>

      <!-- + New Canvas Button -->
      <button class="btn-new-tab" title="新建业务视图画布" @click="handleCreateCanvas">
        <el-icon><Plus /></el-icon>
        <span>新建视图</span>
      </button>
    </div>

    <!-- Right: Import from Project Table Pool Button -->
    <div class="tabs-right-actions">
      <el-button
        size="small"
        type="primary"
        plain
        class="btn-import-pool"
        @click="openImportDialog"
      >
        <el-icon><FolderAdd /></el-icon>
        <span>从项目表库引入</span>
      </el-button>
    </div>

    <!-- Dialog: Import Tables from Project Pool -->
    <el-dialog
      v-model="importDialogVisible"
      title="从项目表库引入已有数据表"
      width="640px"
      destroy-on-close
    >
      <div class="import-dialog-content">
        <p class="dialog-desc">
          将当前项目「{{ projectStore.currentProject?.name }}」中已导入但尚未加入当前视图的公共表引入画布中参与连线。
        </p>
        <el-input
          v-model="searchPoolKeyword"
          placeholder="搜索表名或中文注释..."
          clearable
          size="small"
          class="pool-search-input"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>

        <el-table
          ref="tablePoolRef"
          :data="filteredProjectTables"
          border
          size="small"
          max-height="300"
          row-key="id"
          @selection-change="handleSelectionChange"
        >
          <el-table-column
            type="selection"
            width="45"
            align="center"
            :selectable="(row) => !isTableInCurrentCanvas(row.id)"
          />
          <el-table-column prop="table_name" label="表英文名" min-width="160" />
          <el-table-column prop="table_comment" label="中文注释" min-width="160">
            <template #default="{ row }">
              <span v-if="row.table_comment">{{ row.table_comment }}</span>
              <span v-else class="text-muted">无注释</span>
            </template>
          </el-table-column>
          <el-table-column label="字段数" width="70" align="center">
            <template #default="{ row }">
              {{ row.columns?.length || 0 }}
            </template>
          </el-table-column>
          <el-table-column label="视图状态" width="110" align="center">
            <template #default="{ row }">
              <el-tag v-if="isTableInCurrentCanvas(row.id)" type="info" size="small" effect="plain">
                已在视图中
              </el-tag>
              <el-tag v-else type="success" size="small" effect="light">
                可引入
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <span class="selected-hint">
            已选择 {{ selectedTableIds.length }} 张新表
          </span>
          <div>
            <el-button @click="importDialogVisible = false">取消</el-button>
            <el-button
              type="primary"
              :disabled="selectedTableIds.length === 0"
              @click="confirmImportTables"
            >
              确认引入画布
            </el-button>
          </div>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useCanvasStore } from '@/stores/canvasStore'
import { useProjectStore } from '@/stores/projectStore'
import { ElMessageBox, ElMessage } from 'element-plus'
import { DataBoard, Plus, Edit, Close, FolderAdd, Search } from '@element-plus/icons-vue'
import type { TableMeta } from '@/types'

const canvasStore = useCanvasStore()
const projectStore = useProjectStore()

const importDialogVisible = ref(false)
const searchPoolKeyword = ref('')
const selectedTableIds = ref<string[]>([])

const isTableInCurrentCanvas = (tableId: string): boolean => {
  return canvasStore.nodes.some(n => n.data?.tableId === tableId)
}

const filteredProjectTables = computed(() => {
  const list: TableMeta[] = projectStore.currentProject?.tables || []
  if (!searchPoolKeyword.value.trim()) return list
  const kw = searchPoolKeyword.value.trim().toLowerCase()
  return list.filter(t =>
    t.table_name.toLowerCase().includes(kw) ||
    (t.table_comment && t.table_comment.toLowerCase().includes(kw))
  )
})

const handleCreateCanvas = () => {
  if (!projectStore.currentProject) {
    ElMessage.warning('请先选择项目')
    return
  }

  ElMessageBox.prompt('请输入新业务视图名称（如：专项排查流、核心信贷报送）：', '新建业务视图', {
    confirmButtonText: '创建',
    cancelButtonText: '取消',
    inputPattern: /\S+/,
    inputErrorMessage: '视图名称不能为空',
  }).then(async ({ value }) => {
    if (value && value.trim()) {
      await canvasStore.createCanvas(projectStore.currentProject!.id, value.trim())
    }
  }).catch(() => {})
}

const handleRename = (canvas: any) => {
  ElMessageBox.prompt('修改业务视图名称：', '重命名视图', {
    confirmButtonText: '保存',
    cancelButtonText: '取消',
    inputValue: canvas.name,
    inputPattern: /\S+/,
    inputErrorMessage: '视图名称不能为空',
  }).then(async ({ value }) => {
    if (value && value.trim() && value.trim() !== canvas.name) {
      await canvasStore.updateCanvasName(canvas.id, value.trim())
    }
  }).catch(() => {})
}

const handleDelete = (canvas: any) => {
  if (canvasStore.canvases.length <= 1) {
    ElMessage.warning('不能删除项目中最后一个画布')
    return
  }

  ElMessageBox.confirm(
    `确定删除业务视图「${canvas.name}」吗？\n注意：此操作仅删除视图布局，不会删除项目中的底层数据表结构。`,
    '删除业务视图',
    {
      confirmButtonText: '确认删除',
      cancelButtonText: '取消',
      type: 'warning',
    }
  ).then(async () => {
    if (projectStore.currentProject) {
      await canvasStore.deleteCanvas(canvas.id, projectStore.currentProject.id)
    }
  }).catch(() => {})
}

const openImportDialog = () => {
  if (!projectStore.currentProject) {
    ElMessage.warning('请先选择项目')
    return
  }
  selectedTableIds.value = []
  searchPoolKeyword.value = ''
  importDialogVisible.value = true
}

const handleSelectionChange = (selection: TableMeta[]) => {
  selectedTableIds.value = selection
    .filter(t => !isTableInCurrentCanvas(t.id))
    .map(t => t.id)
}

const confirmImportTables = async () => {
  if (selectedTableIds.value.length === 0) return
  await canvasStore.addTablesToCurrentCanvas(selectedTableIds.value)
  importDialogVisible.value = false
}
</script>

<style scoped>
.canvas-tabs-bar {
  height: 38px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  box-sizing: border-box;
  user-select: none;
  z-index: 15;
}

.tabs-scroll-container {
  display: flex;
  align-items: center;
  gap: 4px;
  overflow-x: auto;
  max-width: calc(100% - 170px);
}

.tabs-scroll-container::-webkit-scrollbar {
  height: 3px;
}

.tabs-scroll-container::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 2px;
}

.canvas-tab {
  height: 30px;
  padding: 0 10px;
  background: transparent;
  border-radius: 6px;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #64748b;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
  border: 1px solid transparent;
}

.canvas-tab:hover {
  background: #f1f5f9;
  color: #1e293b;
}

.canvas-tab.is-active {
  background: #ffffff;
  color: #2563eb;
  font-weight: 600;
  border-color: #cbd5e1;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.05);
}

.tab-icon {
  font-size: 14px;
}

.tab-title {
  max-width: 130px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-badge {
  font-size: 10px;
  background: #f1f5f9;
  color: #64748b;
  padding: 1px 5px;
  border-radius: 10px;
  font-weight: normal;
}

.canvas-tab.is-active .tab-badge {
  background: #eff6ff;
  color: #2563eb;
  font-weight: 600;
}

.tab-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: 2px;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.canvas-tab:hover .tab-actions,
.canvas-tab.is-active .tab-actions {
  opacity: 1;
}

.action-icon {
  font-size: 12px;
  padding: 2px;
  border-radius: 3px;
  color: #94a3b8;
  transition: all 0.15s ease;
}

.action-icon:hover {
  background: #e2e8f0;
  color: #1e293b;
}

.close-icon:hover {
  color: #ef4444;
}

.btn-new-tab {
  height: 26px;
  padding: 0 8px;
  display: flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  border: 1px dashed #cbd5e1;
  border-radius: 5px;
  color: #64748b;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
  margin-left: 4px;
  white-space: nowrap;
}

.btn-new-tab:hover {
  background: #ffffff;
  color: #2563eb;
  border-color: #3b82f6;
}

.tabs-right-actions {
  display: flex;
  align-items: center;
}

.btn-import-pool {
  font-size: 12px;
  height: 28px;
  padding: 0 10px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.import-dialog-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-desc {
  font-size: 12px;
  color: #64748b;
  margin: 0;
  line-height: 1.5;
}

.pool-search-input {
  width: 240px;
}

.text-muted {
  color: #94a3b8;
  font-size: 12px;
}

.dialog-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.selected-hint {
  font-size: 12px;
  color: #64748b;
}
</style>
