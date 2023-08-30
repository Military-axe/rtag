pub mod cli;
pub mod data;
pub mod rtag_config;

use cli::{parse_cli, Opt};
use data::Db;
use log::{error, info};
use rtag_config::read_config;
#[allow(unused_imports)]
use std::env::{set_var, var};
use std::process::exit;

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
    set_var("RUST_LOG", "info");
    set_var("RTAG", "D:/Documents/git_down/rtag_data/config");
    env_logger::init();
    info!("start");

    // 获取RTAG环境变量
    let config_file_path = match var("RTAG") {
        Ok(x) => x,
        Err(_) => {
            error!("[!] \"RATG\" environment variable not found");
            exit(0);
        }
    };
    // 打开RTAG变量中记录的配置文件路径
    let config = read_config(&(config_file_path + "/rtag.toml"));
    let db_con = Db::new(&config.mongodb_url, &config.database_name)
        .await
        .unwrap();

    info!("[+] connect database");
    let opt = parse_cli();
    match_func(db_con, opt).await?;
    Ok(())
}
