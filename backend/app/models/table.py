import uuid
from sqlalchemy import Column, String, Float, ForeignKey, Text
from sqlalchemy.orm import relationship
from app.core.database import Base


class TableMeta(Base):
    __tablename__ = "table_meta"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    project_id = Column(String(36), ForeignKey("projects.id", ondelete="CASCADE"), nullable=False)
    table_name = Column(String(150), nullable=False)
    table_comment = Column(Text, nullable=True, default="")
    pos_x = Column(Float, nullable=False, default=100.0)
    pos_y = Column(Float, nullable=False, default=100.0)

    # Relationships
    project = relationship("Project", back_populates="tables")
    columns = relationship("ColumnMeta", back_populates="table", cascade="all, delete-orphan", order_by="ColumnMeta.sort_order")
