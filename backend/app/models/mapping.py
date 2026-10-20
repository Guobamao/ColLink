import uuid
from datetime import datetime
from sqlalchemy import Column, String, DateTime, ForeignKey
from sqlalchemy.orm import relationship
from app.core.database import Base


class FieldMapping(Base):
    __tablename__ = "field_mappings"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    project_id = Column(String(36), ForeignKey("projects.id", ondelete="CASCADE"), nullable=False)
    source_table_id = Column(String(36), ForeignKey("table_meta.id", ondelete="CASCADE"), nullable=False)
    source_column_id = Column(String(36), ForeignKey("column_meta.id", ondelete="CASCADE"), nullable=False)
    target_table_id = Column(String(36), ForeignKey("table_meta.id", ondelete="CASCADE"), nullable=False)
    target_column_id = Column(String(36), ForeignKey("column_meta.id", ondelete="CASCADE"), nullable=False)
    created_at = Column(DateTime, default=datetime.utcnow)

    # Relationships
    project = relationship("Project", back_populates="mappings")
    source_table = relationship("TableMeta", foreign_keys=[source_table_id])
    source_column = relationship("ColumnMeta", foreign_keys=[source_column_id])
    target_table = relationship("TableMeta", foreign_keys=[target_table_id])
    target_column = relationship("ColumnMeta", foreign_keys=[target_column_id])
