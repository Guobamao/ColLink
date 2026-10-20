import urllib.parse
from fastapi import APIRouter, Depends, HTTPException, UploadFile, File
from fastapi.responses import StreamingResponse, JSONResponse
from sqlalchemy.orm import Session
from app.core.database import get_db
from app.models.project import Project
from app.schemas.project import ProjectOut
from app.services.exporter import ExporterService

router = APIRouter()


@router.get("/excel/{project_id}")
def export_excel_report(project_id: str, db: Session = Depends(get_db)):
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="项目不存在")

    try:
        excel_stream = ExporterService.export_excel(db, project_id)
        # URL encode filename for UTF-8 compatibility
        filename = f"{project.name}_字段血缘映射表.xlsx"
        encoded_filename = urllib.parse.quote(filename)

        return StreamingResponse(
            excel_stream,
            media_type="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            headers={
                "Content-Disposition": f"attachment; filename*=UTF-8''{encoded_filename}"
            }
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"导出 Excel 失败: {str(e)}")


@router.get("/json/{project_id}")
def export_project_json(project_id: str, db: Session = Depends(get_db)):
    try:
        data = ExporterService.export_project_json(db, project_id)
        return data
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"导出 JSON 备份失败: {str(e)}")


@router.post("/restore-json", response_model=ProjectOut)
async def restore_project_json(
    file: UploadFile = File(...),
    db: Session = Depends(get_db)
):
    try:
        contents = await file.read()
        import json
        data = json.loads(contents.decode("utf-8"))
        restored = ExporterService.import_project_json(db, data)

        return ProjectOut(
            id=restored.id,
            name=restored.name,
            description=restored.description or "",
            created_at=restored.created_at,
            updated_at=restored.updated_at,
            table_count=len(restored.tables),
            mapping_count=len(restored.mappings)
        )
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"恢复项目失败: {str(e)}")
