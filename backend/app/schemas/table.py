from typing import List, Optional
from pydantic import BaseModel


class ColumnBase(BaseModel):
    column_name: str
    data_type: str = "VARCHAR(255)"
    column_comment: Optional[str] = ""
    sort_order: int = 0


class ColumnCreate(ColumnBase):
    pass


class ColumnOut(ColumnBase):
    id: str
    table_id: str

    class Config:
        from_attributes = True


class TableBase(BaseModel):
    table_name: str
    table_comment: Optional[str] = ""
    pos_x: float = 100.0
    pos_y: float = 100.0


class TableCreate(TableBase):
    columns: Optional[List[ColumnCreate]] = []


class TableUpdate(BaseModel):
    table_name: Optional[str] = None
    table_comment: Optional[str] = None
    pos_x: Optional[float] = None
    pos_y: Optional[float] = None


class TablePositionUpdate(BaseModel):
    id: str
    pos_x: float
    pos_y: float


class TableOut(TableBase):
    id: str
    project_id: str

    class Config:
        from_attributes = True


class TableWithColumnsOut(TableOut):
    columns: List[ColumnOut] = []

    class Config:
        from_attributes = True
