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
        <span class="tab-badge" :title="`当前视图包含 ${c.table_count || 0} 张表`">
          {{ c.table_count || 0 }}
        </span>

        <!-- Tab Actions: Edit / Close -->
        <span class="tab-actions" @click.stop>
          <el-icon
            class="action-icon edit-icon"
            title="重命名画布视图"
            @click="handleRename(c)"
          >
            <Edit />
          </el-icon>
          <el-icon
            v-if="canvasStore.canvases.length > 1"
            class="action-icon close-icon"
            title="关闭并删除此画布视图"
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

    <!-- Right: Pool Summary & Import Button -->
    <div class="tabs-right-actions">
      <span class="pool-stat-label">
        项目表库共 <strong>{{ projectStore.currentProject?.tables?.length || 0 }}</strong> 张表
      </span>
      <el-button
        size="small"
        class="btn-import-pool"
        @click="openImportDialog"
      >
        <el-icon><FolderAdd /></el-icon>
        <span>从表库引入</span>
      </el-button>
    </div>

    <!-- Dialog: Import Tables from Project Pool -->
    <el-dialog
      v-model="importDialogVisible"
      title="从项目表库引入数据表"
      width="680px"
      destroy-on-close
    >
      <div class="import-dialog-content">
        <p class="dialog-desc">
          将当前项目「<strong>{{ projectStore.currentProject?.name }}</strong>」中尚未加入当前视图的公共表引入画布中参与连线。
        </p>
        <div class="dialog-search-bar">
          <el-input
            v-model="searchPoolKeyword"
            placeholder="按英文表名或中文注释过滤..."
            clearable
            size="small"
            class="pool-search-input"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>

        <el-table
          ref="tablePoolRef"
          :data="filteredProjectTables"
          border
          size="small"
          max-height="320"
          row-key="id"
          @selection-change="handleSelectionChange"
        >
          <el-table-column
            type="selection"
            width="42"
            align="center"
            :selectable="(row) => !isTableInCurrentCanvas(row.id)"
          />
          <el-table-column prop="table_name" label="表英文名" min-width="160">
            <template #default="{ row }">
              <span class="pool-tbl-name">{{ row.table_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="table_comment" label="中文业务注释" min-width="160">
            <template #default="{ row }">
              <span v-if="row.table_comment" class="pool-tbl-comment">{{ row.table_comment }}</span>
              <span v-else class="text-muted">无注释</span>
            </template>
          </el-table-column>
          <el-table-column label="字段数" width="75" align="center">
            <template #default="{ row }">
              <span class="pool-col-count">{{ row.columns?.length || 0 }}</span>
            </template>
          </el-table-column>
          <el-table-column label="视图状态" width="105" align="center">
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
            已勾选 <strong style="color: var(--cl-primary)">{{ selectedTableIds.length }}</strong> 张新数据表
          </span>
          <div class="footer-btn-group">
            <el-button @click="importDialogVisible = false">取消</el-button>
            <el-button
              type="primary"
              :disabled="selectedTableIds.length === 0"
              @click="confirmImportTables"
            >
              引入所选数据表
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
  border-bottom: 1px solid var(--cl-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  box-sizing: border-box;
  user-select: none;
  z-index: 40;
}

.tabs-scroll-container {
  display: flex;
  align-items: center;
  gap: 2px;
  overflow-x: auto;
  max-width: calc(100% - 240px);
  height: 100%;
}

.tabs-scroll-container::-webkit-scrollbar {
  height: 2px;
}

.tabs-scroll-container::-webkit-scrollbar-thumb {
  background: var(--cl-border-strong);
  border-radius: 2px;
}

.canvas-tab {
  height: 37px;
  padding: 0 12px;
  background: transparent;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  color: var(--cl-text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
  border-bottom: 2px solid transparent;
  border-right: 1px solid rgba(226, 232, 240, 0.6);
  position: relative;
}

.canvas-tab:hover {
  background: rgba(241, 245, 249, 0.8);
  color: var(--cl-text-primary);
}

.canvas-tab.is-active {
  background: #ffffff;
  color: var(--cl-primary);
  font-weight: 600;
  border-bottom-color: var(--cl-primary);
}

.tab-icon {
  font-size: 13px;
}

.tab-title {
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-badge {
  font-family: var(--cl-font-mono);
  font-size: 10px;
  background: #f1f5f9;
  color: var(--cl-text-muted);
  padding: 1px 5px;
  border-radius: 4px;
  line-height: 1.2;
}

.canvas-tab.is-active .tab-badge {
  background: var(--cl-primary-light);
  color: var(--cl-primary);
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
  font-size: 11px;
  padding: 2px;
  border-radius: 3px;
  color: var(--cl-text-faint);
  transition: all 0.15s ease;
}

.action-icon:hover {
  background: #e2e8f0;
  color: var(--cl-text-primary);
}

.close-icon:hover {
  color: var(--cl-danger);
  background: var(--cl-danger-light);
}

.btn-new-tab {
  height: 24px;
  padding: 0 8px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  border: 1px dashed var(--cl-border-strong);
  border-radius: 4px;
  color: var(--cl-text-muted);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
  margin-left: 6px;
  white-space: nowrap;
}

.btn-new-tab:hover {
  background: #ffffff;
  color: var(--cl-primary);
  border-color: var(--cl-primary);
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.05);
}

.tabs-right-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.pool-stat-label {
  font-size: 11px;
  color: var(--cl-text-muted);
}

.pool-stat-label strong {
  font-family: var(--cl-font-mono);
  color: var(--cl-text-primary);
}

.btn-import-pool {
  font-size: 12px;
  height: 26px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  border-radius: 5px;
  background: #ffffff;
  border: 1px solid var(--cl-border);
  color: var(--cl-text-secondary);
}

.btn-import-pool:hover {
  background: var(--cl-surface-hover);
  color: var(--cl-primary);
  border-color: var(--cl-border-strong);
}

.import-dialog-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-desc {
  font-size: 13px;
  color: var(--cl-text-muted);
  margin: 0;
  line-height: 1.5;
}

.dialog-search-bar {
  display: flex;
  justify-content: flex-start;
}

.pool-search-input {
  width: 260px;
}

.pool-tbl-name {
  font-family: var(--cl-font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--cl-text-primary);
}

.pool-tbl-comment {
  font-size: 12px;
  color: var(--cl-text-muted);
}

.pool-col-count {
  font-family: var(--cl-font-mono);
  font-size: 11px;
  color: var(--cl-text-secondary);
}

.text-muted {
  color: var(--cl-text-faint);
  font-size: 12px;
}

.dialog-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.selected-hint {
  font-size: 12px;
  color: var(--cl-text-muted);
}

.footer-btn-group {
  display: flex;
  gap: 10px;
}
</style>
