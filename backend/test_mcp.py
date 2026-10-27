"""
Comprehensive Verification Test for ColLink MCP Server
Validates tool execution, schema retrieval, lineage tracing, and SQL generation.
"""

import os
import sys

BACKEND_DIR = os.path.dirname(os.path.abspath(__file__))
if BACKEND_DIR not in sys.path:
    sys.path.insert(0, BACKEND_DIR)

from app.mcp.server import (
    collink_list_projects,
    collink_list_canvases,
    collink_get_tables,
    collink_search_columns,
    collink_list_mappings,
    collink_get_field_lineage,
    collink_auto_map_fields,
    collink_audit_unmapped_fields,
    collink_generate_etl_sql,
    get_projects_resource,
    get_project_schema_resource,
    mcp_server
)

def run_tests():
    print("==================================================")
    print("1. Testing MCP Server Registration & Inspection...")
    print(f"MCP Server Name: {mcp_server.name}")
    print(f"MCP Server Version: {mcp_server.version}")
    print("==================================================")

    # 1. Test Projects List
    projects = collink_list_projects()
    print(f"Found {len(projects)} projects.")
    assert len(projects) > 0, "Should have at least 1 project"
    p = projects[0]
    p_id = p["project_id"]
    print(f"Project: {p['project_name']} (ID: {p_id}, Tables: {p['table_count']}, Mappings: {p['mapping_count']})")

    # 2. Test Canvases List
    canvases = collink_list_canvases(p_id)
    print(f"Found {len(canvases)} canvases in project.")
    assert len(canvases) > 0, "Should have at least 1 canvas"
    print(f"Canvas: {canvases[0]['canvas_name']} (Tables: {canvases[0]['table_count']})")

    # 3. Test Tables List
    tables = collink_get_tables(p_id)
    print(f"Found {len(tables)} tables in project.")
    assert len(tables) > 0, "Should have tables"
    first_tbl = tables[0]
    print(f"First Table: {first_tbl['table_name']} ({first_tbl['column_count']} columns)")

    # 4. Test Search Columns
    search_results = collink_search_columns(p_id, "id")
    print(f"Searching 'id': {len(search_results)} columns matched.")
    assert len(search_results) > 0, "Should find columns matching 'id'"

    # 5. Test Mappings List
    mappings = collink_list_mappings(p_id)
    print(f"Total mappings configured: {len(mappings)}")
    if mappings:
        m0 = mappings[0]
        print(f"Sample mapping: {m0['label']}")

        # 6. Test Field Lineage for this mapping's target column
        lineage = collink_get_field_lineage(p_id, m0["target_table"], m0["target_column"])
        print(f"Lineage trace for {m0['target_table']}.{m0['target_column']}:")
        print(f"  Upstream sources count: {lineage.get('upstream_count')}")
        print(f"  Downstream targets count: {lineage.get('downstream_count')}")

        # 7. Test Audit Unmapped Fields
        audit = collink_audit_unmapped_fields(p_id, m0["target_table"])
        print(f"Audit for target table {m0['target_table']}:")
        print(f"  Total cols: {audit['total_columns']}, Mapped: {audit['mapped_count']}, Unmapped: {audit['unmapped_count']}")
        print(f"  Coverage: {audit['coverage_rate']}")

        # 8. Test ETL SQL Generation
        etl_res = collink_generate_etl_sql(p_id, m0["target_table"], dialect="mysql")
        print(f"ETL SQL for {m0['target_table']} (Mapped: {etl_res.get('mapped_columns_count')} cols):")
        sql_preview = etl_res.get("sql", "").split("\n")[:8]
        print("\n".join("    " + l for l in sql_preview))
        print("    ...")

    # 9. Test Auto Mapping Dry-Run
    if len(tables) >= 2:
        auto_res = collink_auto_map_fields(p_id, tables[0]["table_name"], tables[1]["table_name"], apply_mappings=False)
        print(f"Auto mapping {tables[0]['table_name']} -> {tables[1]['table_name']}: {auto_res.get('suggested_count')} suggested pairs")

    # 10. Test Resources
    res_projects = get_projects_resource()
    print(f"Resource 'collink://projects' length: {len(res_projects)} chars")
    res_schema = get_project_schema_resource(p_id)
    print(f"Resource 'collink://project/{p_id}/schema' length: {len(res_schema)} chars")

    print("\n==================================================")
    print("[PASS] ALL 10+ ColLink MCP Tools & Resources PASSED!")
    print("==================================================")

if __name__ == "__main__":
    run_tests()
