from typing import List, Optional
from pydantic import BaseModel


class ParsedColumn(BaseModel):
    column_name: str
    data_type: str = "VARCHAR(255)"
    column_comment: Optional[str] = ""
    sort_order: int = 0


class ParsedTable(BaseModel):
    table_name: str
    table_comment: Optional[str] = ""
    columns: List[ParsedColumn] = []


class DdlParseRequest(BaseModel):
    sql_text: str
    dialect: Optional[str] = "dm"  # dm, mysql, postgres, oracle, general


class DdlImportRequest(BaseModel):
    project_id: str
    tables: List[ParsedTable]
    canvas_id: Optional[str] = None
