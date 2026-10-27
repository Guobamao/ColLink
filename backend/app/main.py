from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.core.database import Base, engine, SessionLocal
from app.api.api import api_router
from app.models.project import Project
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.models.mapping import FieldMapping
from app.models.canvas import CanvasView, CanvasNodeLayout

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
    title="ColLink API",
    description="ColLink 后端服务",
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

import os
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles

app.include_router(api_router, prefix="/api")

# Mount MCP Server (SSE transport for web and network-based LLM agents)
try:
    from app.mcp.server import mcp_server
    app.mount("/mcp", mcp_server.sse_app())
except Exception as e:
    import logging
    logging.warning(f"Failed to mount MCP server SSE app: {e}")

# Static files & SPA frontend serving
BASE_DIR = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DIST_DIR = os.path.join(BASE_DIR, "frontend", "dist")

if os.path.exists(DIST_DIR):
    assets_dir = os.path.join(DIST_DIR, "assets")
    if os.path.exists(assets_dir):
        app.mount("/assets", StaticFiles(directory=assets_dir), name="assets")

    @app.get("/{full_path:path}")
    async def serve_spa(full_path: str):
        target_file = os.path.join(DIST_DIR, full_path)
        if full_path and os.path.isfile(target_file):
            return FileResponse(target_file)
        
        response = FileResponse(os.path.join(DIST_DIR, "index.html"))
        response.headers["Cache-Control"] = "no-cache, no-store, must-revalidate, max-age=0"
        response.headers["Pragma"] = "no-cache"
        response.headers["Expires"] = "0"
        return response
else:
    @app.get("/")
    def health_check():
        return {"status": "ok", "app": "ColLink API", "version": "1.0.0"}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("app.main:app", host="0.0.0.0", port=8000, reload=True)
