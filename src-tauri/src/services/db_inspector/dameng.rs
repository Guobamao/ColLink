use super::{DbConfig, DbInspector, DbTableSummary};
use crate::models::{ParsedColumn, ParsedTable};
use async_trait::async_trait;
use dm_pure::{DmClient, ExecuteResult, Value};
use std::collections::HashMap;

pub struct DamengInspector {
    config: DbConfig,
}

impl DamengInspector {
    pub fn new(config: DbConfig) -> Self {
        Self { config }
    }

    fn connect(&self) -> Result<DmClient, String> {
        let mut client = DmClient::new(
            &self.config.host,
            self.config.port,
            &self.config.username,
            &self.config.password,
        );
        client
            .connect()
            .map_err(|e| Self::format_error(e))?;
        Ok(client)
    }

    fn get_schema(&self) -> String {
        let trimmed = self.config.database.trim();
        if trimmed.is_empty() {
            self.config.username.trim().to_uppercase()
        } else {
            trimmed.to_uppercase()
        }
    }

    fn format_error(e: impl std::fmt::Debug) -> String {
        let err_str = format!("{:?}", e);
        if err_str.contains("Login") || err_str.contains("password") || err_str.contains("用户") {
            format!("达梦数据库认证失败: 用户名或密码错误 ({})", err_str)
        } else if err_str.contains("Connection refused") || err_str.contains("10061") {
            format!(
                "达梦数据库连接被拒绝: 目标端口 5236 不可达，请确认达梦数据库服务已启动 ({})",
                err_str
            )
        } else if err_str.contains("timeout") || err_str.contains("timed out") {
            format!("达梦数据库连接超时: 目标主机无响应 ({})", err_str)
        } else {
            format!("达梦数据库操作异常: {}", err_str)
        }
    }
}

fn val_to_str(val: &Value) -> String {
    match val {
        Value::Str(s) => s.trim().to_string(),
        Value::Int(i) => i.to_string(),
        Value::BigInt(i) => i.to_string(),
        Value::SmallInt(i) => i.to_string(),
        Value::TinyInt(i) => i.to_string(),
        Value::Decimal(s) => s.trim().to_string(),
        Value::Double(f) => f.to_string(),
        Value::Real(f) => f.to_string(),
        Value::Date(d) => d.to_string(),
        Value::DateTime(dt) => dt.to_string(),
        Value::Time(t) => t.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Bytes(b) => String::from_utf8_lossy(b).to_string(),
        Value::Raw(b) => String::from_utf8_lossy(b).to_string(),
        Value::Interval(s) => s.clone(),
        Value::Null => String::new(),
    }
}

fn format_dm_type(dtype: &str, length: i64, precision: i64, scale: i64) -> String {
    let upper = dtype.to_uppercase();
    if upper == "VARCHAR" || upper == "VARCHAR2" || upper == "CHAR" {
        if length > 0 {
            format!("{}({})", upper, length)
        } else {
            upper
        }
    } else if upper == "NUMBER" || upper == "DECIMAL" || upper == "NUMERIC" {
        if precision > 0 {
            if scale > 0 {
                format!("{}({}, {})", upper, precision, scale)
            } else {
                format!("{}({})", upper, precision)
            }
        } else {
            upper
        }
    } else {
        upper
    }
}

#[async_trait]
impl DbInspector for DamengInspector {
    async fn test_connection(&self) -> Result<(), String> {
        let mut client = self.connect()?;
        let res = client
            .execute("SELECT 1 FROM DUAL", &[])
            .map_err(|e| Self::format_error(e))?;
        client.close();
        match res {
            ExecuteResult::Query { .. } => Ok(()),
            _ => Ok(()),
        }
    }

    async fn list_tables(&self) -> Result<Vec<DbTableSummary>, String> {
        let mut client = self.connect()?;
        let schema = self.get_schema();

        let sql = format!(
            "SELECT 
                T.TABLE_NAME, 
                COALESCE(C.COMMENTS, '') AS COMMENTS,
                (SELECT COUNT(*) FROM ALL_TAB_COLS TC 
                 WHERE TC.OWNER = T.OWNER AND TC.TABLE_NAME = T.TABLE_NAME) AS COL_COUNT
             FROM ALL_TABLES T
             LEFT JOIN ALL_TAB_COMMENTS C 
               ON T.OWNER = C.OWNER AND T.TABLE_NAME = C.TABLE_NAME
             WHERE T.OWNER = '{}'
               AND T.OWNER NOT IN ('SYS', 'CTISYS', 'SYSAUDITOR', 'SYSSSO')
             ORDER BY T.TABLE_NAME",
            schema.replace('\'', "''")
        );

        let res = client
            .execute(&sql, &[])
            .map_err(|e| Self::format_error(e))?;
        client.close();

        let mut summaries = Vec::new();
        if let ExecuteResult::Query { columns: _, rows } = res {
            for row in rows {
                let tbl_name = row.first().map(val_to_str).unwrap_or_default();
                let comment = row.get(1).map(val_to_str).unwrap_or_default();
                let col_count = row.get(2).and_then(|v| v.as_i64()).map(|n| n as usize);

                if !tbl_name.is_empty() {
                    summaries.push(DbTableSummary {
                        table_name: tbl_name,
                        table_comment: if comment.is_empty() {
                            None
                        } else {
                            Some(comment)
                        },
                        column_count: col_count,
                    });
                }
            }
        }

        Ok(summaries)
    }

    async fn inspect_tables(&self, table_names: &[String]) -> Result<Vec<ParsedTable>, String> {
        if table_names.is_empty() {
            return Ok(Vec::new());
        }

        let mut client = self.connect()?;
        let schema = self.get_schema();

        let mut table_comment_map: HashMap<String, String> = HashMap::new();
        let mut table_cols_map: HashMap<String, Vec<ParsedColumn>> = HashMap::new();

        // 分批执行（每批 50 张表）
        for chunk in table_names.chunks(50) {
            let in_clause = chunk
                .iter()
                .map(|t| format!("'{}'", t.replace('\'', "''")))
                .collect::<Vec<_>>()
                .join(", ");

            let sql = format!(
                "SELECT 
                    C.TABLE_NAME, 
                    C.COLUMN_NAME, 
                    C.DATA_TYPE, 
                    COALESCE(C.DATA_LENGTH, 0) AS DATA_LENGTH, 
                    COALESCE(C.DATA_PRECISION, 0) AS DATA_PRECISION, 
                    COALESCE(C.DATA_SCALE, 0) AS DATA_SCALE, 
                    COALESCE(M.COMMENTS, '') AS COLUMN_COMMENT, 
                    C.COLUMN_ID,
                    COALESCE(TC.COMMENTS, '') AS TABLE_COMMENT,
                    CASE WHEN PK.COLUMN_NAME IS NOT NULL THEN 1 ELSE 0 END AS IS_PK
                 FROM ALL_TAB_COLS C
                 LEFT JOIN ALL_COL_COMMENTS M 
                   ON C.OWNER = M.OWNER AND C.TABLE_NAME = M.TABLE_NAME AND C.COLUMN_NAME = M.COLUMN_NAME
                 LEFT JOIN ALL_TAB_COMMENTS TC
                   ON C.OWNER = TC.OWNER AND C.TABLE_NAME = TC.TABLE_NAME
                 LEFT JOIN (
                     SELECT CC.OWNER, CC.TABLE_NAME, CC.COLUMN_NAME
                     FROM ALL_CONSTRAINTS CONS
                     JOIN ALL_CONS_COLUMNS CC 
                       ON CONS.OWNER = CC.OWNER AND CONS.CONSTRAINT_NAME = CC.CONSTRAINT_NAME
                     WHERE CONS.CONSTRAINT_TYPE = 'P'
                 ) PK ON C.OWNER = PK.OWNER AND C.TABLE_NAME = PK.TABLE_NAME AND C.COLUMN_NAME = PK.COLUMN_NAME
                 WHERE C.OWNER = '{}' AND C.TABLE_NAME IN ({})
                 ORDER BY C.TABLE_NAME, C.COLUMN_ID",
                schema.replace('\'', "''"),
                in_clause
            );

            let res = client
                .execute(&sql, &[])
                .map_err(|e| Self::format_error(e))?;

            if let ExecuteResult::Query { columns: _, rows } = res {
                for row in rows {
                    let tbl_name = row.first().map(val_to_str).unwrap_or_default();
                    let col_name = row.get(1).map(val_to_str).unwrap_or_default();
                    let dtype = row.get(2).map(val_to_str).unwrap_or_default();
                    let length = row.get(3).and_then(|v| v.as_i64()).unwrap_or(0);
                    let prec = row.get(4).and_then(|v| v.as_i64()).unwrap_or(0);
                    let scale = row.get(5).and_then(|v| v.as_i64()).unwrap_or(0);
                    let col_comment = row.get(6).map(val_to_str).unwrap_or_default();
                    let ordinal = row
                        .get(7)
                        .and_then(|v| v.as_i64())
                        .map(|n| n as i32)
                        .unwrap_or(1);
                    let tbl_comment = row.get(8).map(val_to_str).unwrap_or_default();
                    let is_pk = row
                        .get(9)
                        .and_then(|v| v.as_i64())
                        .map(|n| n == 1)
                        .unwrap_or(false);

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

                    let full_type = format_dm_type(&dtype, length, prec, scale);
                    let col = ParsedColumn {
                        column_name: col_name,
                        data_type: full_type,
                        column_comment: final_comment,
                        sort_order: ordinal,
                    };

                    table_cols_map.entry(tbl_name).or_default().push(col);
                }
            }
        }

        client.close();

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
