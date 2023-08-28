pub mod cli;
pub mod data;

use cli::{parse_cli, Opt};
use data::Db;
use log::info;

#[allow(dead_code)]
async fn test_search_tags() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "mongodb://localhost:27017";
    let app_name = "rtag".to_string();
    let db_con = Db::new(addr, app_name).await.unwrap();
    db_con.search_tag(&vec!["test".to_string()]).await?;

    Ok(())
}

/// match_func是根据命令行参数，调用不同功能的接口位置
/// TODO: 继续开发
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

    if let Some(value) = opt.value {
        // TODO: 查询所有存在此字符串的值，以及对应的tag
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "mongodb://localhost:27017";
    let app_name = "rtag".to_string();
    let mut db_con = Db::new(addr, app_name).await.unwrap();
    // let opt = parse_cli();
    // match_func(db_con, opt).await?;
    db_con.update_tag(&vec!["123".to_string()], "xxx").await?;

    Ok(())
}
