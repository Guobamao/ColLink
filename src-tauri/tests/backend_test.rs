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
