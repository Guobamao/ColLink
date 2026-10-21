from datetime import datetime
from typing import List, Optional
from pydantic import BaseModel
from app.schemas.table import TableWithColumnsOut
from app.schemas.mapping import MappingOut


class CanvasBase(BaseModel):
    name: str
    sort_order: Optional[int] = 0


class CanvasCreate(CanvasBase):
    pass


class CanvasUpdate(BaseModel):
    name: Optional[str] = None
    sort_order: Optional[int] = None


class CanvasOut(CanvasBase):
    id: str
    project_id: str
    created_at: datetime
    table_count: Optional[int] = 0

    class Config:
        from_attributes = True


class CanvasTableOut(TableWithColumnsOut):
    color_tag: str = "blue"

    class Config:
        from_attributes = True


class CanvasDetailOut(CanvasOut):
    tables: List[CanvasTableOut] = []
    mappings: List[MappingOut] = []

    class Config:
        from_attributes = True


class CanvasAddTables(BaseModel):
    table_ids: List[str]


class CanvasPositionUpdate(BaseModel):
    id: str  # table_id
    pos_x: float
    pos_y: float


class CanvasColorUpdate(BaseModel):
    color_tag: str
