"""
Path Discovery Service
Calculates multi-hop association paths between two tables strictly based on existing field lineage mappings.
No heuristic or unverified guesses are used.
"""

from typing import List, Dict, Any, Optional, Set, Tuple
from collections import defaultdict, deque
from sqlalchemy.orm import Session
from app.models.table import TableMeta
from app.models.column import ColumnMeta
from app.models.mapping import FieldMapping


class PathDiscoveryService:
    @classmethod
    def discover_paths(
        cls,
        db: Session,
        project_id: str,
        source_table_id: str,
        target_table_id: str,
        max_hops: int = 4
    ) -> List[Dict[str, Any]]:
        """
        Find association paths between source_table_id and target_table_id
        strictly based on existing configured field lineage mappings.
        """
        if source_table_id == target_table_id:
            return []

        # 1. Load all tables and columns in this project
        tables = db.query(TableMeta).filter(TableMeta.project_id == project_id).all()
        table_map: Dict[str, TableMeta] = {t.id: t for t in tables}

        if source_table_id not in table_map or target_table_id not in table_map:
            return []

        src_tbl = table_map[source_table_id]
        tgt_tbl = table_map[target_table_id]

        # 2. Load explicit mappings
        mappings = db.query(FieldMapping).filter(FieldMapping.project_id == project_id).all()

        col_map: Dict[str, ColumnMeta] = {}
        for t in tables:
            for c in t.columns:
                col_map[c.id] = c

        adj: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
        seen_pairs: Set[Tuple[str, str, str, str]] = set()

        # Build adjacency graph strictly from explicit field mappings
        for m in mappings:
            sc = col_map.get(m.source_column_id)
            tc = col_map.get(m.target_column_id)
            st = table_map.get(m.source_table_id)
            tt = table_map.get(m.target_table_id)

            if sc and tc and st and tt and m.source_table_id != m.target_table_id:
                k_forward = (m.source_table_id, m.target_table_id, sc.id, tc.id)
                if k_forward not in seen_pairs:
                    seen_pairs.add(k_forward)
                    adj[m.source_table_id].append({
                        "neighbor_table_id": m.target_table_id,
                        "from_column": sc,
                        "to_column": tc,
                        "from_table": st,
                        "to_table": tt,
                        "relation_type": "EXPLICIT",
                        "match_reason": "已建立血缘连线",
                        "confidence": 1.0
                    })

                k_backward = (m.target_table_id, m.source_table_id, tc.id, sc.id)
                if k_backward not in seen_pairs:
                    seen_pairs.add(k_backward)
                    adj[m.target_table_id].append({
                        "neighbor_table_id": m.source_table_id,
                        "from_column": tc,
                        "to_column": sc,
                        "from_table": tt,
                        "to_table": st,
                        "relation_type": "EXPLICIT",
                        "match_reason": "已建立血缘连线(反向溯源)",
                        "confidence": 1.0
                    })

        # 3. BFS Multi-Hop Path Search (Strictly along existing lineage edges)
        queue = deque([(source_table_id, [], {source_table_id})])
        found_paths: List[List[Dict[str, Any]]] = []
        seen_table_seqs: Set[Tuple[str, ...]] = set()

        expansions = 0
        MAX_EXPANSIONS = 2000
        TARGET_PATHS_LIMIT = 8

        while queue and len(found_paths) < TARGET_PATHS_LIMIT and expansions < MAX_EXPANSIONS:
            curr_table_id, current_steps, visited = queue.popleft()
            expansions += 1

            if len(current_steps) >= max_hops:
                continue

            for link in adj.get(curr_table_id, []):
                next_table_id = link["neighbor_table_id"]

                # Target table reached!
                if next_table_id == target_table_id:
                    new_steps = current_steps + [link]
                    table_seq = tuple(s["from_table"].id for s in new_steps) + (target_table_id,)
                    if table_seq not in seen_table_seqs:
                        seen_table_seqs.add(table_seq)
                        found_paths.append(new_steps)
                    continue

                if next_table_id not in visited and len(current_steps) + 1 < max_hops:
                    queue.append((
                        next_table_id,
                        current_steps + [link],
                        visited | {next_table_id}
                    ))

        # 4. Format, Rank, and Generate SQL for each path
        formatted_paths = []
        for path_index, steps in enumerate(found_paths):
            path_dto = cls._format_path(src_tbl, tgt_tbl, steps, path_index)
            formatted_paths.append(path_dto)

        # Rank paths: shortest hops first
        formatted_paths.sort(key=lambda p: p["hops"])

        return formatted_paths[:6]

    @classmethod
    def _format_path(
        cls,
        src_table: TableMeta,
        tgt_table: TableMeta,
        steps: List[Dict[str, Any]],
        path_index: int
    ) -> Dict[str, Any]:
        """Format raw graph steps into structured DTO and SQL JOIN snippet."""
        formatted_steps = []
        involved_table_names = [src_table.table_name]

        for idx, step in enumerate(steps):
            from_tbl: TableMeta = step["from_table"]
            to_tbl: TableMeta = step["to_table"]
            from_col: ColumnMeta = step["from_column"]
            to_col: ColumnMeta = step["to_column"]

            if to_tbl.table_name not in involved_table_names:
                involved_table_names.append(to_tbl.table_name)

            formatted_steps.append({
                "step_index": idx + 1,
                "relation_type": "EXPLICIT",
                "match_reason": "已建立血缘连线",
                "confidence": 1.0,
                "from_table": {
                    "id": from_tbl.id,
                    "table_name": from_tbl.table_name,
                    "table_comment": from_tbl.table_comment or "",
                },
                "from_column": {
                    "id": from_col.id,
                    "column_name": from_col.column_name,
                    "data_type": from_col.data_type,
                    "column_comment": from_col.column_comment or "",
                },
                "to_table": {
                    "id": to_tbl.id,
                    "table_name": to_tbl.table_name,
                    "table_comment": to_tbl.table_comment or "",
                },
                "to_column": {
                    "id": to_col.id,
                    "column_name": to_col.column_name,
                    "data_type": to_col.data_type,
                    "column_comment": to_col.column_comment or "",
                },
                "step_label": f"{from_tbl.table_name}.{from_col.column_name} = {to_tbl.table_name}.{to_col.column_name}"
            })

        hops = len(formatted_steps)

        # Generate SQL JOIN Snippet
        sql_lines = ["SELECT"]
        sql_lines.append("  t1.*,")
        sql_lines.append(f"  t{len(involved_table_names)}.*")
        sql_lines.append(f"FROM `{src_table.table_name}` t1")

        for idx, step in enumerate(formatted_steps):
            t_prev_alias = f"t{idx + 1}"
            t_curr_alias = f"t{idx + 2}"
            to_tbl_name = step["to_table"]["table_name"]
            from_col_name = step["from_column"]["column_name"]
            to_col_name = step["to_column"]["column_name"]

            sql_lines.append(
                f"JOIN `{to_tbl_name}` {t_curr_alias} ON {t_prev_alias}.`{from_col_name}` = {t_curr_alias}.`{to_col_name}`"
            )

        sql_snippet = "\n".join(sql_lines) + ";"

        # Path summary title (e.g. A -> B -> C)
        summary_title = " -> ".join(involved_table_names)

        # Collect all intermediate table IDs
        intermediate_table_ids = [
            step["to_table"]["id"]
            for step in formatted_steps[:-1]
        ]

        return {
            "path_id": f"path_{path_index + 1}",
            "summary_title": summary_title,
            "hops": hops,
            "is_fully_explicit": True,
            "avg_confidence": 1.0,
            "involved_table_ids": [src_table.id] + [s["to_table"]["id"] for s in formatted_steps],
            "intermediate_table_ids": intermediate_table_ids,
            "steps": formatted_steps,
            "sql_snippet": sql_snippet
        }
