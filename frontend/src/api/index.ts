import { invoke } from '@tauri-apps/api/core'
import type {
  Project,
  TableMeta,
  FieldMapping,
  ParsedTable,
  DbTableSummary,
  CanvasView,
  CanvasDetail,
  RelationPathResult,
} from '@/types'
import { toast } from '@/utils/toast'

export const api = {
  // Projects
  getProjects: () => invoke<Project[]>('get_projects'),
  getProjectDetail: (id: string) => invoke<Project>('get_project_detail', { id }),
  createProject: (data: { name: string; description?: string }) =>
    invoke<Project>('create_project', { input: data }),
  updateProject: (id: string, data: { name?: string; description?: string }) =>
    invoke<Project>('update_project', { id, input: data }),
  deleteProject: (id: string) => invoke<void>('delete_project', { id }),

  // Canvases (Subject Views)
  getCanvases: (projectId: string) => invoke<CanvasView[]>('get_canvases', { projectId }),
  getCanvasDetail: (canvasId: string) => invoke<CanvasDetail>('get_canvas_detail', { canvasId }),
  createCanvas: (projectId: string, data: { name: string }) =>
    invoke<CanvasView>('create_canvas', { projectId, input: data }),
  updateCanvas: (canvasId: string, data: { name?: string; sort_order?: number }) =>
    invoke<CanvasView>('update_canvas', { canvasId, input: data }),
  deleteCanvas: (canvasId: string) => invoke<void>('delete_canvas', { canvasId }),
  addTablesToCanvas: (canvasId: string, tableIds: string[]) =>
    invoke<void>('add_tables_to_canvas', { canvasId, tableIds }),
  removeTableFromCanvas: (canvasId: string, tableId: string) =>
    invoke<void>('remove_table_from_canvas', { canvasId, tableId }),
  updateCanvasPositions: (
    canvasId: string,
    positions: { id: string; pos_x: number; pos_y: number }[]
  ) => invoke<void>('update_canvas_positions', { canvasId, positions }),
  updateCanvasTableColor: (canvasId: string, tableId: string, colorTag: string) =>
    invoke<void>('update_canvas_table_color', { canvasId, tableId, colorTag }),

  // Tables
  createTable: (projectId: string, data: any) =>
    invoke<TableMeta>('create_table', { projectId, input: data }),
  updateTable: (
    id: string,
    data: { table_name?: string; table_comment?: string; pos_x?: number; pos_y?: number }
  ) => invoke<TableMeta>('update_table', { id, input: data }),
  updateTablePositions: (positions: { id: string; pos_x: number; pos_y: number }[]) =>
    invoke<void>('update_table_positions', { positions }),
  deleteTable: (id: string) => invoke<void>('delete_table', { id }),

  // Mappings
  getMappings: (projectId: string) => invoke<FieldMapping[]>('get_mappings', { projectId }),
  createMapping: (
    projectId: string,
    data: {
      source_table_id: string
      source_column_id: string
      target_table_id: string
      target_column_id: string
    }
  ) => invoke<FieldMapping>('create_mapping', { projectId, input: data }),
  batchCreateMappings: (
    projectId: string,
    mappings: Array<{
      source_table_id: string
      source_column_id: string
      target_table_id: string
      target_column_id: string
    }>
  ) => invoke<FieldMapping[]>('batch_create_mappings', { projectId, mappings }),
  deleteMapping: (id: string) => invoke<void>('delete_mapping', { id }),
  batchDeleteMappings: (ids: string[]) =>
    invoke<{ deleted_count: number }>('batch_delete_mappings', { ids }),
  discoverRelationPath: (
    projectId: string,
    data: { source_table_id: string; target_table_id: string; max_hops?: number }
  ) => invoke<RelationPathResult>('discover_relation_path', { projectId, input: data }),

  // DDL
  parseDdlText: (data: { sql_text: string; dialect?: string }) =>
    invoke<ParsedTable[]>('parse_ddl_text', { input: data }),
  parseDdlFile: async (formData: FormData): Promise<ParsedTable[]> => {
    const file = formData.get('file') as File | null
    const dialect = formData.get('dialect') as string | null
    const content = file ? await file.text() : ''
    return invoke<ParsedTable[]>('parse_ddl_file', {
      content,
      dialect: dialect || undefined,
    })
  },
  importDdlTables: (projectId: string, tables: ParsedTable[], canvasId?: string) =>
    invoke<{
      tables: TableMeta[]
      summary: {
        tables_created: number
        tables_updated: number
        columns_added: number
        columns_updated: number
        mappings_preserved: number
        message: string
      }
    }>('import_ddl_tables', {
      projectId,
      tables,
      canvasId: canvasId || undefined,
    }),

  // DB Connection
  testDbConnection: (config: any) =>
    invoke<{ success: boolean; message: string }>('test_db_connection', { config }),
  listDbTables: (config: any) =>
    invoke<DbTableSummary[]>('list_db_tables', { config }),
  inspectSingleTable: (config: any, tableName: string) =>
    invoke<ParsedTable>('inspect_single_table', { config, tableName }),
  importDbTables: (
    projectId: string,
    config: any,
    selectedTables: string[],
    canvasId?: string
  ) =>
    invoke<{
      tables: TableMeta[]
      summary: {
        tables_created: number
        tables_updated: number
        columns_added: number
        columns_updated: number
        mappings_preserved: number
        message: string
      }
    }>('import_db_tables', {
      projectId,
      canvasId: canvasId || undefined,
      config,
      selectedTables,
    }),

  // Export / Backup
  exportExcel: async (projectId: string, projectName: string = '项目'): Promise<void> => {
    const toastId = toast.loading('正在准备导出 Excel 文件...')
    try {
      const dataUrl = await invoke<string>('export_excel', {
        projectId,
        projectName,
      })
      const dateStr = new Date().toISOString().slice(0, 10)
      const fileName = `${projectName}_字段血缘映射表_${dateStr}.xlsx`

      const link = document.createElement('a')
      link.style.display = 'none'
      link.href = dataUrl
      link.setAttribute('download', fileName)
      document.body.appendChild(link)
      link.click()

      setTimeout(() => {
        document.body.removeChild(link)
      }, 300)

      toast.success('Excel 导出成功', { id: toastId })
    } catch (err: any) {
      toast.error('导出失败: ' + (err || '未知异常'), { id: toastId })
      throw err
    }
  },

  exportProjectJson: async (projectId: string, projectName: string = '项目'): Promise<void> => {
    const toastId = toast.loading('正在生成 JSON 备份快照...')
    try {
      const jsonStr = await invoke<string>('export_project_json', { projectId })
      const dateStr = new Date().toISOString().slice(0, 10)
      const fileName = `${projectName}_备份快照_${dateStr}.json`

      const blob = new Blob([jsonStr], { type: 'application/json;charset=utf-8' })
      const blobUrl = window.URL.createObjectURL(blob)

      const link = document.createElement('a')
      link.style.display = 'none'
      link.href = blobUrl
      link.setAttribute('download', fileName)
      document.body.appendChild(link)
      link.click()

      setTimeout(() => {
        document.body.removeChild(link)
        window.URL.revokeObjectURL(blobUrl)
      }, 300)

      toast.success('JSON 备份快照导出成功', { id: toastId })
    } catch (err: any) {
      toast.error('导出失败: ' + (err || '未知异常'), { id: toastId })
      throw err
    }
  },

  getExcelExportUrl: (projectId: string) => `/export/excel/${projectId}`,
  getJsonExportUrl: (projectId: string) => `/export/json/${projectId}`,

  restoreProjectJson: async (formData: FormData): Promise<Project> => {
    const file = formData.get('file') as File | null
    if (!file) {
      throw new Error('未选择备份文件')
    }
    const jsonContent = await file.text()
    return invoke<Project>('restore_project_json', { jsonContent })
  },

  // System & Paths
  getAppInfo: () =>
    invoke<{ db_path: string; mcp_path: string; app_version: string }>('get_app_info'),
  openPathInExplorer: (path: string) => invoke<void>('open_path_in_explorer', { path }),
}
