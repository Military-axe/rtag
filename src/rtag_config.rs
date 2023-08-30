use log::error;
use serde::Deserialize;
use std::{fs::File, io::Read, process::exit};
use toml;

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub mongodb_url: String,
    pub database_name: Option<String>,
    pub tags_collect: Option<String>,
    pub values_collect: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: Database,
}

#[derive(Debug)]
pub struct Conf {
    pub mongodb_url: String,
    pub database_name: String,
    pub tags_collect: String,
    pub values_collect: String,
}

pub fn read_config(config_path: &String) -> Conf {
    let mut str_val = String::new();
    let mut file = match File::open(config_path) {
        Ok(f) => f,
        Err(e) => {
            error!("[!] no such file {} exception:{}", config_path, e);
            exit(0);
        }
    };

    match file.read_to_string(&mut str_val) {
        Ok(s) => s,
        Err(e) => panic!("Error Reading file: {}", e),
    };
    let conf: Config = toml::from_str(&str_val).unwrap();
    let config = conf.database;

    let name = config.database_name.unwrap_or("rtag".to_string());
    let collect1 = config.tags_collect.unwrap_or("tags".to_string());
    let collect2 = config.values_collect.unwrap_or("values".to_string());

    let conf = Conf {
        mongodb_url: config.mongodb_url,
        database_name: name,
        tags_collect: collect1,
        values_collect: collect2,
    };

    return conf;
}
