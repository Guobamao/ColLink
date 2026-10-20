from typing import List, Dict, Any, Optional
import pymysql
import psycopg2
from app.schemas.connection import DbConnectionRequest, DbTableSummary
from app.schemas.ddl import ParsedTable, ParsedColumn


class DbConnectorService:
    @staticmethod
    def test_connection(req: DbConnectionRequest) -> Dict[str, Any]:
        """Test whether connection to the database succeeds."""
        db_type = req.db_type.lower()
        try:
            if db_type == "mysql":
                conn = pymysql.connect(
                    host=req.host,
                    port=req.port,
                    user=req.username,
                    password=req.password,
                    database=req.database,
                    connect_timeout=5,
                    charset="utf8mb4"
                )
                conn.close()
                return {"success": True, "message": "MySQL 数据库连接成功！"}

            elif db_type in ("postgres", "postgresql"):
                conn = psycopg2.connect(
                    host=req.host,
                    port=req.port,
                    dbname=req.database,
                    user=req.username,
                    password=req.password,
                    connect_timeout=5
                )
                conn.close()
                return {"success": True, "message": "PostgreSQL 数据库连接成功！"}

            elif db_type in ("dm", "dameng"):
                # 1. Try dmPython
                try:
                    import dmPython  # type: ignore
                    try:
                        conn = dmPython.connect(
                            user=req.username,
                            password=req.password,
                            server=req.host,
                            port=req.port
                        )
                        conn.close()
                        return {"success": True, "message": "达梦数据库 (dmPython) 连接成功！"}
                    except Exception as conn_err:
                        return {
                            "success": False,
                            "message": f"达梦驱动已加载，但连接服务端失败: {str(conn_err)}"
                        }
                except ImportError:
                    pass

                # 2. Try pyodbc if dm driver installed
                try:
                    import pyodbc  # type: ignore
                    try:
                        conn_str = f"DRIVER={{DM8 ODBC DRIVER}};SERVER={req.host};PORT={req.port};UID={req.username};PWD={req.password};"
                        conn = pyodbc.connect(conn_str, timeout=5)
                        conn.close()
                        return {"success": True, "message": "达梦数据库 (ODBC) 连接成功！"}
                    except Exception as odbc_err:
                        return {
                            "success": False,
                            "message": f"ODBC 驱动已加载，但连接服务端失败: {str(odbc_err)}"
                        }
                except Exception:
                    pass

                return {
                    "success": False,
                    "message": "未能加载达梦数据库原生驱动 (dmPython / DM8 ODBC DRIVER)。您可以直接使用「DDL 导入」快速解析并加载表结构，或在服务器上安装 dmPython。"
                }

            else:
                return {"success": False, "message": f"不支持的数据库类型: {req.db_type}"}

        except Exception as e:
            return {"success": False, "message": f"连接失败: {str(e)}"}

    @staticmethod
    def list_tables(req: DbConnectionRequest) -> List[DbTableSummary]:
        """List tables and comments from database dictionary."""
        db_type = req.db_type.lower()
        results: List[DbTableSummary] = []

        if db_type == "mysql":
            conn = pymysql.connect(
                host=req.host,
                port=req.port,
                user=req.username,
                password=req.password,
                database=req.database,
                connect_timeout=10,
                charset="utf8mb4"
            )
            with conn.cursor() as cursor:
                cursor.execute("""
                    SELECT TABLE_NAME, TABLE_COMMENT 
                    FROM information_schema.TABLES 
                    WHERE TABLE_SCHEMA = %s AND TABLE_TYPE = 'BASE TABLE'
                    ORDER BY TABLE_NAME
                """, (req.database,))
                for row in cursor.fetchall():
                    results.append(DbTableSummary(
                        table_name=str(row[0]),
                        table_comment=str(row[1] or "")
                    ))
            conn.close()

        elif db_type in ("postgres", "postgresql"):
            schema_name = req.schema_name or "public"
            conn = psycopg2.connect(
                host=req.host,
                port=req.port,
                dbname=req.database,
                user=req.username,
                password=req.password,
                connect_timeout=10
            )
            with conn.cursor() as cursor:
                cursor.execute("""
                    SELECT c.relname as table_name, 
                           COALESCE(obj_description(c.oid, 'pg_class'), '') as table_comment
                    FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relkind = 'r' AND n.nspname = %s
                    ORDER BY c.relname
                """, (schema_name,))
                for row in cursor.fetchall():
                    results.append(DbTableSummary(
                        table_name=str(row[0]),
                        table_comment=str(row[1] or "")
                    ))
            conn.close()

        elif db_type in ("dm", "dameng"):
            schema_name = (req.schema_name or req.database or req.username).upper()
            try:
                import dmPython  # type: ignore
                conn = dmPython.connect(
                    user=req.username,
                    password=req.password,
                    server=req.host,
                    port=req.port
                )
                cursor = conn.cursor()
                cursor.execute("""
                    SELECT TABLE_NAME, COMMENTS 
                    FROM ALL_TAB_COMMENTS 
                    WHERE (OWNER = ? OR OWNER = ?) AND TABLE_TYPE = 'TABLE'
                    ORDER BY TABLE_NAME
                """, (schema_name, req.username.upper()))
                rows = cursor.fetchall()

                # If no tables found by owner, fallback to USER_TAB_COMMENTS
                if not rows:
                    cursor.execute("""
                        SELECT TABLE_NAME, COMMENTS 
                        FROM USER_TAB_COMMENTS 
                        WHERE TABLE_TYPE = 'TABLE'
                        ORDER BY TABLE_NAME
                    """)
                    rows = cursor.fetchall()

                for row in rows:
                    results.append(DbTableSummary(
                        table_name=str(row[0]),
                        table_comment=str(row[1] or "")
                    ))
                conn.close()
            except ImportError:
                raise RuntimeError("未检测到达梦 python 驱动 (dmPython)。请使用 DDL 解析导入或配置驱动。")

        return results

    @staticmethod
    def fetch_tables_metadata(
        req: DbConnectionRequest,
        selected_tables: Optional[List[str]] = None
    ) -> List[ParsedTable]:
        """Fetch complete table and column metadata for selected or all tables."""
        db_type = req.db_type.lower()
        tables: List[ParsedTable] = []

        if db_type == "mysql":
            conn = pymysql.connect(
                host=req.host,
                port=req.port,
                user=req.username,
                password=req.password,
                database=req.database,
                charset="utf8mb4"
            )
            with conn.cursor() as cursor:
                tbl_query = """
                    SELECT TABLE_NAME, TABLE_COMMENT 
                    FROM information_schema.TABLES 
                    WHERE TABLE_SCHEMA = %s AND TABLE_TYPE = 'BASE TABLE'
                """
                params = [req.database]
                if selected_tables:
                    placeholders = ",".join(["%s"] * len(selected_tables))
                    tbl_query += f" AND TABLE_NAME IN ({placeholders})"
                    params.extend(selected_tables)
                tbl_query += " ORDER BY TABLE_NAME"

                cursor.execute(tbl_query, params)
                table_rows = cursor.fetchall()

                for tbl_row in table_rows:
                    t_name = str(tbl_row[0])
                    t_comment = str(tbl_row[1] or "")

                    cursor.execute("""
                        SELECT COLUMN_NAME, COLUMN_TYPE, COLUMN_COMMENT, ORDINAL_POSITION
                        FROM information_schema.COLUMNS
                        WHERE TABLE_SCHEMA = %s AND TABLE_NAME = %s
                        ORDER BY ORDINAL_POSITION
                    """, (req.database, t_name))

                    columns: List[ParsedColumn] = []
                    for c_row in cursor.fetchall():
                        columns.append(ParsedColumn(
                            column_name=str(c_row[0]),
                            data_type=str(c_row[1]).upper(),
                            column_comment=str(c_row[2] or ""),
                            sort_order=int(c_row[3])
                        ))

                    tables.append(ParsedTable(
                        table_name=t_name,
                        table_comment=t_comment,
                        columns=columns
                    ))
            conn.close()

        elif db_type in ("postgres", "postgresql"):
            schema_name = req.schema_name or "public"
            conn = psycopg2.connect(
                host=req.host,
                port=req.port,
                dbname=req.database,
                user=req.username,
                password=req.password
            )
            with conn.cursor() as cursor:
                tbl_query = """
                    SELECT c.relname as table_name, 
                           COALESCE(obj_description(c.oid, 'pg_class'), '') as table_comment
                    FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relkind = 'r' AND n.nspname = %s
                """
                params = [schema_name]
                if selected_tables:
                    placeholders = ",".join(["%s"] * len(selected_tables))
                    tbl_query += f" AND c.relname IN ({placeholders})"
                    params.extend(selected_tables)
                tbl_query += " ORDER BY c.relname"

                cursor.execute(tbl_query, params)
                table_rows = cursor.fetchall()

                for tbl_row in table_rows:
                    t_name = str(tbl_row[0])
                    t_comment = str(tbl_row[1] or "")

                    cursor.execute("""
                        SELECT
                            a.attname as column_name,
                            format_type(a.atttypid, a.atttypmod) as data_type,
                            COALESCE(col_description(a.attrelid, a.attnum), '') as column_comment,
                            a.attnum as sort_order
                        FROM pg_attribute a
                        JOIN pg_class c ON c.oid = a.attrelid
                        JOIN pg_namespace n ON n.oid = c.relnamespace
                        WHERE c.relname = %s AND n.nspname = %s AND a.attnum > 0 AND NOT a.attisdropped
                        ORDER BY a.attnum
                    """, (t_name, schema_name))

                    columns: List[ParsedColumn] = []
                    for c_row in cursor.fetchall():
                        columns.append(ParsedColumn(
                            column_name=str(c_row[0]),
                            data_type=str(c_row[1]).upper(),
                            column_comment=str(c_row[2] or ""),
                            sort_order=int(c_row[3])
                        ))

                    tables.append(ParsedTable(
                        table_name=t_name,
                        table_comment=t_comment,
                        columns=columns
                    ))
            conn.close()

        elif db_type in ("dm", "dameng"):
            schema_name = (req.schema_name or req.database or req.username).upper()
            try:
                import dmPython  # type: ignore
                conn = dmPython.connect(
                    user=req.username,
                    password=req.password,
                    server=req.host,
                    port=req.port
                )
                cursor = conn.cursor()
                tbl_query = """
                    SELECT TABLE_NAME, COMMENTS 
                    FROM ALL_TAB_COMMENTS 
                    WHERE (OWNER = ? OR OWNER = ?) AND TABLE_TYPE = 'TABLE'
                """
                params = [schema_name, req.username.upper()]
                if selected_tables:
                    upper_tables = [t.upper() for t in selected_tables]
                    placeholders = ",".join(["?"] * len(upper_tables))
                    tbl_query += f" AND TABLE_NAME IN ({placeholders})"
                    params.extend(upper_tables)
                tbl_query += " ORDER BY TABLE_NAME"

                cursor.execute(tbl_query, params)
                table_rows = cursor.fetchall()

                if not table_rows and not selected_tables:
                    cursor.execute("""
                        SELECT TABLE_NAME, COMMENTS 
                        FROM USER_TAB_COMMENTS 
                        WHERE TABLE_TYPE = 'TABLE'
                        ORDER BY TABLE_NAME
                    """)
                    table_rows = cursor.fetchall()

                for tbl_row in table_rows:
                    t_name = str(tbl_row[0])
                    t_comment = str(tbl_row[1] or "")

                    cursor.execute("""
                        SELECT c.COLUMN_NAME, c.DATA_TYPE, m.COMMENTS, c.COLUMN_ID
                        FROM ALL_TAB_COLUMNS c
                        LEFT JOIN ALL_COL_COMMENTS m ON c.OWNER = m.OWNER AND c.TABLE_NAME = m.TABLE_NAME AND c.COLUMN_NAME = m.COLUMN_NAME
                        WHERE (c.OWNER = ? OR c.OWNER = ?) AND c.TABLE_NAME = ?
                        ORDER BY c.COLUMN_ID
                    """, (schema_name, req.username.upper(), t_name))

                    columns: List[ParsedColumn] = []
                    for c_row in cursor.fetchall():
                        columns.append(ParsedColumn(
                            column_name=str(c_row[0]),
                            data_type=str(c_row[1] or "VARCHAR").upper(),
                            column_comment=str(c_row[2] or ""),
                            sort_order=int(c_row[3] or 0)
                        ))

                    tables.append(ParsedTable(
                        table_name=t_name,
                        table_comment=t_comment,
                        columns=columns
                    ))
                conn.close()
            except ImportError:
                raise RuntimeError("未检测到达梦 python 驱动 (dmPython)。请使用 DDL 解析导入或配置驱动。")

        return tables
