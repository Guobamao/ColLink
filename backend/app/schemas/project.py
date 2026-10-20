from datetime import datetime
from typing import List, Optional
from pydantic import BaseModel
from app.schemas.table import TableWithColumnsOut
from app.schemas.mapping import MappingOut


class ProjectBase(BaseModel):
    name: str
    description: Optional[str] = ""


class ProjectCreate(ProjectBase):
    pass


class ProjectUpdate(BaseModel):
    name: Optional[str] = None
    description: Optional[str] = None


class ProjectOut(ProjectBase):
    id: str
    created_at: datetime
    updated_at: datetime
    table_count: Optional[int] = 0
    mapping_count: Optional[int] = 0

    class Config:
        from_attributes = True


class ProjectDetailOut(ProjectOut):
    tables: List[TableWithColumnsOut] = []
    mappings: List[MappingOut] = []

    class Config:
        from_attributes = True
