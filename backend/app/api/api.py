from fastapi import APIRouter
from app.api.routes import projects, tables, mappings, ddl, connections, export

api_router = APIRouter()

api_router.include_router(projects.router, prefix="/projects", tags=["projects"])
api_router.include_router(tables.router, prefix="/tables", tags=["tables"])
api_router.include_router(mappings.router, prefix="/mappings", tags=["mappings"])
api_router.include_router(ddl.router, prefix="/ddl", tags=["ddl"])
api_router.include_router(connections.router, prefix="/connections", tags=["connections"])
api_router.include_router(export.router, prefix="/export", tags=["export"])
