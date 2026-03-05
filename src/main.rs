use rs_liquibase::Liquibase;
use sqlx::any::AnyPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: rs-liquibase <db_url> <master_xml>");
        return Ok(());
    }

    // 必须安装驱动以支持动态识别数据库前缀
    sqlx::any::install_default_drivers();

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&args[1])
        .await?;

    let lb = Liquibase::new(pool);
    lb.run(&args[2]).await?;

    Ok(())
}
