from typing import List
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from app.core.database import get_db
from app.models.mapping import FieldMapping
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.schemas.mapping import MappingCreate, MappingOut, MappingDetailOut

router = APIRouter()


@router.get("", response_model=List[MappingDetailOut])
def list_mappings(project_id: str, db: Session = Depends(get_db)):
    mappings = db.query(FieldMapping).filter(FieldMapping.project_id == project_id).all()
    results = []

    for m in mappings:
        src_table = db.query(TableMeta).filter(TableMeta.id == m.source_table_id).first()
        src_col = db.query(ColumnMeta).filter(ColumnMeta.id == m.source_column_id).first()
        tgt_table = db.query(TableMeta).filter(TableMeta.id == m.target_table_id).first()
        tgt_col = db.query(ColumnMeta).filter(ColumnMeta.id == m.target_column_id).first()

        results.append(MappingDetailOut(
            id=m.id,
            project_id=m.project_id,
            source_table_id=m.source_table_id,
            source_column_id=m.source_column_id,
            target_table_id=m.target_table_id,
            target_column_id=m.target_column_id,
            created_at=m.created_at,
            source_table_name=src_table.table_name if src_table else "",
            source_table_comment=src_table.table_comment if src_table else "",
            source_column_name=src_col.column_name if src_col else "",
            source_column_comment=src_col.column_comment if src_col else "",
            source_data_type=src_col.data_type if src_col else "",
            target_table_name=tgt_table.table_name if tgt_table else "",
            target_table_comment=tgt_table.table_comment if tgt_table else "",
            target_column_name=tgt_col.column_name if tgt_col else "",
            target_column_comment=tgt_col.column_comment if tgt_col else "",
            target_data_type=tgt_col.data_type if tgt_col else "",
        ))
    return results


@router.post("", response_model=MappingOut)
def create_mapping(project_id: str, data: MappingCreate, db: Session = Depends(get_db)):
    # Check if identical mapping already exists
    existing = db.query(FieldMapping).filter(
        FieldMapping.project_id == project_id,
        FieldMapping.source_table_id == data.source_table_id,
        FieldMapping.source_column_id == data.source_column_id,
        FieldMapping.target_table_id == data.target_table_id,
        FieldMapping.target_column_id == data.target_column_id,
    ).first()

    if existing:
        return existing

    mapping = FieldMapping(
        project_id=project_id,
        source_table_id=data.source_table_id,
        source_column_id=data.source_column_id,
        target_table_id=data.target_table_id,
        target_column_id=data.target_column_id
    )
    db.add(mapping)
    db.commit()
    db.refresh(mapping)
    return mapping


@router.delete("/{mapping_id}")
def delete_mapping(mapping_id: str, db: Session = Depends(get_db)):
    mapping = db.query(FieldMapping).filter(FieldMapping.id == mapping_id).first()
    if not mapping:
        raise HTTPException(status_code=404, detail="Mapping not found")

    db.delete(mapping)
    db.commit()
    return {"message": "Mapping deleted successfully"}
