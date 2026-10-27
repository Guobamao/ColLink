"""
ColLink MCP (Model Context Protocol) Server
Exposes ColLink metadata, field lineage, DDL parser, and ETL generation tools to LLMs.
Compatible with MCP 2.x SDK (MCPServer).
"""

import os
import sys
from typing import Optional, List, Dict, Any
from mcp.server.mcpserver import MCPServer

# Ensure project root is in sys.path
CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
APP_DIR = os.path.dirname(CURRENT_DIR)
BACKEND_DIR = os.path.dirname(APP_DIR)
if BACKEND_DIR not in sys.path:
    sys.path.insert(0, BACKEND_DIR)

from app.core.database import SessionLocal
from app.models.project import Project
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.models.mapping import FieldMapping
from app.models.canvas import CanvasView
from app.services.ddl_parser import DdlParser

# Initialize MCP Server
mcp_server = MCPServer(
    name="ColLink-Metadata-Server",
    version="1.0.0",
    description="ColLink 可视化数据血缘与字段映射平台的 MCP 服务，提供元数据检索、字段血缘追溯、智能映射与 ETL 生成能力。"
)


# ==============================================================================
# 1. Projects & Canvas Tools (项目与画布)
# ==============================================================================

@mcp_server.tool(
    name="collink_list_projects",
    description="列出 ColLink 中所有数据血缘项目，包含项目 ID、名称、描述、表数量、映射关系数量。"
)
def collink_list_projects() -> List[Dict[str, Any]]:
    with SessionLocal() as db:
        projects = db.query(Project).all()
        results = []
        for p in projects:
            table_count = db.query(TableMeta).filter(TableMeta.project_id == p.id).count()
            mapping_count = db.query(FieldMapping).filter(FieldMapping.project_id == p.id).count()
            results.append({
                "project_id": p.id,
                "project_name": p.name,
                "description": p.description or "",
                "table_count": table_count,
                "mapping_count": mapping_count,
                "created_at": str(p.created_at) if p.created_at else ""
            })
        return results


@mcp_server.tool(
    name="collink_list_canvases",
    description="获取指定项目下的所有主题画布（业务主题视图/全景总览），返回画布名称、是否默认以及包含的数据表 ID。"
)
def collink_list_canvases(project_id: str) -> List[Dict[str, Any]]:
    with SessionLocal() as db:
        canvases = db.query(CanvasView).filter(CanvasView.project_id == project_id).all()
        results = []
        for c in canvases:
            layouts = c.node_layouts or []
            table_ids = [l.table_id for l in layouts]
            results.append({
                "canvas_id": c.id,
                "canvas_name": c.name,
                "sort_order": c.sort_order,
                "table_count": len(table_ids),
                "table_ids": table_ids
            })
        return results


# ==============================================================================
# 2. Metadata & Schema Tools (元数据与表结构)
# ==============================================================================

@mcp_server.tool(
    name="collink_get_tables",
    description="获取指定项目下所有表或特定表的详细结构定义，包含表名、中文注释及所有字段（名称、数据类型、注释、是否主键）。"
)
def collink_get_tables(project_id: str, table_name: Optional[str] = None) -> List[Dict[str, Any]]:
    with SessionLocal() as db:
        query = db.query(TableMeta).filter(TableMeta.project_id == project_id)
        if table_name:
            query = query.filter(TableMeta.table_name.ilike(table_name))
        
        tables = query.all()
        results = []
        for t in tables:
            cols = []
            for c in t.columns:
                is_pk = c.column_name.lower() == 'id' or c.column_name.lower().endswith('_id')
                cols.append({
                    "column_id": c.id,
                    "column_name": c.column_name,
                    "data_type": c.data_type,
                    "column_comment": c.column_comment or "",
                    "is_primary_key": is_pk
                })
            results.append({
                "table_id": t.id,
                "table_name": t.table_name,
                "table_comment": t.table_comment or "",
                "column_count": len(cols),
                "columns": cols
            })
        return results


@mcp_server.tool(
    name="collink_search_columns",
    description="跨表模糊搜索字段。在项目所有数据表中按字段名或中文注释检索关键词（如'金额', 'user_id', '创建时间'），快速定位指标与属性。"
)
def collink_search_columns(project_id: str, keyword: str) -> List[Dict[str, Any]]:
    with SessionLocal() as db:
        tables = db.query(TableMeta).filter(TableMeta.project_id == project_id).all()
        matches = []
        kw_lower = keyword.strip().lower()

        for t in tables:
            for c in t.columns:
                name_hit = kw_lower in c.column_name.lower()
                comment_hit = bool(c.column_comment and kw_lower in c.column_comment.lower())
                if name_hit or comment_hit:
                    is_pk = c.column_name.lower() == 'id' or c.column_name.lower().endswith('_id')
                    matches.append({
                        "table_name": t.table_name,
                        "table_comment": t.table_comment or "",
                        "column_name": c.column_name,
                        "data_type": c.data_type,
                        "column_comment": c.column_comment or "",
                        "is_primary_key": is_pk,
                        "match_type": "name" if name_hit else "comment"
                    })
        return matches


# ==============================================================================
# 3. Field Lineage & Mappings Tools (字段血缘与映射管理)
# ==============================================================================

@mcp_server.tool(
    name="collink_get_field_lineage",
    description="查询指定表字段的完整上下游血缘关系。向上追踪源头来源字段（Upstream），向下追踪派生下游字段（Downstream）。"
)
def collink_get_field_lineage(project_id: str, table_name: str, column_name: str) -> Dict[str, Any]:
    with SessionLocal() as db:
        target_tbl = db.query(TableMeta).filter(
            TableMeta.project_id == project_id,
            TableMeta.table_name.ilike(table_name)
        ).first()
        if not target_tbl:
            return {"error": f"Table '{table_name}' not found in project '{project_id}'"}

        target_col = db.query(ColumnMeta).filter(
            ColumnMeta.table_id == target_tbl.id,
            ColumnMeta.column_name.ilike(column_name)
        ).first()
        if not target_col:
            return {"error": f"Column '{column_name}' not found in table '{table_name}'"}

        # Upstream: where this column is the target
        up_mappings = db.query(FieldMapping).filter(
            FieldMapping.project_id == project_id,
            FieldMapping.target_column_id == target_col.id
        ).all()
        upstream = []
        for m in up_mappings:
            st = db.query(TableMeta).filter(TableMeta.id == m.source_table_id).first()
            sc = db.query(ColumnMeta).filter(ColumnMeta.id == m.source_column_id).first()
            if st and sc:
                upstream.append({
                    "mapping_id": m.id,
                    "source_table": st.table_name,
                    "source_column": sc.column_name,
                    "source_type": sc.data_type,
                    "source_comment": sc.column_comment or ""
                })

        # Downstream: where this column is the source
        down_mappings = db.query(FieldMapping).filter(
            FieldMapping.project_id == project_id,
            FieldMapping.source_column_id == target_col.id
        ).all()
        downstream = []
        for m in down_mappings:
            tt = db.query(TableMeta).filter(TableMeta.id == m.target_table_id).first()
            tc = db.query(ColumnMeta).filter(ColumnMeta.id == m.target_column_id).first()
            if tt and tc:
                downstream.append({
                    "mapping_id": m.id,
                    "target_table": tt.table_name,
                    "target_column": tc.column_name,
                    "target_type": tc.data_type,
                    "target_comment": tc.column_comment or ""
                })

        return {
            "table_name": target_tbl.table_name,
            "column_name": target_col.column_name,
            "data_type": target_col.data_type,
            "column_comment": target_col.column_comment or "",
            "upstream_count": len(upstream),
            "upstream": upstream,
            "downstream_count": len(downstream),
            "downstream": downstream
        }


@mcp_server.tool(
    name="collink_list_mappings",
    description="查询项目中已配置的字段映射关系清单，支持按源表名或目标表名进行过滤。"
)
def collink_list_mappings(
    project_id: str,
    source_table: Optional[str] = None,
    target_table: Optional[str] = None
) -> List[Dict[str, Any]]:
    with SessionLocal() as db:
        mappings = db.query(FieldMapping).filter(FieldMapping.project_id == project_id).all()
        results = []

        table_cache: Dict[str, TableMeta] = {}
        col_cache: Dict[str, ColumnMeta] = {}

        def get_table(tid: str):
            if tid not in table_cache:
                table_cache[tid] = db.query(TableMeta).filter(TableMeta.id == tid).first()
            return table_cache[tid]

        def get_col(cid: str):
            if cid not in col_cache:
                col_cache[cid] = db.query(ColumnMeta).filter(ColumnMeta.id == cid).first()
            return col_cache[cid]

        for m in mappings:
            st = get_table(m.source_table_id)
            sc = get_col(m.source_column_id)
            tt = get_table(m.target_table_id)
            tc = get_col(m.target_column_id)

            if not (st and sc and tt and tc):
                continue

            if source_table and st.table_name.lower() != source_table.lower():
                continue
            if target_table and tt.table_name.lower() != target_table.lower():
                continue

            results.append({
                "mapping_id": m.id,
                "source_table": st.table_name,
                "source_column": sc.column_name,
                "source_data_type": sc.data_type,
                "target_table": tt.table_name,
                "target_column": tc.column_name,
                "target_data_type": tc.data_type,
                "label": f"{st.table_name}.{sc.column_name} → {tt.table_name}.{tc.column_name}"
            })

        return results


@mcp_server.tool(
    name="collink_create_mapping",
    description="在源表字段与目标表字段之间创建一条字段映射血缘连线。"
)
def collink_create_mapping(
    project_id: str,
    source_table: str,
    source_column: str,
    target_table: str,
    target_column: str
) -> Dict[str, Any]:
    with SessionLocal() as db:
        st = db.query(TableMeta).filter(TableMeta.project_id == project_id, TableMeta.table_name.ilike(source_table)).first()
        if not st:
            return {"success": False, "error": f"源表 '{source_table}' 不存在"}
        sc = db.query(ColumnMeta).filter(ColumnMeta.table_id == st.id, ColumnMeta.column_name.ilike(source_column)).first()
        if not sc:
            return {"success": False, "error": f"源字段 '{source_column}' 在表 '{source_table}' 中不存在"}

        tt = db.query(TableMeta).filter(TableMeta.project_id == project_id, TableMeta.table_name.ilike(target_table)).first()
        if not tt:
            return {"success": False, "error": f"目标表 '{target_table}' 不存在"}
        tc = db.query(ColumnMeta).filter(ColumnMeta.table_id == tt.id, ColumnMeta.column_name.ilike(target_column)).first()
        if not tc:
            return {"success": False, "error": f"目标字段 '{target_column}' 在表 '{target_table}' 中不存在"}

        # Check existing
        existing = db.query(FieldMapping).filter(
            FieldMapping.project_id == project_id,
            FieldMapping.source_table_id == st.id,
            FieldMapping.source_column_id == sc.id,
            FieldMapping.target_table_id == tt.id,
            FieldMapping.target_column_id == tc.id
        ).first()

        if existing:
            return {
                "success": True,
                "message": "映射已存在，无需重复创建",
                "mapping_id": existing.id
            }

        mapping = FieldMapping(
            project_id=project_id,
            source_table_id=st.id,
            source_column_id=sc.id,
            target_table_id=tt.id,
            target_column_id=tc.id
        )
        db.add(mapping)
        db.commit()
        db.refresh(mapping)

        return {
            "success": True,
            "message": f"成功建立映射: {st.table_name}.{sc.column_name} → {tt.table_name}.{tc.column_name}",
            "mapping_id": mapping.id
        }


@mcp_server.tool(
    name="collink_batch_create_mappings",
    description="在两张表之间批量建立多个字段映射关系。传入 field_pairs 如 [['source_col1', 'target_col1'], ['source_col2', 'target_col2']]."
)
def collink_batch_create_mappings(
    project_id: str,
    source_table: str,
    target_table: str,
    field_pairs: List[List[str]]
) -> Dict[str, Any]:
    with SessionLocal() as db:
        st = db.query(TableMeta).filter(TableMeta.project_id == project_id, TableMeta.table_name.ilike(source_table)).first()
        tt = db.query(TableMeta).filter(TableMeta.project_id == project_id, TableMeta.table_name.ilike(target_table)).first()
        if not st or not tt:
            return {"success": False, "error": f"源表或目标表不存在: {source_table}, {target_table}"}

        src_cols = {c.column_name.lower(): c.id for c in st.columns}
        tgt_cols = {c.column_name.lower(): c.id for c in tt.columns}

        created_count = 0
        errors = []

        for pair in field_pairs:
            if len(pair) < 2:
                continue
            sc_name, tc_name = pair[0], pair[1]
            sc_id = src_cols.get(sc_name.lower())
            tc_id = tgt_cols.get(tc_name.lower())

            if not sc_id:
                errors.append(f"源字段不存在: {sc_name}")
                continue
            if not tc_id:
                errors.append(f"目标字段不存在: {tc_name}")
                continue

            existing = db.query(FieldMapping).filter(
                FieldMapping.project_id == project_id,
                FieldMapping.source_table_id == st.id,
                FieldMapping.source_column_id == sc_id,
                FieldMapping.target_table_id == tt.id,
                FieldMapping.target_column_id == tc_id
            ).first()

            if not existing:
                mapping = FieldMapping(
                    project_id=project_id,
                    source_table_id=st.id,
                    source_column_id=sc_id,
                    target_table_id=tt.id,
                    target_column_id=tc_id
                )
                db.add(mapping)
                created_count += 1

        db.commit()
        return {
            "success": True,
            "created_count": created_count,
            "errors": errors,
            "message": f"成功建立 {created_count} 条映射关系"
        }


@mcp_server.tool(
    name="collink_auto_map_fields",
    description="对源表与目标表执行智能映射推断（基于同名字段、注释相似度与类型匹配），返回推荐的匹配候选对。若 apply_mappings=True 则直接持久化写入映射。"
)
def collink_auto_map_fields(
    project_id: str,
    source_table: str,
    target_table: str,
    apply_mappings: bool = False
) -> Dict[str, Any]:
    with SessionLocal() as db:
        st = db.query(TableMeta).filter(TableMeta.project_id == project_id, TableMeta.table_name.ilike(source_table)).first()
        tt = db.query(TableMeta).filter(TableMeta.project_id == project_id, TableMeta.table_name.ilike(target_table)).first()
        if not st or not tt:
            return {"error": f"表未找到: {source_table} 或 {target_table}"}

        candidates = []
        applied_count = 0

        for tc in tt.columns:
            tc_name = tc.column_name.lower()
            tc_comment = (tc.column_comment or "").lower()

            best_match = None
            best_score = 0.0
            match_reason = ""

            for sc in st.columns:
                sc_name = sc.column_name.lower()
                sc_comment = (sc.column_comment or "").lower()

                # Rule 1: Exact name match
                if sc_name == tc_name:
                    best_match = sc
                    best_score = 1.0
                    match_reason = "字段名完全一致"
                    break
                
                # Rule 2: Substring or prefix match (e.g. user_id vs id)
                if sc_name.endswith(f"_{tc_name}") or tc_name.endswith(f"_{sc_name}"):
                    if best_score < 0.85:
                        best_match = sc
                        best_score = 0.85
                        match_reason = "字段名主干/外键匹配"

                # Rule 3: Comment exact match
                if tc_comment and sc_comment and tc_comment == sc_comment:
                    if best_score < 0.9:
                        best_match = sc
                        best_score = 0.9
                        match_reason = f"中文注释完全相同: '{tc.column_comment}'"

                # Rule 4: Comment substring match
                elif tc_comment and sc_comment and (tc_comment in sc_comment or sc_comment in tc_comment):
                    if best_score < 0.75:
                        best_match = sc
                        best_score = 0.75
                        match_reason = f"中文注释包含匹配: '{sc.column_comment}' ~ '{tc.column_comment}'"

            if best_match and best_score >= 0.7:
                candidate_info = {
                    "source_column": best_match.column_name,
                    "target_column": tc.column_name,
                    "source_comment": best_match.column_comment or "",
                    "target_comment": tc.column_comment or "",
                    "source_type": best_match.data_type,
                    "target_type": tc.data_type,
                    "confidence": round(best_score, 2),
                    "reason": match_reason
                }
                candidates.append(candidate_info)

                if apply_mappings:
                    existing = db.query(FieldMapping).filter(
                        FieldMapping.project_id == project_id,
                        FieldMapping.source_table_id == st.id,
                        FieldMapping.source_column_id == best_match.id,
                        FieldMapping.target_table_id == tt.id,
                        FieldMapping.target_column_id == tc.id
                    ).first()
                    if not existing:
                        db.add(FieldMapping(
                            project_id=project_id,
                            source_table_id=st.id,
                            source_column_id=best_match.id,
                            target_table_id=tt.id,
                            target_column_id=tc.id
                        ))
                        applied_count += 1

        if apply_mappings:
            db.commit()

        return {
            "source_table": st.table_name,
            "target_table": tt.table_name,
            "suggested_count": len(candidates),
            "applied_count": applied_count,
            "candidates": candidates
        }


# ==============================================================================
# 4. Data Quality & Audit (质量审计与覆盖率检查)
# ==============================================================================

@mcp_server.tool(
    name="collink_audit_unmapped_fields",
    description="审计指定目标表的血缘覆盖率，统计已映射与未映射字段，列出缺失来源的孤儿字段清单以供补齐。"
)
def collink_audit_unmapped_fields(project_id: str, target_table: str) -> Dict[str, Any]:
    with SessionLocal() as db:
        tt = db.query(TableMeta).filter(TableMeta.project_id == project_id, TableMeta.table_name.ilike(target_table)).first()
        if not tt:
            return {"error": f"目标表 '{target_table}' 不存在"}

        mapped_col_ids = set(
            row[0] for row in db.query(FieldMapping.target_column_id).filter(
                FieldMapping.project_id == project_id,
                FieldMapping.target_table_id == tt.id
            ).all()
        )

        mapped_cols = []
        unmapped_cols = []

        for c in tt.columns:
            is_pk = c.column_name.lower() == 'id' or c.column_name.lower().endswith('_id')
            info = {
                "column_name": c.column_name,
                "data_type": c.data_type,
                "column_comment": c.column_comment or "",
                "is_primary_key": is_pk
            }
            if c.id in mapped_col_ids:
                mapped_cols.append(info)
            else:
                unmapped_cols.append(info)

        total = len(tt.columns)
        coverage_rate = f"{(len(mapped_cols) / total * 100):.1f}%" if total > 0 else "0.0%"

        return {
            "target_table": tt.table_name,
            "table_comment": tt.table_comment or "",
            "total_columns": total,
            "mapped_count": len(mapped_cols),
            "unmapped_count": len(unmapped_cols),
            "coverage_rate": coverage_rate,
            "unmapped_columns": unmapped_cols
        }


# ==============================================================================
# 5. DDL & ETL Tools (DDL 解析导入与 ETL SQL 生成)
# ==============================================================================

@mcp_server.tool(
    name="collink_import_ddl",
    description="解析建表 DDL SQL 脚本并在指定项目中创建表结构与字段模型（支持 MySQL, Oracle, PostgreSQL, 达梦 等方言语法）。"
)
def collink_import_ddl(project_id: str, ddl_sql: str) -> Dict[str, Any]:
    with SessionLocal() as db:
        p = db.query(Project).filter(Project.id == project_id).first()
        if not p:
            return {"success": False, "error": f"项目 ID '{project_id}' 不存在"}

        try:
            parsed_tables = DdlParser.parse(ddl_sql)
        except Exception as e:
            return {"success": False, "error": f"DDL 解析失败: {str(e)}"}

        if not parsed_tables:
            return {"success": False, "error": "未从 SQL 中识别出有效的 CREATE TABLE 语句"}

        imported_tables = []
        for pt in parsed_tables:
            # Check if table already exists in this project
            existing_t = db.query(TableMeta).filter(
                TableMeta.project_id == project_id,
                TableMeta.table_name == pt.table_name
            ).first()

            if existing_t:
                # Update existing table comment
                if pt.table_comment:
                    existing_t.table_comment = pt.table_comment
                t_obj = existing_t
            else:
                t_obj = TableMeta(
                    project_id=project_id,
                    table_name=pt.table_name,
                    table_comment=pt.table_comment or "",
                    pos_x=100 + len(imported_tables) * 40,
                    pos_y=100 + len(imported_tables) * 40
                )
                db.add(t_obj)
                db.flush()

            # Insert columns
            existing_col_names = {c.column_name.upper(): c for c in t_obj.columns}
            for pc in pt.columns:
                if pc.column_name.upper() not in existing_col_names:
                    col_obj = ColumnMeta(
                        table_id=t_obj.id,
                        column_name=pc.column_name,
                        data_type=pc.data_type,
                        column_comment=pc.column_comment or "",
                        sort_order=len(t_obj.columns)
                    )
                    db.add(col_obj)

            imported_tables.append({
                "table_name": pt.table_name,
                "comment": pt.table_comment,
                "column_count": len(pt.columns)
            })

        db.commit()
        return {
            "success": True,
            "imported_tables_count": len(imported_tables),
            "tables": imported_tables,
            "message": f"成功导入 {len(imported_tables)} 张数据表模型"
        }


@mcp_server.tool(
    name="collink_generate_etl_sql",
    description="根据当前项目已配置的字段映射关系，为指定目标表自动生成标准的 INSERT INTO ... SELECT ... ETL 转换 SQL 脚本。"
)
def collink_generate_etl_sql(project_id: str, target_table: str, dialect: str = "mysql") -> Dict[str, Any]:
    with SessionLocal() as db:
        tt = db.query(TableMeta).filter(TableMeta.project_id == project_id, TableMeta.table_name.ilike(target_table)).first()
        if not tt:
            return {"error": f"目标表 '{target_table}' 不存在"}

        mappings = db.query(FieldMapping).filter(
            FieldMapping.project_id == project_id,
            FieldMapping.target_table_id == tt.id
        ).all()

        if not mappings:
            return {
                "target_table": tt.table_name,
                "error": "该目标表尚未配置任何上游字段映射关系，无法生成 ETL 语句",
                "sql": ""
            }

        # Group mappings by source table
        source_tables: Dict[str, TableMeta] = {}
        target_cols: Dict[str, ColumnMeta] = {c.id: c for c in tt.columns}
        col_mappings: List[Dict[str, Any]] = []

        for m in mappings:
            if m.source_table_id not in source_tables:
                st = db.query(TableMeta).filter(TableMeta.id == m.source_table_id).first()
                if st:
                    source_tables[m.source_table_id] = st
            
            st = source_tables.get(m.source_table_id)
            sc = db.query(ColumnMeta).filter(ColumnMeta.id == m.source_column_id).first()
            tc = target_cols.get(m.target_column_id)

            if st and sc and tc:
                col_mappings.append({
                    "src_table": st.table_name,
                    "src_col": sc.column_name,
                    "src_type": sc.data_type,
                    "tgt_col": tc.column_name,
                    "tgt_type": tc.data_type,
                    "comment": tc.column_comment or sc.column_comment or ""
                })

        if not col_mappings:
            return {"error": "映射有效字段为空"}

        primary_src_table = col_mappings[0]["src_table"]

        # Generate SQL
        target_col_names = [f"  `{m['tgt_col']}`" if dialect == "mysql" else f'  "{m["tgt_col"]}"' for m in col_mappings]
        select_expressions = []
        for m in col_mappings:
            col_str = f"s.`{m['src_col']}`" if dialect == "mysql" else f's."{m["src_col"]}"'
            as_str = f"`{m['tgt_col']}`" if dialect == "mysql" else f'"{m["tgt_col"]}"'
            comment_str = f" -- {m['comment']}" if m['comment'] else ""
            select_expressions.append(f"  {col_str} AS {as_str},{comment_str}")

        # Remove trailing comma on last expression
        if select_expressions:
            last_idx = select_expressions[-1].find(",")
            if last_idx != -1:
                select_expressions[-1] = select_expressions[-1][:last_idx] + select_expressions[-1][last_idx+1:]

        tgt_table_id = f"`{tt.table_name}`" if dialect == "mysql" else f'"{tt.table_name}"'
        src_table_id = f"`{primary_src_table}`" if dialect == "mysql" else f'"{primary_src_table}"'

        sql = f"""-- ======================================================================
-- ETL 转换脚本: {primary_src_table} -> {tt.table_name}
-- 自动生成于 ColLink 数据血缘平台 (方言: {dialect})
-- ======================================================================

INSERT INTO {tgt_table_id} (
{',\n'.join(target_col_names)}
)
SELECT
{'\n'.join(select_expressions)}
FROM {src_table_id} s;
"""

        return {
            "target_table": tt.table_name,
            "source_tables": [st.table_name for st in source_tables.values()],
            "mapped_columns_count": len(col_mappings),
            "sql": sql
        }


# ==============================================================================
# 6. Resources (只读元数据上下文资源)
# ==============================================================================

@mcp_server.resource(
    uri="collink://projects",
    name="ColLink Projects List",
    description="ColLink 中所有数据血缘项目概要列表"
)
def get_projects_resource() -> str:
    with SessionLocal() as db:
        projects = db.query(Project).all()
        lines = ["# ColLink 数据血缘项目清单\n"]
        for p in projects:
            t_count = db.query(TableMeta).filter(TableMeta.project_id == p.id).count()
            m_count = db.query(FieldMapping).filter(FieldMapping.project_id == p.id).count()
            lines.append(f"- **项目名称**: {p.name} (ID: `{p.id}`)")
            lines.append(f"  - 描述: {p.description or '无'}")
            lines.append(f"  - 数据表数: {t_count} | 字段血缘映射数: {m_count}\n")
        return "\n".join(lines)


@mcp_server.resource(
    uri="collink://project/{project_id}/schema",
    name="Project Schema Context",
    description="指定项目的全量数据字典结构（表、字段、数据类型及注释），直接作为 LLM 上下文。"
)
def get_project_schema_resource(project_id: str) -> str:
    with SessionLocal() as db:
        p = db.query(Project).filter(Project.id == project_id).first()
        if not p:
            return f"Project '{project_id}' not found."

        tables = db.query(TableMeta).filter(TableMeta.project_id == project_id).all()
        lines = [f"# 项目数据字典: {p.name} (`{p.id}`)\n"]

        for t in tables:
            comment_str = f" ({t.table_comment})" if t.table_comment else ""
            lines.append(f"## 表: `{t.table_name}`{comment_str}")
            lines.append("| 字段名 | 数据类型 | 主键 | 中文注释 |")
            lines.append("| :--- | :--- | :---: | :--- |")
            for c in t.columns:
                is_pk = c.column_name.lower() == 'id' or c.column_name.lower().endswith('_id')
                pk_str = "PK" if is_pk else ""
                lines.append(f"| `{c.column_name}` | {c.data_type} | {pk_str} | {c.column_comment or ''} |")
            lines.append("")

        return "\n".join(lines)


# ==============================================================================
# 7. Prompts (预设提示词模板)
# ==============================================================================

@mcp_server.prompt(
    name="lineage_quality_audit",
    description="引导大模型对指定数据血缘项目的映射覆盖率和规则完整性进行全面质量审计。"
)
def lineage_quality_audit_prompt(project_id: str) -> str:
    return f"""你是一名资深数据仓库架构师与数据治理专家。
请调用 ColLink MCP 工具，对项目 ID 为 `{project_id}` 的数据血缘进行质量审计：
1. 首先调用 `collink_get_tables` 查看当前项目下的所有表和字段；
2. 针对目标层数据表，调用 `collink_audit_unmapped_fields` 检查哪些关键字段尚未映射来源；
3. 调用 `collink_list_mappings` 审查已配置的映射关系是否合理，是否存在数据类型不兼容的风险；
4. 输出一份清晰的《数据血缘完整度体检报告》，并列出优先需要补齐的字段映射建议。"""


# ==============================================================================
# Server Runner
# ==============================================================================

def main():
    """Run MCP server in stdio mode by default (ideal for Claude Desktop / Cursor / Antigravity)."""
    mcp_server.run(transport="stdio")


if __name__ == "__main__":
    main()
