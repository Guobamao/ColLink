import io
import json
from datetime import datetime
from typing import Dict, Any, List
from openpyxl import Workbook
from openpyxl.styles import Font, PatternFill, Alignment, Border, Side
from openpyxl.utils import get_column_letter
from sqlalchemy.orm import Session
from app.models.project import Project
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.models.mapping import FieldMapping


class ExporterService:
    @staticmethod
    def export_excel(db: Session, project_id: str) -> io.BytesIO:
        """Generate formatted Excel report containing field mappings and table dictionaries."""
        project = db.query(Project).filter(Project.id == project_id).first()
        if not project:
            raise ValueError("Project not found")

        # Load all tables & columns for this project
        tables = db.query(TableMeta).filter(TableMeta.project_id == project_id).all()
        table_map = {t.id: t for t in tables}

        col_map = {}
        for t in tables:
            for c in t.columns:
                col_map[c.id] = c

        mappings = db.query(FieldMapping).filter(FieldMapping.project_id == project_id).all()

        wb = Workbook()

        # Styles
        header_fill = PatternFill(start_color="1F4E79", end_color="1F4E79", fill_type="solid")
        header_font = Font(name="Microsoft YaHei", size=11, bold=True, color="FFFFFF")
        sub_fill = PatternFill(start_color="2F5597", end_color="2F5597", fill_type="solid")
        zebra_fill = PatternFill(start_color="F2F5F9", end_color="F2F5F9", fill_type="solid")
        border_thin = Border(
            left=Side(style='thin', color='D9D9D9'),
            right=Side(style='thin', color='D9D9D9'),
            top=Side(style='thin', color='D9D9D9'),
            bottom=Side(style='thin', color='D9D9D9')
        )
        content_font = Font(name="Microsoft YaHei", size=10)
        center_align = Alignment(horizontal='center', vertical='center')
        left_align = Alignment(horizontal='left', vertical='center')

        # Sheet 1: 字段映射清单
        ws_mappings = wb.active
        ws_mappings.title = "字段映射清单"

        mapping_headers = [
            "序号", "源表英文名", "源表中文注释", "源字段名", "源字段类型", "源字段注释",
            "目标表英文名", "目标表中文注释", "目标字段名", "目标字段类型", "目标字段注释", "映射创建时间"
        ]
        ws_mappings.append(mapping_headers)

        for col_idx in range(1, len(mapping_headers) + 1):
            cell = ws_mappings.cell(row=1, column=col_idx)
            cell.fill = header_fill
            cell.font = header_font
            cell.alignment = center_align
        ws_mappings.row_dimensions[1].height = 28

        for idx, m in enumerate(mappings, 1):
            src_table = table_map.get(m.source_table_id)
            src_col = col_map.get(m.source_column_id)
            tgt_table = table_map.get(m.target_table_id)
            tgt_col = col_map.get(m.target_column_id)

            created_str = m.created_at.strftime("%Y-%m-%d %H:%M:%S") if m.created_at else ""

            row_data = [
                idx,
                src_table.table_name if src_table else "",
                src_table.table_comment if src_table else "",
                src_col.column_name if src_col else "",
                src_col.data_type if src_col else "",
                src_col.column_comment if src_col else "",
                tgt_table.table_name if tgt_table else "",
                tgt_table.table_comment if tgt_table else "",
                tgt_col.column_name if tgt_col else "",
                tgt_col.data_type if tgt_col else "",
                tgt_col.column_comment if tgt_col else "",
                created_str
            ]
            ws_mappings.append(row_data)
            row_idx = idx + 1
            ws_mappings.row_dimensions[row_idx].height = 22

            is_zebra = (idx % 2 == 0)
            for c_idx in range(1, len(row_data) + 1):
                cell = ws_mappings.cell(row=row_idx, column=c_idx)
                cell.font = content_font
                cell.border = border_thin
                cell.alignment = center_align if c_idx in (1, 5, 10, 12) else left_align
                if is_zebra:
                    cell.fill = zebra_fill

        # Auto column width
        for col in ws_mappings.columns:
            max_len = max(len(str(cell.value or '')) for cell in col)
            col_letter = get_column_letter(col[0].column)
            ws_mappings.column_dimensions[col_letter].width = max(max_len * 1.5 + 4, 12)

        # Sheet 2: 数据表结构字典
        ws_dict = wb.create_sheet(title="数据表结构字典")
        dict_headers = ["表英文名", "表中文注释", "字段序号", "字段名", "字段类型", "字段注释"]
        ws_dict.append(dict_headers)

        for col_idx in range(1, len(dict_headers) + 1):
            cell = ws_dict.cell(row=1, column=col_idx)
            cell.fill = sub_fill
            cell.font = header_font
            cell.alignment = center_align
        ws_dict.row_dimensions[1].height = 28

        curr_row = 2
        for t in tables:
            for c in t.columns:
                ws_dict.append([
                    t.table_name,
                    t.table_comment,
                    c.sort_order,
                    c.column_name,
                    c.data_type,
                    c.column_comment
                ])
                ws_dict.row_dimensions[curr_row].height = 20
                for c_idx in range(1, len(dict_headers) + 1):
                    cell = ws_dict.cell(row=curr_row, column=c_idx)
                    cell.font = content_font
                    cell.border = border_thin
                    cell.alignment = center_align if c_idx in (3, 5) else left_align
                curr_row += 1

        for col in ws_dict.columns:
            max_len = max(len(str(cell.value or '')) for cell in col)
            col_letter = get_column_letter(col[0].column)
            ws_dict.column_dimensions[col_letter].width = max(max_len * 1.5 + 4, 14)

        output = io.BytesIO()
        wb.save(output)
        output.seek(0)
        return output

    @staticmethod
    def export_project_json(db: Session, project_id: str) -> Dict[str, Any]:
        """Export full project data structure to JSON."""
        project = db.query(Project).filter(Project.id == project_id).first()
        if not project:
            raise ValueError("Project not found")

        tables = db.query(TableMeta).filter(TableMeta.project_id == project_id).all()
        mappings = db.query(FieldMapping).filter(FieldMapping.project_id == project_id).all()

        return {
            "version": "1.0",
            "exported_at": datetime.utcnow().isoformat(),
            "project": {
                "id": project.id,
                "name": project.name,
                "description": project.description,
            },
            "tables": [
                {
                    "id": t.id,
                    "table_name": t.table_name,
                    "table_comment": t.table_comment,
                    "pos_x": t.pos_x,
                    "pos_y": t.pos_y,
                    "columns": [
                        {
                            "id": c.id,
                            "column_name": c.column_name,
                            "data_type": c.data_type,
                            "column_comment": c.column_comment,
                            "sort_order": c.sort_order
                        }
                        for c in t.columns
                    ]
                }
                for t in tables
            ],
            "mappings": [
                {
                    "id": m.id,
                    "source_table_id": m.source_table_id,
                    "source_column_id": m.source_column_id,
                    "target_table_id": m.target_table_id,
                    "target_column_id": m.target_column_id,
                    "created_at": m.created_at.isoformat() if m.created_at else None
                }
                for m in mappings
            ]
        }

    @staticmethod
    def import_project_json(db: Session, data: Dict[str, Any]) -> Project:
        """Import or restore project data from JSON."""
        proj_data = data.get("project", {})
        proj_name = proj_data.get("name", "导入的项目")
        proj_desc = proj_data.get("description", "")

        new_project = Project(
            name=f"{proj_name} (备份恢复)",
            description=proj_desc
        )
        db.add(new_project)
        db.flush()

        # Map old IDs to new entities to preserve mappings
        old_to_new_table_id = {}
        old_to_new_col_id = {}

        for t_info in data.get("tables", []):
            new_table = TableMeta(
                project_id=new_project.id,
                table_name=t_info.get("table_name", "UNNAMED"),
                table_comment=t_info.get("table_comment", ""),
                pos_x=t_info.get("pos_x", 100.0),
                pos_y=t_info.get("pos_y", 100.0)
            )
            db.add(new_table)
            db.flush()
            old_to_new_table_id[t_info["id"]] = new_table.id

            for c_info in t_info.get("columns", []):
                new_col = ColumnMeta(
                    table_id=new_table.id,
                    column_name=c_info.get("column_name", ""),
                    data_type=c_info.get("data_type", "VARCHAR(255)"),
                    column_comment=c_info.get("column_comment", ""),
                    sort_order=c_info.get("sort_order", 0)
                )
                db.add(new_col)
                db.flush()
                old_to_new_col_id[c_info["id"]] = new_col.id

        for m_info in data.get("mappings", []):
            old_s_tid = m_info.get("source_table_id")
            old_s_cid = m_info.get("source_column_id")
            old_t_tid = m_info.get("target_table_id")
            old_t_cid = m_info.get("target_column_id")

            if old_s_tid in old_to_new_table_id and old_t_tid in old_to_new_table_id and \
               old_s_cid in old_to_new_col_id and old_t_cid in old_to_new_col_id:
                new_mapping = FieldMapping(
                    project_id=new_project.id,
                    source_table_id=old_to_new_table_id[old_s_tid],
                    source_column_id=old_to_new_col_id[old_s_cid],
                    target_table_id=old_to_new_table_id[old_t_tid],
                    target_column_id=old_to_new_col_id[old_t_cid]
                )
                db.add(new_mapping)

        db.commit()
        db.refresh(new_project)
        return new_project
