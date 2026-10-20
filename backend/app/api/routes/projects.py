from typing import List
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from sqlalchemy import func
from app.core.database import get_db
from app.models.project import Project
from app.models.table import TableMeta
from app.models.mapping import FieldMapping
from app.schemas.project import ProjectCreate, ProjectUpdate, ProjectOut, ProjectDetailOut
from app.schemas.table import TableWithColumnsOut, ColumnOut
from app.schemas.mapping import MappingOut

router = APIRouter()


@router.get("", response_model=List[ProjectOut])
def list_projects(db: Session = Depends(get_db)):
    projects = db.query(Project).order_by(Project.updated_at.desc()).all()
    results = []
    for p in projects:
        t_count = db.query(func.count(TableMeta.id)).filter(TableMeta.project_id == p.id).scalar() or 0
        m_count = db.query(func.count(FieldMapping.id)).filter(FieldMapping.project_id == p.id).scalar() or 0
        p_out = ProjectOut(
            id=p.id,
            name=p.name,
            description=p.description or "",
            created_at=p.created_at,
            updated_at=p.updated_at,
            table_count=t_count,
            mapping_count=m_count
        )
        results.append(p_out)
    return results


@router.post("", response_model=ProjectOut)
def create_project(data: ProjectCreate, db: Session = Depends(get_db)):
    project = Project(
        name=data.name,
        description=data.description or ""
    )
    db.add(project)
    db.commit()
    db.refresh(project)
    return ProjectOut(
        id=project.id,
        name=project.name,
        description=project.description or "",
        created_at=project.created_at,
        updated_at=project.updated_at,
        table_count=0,
        mapping_count=0
    )


@router.get("/{project_id}", response_model=ProjectDetailOut)
def get_project_detail(project_id: str, db: Session = Depends(get_db)):
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    tables = db.query(TableMeta).filter(TableMeta.project_id == project_id).all()
    mappings = db.query(FieldMapping).filter(FieldMapping.project_id == project_id).all()

    tables_out = []
    for t in tables:
        cols_out = [
            ColumnOut(
                id=c.id,
                table_id=c.table_id,
                column_name=c.column_name,
                data_type=c.data_type,
                column_comment=c.column_comment or "",
                sort_order=c.sort_order
            )
            for c in t.columns
        ]
        tables_out.append(TableWithColumnsOut(
            id=t.id,
            project_id=t.project_id,
            table_name=t.table_name,
            table_comment=t.table_comment or "",
            pos_x=t.pos_x,
            pos_y=t.pos_y,
            columns=cols_out
        ))

    mappings_out = [
        MappingOut(
            id=m.id,
            project_id=m.project_id,
            source_table_id=m.source_table_id,
            source_column_id=m.source_column_id,
            target_table_id=m.target_table_id,
            target_column_id=m.target_column_id,
            created_at=m.created_at
        )
        for m in mappings
    ]

    return ProjectDetailOut(
        id=project.id,
        name=project.name,
        description=project.description or "",
        created_at=project.created_at,
        updated_at=project.updated_at,
        table_count=len(tables),
        mapping_count=len(mappings),
        tables=tables_out,
        mappings=mappings_out
    )


@router.put("/{project_id}", response_model=ProjectOut)
def update_project(project_id: str, data: ProjectUpdate, db: Session = Depends(get_db)):
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    if data.name is not None:
        project.name = data.name
    if data.description is not None:
        project.description = data.description

    db.commit()
    db.refresh(project)

    t_count = db.query(func.count(TableMeta.id)).filter(TableMeta.project_id == project.id).scalar() or 0
    m_count = db.query(func.count(FieldMapping.id)).filter(FieldMapping.project_id == project.id).scalar() or 0

    return ProjectOut(
        id=project.id,
        name=project.name,
        description=project.description or "",
        created_at=project.created_at,
        updated_at=project.updated_at,
        table_count=t_count,
        mapping_count=m_count
    )


@router.delete("/{project_id}")
def delete_project(project_id: str, db: Session = Depends(get_db)):
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    db.delete(project)
    db.commit()
    return {"message": "Project deleted successfully"}
