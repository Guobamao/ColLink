from typing import List
from fastapi import APIRouter, Depends, HTTPException, UploadFile, File, Form
from sqlalchemy.orm import Session
from app.core.database import get_db
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.models.canvas import CanvasView, CanvasNodeLayout
from app.schemas.ddl import DdlParseRequest, DdlImportRequest, ParsedTable
from app.schemas.table import TableWithColumnsOut, ColumnOut
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


@router.post("/import", response_model=List[TableWithColumnsOut])
def import_ddl_tables(req: DdlImportRequest, db: Session = Depends(get_db)):
    """Import parsed tables directly into project canvas with neat positioning."""
    created_tables: List[TableWithColumnsOut] = []

    # Get current count of tables in project to offset new nodes
    existing_count = db.query(TableMeta).filter(TableMeta.project_id == req.project_id).count()

    for idx, t in enumerate(req.tables):
        # Grid positioning: 3 tables per row
        total_idx = existing_count + idx
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

        created_tables.append(TableWithColumnsOut(
            id=table.id,
            project_id=table.project_id,
            table_name=table.table_name,
            table_comment=table.table_comment or "",
            pos_x=table.pos_x,
            pos_y=table.pos_y,
            columns=cols_out
        ))

        # Also add to canvas view
        target_canvas_id = req.canvas_id
        if not target_canvas_id:
            first_canvas = db.query(CanvasView).filter(CanvasView.project_id == req.project_id).order_by(CanvasView.sort_order.asc()).first()
            if first_canvas:
                target_canvas_id = first_canvas.id

        if target_canvas_id:
            layout = CanvasNodeLayout(
                canvas_id=target_canvas_id,
                table_id=table.id,
                position_x=table.pos_x,
                position_y=table.pos_y,
                color_tag="blue"
            )
            db.add(layout)

    db.commit()
    return created_tables
