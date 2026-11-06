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
            .map_err(|e| Self::format_error(e))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                log::error!("PostgreSQL 连接底层异常: {}", e);
            }
        });

        Ok(client)
    }

    fn format_error(e: impl std::fmt::Display) -> String {
        let err_str = e.to_string();
        if err_str.contains("password authentication failed") || err_str.contains("28P01") {
            format!("PostgreSQL 认证失败: 用户名或密码错误 ({})", err_str)
        } else if err_str.contains("database") && err_str.contains("does not exist")
            || err_str.contains("3D000")
        {
            format!("PostgreSQL 数据库不存在: 请核对数据库名称 ({})", err_str)
        } else if err_str.contains("Connection refused") || err_str.contains("10061") {
            format!(
                "PostgreSQL 连接被拒绝: 无法连接目标服务，请检查服务状态与防火墙 ({})",
                err_str
            )
        } else if err_str.contains("timeout") || err_str.contains("timed out") {
            format!("PostgreSQL 连接超时: 目标主机无响应 ({})", err_str)
        } else {
            format!("PostgreSQL 操作异常: {}", err_str)
        }
    }
}

#[async_trait]
impl DbInspector for PostgresInspector {
    async fn test_connection(&self) -> Result<(), String> {
        let client = self.connect().await?;
        client
            .execute("SELECT 1", &[])
            .await
            .map_err(|e| Self::format_error(e))?;
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
            WHERE c.relkind IN ('r', 'p', 'v', 'm')
              AND n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
            GROUP BY c.relname, c.oid, n.nspname
            ORDER BY c.relname;
        ";

        let rows = client
            .query(sql, &[])
            .await
            .map_err(|e| Self::format_error(e))?;

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

        let mut table_comment_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut table_cols_map: std::collections::HashMap<String, Vec<ParsedColumn>> =
            std::collections::HashMap::new();

        let sql = "
            SELECT 
                c.relname::text AS table_name,
                COALESCE(obj_description(c.oid, 'pg_class'), '')::text AS table_comment,
                a.attname::text AS column_name,
                format_type(a.atttypid, a.atttypmod)::text AS data_type,
                COALESCE(col_description(c.oid, a.attnum), '')::text AS column_comment,
                a.attnum::int AS sort_order,
                EXISTS (
                    SELECT 1 
                    FROM pg_index i 
                    WHERE i.indrelid = c.oid 
                      AND a.attnum = ANY(i.indkey) 
                      AND i.indisprimary
                ) AS is_pk
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_attribute a ON a.attrelid = c.oid
            WHERE c.relkind IN ('r', 'p', 'v', 'm')
              AND n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
              AND a.attnum > 0 
              AND NOT a.attisdropped
              AND c.relname = ANY($1)
            ORDER BY c.relname, a.attnum;
        ";

        // 分批执行（每批 50 个表）
        for chunk in table_names.chunks(50) {
            let chunk_vec: Vec<String> = chunk.to_vec();
            let rows = client
                .query(sql, &[&chunk_vec])
                .await
                .map_err(|e| Self::format_error(e))?;

            for row in rows {
                let tbl_name: String = row.get("table_name");
                let tbl_comment: String = row.get("table_comment");
                let col_name: String = row.get("column_name");
                let data_type: String = row.get("data_type");
                let col_comment: String = row.get("column_comment");
                let ordinal: i32 = row.get("sort_order");
                let is_pk: bool = row.get("is_pk");

                if !tbl_comment.is_empty() {
                    table_comment_map.insert(tbl_name.clone(), tbl_comment);
                }

                let final_comment = if is_pk {
                    if col_comment.is_empty() {
                        Some("主键".to_string())
                    } else if !col_comment.contains("主键") {
                        Some(format!("{} (主键)", col_comment))
                    } else {
                        Some(col_comment)
                    }
                } else if col_comment.is_empty() {
                    None
                } else {
                    Some(col_comment)
                };

                let col = ParsedColumn {
                    column_name: col_name,
                    data_type: data_type.to_uppercase(),
                    column_comment: final_comment,
                    sort_order: ordinal,
                };

                table_cols_map.entry(tbl_name).or_default().push(col);
            }
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
