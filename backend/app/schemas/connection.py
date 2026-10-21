from typing import List, Optional
from pydantic import BaseModel
from app.schemas.ddl import ParsedTable


class DbConnectionRequest(BaseModel):
    db_type: str  # dm, mysql, postgres
    host: str
    port: int
    database: str
    schema_name: Optional[str] = None
    username: str
    password: str


class DbTableSummary(BaseModel):
    table_name: str
    table_comment: Optional[str] = ""
    column_count: Optional[int] = 0


class DbMetadataImportRequest(BaseModel):
    project_id: str
    connection: DbConnectionRequest
    selected_tables: Optional[List[str]] = None  # None or empty = all tables
    canvas_id: Optional[str] = None
