from fastapi import APIRouter
from app.api.routes import projects, tables, mappings, ddl, connections, export, canvases

import time

api_router = APIRouter()

# Global state for client lifecycle management
APP_STATE = {
    "last_heartbeat": time.time(),
    "has_connected": False,
    "shutdown_requested_at": None,
    "started_at": time.time(),
}

api_router.include_router(projects.router, prefix="/projects", tags=["projects"])
api_router.include_router(canvases.router, prefix="/canvases", tags=["canvases"])
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
    APP_STATE["has_connected"] = True
    APP_STATE["shutdown_requested_at"] = None  # Cancel shutdown if user merely refreshed
    return {"status": "alive"}


@api_router.post("/shutdown")
def shutdown():
    APP_STATE["shutdown_requested_at"] = time.time()
    return {"status": "shutdown_scheduled"}
