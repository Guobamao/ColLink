use super::{DbConfig, DbInspector, DbTableSummary};
use crate::models::{ParsedColumn, ParsedTable};
use async_trait::async_trait;
use mysql_async::prelude::*;
use mysql_async::{OptsBuilder, Pool};
use std::time::Duration;

pub struct MysqlInspector {
    config: DbConfig,
}

impl MysqlInspector {
    pub fn new(config: DbConfig) -> Self {
        Self { config }
    }

    fn get_pool(&self) -> Result<Pool, String> {
        let opts = OptsBuilder::default()
            .ip_or_hostname(&self.config.host)
            .tcp_port(self.config.port)
            .user(Some(&self.config.username))
            .pass(Some(&self.config.password))
            .db_name(Some(&self.config.database))
            .prefer_socket(false);

        Ok(Pool::new(opts))
    }

    fn format_error(e: impl std::fmt::Display) -> String {
        let err_str = e.to_string();
        if err_str.contains("Access denied") || err_str.contains("1045") {
            format!("MySQL 认证失败: 用户名或密码错误 ({})", err_str)
        } else if err_str.contains("Unknown database") || err_str.contains("1049") {
            format!("MySQL 数据库不存在: 请核对数据库名称 ({})", err_str)
        } else if err_str.contains("Connection refused") || err_str.contains("10061") {
            format!(
                "MySQL 连接被拒绝: 无法连接到目标主机或端口，请检查数据库服务是否开启 ({})",
                err_str
            )
        } else if err_str.contains("timed out") || err_str.contains("timeout") {
            format!("MySQL 连接超时: 网络延迟过高或目标地址不可达 ({})", err_str)
        } else {
            format!("MySQL 操作异常: {}", err_str)
        }
    }
}

#[async_trait]
impl DbInspector for MysqlInspector {
    async fn test_connection(&self) -> Result<(), String> {
        let pool = self.get_pool()?;
        let fut = async {
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|e| Self::format_error(e))?;
            conn.query_drop("SELECT 1")
                .await
                .map_err(|e| Self::format_error(e))?;
            let _ = pool.disconnect().await;
            Ok::<(), String>(())
        };

        match tokio::time::timeout(Duration::from_secs(8), fut).await {
            Ok(res) => res,
            Err(_) => Err("MySQL 连接超时 (超过 8 秒未响应)，请检查主机 IP、端口及网络防火墙。".to_string()),
        }
    }

    async fn list_tables(&self) -> Result<Vec<DbTableSummary>, String> {
        let pool = self.get_pool()?;
        let db_name = self.config.database.clone();

        let fut = async move {
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|e| Self::format_error(e))?;

            let sql = "
                SELECT 
                    t.TABLE_NAME, 
                    COALESCE(t.TABLE_COMMENT, ''),
                    (SELECT COUNT(*) FROM information_schema.COLUMNS c 
                     WHERE c.TABLE_SCHEMA = t.TABLE_SCHEMA AND c.TABLE_NAME = t.TABLE_NAME) AS col_count
                FROM information_schema.TABLES t
                WHERE t.TABLE_SCHEMA = ? 
                  AND t.TABLE_TYPE IN ('BASE TABLE', 'SYSTEM VERSIONED')
                  AND t.TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys')
                ORDER BY t.TABLE_NAME;
            ";

            let rows: Vec<(String, String, Option<usize>)> = conn
                .exec(sql, (&db_name,))
                .await
                .map_err(|e| Self::format_error(e))?;

            let _ = pool.disconnect().await;

            let summaries = rows
                .into_iter()
                .map(|(name, comment, count)| DbTableSummary {
                    table_name: name,
                    table_comment: if comment.is_empty() {
                        None
                    } else {
                        Some(comment)
                    },
                    column_count: count,
                })
                .collect();

            Ok::<Vec<DbTableSummary>, String>(summaries)
        };

        match tokio::time::timeout(Duration::from_secs(12), fut).await {
            Ok(res) => res,
            Err(_) => Err("查询 MySQL 表清单超时，请检查数据库负载与网络连接。".to_string()),
        }
    }

    async fn inspect_tables(&self, table_names: &[String]) -> Result<Vec<ParsedTable>, String> {
        if table_names.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.get_pool()?;
        let db_name = self.config.database.clone();
        let names_vec = table_names.to_vec();

        let fut = async move {
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|e| Self::format_error(e))?;

            let mut table_comment_map: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            let mut table_cols_map: std::collections::HashMap<String, Vec<ParsedColumn>> =
                std::collections::HashMap::new();

            for chunk in names_vec.chunks(50) {
                let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

                let tbl_sql = format!(
                    "SELECT TABLE_NAME, COALESCE(TABLE_COMMENT, '') 
                     FROM information_schema.TABLES 
                     WHERE TABLE_SCHEMA = ? AND TABLE_NAME IN ({})",
                    placeholders
                );

                let mut tbl_params: Vec<mysql_async::Value> = vec![db_name.clone().into()];
                for t in chunk {
                    tbl_params.push(t.clone().into());
                }

                let tbl_rows: Vec<(String, String)> = conn
                    .exec(tbl_sql, tbl_params)
                    .await
                    .map_err(|e| Self::format_error(e))?;

                for (tbl, comment) in tbl_rows {
                    if !comment.is_empty() {
                        table_comment_map.insert(tbl, comment);
                    }
                }

                let col_sql = format!(
                    "SELECT 
                        TABLE_NAME, 
                        COLUMN_NAME, 
                        COLUMN_TYPE, 
                        COALESCE(COLUMN_COMMENT, ''), 
                        ORDINAL_POSITION,
                        COALESCE(COLUMN_KEY, '')
                     FROM information_schema.COLUMNS
                     WHERE TABLE_SCHEMA = ? AND TABLE_NAME IN ({})
                     ORDER BY TABLE_NAME, ORDINAL_POSITION",
                    placeholders
                );

                let mut col_params: Vec<mysql_async::Value> = vec![db_name.clone().into()];
                for t in chunk {
                    col_params.push(t.clone().into());
                }

                let col_rows: Vec<(String, String, String, String, i32, String)> = conn
                    .exec(col_sql, col_params)
                    .await
                    .map_err(|e| Self::format_error(e))?;

                for (tbl_name, col_name, col_type, col_comment, ordinal, col_key) in col_rows {
                    let is_pk = col_key.eq_ignore_ascii_case("PRI");
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
                        data_type: col_type.to_uppercase(),
                        column_comment: final_comment,
                        sort_order: ordinal,
                    };
                    table_cols_map.entry(tbl_name).or_default().push(col);
                }
            }

            let _ = pool.disconnect().await;

            let mut result = Vec::new();
            for t in &names_vec {
                let cols = table_cols_map.remove(t).unwrap_or_default();
                let comment = table_comment_map.remove(t);
                result.push(ParsedTable {
                    table_name: t.clone(),
                    table_comment: comment,
                    columns: cols,
                });
            }

            Ok::<Vec<ParsedTable>, String>(result)
        };

        match tokio::time::timeout(Duration::from_secs(30), fut).await {
            Ok(res) => res,
            Err(_) => Err("提取 MySQL 字段元数据超时，请检查数据表数量或网络连通性。".to_string()),
        }
    }
}
