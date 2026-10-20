from typing import List
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from app.core.database import get_db
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.schemas.table import (
    TableCreate, TableUpdate, TablePositionUpdate,
    TableWithColumnsOut, ColumnOut
)

router = APIRouter()


@router.post("", response_model=TableWithColumnsOut)
def create_table(project_id: str, data: TableCreate, db: Session = Depends(get_db)):
    table = TableMeta(
        project_id=project_id,
        table_name=data.table_name,
        table_comment=data.table_comment or "",
        pos_x=data.pos_x,
        pos_y=data.pos_y
    )
    db.add(table)
    db.flush()

    for col_data in data.columns or []:
        col = ColumnMeta(
            table_id=table.id,
            column_name=col_data.column_name,
            data_type=col_data.data_type,
            column_comment=col_data.column_comment or "",
            sort_order=col_data.sort_order
        )
        db.add(col)

    db.commit()
    db.refresh(table)

    return TableWithColumnsOut(
        id=table.id,
        project_id=table.project_id,
        table_name=table.table_name,
        table_comment=table.table_comment or "",
        pos_x=table.pos_x,
        pos_y=table.pos_y,
        columns=[
            ColumnOut(
                id=c.id,
                table_id=c.table_id,
                column_name=c.column_name,
                data_type=c.data_type,
                column_comment=c.column_comment or "",
                sort_order=c.sort_order
            )
            for c in table.columns
        ]
    )


@router.put("/positions")
def update_table_positions(
    positions: List[TablePositionUpdate],
    db: Session = Depends(get_db)
):
    """Batch update table card coordinates from canvas movement."""
    for pos in positions:
        table = db.query(TableMeta).filter(TableMeta.id == pos.id).first()
        if table:
            table.pos_x = pos.pos_x
            table.pos_y = pos.pos_y
    db.commit()
    return {"message": "Positions updated"}


@router.put("/{table_id}", response_model=TableWithColumnsOut)
def update_table(table_id: str, data: TableUpdate, db: Session = Depends(get_db)):
    table = db.query(TableMeta).filter(TableMeta.id == table_id).first()
    if not table:
        raise HTTPException(status_code=404, detail="Table not found")

    if data.table_name is not None:
        table.table_name = data.table_name
    if data.table_comment is not None:
        table.table_comment = data.table_comment
    if data.pos_x is not None:
        table.pos_x = data.pos_x
    if data.pos_y is not None:
        table.pos_y = data.pos_y

    db.commit()
    db.refresh(table)

    return TableWithColumnsOut(
        id=table.id,
        project_id=table.project_id,
        table_name=table.table_name,
        table_comment=table.table_comment or "",
        pos_x=table.pos_x,
        pos_y=table.pos_y,
        columns=[
            ColumnOut(
                id=c.id,
                table_id=c.table_id,
                column_name=c.column_name,
                data_type=c.data_type,
                column_comment=c.column_comment or "",
                sort_order=c.sort_order
            )
            for c in table.columns
        ]
    )


@router.delete("/{table_id}")
def delete_table(table_id: str, db: Session = Depends(get_db)):
    table = db.query(TableMeta).filter(TableMeta.id == table_id).first()
    if not table:
        raise HTTPException(status_code=404, detail="Table not found")

    db.delete(table)
    db.commit()
    return {"message": "Table deleted successfully"}
