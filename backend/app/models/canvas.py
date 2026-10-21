import uuid
from datetime import datetime
from sqlalchemy import Column, String, Integer, Float, DateTime, ForeignKey
from sqlalchemy.orm import relationship
from app.core.database import Base


class CanvasView(Base):
    __tablename__ = "canvas_views"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    project_id = Column(String(36), ForeignKey("projects.id", ondelete="CASCADE"), nullable=False)
    name = Column(String(100), nullable=False)
    sort_order = Column(Integer, default=0)
    created_at = Column(DateTime, default=datetime.utcnow)

    # Relationships
    project = relationship("Project", back_populates="canvases")
    node_layouts = relationship("CanvasNodeLayout", back_populates="canvas", cascade="all, delete-orphan")


class CanvasNodeLayout(Base):
    __tablename__ = "canvas_node_layouts"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    canvas_id = Column(String(36), ForeignKey("canvas_views.id", ondelete="CASCADE"), nullable=False)
    table_id = Column(String(36), ForeignKey("table_meta.id", ondelete="CASCADE"), nullable=False)
    position_x = Column(Float, nullable=False, default=100.0)
    position_y = Column(Float, nullable=False, default=100.0)
    color_tag = Column(String(30), nullable=False, default="blue")
    created_at = Column(DateTime, default=datetime.utcnow)

    # Relationships
    canvas = relationship("CanvasView", back_populates="node_layouts")
    table = relationship("TableMeta", back_populates="node_layouts")
