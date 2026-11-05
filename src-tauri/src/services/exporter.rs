use chrono::Utc;
use rusqlite::Connection;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ProjectBackupData {
    pub version: String,
    pub exported_at: String,
    pub project: ProjectBackupMeta,
    pub tables: Vec<TableBackupMeta>,
    pub mappings: Vec<MappingBackupMeta>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectBackupMeta {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TableBackupMeta {
    pub id: String,
    pub table_name: String,
    pub table_comment: Option<String>,
    pub pos_x: f64,
    pub pos_y: f64,
    pub columns: Vec<ColumnBackupMeta>,
}

#[derive(Serialize, Deserialize)]
pub struct ColumnBackupMeta {
    pub id: String,
    pub column_name: String,
    pub data_type: String,
    pub column_comment: Option<String>,
    pub sort_order: i32,
}

#[derive(Serialize, Deserialize)]
pub struct MappingBackupMeta {
    pub id: String,
    pub source_table_id: String,
    pub source_column_id: String,
    pub target_table_id: String,
    pub target_column_id: String,
    pub created_at: Option<String>,
}

pub struct ExporterService;

impl ExporterService {
    pub fn export_excel(
        conn: &Connection,
        project_id: &str,
    ) -> std::result::Result<Vec<u8>, String> {
        let tables = crate::db::tables::list_tables_for_project(conn, project_id)
            .map_err(|e| format!("加载表失败: {}", e))?;
        let mappings = crate::db::mappings::list_mappings(conn, project_id)
            .map_err(|e| format!("加载映射失败: {}", e))?;

        let mut wb = Workbook::new();

        // Styles
        let header_format = Format::new()
            .set_font_name("Microsoft YaHei")
            .set_font_size(11.0)
            .set_bold()
            .set_font_color(Color::RGB(0xFFFFFF))
            .set_background_color(Color::RGB(0x1F4E79))
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter);

        let sub_header_format = Format::new()
            .set_font_name("Microsoft YaHei")
            .set_font_size(11.0)
            .set_bold()
            .set_font_color(Color::RGB(0xFFFFFF))
            .set_background_color(Color::RGB(0x2F5597))
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter);

        let cell_center = Format::new()
            .set_font_name("Microsoft YaHei")
            .set_font_size(10.0)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xD9D9D9));

        let cell_left = Format::new()
            .set_font_name("Microsoft YaHei")
            .set_font_size(10.0)
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xD9D9D9));

        let zebra_center = Format::new()
            .set_font_name("Microsoft YaHei")
            .set_font_size(10.0)
            .set_background_color(Color::RGB(0xF2F5F9))
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xD9D9D9));

        let zebra_left = Format::new()
            .set_font_name("Microsoft YaHei")
            .set_font_size(10.0)
            .set_background_color(Color::RGB(0xF2F5F9))
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xD9D9D9));

        // Sheet 1: 字段映射清单
        let ws_mappings = wb.add_worksheet();
        ws_mappings
            .set_name("字段映射清单")
            .map_err(|e| e.to_string())?;

        let mapping_headers = [
            "序号",
            "源表英文名",
            "源表中文注释",
            "源字段名",
            "源字段类型",
            "源字段注释",
            "目标表英文名",
            "目标表中文注释",
            "目标字段名",
            "目标字段类型",
            "目标字段注释",
            "映射创建时间",
        ];

        ws_mappings.set_row_height(0, 28.0).map_err(|e| e.to_string())?;
        for (col_idx, &header) in mapping_headers.iter().enumerate() {
            ws_mappings
                .write_string_with_format(0, col_idx as u16, header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (idx, m) in mappings.iter().enumerate() {
            let row_idx = (idx + 1) as u32;
            ws_mappings.set_row_height(row_idx, 22.0).map_err(|e| e.to_string())?;
            let is_zebra = idx % 2 == 1;

            let c_fmt = if is_zebra { &zebra_center } else { &cell_center };
            let l_fmt = if is_zebra { &zebra_left } else { &cell_left };

            ws_mappings
                .write_number_with_format(row_idx, 0, (idx + 1) as f64, c_fmt)
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    1,
                    m.source_table_name.as_deref().unwrap_or(""),
                    l_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    2,
                    m.source_table_comment.as_deref().unwrap_or(""),
                    l_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    3,
                    m.source_column_name.as_deref().unwrap_or(""),
                    l_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    4,
                    m.source_data_type.as_deref().unwrap_or(""),
                    c_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    5,
                    m.source_column_comment.as_deref().unwrap_or(""),
                    l_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    6,
                    m.target_table_name.as_deref().unwrap_or(""),
                    l_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    7,
                    m.target_table_comment.as_deref().unwrap_or(""),
                    l_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    8,
                    m.target_column_name.as_deref().unwrap_or(""),
                    l_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    9,
                    m.target_data_type.as_deref().unwrap_or(""),
                    c_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    10,
                    m.target_column_comment.as_deref().unwrap_or(""),
                    l_fmt,
                )
                .map_err(|e| e.to_string())?;
            ws_mappings
                .write_string_with_format(
                    row_idx,
                    11,
                    m.created_at.as_deref().unwrap_or(""),
                    c_fmt,
                )
                .map_err(|e| e.to_string())?;
        }

        // Set column widths for Sheet 1
        let col_widths = [8.0, 20.0, 22.0, 20.0, 14.0, 22.0, 20.0, 22.0, 20.0, 14.0, 22.0, 20.0];
        for (c, &w) in col_widths.iter().enumerate() {
            ws_mappings.set_column_width(c as u16, w).map_err(|e| e.to_string())?;
        }

        // Sheet 2: 数据表结构字典
        let ws_dict = wb.add_worksheet();
        ws_dict
            .set_name("数据表结构字典")
            .map_err(|e| e.to_string())?;

        let dict_headers = [
            "表英文名",
            "表中文注释",
            "字段序号",
            "字段名",
            "字段类型",
            "字段注释",
        ];

        ws_dict.set_row_height(0, 28.0).map_err(|e| e.to_string())?;
        for (col_idx, &header) in dict_headers.iter().enumerate() {
            ws_dict
                .write_string_with_format(0, col_idx as u16, header, &sub_header_format)
                .map_err(|e| e.to_string())?;
        }

        let mut curr_row: u32 = 1;
        for t in &tables {
            for c in &t.columns {
                ws_dict.set_row_height(curr_row, 20.0).map_err(|e| e.to_string())?;
                ws_dict
                    .write_string_with_format(curr_row, 0, &t.table_name, &cell_left)
                    .map_err(|e| e.to_string())?;
                ws_dict
                    .write_string_with_format(
                        curr_row,
                        1,
                        t.table_comment.as_deref().unwrap_or(""),
                        &cell_left,
                    )
                    .map_err(|e| e.to_string())?;
                ws_dict
                    .write_number_with_format(curr_row, 2, c.sort_order as f64, &cell_center)
                    .map_err(|e| e.to_string())?;
                ws_dict
                    .write_string_with_format(curr_row, 3, &c.column_name, &cell_left)
                    .map_err(|e| e.to_string())?;
                ws_dict
                    .write_string_with_format(curr_row, 4, &c.data_type, &cell_center)
                    .map_err(|e| e.to_string())?;
                ws_dict
                    .write_string_with_format(
                        curr_row,
                        5,
                        c.column_comment.as_deref().unwrap_or(""),
                        &cell_left,
                    )
                    .map_err(|e| e.to_string())?;
                curr_row += 1;
            }
        }

        let dict_widths = [22.0, 24.0, 10.0, 22.0, 16.0, 24.0];
        for (c, &w) in dict_widths.iter().enumerate() {
            ws_dict.set_column_width(c as u16, w).map_err(|e| e.to_string())?;
        }

        wb.save_to_buffer().map_err(|e| e.to_string())
    }

    pub fn export_project_json(
        conn: &Connection,
        project_id: &str,
    ) -> std::result::Result<String, String> {
        let project = crate::db::projects::get_project_detail(conn, project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Project not found".to_string())?;

        let tables = crate::db::tables::list_tables_for_project(conn, project_id)
            .map_err(|e| e.to_string())?;
        let mappings = crate::db::mappings::list_mappings(conn, project_id)
            .map_err(|e| e.to_string())?;

        let backup = ProjectBackupData {
            version: "1.0".to_string(),
            exported_at: Utc::now().to_rfc3339(),
            project: ProjectBackupMeta {
                id: project.id,
                name: project.name,
                description: project.description,
            },
            tables: tables
                .into_iter()
                .map(|t| TableBackupMeta {
                    id: t.id,
                    table_name: t.table_name,
                    table_comment: t.table_comment,
                    pos_x: t.pos_x,
                    pos_y: t.pos_y,
                    columns: t
                        .columns
                        .into_iter()
                        .map(|c| ColumnBackupMeta {
                            id: c.id,
                            column_name: c.column_name,
                            data_type: c.data_type,
                            column_comment: c.column_comment,
                            sort_order: c.sort_order,
                        })
                        .collect(),
                })
                .collect(),
            mappings: mappings
                .into_iter()
                .map(|m| MappingBackupMeta {
                    id: m.id,
                    source_table_id: m.source_table_id,
                    source_column_id: m.source_column_id,
                    target_table_id: m.target_table_id,
                    target_column_id: m.target_column_id,
                    created_at: m.created_at,
                })
                .collect(),
        };

        serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())
    }

    pub fn import_project_json(
        conn: &Connection,
        json_content: &str,
    ) -> std::result::Result<crate::models::Project, String> {
        let backup: ProjectBackupData =
            serde_json::from_str(json_content).map_err(|e| format!("JSON解析错误: {}", e))?;

        let new_name = format!("{} (备份恢复)", backup.project.name);
        let new_project = crate::db::projects::create_project(
            conn,
            &new_name,
            backup.project.description.as_deref(),
        )
        .map_err(|e| e.to_string())?;

        let mut old_to_new_tbl_id: HashMap<String, String> = HashMap::new();
        let mut old_to_new_col_id: HashMap<String, String> = HashMap::new();

        for t_info in backup.tables {
            let cols: Vec<crate::models::ColumnMeta> = t_info
                .columns
                .iter()
                .map(|c| crate::models::ColumnMeta {
                    id: String::new(),
                    table_id: String::new(),
                    column_name: c.column_name.clone(),
                    data_type: c.data_type.clone(),
                    column_comment: c.column_comment.clone(),
                    sort_order: c.sort_order,
                })
                .collect();

            let created_tbl = crate::db::tables::create_table(
                conn,
                &new_project.id,
                &t_info.table_name,
                t_info.table_comment.as_deref(),
                Some(t_info.pos_x),
                Some(t_info.pos_y),
                &cols,
            )
            .map_err(|e| e.to_string())?;

            old_to_new_tbl_id.insert(t_info.id, created_tbl.id.clone());

            for (orig_c, new_c) in t_info.columns.iter().zip(created_tbl.columns.iter()) {
                old_to_new_col_id.insert(orig_c.id.clone(), new_c.id.clone());
            }
        }

        for m_info in backup.mappings {
            if let (Some(new_st), Some(new_tt), Some(new_sc), Some(new_tc)) = (
                old_to_new_tbl_id.get(&m_info.source_table_id),
                old_to_new_tbl_id.get(&m_info.target_table_id),
                old_to_new_col_id.get(&m_info.source_column_id),
                old_to_new_col_id.get(&m_info.target_column_id),
            ) {
                let _ = crate::db::mappings::create_mapping(
                    conn,
                    &new_project.id,
                    new_st,
                    new_sc,
                    new_tt,
                    new_tc,
                );
            }
        }

        crate::db::projects::get_project_detail(conn, &new_project.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Failed to load restored project".to_string())
    }
}
