from typing import List
from fastapi import APIRouter, Depends, HTTPException, UploadFile, File, Form
from sqlalchemy.orm import Session
from app.core.database import get_db
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.models.canvas import CanvasView, CanvasNodeLayout
from app.schemas.ddl import DdlParseRequest, DdlImportRequest, ParsedTable
from app.schemas.table import TableWithColumnsOut, ColumnOut
from sqlalchemy import func
from app.models.mapping import FieldMapping
from app.schemas.table import TableWithColumnsOut, ColumnOut, ImportResultResponse, SyncSummary
from app.services.ddl_parser import DdlParser

router = APIRouter()


@router.post("/parse", response_model=List[ParsedTable])
def parse_ddl_text(req: DdlParseRequest):
    """Parse DDL text and return structured table/column preview."""
    try:
        tables = DdlParser.parse(req.sql_text, req.dialect or "dm")
        return tables
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"DDL 解析失败: {str(e)}")


@router.post("/parse-file", response_model=List[ParsedTable])
async def parse_ddl_file(
    file: UploadFile = File(...),
    dialect: str = Form("dm")
):
    """Upload .sql file, parse DDL and return structured preview."""
    try:
        contents = await file.read()
        sql_text = contents.decode("utf-8", errors="replace")
        tables = DdlParser.parse(sql_text, dialect)
        return tables
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"文件解析失败: {str(e)}")


@router.post("/import", response_model=ImportResultResponse)
def import_ddl_tables(req: DdlImportRequest, db: Session = Depends(get_db)):
    """Import parsed tables directly into project canvas with schema diff and in-place column preservation."""
    result_tables: List[TableWithColumnsOut] = []
    tables_created = 0
    tables_updated = 0
    columns_added = 0
    columns_updated = 0
    mappings_preserved = 0

    existing_count = db.query(TableMeta).filter(TableMeta.project_id == req.project_id).count()

    target_canvas_id = req.canvas_id
    if not target_canvas_id:
        first_canvas = db.query(CanvasView).filter(CanvasView.project_id == req.project_id).order_by(CanvasView.sort_order.asc()).first()
        if first_canvas:
            target_canvas_id = first_canvas.id

    for idx, t in enumerate(req.tables):
        clean_name = t.table_name.strip()
        existing_table = db.query(TableMeta).filter(
            TableMeta.project_id == req.project_id,
            func.upper(TableMeta.table_name) == clean_name.upper()
        ).first()

        if existing_table:
            # Incremental sync: Table already exists, do NOT delete or recreate!
            tables_updated += 1
            if t.table_comment:
                existing_table.table_comment = t.table_comment

            existing_cols = db.query(ColumnMeta).filter(ColumnMeta.table_id == existing_table.id).all()
            existing_col_map = {c.column_name.strip().upper(): c for c in existing_cols}

            # Count preserved mappings for this table
            col_ids = [c.id for c in existing_cols]
            if col_ids:
                preserved = db.query(FieldMapping).filter(
                    (FieldMapping.source_column_id.in_(col_ids)) | (FieldMapping.target_column_id.in_(col_ids))
                ).count()
                mappings_preserved += preserved

            for c in t.columns:
                c_name_key = c.column_name.strip().upper()
                if c_name_key in existing_col_map:
                    # In-place update preserving column ID and mappings!
                    matched_col = existing_col_map[c_name_key]
                    matched_col.data_type = c.data_type
                    if c.column_comment:
                        matched_col.column_comment = c.column_comment
                    matched_col.sort_order = c.sort_order
                    columns_updated += 1
                else:
                    # New column added to existing table
                    new_col = ColumnMeta(
                        table_id=existing_table.id,
                        column_name=c.column_name,
                        data_type=c.data_type,
                        column_comment=c.column_comment or "",
                        sort_order=c.sort_order
                    )
                    db.add(new_col)
                    columns_added += 1

            db.flush()

            # Ensure canvas node layout exists
            if target_canvas_id:
                layout = db.query(CanvasNodeLayout).filter(
                    CanvasNodeLayout.canvas_id == target_canvas_id,
                    CanvasNodeLayout.table_id == existing_table.id
                ).first()
                if not layout:
                    db.add(CanvasNodeLayout(
                        canvas_id=target_canvas_id,
                        table_id=existing_table.id,
                        position_x=existing_table.pos_x,
                        position_y=existing_table.pos_y,
                        color_tag="blue"
                    ))

            cols_out = db.query(ColumnMeta).filter(ColumnMeta.table_id == existing_table.id).order_by(ColumnMeta.sort_order.asc()).all()
            result_tables.append(TableWithColumnsOut(
                id=existing_table.id,
                project_id=existing_table.project_id,
                table_name=existing_table.table_name,
                table_comment=existing_table.table_comment or "",
                pos_x=existing_table.pos_x,
                pos_y=existing_table.pos_y,
                columns=[ColumnOut.from_orm(co) for co in cols_out]
            ))
        else:
            # Brand new table
            tables_created += 1
            total_idx = existing_count + tables_created - 1
            col_idx = total_idx % 3
            row_idx = total_idx // 3
            pos_x = 80.0 + col_idx * 400.0
            pos_y = 80.0 + row_idx * 360.0

            table = TableMeta(
                project_id=req.project_id,
                table_name=t.table_name,
                table_comment=t.table_comment or "",
                pos_x=pos_x,
                pos_y=pos_y
            )
            db.add(table)
            db.flush()

            cols_out = []
            for c in t.columns:
                col = ColumnMeta(
                    table_id=table.id,
                    column_name=c.column_name,
                    data_type=c.data_type,
                    column_comment=c.column_comment or "",
                    sort_order=c.sort_order
                )
                db.add(col)
                db.flush()
                cols_out.append(ColumnOut(
                    id=col.id,
                    table_id=col.table_id,
                    column_name=col.column_name,
                    data_type=col.data_type,
                    column_comment=col.column_comment or "",
                    sort_order=col.sort_order
                ))
                columns_added += 1

            if target_canvas_id:
                layout = CanvasNodeLayout(
                    canvas_id=target_canvas_id,
                    table_id=table.id,
                    position_x=table.pos_x,
                    position_y=table.pos_y,
                    color_tag="blue"
                )
                db.add(layout)

            result_tables.append(TableWithColumnsOut(
                id=table.id,
                project_id=table.project_id,
                table_name=table.table_name,
                table_comment=table.table_comment or "",
                pos_x=table.pos_x,
                pos_y=table.pos_y,
                columns=cols_out
            ))

    db.commit()

    # Build summary message
    summary_parts = []
    if tables_created > 0:
        summary_parts.append(f"新增 {tables_created} 张表")
    if tables_updated > 0:
        summary_parts.append(f"增量同步 {tables_updated} 张已有表 (新增 {columns_added} 个字段, 原地更新 {columns_updated} 个字段)")
    if mappings_preserved > 0:
        summary_parts.append(f"安全保留 {mappings_preserved} 条既有连线映射")

    summary_msg = "、".join(summary_parts) if summary_parts else "未发生结构变动"
    full_message = f"表结构同步完成：{summary_msg}。"

    summary = SyncSummary(
        tables_created=tables_created,
        tables_updated=tables_updated,
        columns_added=columns_added,
        columns_updated=columns_updated,
        mappings_preserved=mappings_preserved,
        message=full_message
    )

    return ImportResultResponse(tables=result_tables, summary=summary)

