use rs_liquibase::Liquibase;
use sqlx::any::AnyPoolOptions;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 定义一个本地 SQLite 数据库文件
    //cargo run --example run_migrations
    let db_path = "test_database.db";
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    // 如果旧的测试库存在，先删除它以保证测试环境纯净
    if Path::new(db_path).exists() {
        fs::remove_file(db_path)?;
    }

    println!("连接数据库: {}", db_url);

    // 2. 初始化数据库连接
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    // 3. 实例化我们的库
    let lb = Liquibase::new(pool);

    // 4. 执行迁移
    println!("--- 开始第一次迁移 ---");
    lb.run("examples/master.xml").await?;

    // 5. 验证数据是否插入成功
    let pool_for_check = AnyPoolOptions::new().connect(&db_url).await?;
    let row: (i64, String) = sqlx::query_as("SELECT id, name FROM test_user WHERE id = 1")
        .fetch_one(&pool_for_check)
        .await?;

    println!("验证结果: ID={}, Name={}", row.0, row.1);
    assert_eq!(row.1, "Abel");

    println!("--- 开始第二次迁移（预期应该跳过已执行的脚本） ---");
    lb.run("examples/master.xml").await?;

    println!("✅ 测试顺利通过！");

    Ok(())
}
