# rs-liquibase

[English](README.md) | [简体中文](README-ZH.MD)

---

rs-liquibase is a lightweight database version management tool written in Rust. It is deeply compatible with the Liquibase XML format, designed to provide a faster, smaller, and zero-dependency migration experience compared to the traditional Java-based Liquibase.


✨ Features
Lightweight & Fast: Built with Rust, no JVM dependency, with millisecond-level startup.

Compatibility: Supports standard Liquibase XML formats (including include, changeSet, and sql tags).

Security: Built-in SHA256 Checksum validation to automatically intercept unauthorized tampering of previously executed scripts.

Smart SQL Splitting: Automatically parses multi-line SQL, supports comment filtering, and adapts to database drivers that do not support multi-statement execution.

Multi-Database Support: Powered by sqlx, with native support for PostgreSQL, MySQL, and SQLite.

🚀 Quick Start
1. Prepare Changelog Files
   master.xml

```XML
<?xml version="1.0" encoding="utf-8"?>
<databaseChangeLog xmlns="http://www.liquibase.org/xml/ns/dbchangelog">
    <include file="001_init_table.xml"/>
</databaseChangeLog>
```
001_init_table.xml

```XML
<databaseChangeLog xmlns="http://www.liquibase.org/xml/ns/dbchangelog">
<changeSet id="abel-2026-03-05" author="abel">
<sql>
CREATE TABLE users (id INT PRIMARY KEY, name TEXT);
-- Supports single-line comments
INSERT INTO users VALUES (1, 'Gemini');
</sql>
</changeSet>
</databaseChangeLog>
```
2. Usage as a Library
   Add the following to your Cargo.toml:

Ini, TOML
```toml
[dependencies]
rs-liquibase = { git = "https://github.com/asdtiang/rs-liquibase.git" }
sqlx = { version = "0.7", features = [
"runtime-tokio-native-tls",
"any",      # Required: enable 'any' driver
"mysql",    # Enable specific database drivers as needed
"postgres",
"sqlite"
] }

```
Update Command: cargo update -p rs-liquibase

3. Execution Example
```Rust
   use rs_liquibase::Liquibase;
   use sqlx::any::AnyPoolOptions;

   #[tokio::main]
   async fn main() -> anyhow::Result<()> {
   // Install drivers to support dynamic database identification
    sqlx::any::install_default_drivers();

    let pool = AnyPoolOptions::new()
        .connect("sqlite://test.db?mode=rwc")
        .await?;

    let lb = Liquibase::new(pool);
    lb.run("migrations/master.xml").await?;
    
    Ok(())
}
```
🔗 RBatis Integration
rs-liquibase works perfectly alongside ORMs like RBatis. The recommended flow is to ensure the database schema is migrated before initializing the ORM.

```Rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
let db_url = "mysql://root:123456@localhost:3306/mydb";

    // --- Step 1: Use rs-liquibase to ensure schema version is correct ---
    sqlx::any::install_default_drivers();
    let migrate_pool = sqlx::AnyPool::connect(db_url).await?;
    let lb = rs_liquibase::Liquibase::new(migrate_pool);
    lb.run("migrations/master.xml").await?; 
    println!("✅ Database migration completed");

    // --- Step 2: Initialize RBatis for business logic ---
    let rb = RBatis::new();
    // RBatis will initialize its underlying driver based on the URL
    rb.init(sqlx::mysql::MySqlDriver {}, db_url)?;
    println!("🚀 RBatis is ready");

    // Now you can safely use rb.query(...) as tables are guaranteed to exist
    Ok(())
}
```
🛠️ Technical Implementation
Smart SQL Cleaning
To solve the issue where some database drivers cannot execute multiple statements at once, rs-liquibase processes the content within <sql> tags as follows:

Line Scanning: Filters out empty lines.

Comment Removal: Automatically ignores lines starting with -- or #.

Statement Reassembly: Splits SQL by the semicolon ; delimiter, ensuring each command is executed independently and sequentially within a transaction.

Checksum Mechanism
During each migration, the system calculates a hash of the SQL block and stores it in the DATABASECHANGELOG table.

If a local file is modified after execution, a MD5Sum mismatch error is triggered, and the process stops.

This ensures that the production database schema remains strictly consistent with the code repository.

📅 Roadmap
[x] Core XML parsing and SQL split execution

[x] Checksum integrity validation

[x] Cross-database Any driver support

[ ] Support for the rollback tag

[ ] Standalone compiled Binary CLI tool

[ ] Support for DATABASECHANGELOGLOCK (Concurrency safety)

📄 License
MIT