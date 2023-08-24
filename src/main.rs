pub mod data;

use data::Db;

#[allow(dead_code)]
async fn test_search_tags() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "mongodb://localhost:27017";
    let app_name = "rtag".to_string();
    let db_con = Db::new(addr, app_name).await.unwrap();
    db_con.search_tag(&vec!["test".to_string()]).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "mongodb://localhost:27017";
    let app_name = "rtag".to_string();
    // let db_con = Db::new(addr, app_name).await.unwrap();
    Db::init(addr, app_name).await?;

    Ok(())
}