use crate::models::ParsedTable;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod dameng;
pub mod mysql;
pub mod postgres;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub db_type: String, // "dm" | "mysql" | "postgres"
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTableSummary {
    pub table_name: String,
    #[serde(default)]
    pub table_comment: Option<String>,
    #[serde(default)]
    pub column_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTestResult {
    pub success: bool,
    pub message: String,
}

#[async_trait]
pub trait DbInspector: Send + Sync {
    /// 1. 测试数据库连通性与权限
    async fn test_connection(&self) -> Result<(), String>;

    /// 2. 获取当前库/模式下所有数据表摘要（表名、中文注释、字段数）
    async fn list_tables(&self) -> Result<Vec<DbTableSummary>, String>;

    /// 3. 提取勾选数据表的列级完整元数据
    async fn inspect_tables(&self, table_names: &[String]) -> Result<Vec<ParsedTable>, String>;
}

pub fn create_inspector(config: &DbConfig) -> Result<Box<dyn DbInspector>, String> {
    match config.db_type.to_lowercase().as_str() {
        "mysql" => Ok(Box::new(mysql::MysqlInspector::new(config.clone()))),
        "postgres" | "pg" | "postgresql" => {
            Ok(Box::new(postgres::PostgresInspector::new(config.clone())))
        }
        "dm" | "dameng" => Ok(Box::new(dameng::DamengInspector::new(config.clone()))),
        other => Err(format!("暂不支持的数据库类型: {}", other)),
    }
}
