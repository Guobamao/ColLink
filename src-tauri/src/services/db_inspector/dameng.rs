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
            .map_err(|e| format!("达梦数据库连接失败: {:?}", e))?;
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
            .map_err(|e| format!("达梦执行测试语句失败: {:?}", e))?;
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
             ORDER BY T.TABLE_NAME",
            schema.replace('\'', "''")
        );

        let res = client
            .execute(&sql, &[])
            .map_err(|e| format!("查询达梦数据表清单失败: {:?}", e))?;
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

        let in_clause = table_names
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
                COALESCE(TC.COMMENTS, '') AS TABLE_COMMENT
             FROM ALL_TAB_COLS C
             LEFT JOIN ALL_COL_COMMENTS M 
               ON C.OWNER = M.OWNER AND C.TABLE_NAME = M.TABLE_NAME AND C.COLUMN_NAME = M.COLUMN_NAME
             LEFT JOIN ALL_TAB_COMMENTS TC
               ON C.OWNER = TC.OWNER AND C.TABLE_NAME = TC.TABLE_NAME
             WHERE C.OWNER = '{}' AND C.TABLE_NAME IN ({})
             ORDER BY C.TABLE_NAME, C.COLUMN_ID",
            schema.replace('\'', "''"),
            in_clause
        );

        let res = client
            .execute(&sql, &[])
            .map_err(|e| format!("查询达梦数据表字段元数据失败: {:?}", e))?;
        client.close();

        let mut table_comment_map: HashMap<String, String> = HashMap::new();
        let mut table_cols_map: HashMap<String, Vec<ParsedColumn>> = HashMap::new();

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

                if !tbl_comment.is_empty() {
                    table_comment_map.insert(tbl_name.clone(), tbl_comment);
                }

                let full_type = format_dm_type(&dtype, length, prec, scale);
                let col = ParsedColumn {
                    column_name: col_name,
                    data_type: full_type,
                    column_comment: if col_comment.is_empty() {
                        None
                    } else {
                        Some(col_comment)
                    },
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
