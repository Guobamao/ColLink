import uuid
from datetime import datetime
from sqlalchemy import Column, String, DateTime, Text
from sqlalchemy.orm import relationship
from app.core.database import Base


class Project(Base):
    __tablename__ = "projects"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    name = Column(String(100), nullable=False)
    description = Column(Text, nullable=True, default="")
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    # Relationships
    tables = relationship("TableMeta", back_populates="project", cascade="all, delete-orphan")
    mappings = relationship("FieldMapping", back_populates="project", cascade="all, delete-orphan")
    canvases = relationship("CanvasView", back_populates="project", cascade="all, delete-orphan", order_by="CanvasView.sort_order")
