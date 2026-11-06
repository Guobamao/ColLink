use super::{DbConfig, DbInspector, DbTableSummary};
use crate::models::{ParsedColumn, ParsedTable};
use async_trait::async_trait;
use mysql_async::prelude::*;
use mysql_async::{OptsBuilder, Pool};

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
}

#[async_trait]
impl DbInspector for MysqlInspector {
    async fn test_connection(&self) -> Result<(), String> {
        let pool = self.get_pool()?;
        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| format!("MySQL 连接失败: {}", e))?;
        let _ = conn
            .query_drop("SELECT 1")
            .await
            .map_err(|e| format!("MySQL 执行测试查询失败: {}", e))?;
        pool.disconnect().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn list_tables(&self) -> Result<Vec<DbTableSummary>, String> {
        let pool = self.get_pool()?;
        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| format!("MySQL 连接失败: {}", e))?;

        let sql = "
            SELECT 
                t.TABLE_NAME, 
                COALESCE(t.TABLE_COMMENT, ''),
                (SELECT COUNT(*) FROM information_schema.COLUMNS c 
                 WHERE c.TABLE_SCHEMA = t.TABLE_SCHEMA AND c.TABLE_NAME = t.TABLE_NAME) AS col_count
            FROM information_schema.TABLES t
            WHERE t.TABLE_SCHEMA = ? AND t.TABLE_TYPE = 'BASE TABLE'
            ORDER BY t.TABLE_NAME;
        ";

        let rows: Vec<(String, String, Option<usize>)> = conn
            .exec(sql, (&self.config.database,))
            .await
            .map_err(|e| format!("查询 MySQL 表清单失败: {}", e))?;

        pool.disconnect().await.map_err(|e| e.to_string())?;

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

        Ok(summaries)
    }

    async fn inspect_tables(&self, table_names: &[String]) -> Result<Vec<ParsedTable>, String> {
        if table_names.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.get_pool()?;
        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| format!("MySQL 连接失败: {}", e))?;

        // 1. 获取选定表的中文注释
        let tbl_placeholders = table_names
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let tbl_sql = format!(
            "SELECT TABLE_NAME, COALESCE(TABLE_COMMENT, '') 
             FROM information_schema.TABLES 
             WHERE TABLE_SCHEMA = ? AND TABLE_NAME IN ({})",
            tbl_placeholders
        );

        let mut tbl_params: Vec<mysql_async::Value> = vec![self.config.database.clone().into()];
        for t in table_names {
            tbl_params.push(t.clone().into());
        }

        let tbl_rows: Vec<(String, String)> = conn
            .exec(tbl_sql, tbl_params)
            .await
            .map_err(|e| format!("查询 MySQL 表注释失败: {}", e))?;

        let mut table_comment_map: std::collections::HashMap<String, String> =
            tbl_rows.into_iter().collect();

        // 2. 获取列级字段元数据
        let col_placeholders = table_names
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let col_sql = format!(
            "SELECT 
                TABLE_NAME, 
                COLUMN_NAME, 
                COLUMN_TYPE, 
                COALESCE(COLUMN_COMMENT, ''), 
                ORDINAL_POSITION
             FROM information_schema.COLUMNS
             WHERE TABLE_SCHEMA = ? AND TABLE_NAME IN ({})
             ORDER BY TABLE_NAME, ORDINAL_POSITION",
            col_placeholders
        );

        let mut col_params: Vec<mysql_async::Value> = vec![self.config.database.clone().into()];
        for t in table_names {
            col_params.push(t.clone().into());
        }

        let col_rows: Vec<(String, String, String, String, i32)> = conn
            .exec(col_sql, col_params)
            .await
            .map_err(|e| format!("查询 MySQL 列元数据失败: {}", e))?;

        pool.disconnect().await.map_err(|e| e.to_string())?;

        // 按表名聚合字段
        let mut table_cols_map: std::collections::HashMap<String, Vec<ParsedColumn>> =
            std::collections::HashMap::new();

        for (tbl_name, col_name, col_type, col_comment, ordinal) in col_rows {
            let col = ParsedColumn {
                column_name: col_name,
                data_type: col_type.to_uppercase(),
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
            let comment = table_comment_map.remove(t).filter(|c| !c.is_empty());
            result.push(ParsedTable {
                table_name: t.clone(),
                table_comment: comment,
                columns: cols,
            });
        }

        Ok(result)
    }
}
