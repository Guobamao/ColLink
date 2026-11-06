use collink_lib::db::{init_db, projects, canvases, tables, mappings};
use collink_lib::models::ColumnMeta;
use collink_lib::services::ddl_parser::DdlParser;
use collink_lib::services::exporter::ExporterService;
use collink_lib::services::path_discovery::PathDiscoveryService;
use rusqlite::Connection;

#[test]
fn test_database_crud_flow() {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    // 1. Projects
    let proj = projects::create_project(&conn, "测试项目", Some("用于自动化测试的项目")).unwrap();
    assert_eq!(proj.name, "测试项目");

    let p_list = projects::list_projects(&conn).unwrap();
    assert_eq!(p_list.len(), 1);

    // 2. Canvases auto-initialized
    let canvases = canvases::list_canvases(&conn, &proj.id).unwrap();
    assert_eq!(canvases.len(), 1);
    assert_eq!(canvases[0].name, "默认总览");

    // 3. Create Tables
    let t1 = tables::create_table(
        &conn,
        &proj.id,
        "ods_user",
        Some("用户源表"),
        Some(100.0),
        Some(100.0),
        &[
            ColumnMeta {
                id: String::new(),
                table_id: String::new(),
                column_name: "user_id".to_string(),
                data_type: "VARCHAR(64)".to_string(),
                column_comment: Some("用户ID".to_string()),
                sort_order: 1,
            },
            ColumnMeta {
                id: String::new(),
                table_id: String::new(),
                column_name: "user_name".to_string(),
                data_type: "VARCHAR(100)".to_string(),
                column_comment: Some("用户姓名".to_string()),
                sort_order: 2,
            },
        ],
    ).unwrap();
    assert_eq!(t1.columns.len(), 2);

    let t2 = tables::create_table(
        &conn,
        &proj.id,
        "dwd_user",
        Some("用户明细表"),
        Some(500.0),
        Some(100.0),
        &[
            ColumnMeta {
                id: String::new(),
                table_id: String::new(),
                column_name: "usr_id".to_string(),
                data_type: "VARCHAR(64)".to_string(),
                column_comment: Some("用户编号".to_string()),
                sort_order: 1,
            },
        ],
    ).unwrap();

    let t3 = tables::create_table(
        &conn,
        &proj.id,
        "ads_user_stat",
        Some("用户应用汇总表"),
        Some(900.0),
        Some(100.0),
        &[
            ColumnMeta {
                id: String::new(),
                table_id: String::new(),
                column_name: "cust_id".to_string(),
                data_type: "VARCHAR(64)".to_string(),
                column_comment: Some("客户ID".to_string()),
                sort_order: 1,
            },
        ],
    ).unwrap();

    // 4. Mappings (t1.user_id -> t2.usr_id, t2.usr_id -> t3.cust_id)
    let m1 = mappings::create_mapping(
        &conn,
        &proj.id,
        &t1.id,
        &t1.columns[0].id,
        &t2.id,
        &t2.columns[0].id,
    ).unwrap();
    assert_eq!(m1.source_table_name, Some("ods_user".to_string()));

    let _m2 = mappings::create_mapping(
        &conn,
        &proj.id,
        &t2.id,
        &t2.columns[0].id,
        &t3.id,
        &t3.columns[0].id,
    ).unwrap();

    let all_mappings = mappings::list_mappings(&conn, &proj.id).unwrap();
    assert_eq!(all_mappings.len(), 2);

    // 5. Multi-Hop Path Discovery (ods_user -> ads_user_stat)
    let path_result = PathDiscoveryService::discover_paths(
        &conn,
        &proj.id,
        &t1.id,
        &t3.id,
        4,
    ).unwrap();
    assert_eq!(path_result.paths_count, 1);
    assert_eq!(path_result.paths[0].hops, 2);
    assert_eq!(path_result.paths[0].summary_title, "ods_user -> dwd_user -> ads_user_stat");
    assert!(path_result.paths[0].sql_snippet.contains("SELECT"));
    assert!(path_result.paths[0].sql_snippet.contains("JOIN `dwd_user`"));
    assert!(path_result.paths[0].sql_snippet.contains("JOIN `ads_user_stat`"));

    // 6. Excel Export
    let excel_bytes = ExporterService::export_excel(&conn, &proj.id).unwrap();
    assert!(!excel_bytes.is_empty());

    // 7. JSON Export & Restore
    let json_dump = ExporterService::export_project_json(&conn, &proj.id).unwrap();
    assert!(json_dump.contains("测试项目"));

    let restored = ExporterService::import_project_json(&conn, &json_dump).unwrap();
    assert!(restored.name.contains("备份恢复"));
    let restored_tables = tables::list_tables_for_project(&conn, &restored.id).unwrap();
    assert_eq!(restored_tables.len(), 3);
}

#[test]
fn test_ddl_parser_multi_dialect() {
    let sql = r#"
    COMMENT ON TABLE "SYS_USER" IS '系统用户表';
    COMMENT ON COLUMN "SYS_USER"."USER_ID" IS '用户唯一主键';
    COMMENT ON COLUMN "SYS_USER"."USER_NAME" IS '用户登录名';

    CREATE TABLE "SYS_USER" (
        "USER_ID" VARCHAR(36) NOT NULL,
        "USER_NAME" VARCHAR(100) DEFAULT '',
        "AGE" INT COMMENT '用户年龄',
        PRIMARY KEY ("USER_ID")
    ) COMMENT='系统用户表';
    "#;

    let tables = DdlParser::parse(sql, Some("dm"));
    assert_eq!(tables.len(), 1);
    let t = &tables[0];
    assert_eq!(t.table_name, "SYS_USER");
    assert_eq!(t.table_comment, Some("系统用户表".to_string()));
    assert_eq!(t.columns.len(), 3);

    assert_eq!(t.columns[0].column_name, "USER_ID");
    assert_eq!(t.columns[0].column_comment, Some("用户唯一主键".to_string()));

    assert_eq!(t.columns[1].column_name, "USER_NAME");
    assert_eq!(t.columns[1].column_comment, Some("用户登录名".to_string()));

    assert_eq!(t.columns[2].column_name, "AGE");
    assert_eq!(t.columns[2].column_comment, Some("用户年龄".to_string()));
}

#[test]
fn test_user_sql_crash() {
    let sql = r#"
CREATE TABLE "DAILY_REPORT_PUSH_RECORD"
(
"ID" VARCHAR(64) NOT NULL,
"SUBSCRIBE_ID" VARCHAR(64),
"REPORT_ID" VARCHAR(64),
"REPORT_NAME" VARCHAR(266),
"USER_UID" VARCHAR(64),
"USER_UNAME" VARCHAR(255),
"PUSH_TIME" TIMESTAMP(6),
"PUSH_STATUS" INTEGER,
"ERROR_MSG" VARCHAR(2000),
"DIR_ID" VARCHAR(64),
"PUSH_TYPE" INTEGER,
"CREATE_TIME" TIMESTAMP(6),
"REPORT_TYPE" INTEGER DEFAULT 1,
NOT CLUSTER PRIMARY KEY("ID")) STORAGE(ON "MAIN", CLUSTERBTR) ;

COMMENT ON TABLE "DAILY_REPORT_PUSH_RECORD" IS '报告推送记录表';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."CREATE_TIME" IS '创建时间';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."DIR_ID" IS '报告文件夹ID';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."ERROR_MSG" IS '错误信息';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."ID" IS '主键';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."PUSH_STATUS" IS '推送状态(1-成功 0-失败)';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."PUSH_TIME" IS '推送时间';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."PUSH_TYPE" IS '推送类型(1-首页推送 2-eLink推送)';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."REPORT_ID" IS '报告编号';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."REPORT_NAME" IS '报告名称';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."REPORT_TYPE" IS '报告类型(1-静态报告 2-动态报告)';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."SUBSCRIBE_ID" IS '订阅ID';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."USER_UID" IS '用户ID';

COMMENT ON COLUMN "DAILY_REPORT_PUSH_RECORD"."USER_UNAME" IS '用户姓名';
    "#;

    let tables = DdlParser::parse(sql, Some("dm"));
    assert_eq!(tables.len(), 1);
    let t = &tables[0];
    assert_eq!(t.table_name, "DAILY_REPORT_PUSH_RECORD");
    assert_eq!(t.table_comment, Some("报告推送记录表".to_string()));
    assert_eq!(t.columns.len(), 13);
    assert_eq!(t.columns[0].column_name, "ID");
    assert_eq!(t.columns[0].column_comment, Some("主键".to_string()));
    assert_eq!(t.columns[12].column_name, "REPORT_TYPE");
    assert_eq!(t.columns[12].column_comment, Some("报告类型(1-静态报告 2-动态报告)".to_string()));
}

#[test]
fn test_db_inspector_factory() {
    use collink_lib::services::db_inspector::{create_inspector, DbConfig};

    let cfg_mysql = DbConfig {
        db_type: "mysql".to_string(),
        host: "127.0.0.1".to_string(),
        port: 3306,
        database: "test".to_string(),
        username: "root".to_string(),
        password: "pwd".to_string(),
    };
    assert!(create_inspector(&cfg_mysql).is_ok());

    let cfg_pg = DbConfig {
        db_type: "postgres".to_string(),
        host: "127.0.0.1".to_string(),
        port: 5432,
        database: "postgres".to_string(),
        username: "postgres".to_string(),
        password: "pwd".to_string(),
    };
    assert!(create_inspector(&cfg_pg).is_ok());

    let cfg_dm = DbConfig {
        db_type: "dm".to_string(),
        host: "127.0.0.1".to_string(),
        port: 5236,
        database: "SYSDBA".to_string(),
        username: "SYSDBA".to_string(),
        password: "SYSDBA".to_string(),
    };
    assert!(create_inspector(&cfg_dm).is_ok());

    let cfg_invalid = DbConfig {
        db_type: "mongodb".to_string(),
        host: "127.0.0.1".to_string(),
        port: 27017,
        database: "test".to_string(),
        username: "root".to_string(),
        password: "".to_string(),
    };
    assert!(create_inspector(&cfg_invalid).is_err());
}

#[test]
fn test_direct_db_import_and_incremental_mapping_preservation() {
    use collink_lib::models::{ParsedColumn, ParsedTable};

    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    let proj = projects::create_project(&conn, "直连测试项目", None).unwrap();
    let canvases_list = canvases::list_canvases(&conn, &proj.id).unwrap();
    let canvas_id = &canvases_list[0].id;

    // Simulate inspected tables from database
    let table_a = ParsedTable {
        table_name: "T_ORDER".to_string(),
        table_comment: Some("订单表".to_string()),
        columns: vec![
            ParsedColumn {
                column_name: "ORDER_ID".to_string(),
                data_type: "VARCHAR(64)".to_string(),
                column_comment: Some("主键".to_string()),
                sort_order: 1,
            },
            ParsedColumn {
                column_name: "USER_ID".to_string(),
                data_type: "VARCHAR(64)".to_string(),
                column_comment: Some("买家ID".to_string()),
                sort_order: 2,
            },
        ],
    };

    let table_b = ParsedTable {
        table_name: "T_USER".to_string(),
        table_comment: Some("用户表".to_string()),
        columns: vec![
            ParsedColumn {
                column_name: "ID".to_string(),
                data_type: "VARCHAR(64)".to_string(),
                column_comment: Some("主键".to_string()),
                sort_order: 1,
            },
            ParsedColumn {
                column_name: "NAME".to_string(),
                data_type: "VARCHAR(100)".to_string(),
                column_comment: Some("姓名".to_string()),
                sort_order: 2,
            },
        ],
    };

    // First import
    let import_res1 = DdlParser::import_tables(&conn, &proj.id, &[table_a.clone(), table_b.clone()], Some(canvas_id)).unwrap();
    assert_eq!(import_res1.summary.tables_created, 2);

    let tables_after1 = tables::list_tables_for_project(&conn, &proj.id).unwrap();
    assert_eq!(tables_after1.len(), 2);

    let order_table = tables_after1.iter().find(|t| t.table_name == "T_ORDER").unwrap();
    let user_table = tables_after1.iter().find(|t| t.table_name == "T_USER").unwrap();

    let order_user_id_col = order_table.columns.iter().find(|c| c.column_name == "USER_ID").unwrap();
    let user_id_col = user_table.columns.iter().find(|c| c.column_name == "ID").unwrap();

    // Create a mapping between T_ORDER.USER_ID and T_USER.ID
    let mapping = mappings::create_mapping(
        &conn,
        &proj.id,
        &order_table.id,
        &order_user_id_col.id,
        &user_table.id,
        &user_id_col.id,
    ).unwrap();
    assert!(!mapping.id.is_empty());

    // Incremental Re-import (Simulating column addition in database: T_ORDER added AMOUNT column)
    let mut table_a_updated = table_a.clone();
    table_a_updated.columns.push(ParsedColumn {
        column_name: "AMOUNT".to_string(),
        data_type: "DECIMAL(12, 2)".to_string(),
        column_comment: Some("订单金额".to_string()),
        sort_order: 3,
    });

    let import_res2 = DdlParser::import_tables(&conn, &proj.id, &[table_a_updated, table_b], Some(canvas_id)).unwrap();
    assert_eq!(import_res2.summary.tables_updated, 2);
    assert_eq!(import_res2.summary.columns_added, 1);

    // Verify existing mapping is still intact!
    let current_mappings = mappings::list_mappings(&conn, &proj.id).unwrap();
    assert_eq!(current_mappings.len(), 1);
    assert_eq!(current_mappings[0].id, mapping.id);
    assert_eq!(current_mappings[0].source_column_id, order_user_id_col.id);
    assert_eq!(current_mappings[0].target_column_id, user_id_col.id);
}
