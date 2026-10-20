from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.core.database import Base, engine, SessionLocal
from app.api.api import api_router
from app.models.project import Project
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.models.mapping import FieldMapping

# Ensure all tables exist immediately upon import
Base.metadata.create_all(bind=engine)


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Create default demo project if empty
    db = SessionLocal()
    try:
        count = db.query(Project).count()
        if count == 0:
            demo_project = Project(
                name="示例数据治理项目 (达梦/ODS-DWD)",
                description="演示源端 ODS 表与目标端 DWD 明细表之间的字段映射与血缘关系"
            )
            db.add(demo_project)
            db.commit()
    finally:
        db.close()

    yield


app = FastAPI(
    title="DataField Mapper API",
    description="数据字段血缘与映射可视化工具后端服务",
    version="1.0.0",
    lifespan=lifespan
)

# CORS configuration
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(api_router, prefix="/api")


@app.get("/")
def health_check():
    return {"status": "ok", "app": "DataField Mapper API", "version": "1.0.0"}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("app.main:app", host="0.0.0.0", port=8000, reload=True)
