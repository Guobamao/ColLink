from typing import List, Dict, Any
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from app.core.database import get_db
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.models.canvas import CanvasView, CanvasNodeLayout
from app.schemas.connection import DbConnectionRequest, DbTableSummary, DbMetadataImportRequest
from app.schemas.table import TableWithColumnsOut, ColumnOut
from app.services.db_connector import DbConnectorService

router = APIRouter()


@router.post("/test")
def test_connection(req: DbConnectionRequest):
    result = DbConnectorService.test_connection(req)
    return result


@router.post("/tables", response_model=List[DbTableSummary])
def list_db_tables(req: DbConnectionRequest):
    try:
        tables = DbConnectorService.list_tables(req)
        return tables
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"获取表列表失败: {str(e)}")


@router.post("/import", response_model=List[TableWithColumnsOut])
def import_db_tables(req: DbMetadataImportRequest, db: Session = Depends(get_db)):
    try:
        parsed_tables = DbConnectorService.fetch_tables_metadata(
            req.connection,
            req.selected_tables
        )
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"抓取数据库元数据失败: {str(e)}")

    created_tables: List[TableWithColumnsOut] = []
    existing_count = db.query(TableMeta).filter(TableMeta.project_id == req.project_id).count()

    for idx, t in enumerate(parsed_tables):
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
