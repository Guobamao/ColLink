from typing import List
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from sqlalchemy import func
from app.core.database import get_db
from app.models.project import Project
from app.models.table import TableMeta
from app.models.mapping import FieldMapping
from app.models.canvas import CanvasView, CanvasNodeLayout
from app.schemas.canvas import (
    CanvasCreate, CanvasUpdate, CanvasOut,
    CanvasDetailOut, CanvasTableOut, CanvasAddTables,
    CanvasPositionUpdate, CanvasColorUpdate
)
from app.schemas.table import ColumnOut
from app.schemas.mapping import MappingOut

router = APIRouter()


def ensure_default_canvas(db: Session, project_id: str) -> List[CanvasView]:
    """Ensure a project has at least one CanvasView (auto-migrate old projects)."""
    canvases = (
        db.query(CanvasView)
        .filter(CanvasView.project_id == project_id)
        .order_by(CanvasView.sort_order.asc(), CanvasView.created_at.asc())
        .all()
    )
    if canvases:
        return canvases

    # Check project exists
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        return []

    # Create default overview canvas
    default_canvas = CanvasView(
        project_id=project_id,
        name="默认总览",
        sort_order=0
    )
    db.add(default_canvas)
    db.flush()

    # Link all existing tables in this project to the default canvas
    existing_tables = db.query(TableMeta).filter(TableMeta.project_id == project_id).all()
    for t in existing_tables:
        layout = CanvasNodeLayout(
            canvas_id=default_canvas.id,
            table_id=t.id,
            position_x=t.pos_x,
            position_y=t.pos_y,
            color_tag="blue"
        )
        db.add(layout)

    db.commit()
    db.refresh(default_canvas)
    return [default_canvas]


@router.get("/project/{project_id}", response_model=List[CanvasOut])
def list_project_canvases(project_id: str, db: Session = Depends(get_db)):
    """List all canvas views for a project, auto-initializing default canvas if needed."""
    canvases = ensure_default_canvas(db, project_id)
    results = []
    for c in canvases:
        t_count = (
            db.query(func.count(CanvasNodeLayout.id))
            .filter(CanvasNodeLayout.canvas_id == c.id)
            .scalar()
            or 0
        )
        results.append(
            CanvasOut(
                id=c.id,
                project_id=c.project_id,
                name=c.name,
                sort_order=c.sort_order,
                created_at=c.created_at,
                table_count=t_count
            )
        )
    return results


@router.post("/project/{project_id}", response_model=CanvasOut)
def create_canvas(project_id: str, data: CanvasCreate, db: Session = Depends(get_db)):
    """Create a new canvas view under the project."""
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    max_order = (
        db.query(func.max(CanvasView.sort_order))
        .filter(CanvasView.project_id == project_id)
        .scalar()
        or 0
    )

    canvas = CanvasView(
        project_id=project_id,
        name=data.name.strip() or "新建业务视图",
        sort_order=max_order + 1
    )
    db.add(canvas)
    db.commit()
    db.refresh(canvas)

    return CanvasOut(
        id=canvas.id,
        project_id=canvas.project_id,
        name=canvas.name,
        sort_order=canvas.sort_order,
        created_at=canvas.created_at,
        table_count=0
    )


@router.get("/{canvas_id}", response_model=CanvasDetailOut)
def get_canvas_detail(canvas_id: str, db: Session = Depends(get_db)):
    """Get full canvas details: tables with canvas-specific layout coordinates and color tags, plus related mappings."""
    canvas = db.query(CanvasView).filter(CanvasView.id == canvas_id).first()
    if not canvas:
        raise HTTPException(status_code=404, detail="Canvas not found")

    layouts = (
        db.query(CanvasNodeLayout)
        .filter(CanvasNodeLayout.canvas_id == canvas_id)
        .all()
    )

    table_ids = set()
    tables_out = []
    for l in layouts:
        t = l.table
        if not t:
            continue
        table_ids.add(t.id)
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
        tables_out.append(
            CanvasTableOut(
                id=t.id,
                project_id=t.project_id,
                table_name=t.table_name,
                table_comment=t.table_comment or "",
                pos_x=l.position_x,
                pos_y=l.position_y,
                color_tag=l.color_tag or "blue",
                columns=cols_out
            )
        )

    # Filter mappings where both source and target tables exist in this canvas
    mappings = []
    if table_ids:
        mappings = (
            db.query(FieldMapping)
            .filter(
                FieldMapping.project_id == canvas.project_id,
                FieldMapping.source_table_id.in_(table_ids),
                FieldMapping.target_table_id.in_(table_ids)
            )
            .all()
        )

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

    return CanvasDetailOut(
        id=canvas.id,
        project_id=canvas.project_id,
        name=canvas.name,
        sort_order=canvas.sort_order,
        created_at=canvas.created_at,
        table_count=len(tables_out),
        tables=tables_out,
        mappings=mappings_out
    )


@router.put("/{canvas_id}", response_model=CanvasOut)
def update_canvas(canvas_id: str, data: CanvasUpdate, db: Session = Depends(get_db)):
    """Rename or reorder canvas."""
    canvas = db.query(CanvasView).filter(CanvasView.id == canvas_id).first()
    if not canvas:
        raise HTTPException(status_code=404, detail="Canvas not found")

    if data.name is not None:
        canvas.name = data.name.strip()
    if data.sort_order is not None:
        canvas.sort_order = data.sort_order

    db.commit()
    db.refresh(canvas)

    t_count = (
        db.query(func.count(CanvasNodeLayout.id))
        .filter(CanvasNodeLayout.canvas_id == canvas.id)
        .scalar()
        or 0
    )

    return CanvasOut(
        id=canvas.id,
        project_id=canvas.project_id,
        name=canvas.name,
        sort_order=canvas.sort_order,
        created_at=canvas.created_at,
        table_count=t_count
    )


@router.delete("/{canvas_id}")
def delete_canvas(canvas_id: str, db: Session = Depends(get_db)):
    """Delete a canvas view. Cannot delete the last remaining canvas of a project."""
    canvas = db.query(CanvasView).filter(CanvasView.id == canvas_id).first()
    if not canvas:
        raise HTTPException(status_code=404, detail="Canvas not found")

    total_canvases = (
        db.query(func.count(CanvasView.id))
        .filter(CanvasView.project_id == canvas.project_id)
        .scalar()
        or 0
    )
    if total_canvases <= 1:
        raise HTTPException(status_code=400, detail="不能删除项目中最后一个画布")

    db.delete(canvas)
    db.commit()
    return {"message": "Canvas deleted successfully"}


@router.post("/{canvas_id}/tables")
def add_tables_to_canvas(canvas_id: str, data: CanvasAddTables, db: Session = Depends(get_db)):
    """Import existing project tables into this canvas view."""
    canvas = db.query(CanvasView).filter(CanvasView.id == canvas_id).first()
    if not canvas:
        raise HTTPException(status_code=404, detail="Canvas not found")

    existing_layouts = (
        db.query(CanvasNodeLayout)
        .filter(CanvasNodeLayout.canvas_id == canvas_id)
        .all()
    )
    existing_table_ids = {l.table_id for l in existing_layouts}

    idx = len(existing_layouts)
    for t_id in data.table_ids:
        if t_id not in existing_table_ids:
            # Check table belongs to same project
            table = db.query(TableMeta).filter(TableMeta.id == t_id, TableMeta.project_id == canvas.project_id).first()
            if table:
                # Staggered arrangement
                pos_x = 100.0 + (idx % 3) * 360.0
                pos_y = 100.0 + (idx // 3) * 320.0
                layout = CanvasNodeLayout(
                    canvas_id=canvas_id,
                    table_id=t_id,
                    position_x=pos_x,
                    position_y=pos_y,
                    color_tag="blue"
                )
                db.add(layout)
                idx += 1

    db.commit()
    return {"message": "Tables added to canvas successfully"}


@router.delete("/{canvas_id}/tables/{table_id}")
def remove_table_from_canvas(canvas_id: str, table_id: str, db: Session = Depends(get_db)):
    """Remove a table from this canvas view without deleting the underlying project table."""
    layout = (
        db.query(CanvasNodeLayout)
        .filter(CanvasNodeLayout.canvas_id == canvas_id, CanvasNodeLayout.table_id == table_id)
        .first()
    )
    if not layout:
        raise HTTPException(status_code=404, detail="Table is not in this canvas")

    db.delete(layout)
    db.commit()
    return {"message": "Table removed from canvas successfully"}


@router.put("/{canvas_id}/positions")
def update_canvas_positions(
    canvas_id: str,
    positions: List[CanvasPositionUpdate],
    db: Session = Depends(get_db)
):
    """Batch update node positions specifically for this canvas view."""
    for p in positions:
        layout = (
            db.query(CanvasNodeLayout)
            .filter(CanvasNodeLayout.canvas_id == canvas_id, CanvasNodeLayout.table_id == p.id)
            .first()
        )
        if layout:
            layout.position_x = p.pos_x
            layout.position_y = p.pos_y
    db.commit()
    return {"message": "Canvas positions updated"}


@router.put("/{canvas_id}/tables/{table_id}/color")
def update_canvas_table_color(
    canvas_id: str,
    table_id: str,
    data: CanvasColorUpdate,
    db: Session = Depends(get_db)
):
    """Update a table's color tag within this canvas view."""
    layout = (
        db.query(CanvasNodeLayout)
        .filter(CanvasNodeLayout.canvas_id == canvas_id, CanvasNodeLayout.table_id == table_id)
        .first()
    )
    if not layout:
        raise HTTPException(status_code=404, detail="Table is not in this canvas")

    layout.color_tag = data.color_tag
    db.commit()
    return {"message": "Color tag updated", "color_tag": layout.color_tag}
