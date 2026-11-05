use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMeta {
    pub id: String,
    pub table_id: String,
    pub column_name: String,
    pub data_type: String,
    #[serde(default)]
    pub column_comment: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMeta {
    pub id: String,
    pub project_id: String,
    pub table_name: String,
    #[serde(default)]
    pub table_comment: Option<String>,
    pub pos_x: f64,
    pub pos_y: f64,
    #[serde(default)]
    pub columns: Vec<ColumnMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasTableMeta {
    pub id: String,
    pub project_id: String,
    pub table_name: String,
    #[serde(default)]
    pub table_comment: Option<String>,
    pub pos_x: f64,
    pub pos_y: f64,
    pub color_tag: Option<String>,
    #[serde(default)]
    pub columns: Vec<ColumnMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub id: String,
    pub project_id: String,
    pub source_table_id: String,
    pub source_column_id: String,
    pub target_table_id: String,
    pub target_column_id: String,
    pub created_at: Option<String>,
    pub source_table_name: Option<String>,
    pub source_table_comment: Option<String>,
    pub source_column_name: Option<String>,
    pub source_column_comment: Option<String>,
    pub source_data_type: Option<String>,
    pub target_table_name: Option<String>,
    pub target_table_comment: Option<String>,
    pub target_column_name: Option<String>,
    pub target_column_comment: Option<String>,
    pub target_data_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub table_count: Option<usize>,
    pub mapping_count: Option<usize>,
    pub tables: Option<Vec<TableMeta>>,
    pub mappings: Option<Vec<FieldMapping>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasView {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub sort_order: i32,
    pub created_at: String,
    pub table_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasDetail {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub sort_order: i32,
    pub created_at: String,
    pub table_count: usize,
    pub tables: Vec<CanvasTableMeta>,
    pub mappings: Vec<FieldMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub id: String,
    pub pos_x: f64,
    pub pos_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedColumn {
    pub column_name: String,
    pub data_type: String,
    #[serde(default)]
    pub column_comment: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTable {
    pub table_name: String,
    #[serde(default)]
    pub table_comment: Option<String>,
    pub columns: Vec<ParsedColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub tables_created: usize,
    pub tables_updated: usize,
    pub columns_added: usize,
    pub columns_updated: usize,
    pub mappings_preserved: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub tables: Vec<TableMeta>,
    pub summary: ImportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStepTable {
    pub id: String,
    pub table_name: String,
    pub table_comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStepColumn {
    pub id: String,
    pub column_name: String,
    pub data_type: String,
    pub column_comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStep {
    pub step_index: usize,
    pub relation_type: String,
    pub match_reason: String,
    pub confidence: f64,
    pub from_table: PathStepTable,
    pub from_column: PathStepColumn,
    pub to_table: PathStepTable,
    pub to_column: PathStepColumn,
    pub step_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationPath {
    pub path_id: String,
    pub summary_title: String,
    pub hops: usize,
    pub is_fully_explicit: bool,
    pub avg_confidence: f64,
    pub involved_table_ids: Vec<String>,
    pub intermediate_table_ids: Vec<String>,
    pub steps: Vec<PathStep>,
    pub sql_snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationPathResult {
    pub project_id: String,
    pub source_table_id: String,
    pub target_table_id: String,
    pub paths_count: usize,
    pub paths: Vec<RelationPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteResult {
    pub deleted_count: usize,
}
