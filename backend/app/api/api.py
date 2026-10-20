from fastapi import APIRouter
from app.api.routes import projects, tables, mappings, ddl, connections, export

import time

api_router = APIRouter()

# Global state for client lifecycle management
APP_STATE = {
    "last_heartbeat": time.time(),
    "shutdown_requested": False,
    "started_at": time.time(),
}

api_router.include_router(projects.router, prefix="/projects", tags=["projects"])
api_router.include_router(tables.router, prefix="/tables", tags=["tables"])
api_router.include_router(mappings.router, prefix="/mappings", tags=["mappings"])
api_router.include_router(ddl.router, prefix="/ddl", tags=["ddl"])
api_router.include_router(connections.router, prefix="/connections", tags=["connections"])
api_router.include_router(export.router, prefix="/export", tags=["export"])


@api_router.get("/health")
def health():
    return {"status": "ok", "app": "ColLink API", "version": "1.0.0"}


@api_router.post("/heartbeat")
def heartbeat():
    APP_STATE["last_heartbeat"] = time.time()
    return {"status": "alive"}


@api_router.post("/shutdown")
def shutdown():
    APP_STATE["shutdown_requested"] = True
    return {"status": "shutting_down"}
