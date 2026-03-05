pub mod models;

use anyhow::{anyhow, Context, Result};
use models::{ChildChangeLog, MasterChangeLog};
use sha2::{Digest, Sha256};
use sqlx::{Any, Pool, Row};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Liquibase {
    pool: Pool<Any>,
}

impl Liquibase {
    pub fn new(pool: Pool<Any>) -> Self {
        Self { pool }
    }

    /// 初始化管理表
    pub async fn init_metadata_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS DATABASECHANGELOG (
                ID VARCHAR(255) NOT NULL,
                AUTHOR VARCHAR(255) NOT NULL,
                FILENAME VARCHAR(255) NOT NULL,
                DATEEXECUTED TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                MD5SUM VARCHAR(64),
                PRIMARY KEY (ID, AUTHOR, FILENAME)
            );
        "#;
        sqlx::query(sql).execute(&self.pool).await?;
        Ok(())
    }

    /// 核心 SQL 行处理算法
    /// 逻辑：按行扫描 -> 过滤注释/空行 -> 拼接 -> 以分号结束则切分
    fn split_sql_by_line(&self, raw_sql: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();

        for line in raw_sql.lines() {
            let trimmed = line.trim();

            // 1. 过滤空行
            if trimmed.is_empty() {
                continue;
            }

            // 2. 过滤注释行 (支持 -- 和 #)
            if trimmed.starts_with("--") || trimmed.starts_with("#") {
                continue;
            }

            // 3. 拼接当前行
            current_statement.push_str(line);
            current_statement.push('\n');

            // 4. 检查是否以分号结束（代表一条完整的 SQL 结束）
            if trimmed.ends_with(';') {
                statements.push(current_statement.trim().to_string());
                current_statement.clear();
            }
        }

        // 处理最后一行没有分号但仍有内容的情况
        if !current_statement.trim().is_empty() {
            statements.push(current_statement.trim().to_string());
        }

        statements
    }

    pub async fn run(&self, master_path: &str) -> Result<()> {
        self.init_metadata_table().await?;

        let master_abs_path = PathBuf::from(master_path)
            .canonicalize()
            .with_context(|| format!("Cannot find master file: {}", master_path))?;
        let base_dir = master_abs_path.parent().unwrap_or(Path::new("."));

        let master_content = fs::read_to_string(&master_abs_path)?;
        let master: MasterChangeLog = quick_xml::de::from_str(&master_content)?;

        for include in master.includes {
            println!("Includes: {:?}", include);
            let child_path = base_dir.join(&include.file);
            self.process_child_file(child_path.to_str().unwrap())
                .await?;
        }

        Ok(())
    }

    async fn process_child_file(&self, file_path: &str) -> Result<()> {
        println!("file_path: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;
        let child: ChildChangeLog = quick_xml::de::from_str(&content)?;

        for cs in child.change_sets {
            // 计算原始 SQL 内容的哈希（包含注释，确保文件任何改动都能被察觉）
            let current_checksum = self.calculate_checksum(&cs.sql);

            let record =
                sqlx::query("SELECT MD5SUM FROM DATABASECHANGELOG WHERE ID = $1 AND AUTHOR = $2")
                    .bind(&cs.id)
                    .bind(&cs.author)
                    .fetch_optional(&self.pool)
                    .await?;

            match record {
                Some(row) => {
                    let old_checksum: String = row.get(0);
                    if old_checksum != current_checksum {
                        return Err(anyhow!("Checksum mismatch for {}", cs.id));
                    }
                    println!("✅ Skipped: {}", cs.id);
                }
                None => {
                    println!("⏳ Executing: {}", cs.id);
                    let mut tx = self.pool.begin().await?;

                    // 获取处理后的 SQL 语句数组
                    let statements = self.split_sql_by_line(&cs.sql);

                    for sql in statements {
                        sqlx::query(&sql)
                            .execute(&mut *tx)
                            .await
                            .with_context(|| format!("Execution failed in {}: \n{}", cs.id, sql))?;
                    }

                    // 记录历史
                    sqlx::query("INSERT INTO DATABASECHANGELOG (ID, AUTHOR, FILENAME, MD5SUM) VALUES ($1, $2, $3, $4)")
                        .bind(&cs.id)
                        .bind(&cs.author)
                        .bind(file_path)
                        .bind(&current_checksum)
                        .execute(&mut *tx).await?;

                    tx.commit().await?;
                    println!("🚀 Success: {}", cs.id);
                }
            }
        }
        Ok(())
    }

    fn calculate_checksum(&self, sql: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(sql.trim().as_bytes());
        hex::encode(hasher.finalize())
    }
}
