import axios from 'axios'
import type { Project, TableMeta, FieldMapping, ParsedTable, DbTableSummary, CanvasView, CanvasDetail } from '@/types'

const apiClient = axios.create({
  baseURL: '/api',
  timeout: 30000,
})

export const api = {
  // Projects
  getProjects: () => apiClient.get<Project[]>('/projects').then(res => res.data),
  getProjectDetail: (id: string) => apiClient.get<Project>(`/projects/${id}`).then(res => res.data),
  createProject: (data: { name: string; description?: string }) => apiClient.post<Project>('/projects', data).then(res => res.data),
  updateProject: (id: string, data: { name?: string; description?: string }) => apiClient.put<Project>(`/projects/${id}`, data).then(res => res.data),
  deleteProject: (id: string) => apiClient.delete(`/projects/${id}`).then(res => res.data),

  // Canvases (Subject Views)
  getCanvases: (projectId: string) => apiClient.get<CanvasView[]>(`/canvases/project/${projectId}`).then(res => res.data),
  getCanvasDetail: (canvasId: string) => apiClient.get<CanvasDetail>(`/canvases/${canvasId}`).then(res => res.data),
  createCanvas: (projectId: string, data: { name: string }) => apiClient.post<CanvasView>(`/canvases/project/${projectId}`, data).then(res => res.data),
  updateCanvas: (canvasId: string, data: { name?: string; sort_order?: number }) => apiClient.put<CanvasView>(`/canvases/${canvasId}`, data).then(res => res.data),
  deleteCanvas: (canvasId: string) => apiClient.delete(`/canvases/${canvasId}`).then(res => res.data),
  addTablesToCanvas: (canvasId: string, tableIds: string[]) => apiClient.post(`/canvases/${canvasId}/tables`, { table_ids: tableIds }).then(res => res.data),
  removeTableFromCanvas: (canvasId: string, tableId: string) => apiClient.delete(`/canvases/${canvasId}/tables/${tableId}`).then(res => res.data),
  updateCanvasPositions: (canvasId: string, positions: { id: string; pos_x: number; pos_y: number }[]) => apiClient.put(`/canvases/${canvasId}/positions`, positions).then(res => res.data),
  updateCanvasTableColor: (canvasId: string, tableId: string, colorTag: string) => apiClient.put(`/canvases/${canvasId}/tables/${tableId}/color`, { color_tag: colorTag }).then(res => res.data),

  // Tables
  createTable: (projectId: string, data: any) => apiClient.post<TableMeta>(`/tables?project_id=${projectId}`, data).then(res => res.data),
  updateTable: (id: string, data: { table_name?: string; table_comment?: string; pos_x?: number; pos_y?: number }) => apiClient.put<TableMeta>(`/tables/${id}`, data).then(res => res.data),
  updateTablePositions: (positions: { id: string; pos_x: number; pos_y: number }[]) => apiClient.put('/tables/positions', positions).then(res => res.data),
  deleteTable: (id: string) => apiClient.delete(`/tables/${id}`).then(res => res.data),

  // Mappings
  getMappings: (projectId: string) => apiClient.get<FieldMapping[]>(`/mappings?project_id=${projectId}`).then(res => res.data),
  createMapping: (projectId: string, data: { source_table_id: string; source_column_id: string; target_table_id: string; target_column_id: string }) =>
    apiClient.post<FieldMapping>(`/mappings?project_id=${projectId}`, data).then(res => res.data),
  deleteMapping: (id: string) => apiClient.delete(`/mappings/${id}`).then(res => res.data),

  // DDL
  parseDdlText: (data: { sql_text: string; dialect?: string }) => apiClient.post<ParsedTable[]>('/ddl/parse', data).then(res => res.data),
  parseDdlFile: (formData: FormData) => apiClient.post<ParsedTable[]>('/ddl/parse-file', formData, {
    headers: { 'Content-Type': 'multipart/form-data' }
  }).then(res => res.data),
  importDdlTables: (projectId: string, tables: ParsedTable[], canvasId?: string) => apiClient.post<TableMeta[]>('/ddl/import', {
    project_id: projectId,
    tables,
    canvas_id: canvasId
  }).then(res => res.data),

  // DB Connection
  testDbConnection: (data: any) => apiClient.post<{ success: boolean; message: string }>('/connections/test', data).then(res => res.data),
  listDbTables: (data: any) => apiClient.post<DbTableSummary[]>('/connections/tables', data).then(res => res.data),
  importDbTables: (projectId: string, connection: any, selectedTables?: string[], canvasId?: string) => apiClient.post<TableMeta[]>('/connections/import', {
    project_id: projectId,
    connection,
    selected_tables: selectedTables,
    canvas_id: canvasId
  }).then(res => res.data),

  // Export / Backup
  getExcelExportUrl: (projectId: string) => `/api/export/excel/${projectId}`,
  getJsonExportUrl: (projectId: string) => `/api/export/json/${projectId}`,
  restoreProjectJson: (formData: FormData) => apiClient.post<Project>('/export/restore-json', formData, {
    headers: { 'Content-Type': 'multipart/form-data' }
  }).then(res => res.data),
}
