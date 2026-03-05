rs-liquibase
rs-liquibase 是一个用 Rust 编写的轻量级数据库版本管理工具。它深度兼容 Liquibase XML 格式，旨在提供比 Java 版 Liquibase 更快、更小、零依赖的迁移体验。

✨ 特性
轻量且快速：基于 Rust 编写，无 JVM 依赖，启动毫秒级。

兼容性：支持 Liquibase XML 格式（include、changeSet、sql 标签）。

安全性：内置 SHA256 Checksum 校验，自动拦截已执行脚本的非授权篡改。

智能 SQL 拆分：自动解析多行 SQL，支持注释过滤，适配不支持多语句执行的数据库驱动。

多库支持：基于 sqlx，原生支持 PostgreSQL, MySQL, SQLite。

🚀 快速上手
1. 准备 Changelog 文件
   master.xml

XML
<?xml version="1.0" encoding="utf-8"?>
<databaseChangeLog xmlns="http://www.liquibase.org/xml/ns/dbchangelog">
    <include file="001_init_table.xml"/>
</databaseChangeLog>
001_init_table.xml

XML
<databaseChangeLog xmlns="http://www.liquibase.org/xml/ns/dbchangelog">
<changeSet id="abel-2026-03-05" author="abel">
<sql>
CREATE TABLE users (id INT PRIMARY KEY, name TEXT);
-- 支持单行注释
INSERT INTO users VALUES (1, 'Gemini');
</sql>
</changeSet>
</databaseChangeLog>
2. 作为库使用 (Library)
   在你的 Cargo.toml 中添加：

Ini, TOML
```toml
[dependencies]
rs-liquibase = { git = "https://github.com/asdtiang/rs-liquibase.git" }
sqlx = { version = "0.7", features = [
    "runtime-tokio-native-tls", 
    "any",      # 必须开启 any 
    "mysql",
    "postgres", 
    "sqlite"
] }
```
注意选择适合自己的数据库类型。
在 Rust 代码中执行迁移：

```Rust
use rs_liquibase::Liquibase;
use sqlx::any::AnyPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new().connect("sqlite://test.db?mode=rwc").await?;

    let lb = Liquibase::new(pool);
    lb.run("migrations/master.xml").await?;
    
    Ok(())
}
```

RBatis集成

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = "mysql://root:123456@localhost:3306/mydb";

    // --- 第一步：使用 rs-liquibase 确保数据库版本正确 ---
    sqlx::any::install_default_drivers();
    let migrate_pool = sqlx::AnyPool::connect(db_url).await?;
    let lb = rs_liquibase::Liquibase::new(migrate_pool);
    lb.run("migrations/master.xml").await?; 
    println!("✅ 数据库迁移完成");

    // --- 第二 step：初始化 RBatis 进行业务开发 ---
    let rb = RBatis::new();
    // RBatis 内部会自动根据 URL 初始化它需要的底层驱动（通常也是 SQLx）
    rb.init(sqlx::mysql::MySqlDriver {}, db_url)?;
    println!("🚀 RBatis 已就绪");

    // 此时你可以放心地使用 rb.query(...) 了，因为表肯定已经存在
    Ok(())
}
```


🛠️ 技术实现细节
SQL 智能清洗逻辑
为了解决某些数据库驱动无法一次性执行多条语句的问题，rs-liquibase 会对 <sql> 标签内容进行以下处理：

按行扫描：过滤空行。

移除注释：自动忽略 -- 和 # 开头的行。

语句重组：以分号 ; 为界限拆分 SQL，确保每条指令独立、有序地在事务中执行。

Checksum 机制
每次执行迁移时，系统会计算 SQL 块的哈希值并存入 DATABASECHANGELOG 表。

如果本地文件被修改，执行时会触发 MD5Sum mismatch 报错并停止。

这确保了生产环境的数据库结构与代码版本库严格一致。

📅 开发计划 (Roadmap)
[x] 核心 XML 解析与 SQL 拆分执行

[x] Checksum 完整性校验

[x] 跨数据库 Any 驱动支持

[ ] 支持 rollback 标签

[ ] 提供独立编译的二进制 CLI 工具

[ ] 支持 DATABASECHANGELOGLOCK 锁机制（并发安全）

📄 开源协议
MIT