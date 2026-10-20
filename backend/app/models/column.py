import uuid
from sqlalchemy import Column, String, Integer, ForeignKey, Text
from sqlalchemy.orm import relationship
from app.core.database import Base


class ColumnMeta(Base):
    __tablename__ = "column_meta"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    table_id = Column(String(36), ForeignKey("table_meta.id", ondelete="CASCADE"), nullable=False)
    column_name = Column(String(150), nullable=False)
    data_type = Column(String(100), nullable=False, default="VARCHAR(255)")
    column_comment = Column(Text, nullable=True, default="")
    sort_order = Column(Integer, nullable=False, default=0)

    # Relationships
    table = relationship("TableMeta", back_populates="columns")
