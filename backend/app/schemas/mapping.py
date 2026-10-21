from datetime import datetime
from typing import Optional
from pydantic import BaseModel
from app.schemas.table import TableOut, ColumnOut


class MappingBase(BaseModel):
    source_table_id: str
    source_column_id: str
    target_table_id: str
    target_column_id: str


class MappingCreate(MappingBase):
    pass


class MappingOut(MappingBase):
    id: str
    project_id: str
    created_at: datetime

    class Config:
        from_attributes = True


class MappingDetailOut(MappingOut):
    source_table_name: Optional[str] = None
    source_table_comment: Optional[str] = None
    source_column_name: Optional[str] = None
    source_column_comment: Optional[str] = None
    source_data_type: Optional[str] = None

    target_table_name: Optional[str] = None
    target_table_comment: Optional[str] = None
    target_column_name: Optional[str] = None
    target_column_comment: Optional[str] = None
    target_data_type: Optional[str] = None


class BatchMappingCreate(BaseModel):
    mappings: list[MappingCreate]


class BatchDeleteRequest(BaseModel):
    ids: list[str]
