use super::{DbConfig, DbInspector, DbTableSummary};
use crate::models::{ParsedColumn, ParsedTable};
use async_trait::async_trait;
use tokio_postgres::{Config, NoTls};

pub struct PostgresInspector {
    config: DbConfig,
}

impl PostgresInspector {
    pub fn new(config: DbConfig) -> Self {
        Self { config }
    }

    async fn connect(&self) -> Result<tokio_postgres::Client, String> {
        let mut pg_conf = Config::new();
        pg_conf.host(&self.config.host);
        pg_conf.port(self.config.port);
        pg_conf.user(&self.config.username);
        pg_conf.password(&self.config.password);
        pg_conf.dbname(&self.config.database);

        let (client, connection) = pg_conf
            .connect(NoTls)
            .await
            .map_err(|e| format!("PostgreSQL 连接失败: {}", e))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                log::error!("PostgreSQL 连接底层异常: {}", e);
            }
        });

        Ok(client)
    }
}

#[async_trait]
impl DbInspector for PostgresInspector {
    async fn test_connection(&self) -> Result<(), String> {
        let client = self.connect().await?;
        client
            .execute("SELECT 1", &[])
            .await
            .map_err(|e| format!("PostgreSQL 执行测试查询失败: {}", e))?;
        Ok(())
    }

    async fn list_tables(&self) -> Result<Vec<DbTableSummary>, String> {
        let client = self.connect().await?;

        let sql = "
            SELECT 
                c.relname::text AS table_name,
                COALESCE(obj_description(c.oid, 'pg_class'), '')::text AS table_comment,
                COUNT(a.attnum)::int AS column_count
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            LEFT JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum > 0 AND NOT a.attisdropped
            WHERE c.relkind IN ('r', 'p')
              AND n.nspname NOT IN ('pg_catalog', 'information_schema')
            GROUP BY c.relname, c.oid, n.nspname
            ORDER BY c.relname;
        ";

        let rows = client
            .query(sql, &[])
            .await
            .map_err(|e| format!("查询 PostgreSQL 表清单失败: {}", e))?;

        let summaries = rows
            .into_iter()
            .map(|row| {
                let name: String = row.get("table_name");
                let comment: String = row.get("table_comment");
                let col_cnt: i32 = row.get("column_count");
                DbTableSummary {
                    table_name: name,
                    table_comment: if comment.is_empty() {
                        None
                    } else {
                        Some(comment)
                    },
                    column_count: Some(col_cnt as usize),
                }
            })
            .collect();

        Ok(summaries)
    }

    async fn inspect_tables(&self, table_names: &[String]) -> Result<Vec<ParsedTable>, String> {
        if table_names.is_empty() {
            return Ok(Vec::new());
        }

        let client = self.connect().await?;

        let sql = "
            SELECT 
                c.relname::text AS table_name,
                COALESCE(obj_description(c.oid, 'pg_class'), '')::text AS table_comment,
                a.attname::text AS column_name,
                format_type(a.atttypid, a.atttypmod)::text AS data_type,
                COALESCE(col_description(c.oid, a.attnum), '')::text AS column_comment,
                a.attnum::int AS sort_order
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_attribute a ON a.attrelid = c.oid
            WHERE c.relkind IN ('r', 'p')
              AND n.nspname NOT IN ('pg_catalog', 'information_schema')
              AND a.attnum > 0 
              AND NOT a.attisdropped
              AND c.relname = ANY($1)
            ORDER BY c.relname, a.attnum;
        ";

        let rows = client
            .query(sql, &[&table_names])
            .await
            .map_err(|e| format!("查询 PostgreSQL 字段元数据失败: {}", e))?;

        let mut table_comment_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut table_cols_map: std::collections::HashMap<String, Vec<ParsedColumn>> =
            std::collections::HashMap::new();

        for row in rows {
            let tbl_name: String = row.get("table_name");
            let tbl_comment: String = row.get("table_comment");
            let col_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let col_comment: String = row.get("column_comment");
            let ordinal: i32 = row.get("sort_order");

            if !tbl_comment.is_empty() {
                table_comment_map.insert(tbl_name.clone(), tbl_comment);
            }

            let col = ParsedColumn {
                column_name: col_name,
                data_type: data_type.to_uppercase(),
                column_comment: if col_comment.is_empty() {
                    None
                } else {
                    Some(col_comment)
                },
                sort_order: ordinal,
            };

            table_cols_map.entry(tbl_name).or_default().push(col);
        }

        let mut result = Vec::new();
        for t in table_names {
            let cols = table_cols_map.remove(t).unwrap_or_default();
            let comment = table_comment_map.remove(t);
            result.push(ParsedTable {
                table_name: t.clone(),
                table_comment: comment,
                columns: cols,
            });
        }

        Ok(result)
    }
}
