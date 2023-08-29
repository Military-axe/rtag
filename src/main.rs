pub mod cli;
pub mod data;

use cli::{parse_cli, Opt};
use data::Db;
use log::info;
#[allow(unused_imports)]
use std::env::set_var;

#[allow(dead_code)]
async fn test_search_tags() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "mongodb://localhost:27017";
    let app_name = "rtag".to_string();
    let db_con = Db::new(addr, app_name).await.unwrap();
    db_con.search_tag(&vec!["test".to_string()]).await?;

    Ok(())
}

#[allow(dead_code)]
/// match_func是根据命令行参数，调用不同功能的接口位置
async fn match_func(mut db: Db, opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    if !opt.tag.is_empty() {
        if let Some(value) = opt.value {
            // 添加tag对应的值
            db.update_tag(&opt.tag, &value).await?;
            info!("Successfully added field");
        } else {
            // 搜索tags对应的值
            db.search_tag(&opt.tag).await?;
        }

        return Ok(());
    }

    // 查询所有存在此字符串的值，以及对应的tag
    if let Some(value) = opt.value {
        db.find_value(&value).await?
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // set_var("RUST_LOG", "info");
    env_logger::init();
    info!("start");
    let addr = "mongodb://localhost:27017";
    let app_name = "rtag".to_string();
    let db_con = Db::new(addr, app_name).await.unwrap();
    info!("[+] connect database");
    let opt = parse_cli();
    match_func(db_con, opt).await?;
    Ok(())
}
