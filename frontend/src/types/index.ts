export interface ColumnMeta {
  id: string
  table_id: string
  column_name: string
  data_type: string
  column_comment?: string
  sort_order: number
}

export interface TableMeta {
  id: string
  project_id: string
  table_name: string
  table_comment?: string
  pos_x: number
  pos_y: number
  columns: ColumnMeta[]
}

export interface FieldMapping {
  id: string
  project_id: string
  source_table_id: string
  source_column_id: string
  target_table_id: string
  target_column_id: string
  created_at?: string
  source_table_name?: string
  source_table_comment?: string
  source_column_name?: string
  source_column_comment?: string
  source_data_type?: string
  target_table_name?: string
  target_table_comment?: string
  target_column_name?: string
  target_column_comment?: string
  target_data_type?: string
}

export interface Project {
  id: string
  name: string
  description?: string
  created_at?: string
  updated_at?: string
  table_count?: number
  mapping_count?: number
  tables?: TableMeta[]
  mappings?: FieldMapping[]
}

export interface ParsedColumn {
  column_name: string
  data_type: string
  column_comment?: string
  sort_order: number
}

export interface ParsedTable {
  table_name: string
  table_comment?: string
  columns: ParsedColumn[]
}

export interface DbTableSummary {
  table_name: string
  table_comment?: string
  column_count?: number
}

export interface CanvasTableMeta extends TableMeta {
  color_tag?: string
}

export interface CanvasView {
  id: string
  project_id: string
  name: string
  sort_order: number
  created_at: string
  table_count?: number
}

export interface CanvasDetail extends CanvasView {
  tables: CanvasTableMeta[]
  mappings: FieldMapping[]
}

export interface PathStepTable {
  id: string
  table_name: string
  table_comment: string
}

export interface PathStepColumn {
  id: string
  column_name: string
  data_type: string
  column_comment: string
}

export interface PathStep {
  step_index: number
  relation_type: 'EXPLICIT' | 'INFERRED'
  match_reason: string
  confidence: number
  from_table: PathStepTable
  from_column: PathStepColumn
  to_table: PathStepTable
  to_column: PathStepColumn
  step_label: string
}

export interface RelationPath {
  path_id: string
  summary_title: string
  hops: number
  is_fully_explicit: boolean
  avg_confidence: number
  involved_table_ids: string[]
  intermediate_table_ids: string[]
  steps: PathStep[]
  sql_snippet: string
}

export interface RelationPathResult {
  paths_count: number
  source_table: PathStepTable
  target_table: PathStepTable
  paths: RelationPath[]
}

