use colored::Colorize;
use futures::stream::TryStreamExt;
use log::{error, info, warn};
use mongodb::bson::{doc, Bson, Document};
use mongodb::options::FindOneOptions;
use mongodb::{options::ClientOptions, options::UpdateOptions, Client, Collection, Database};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

pub struct Db {
    pub client: Client,
    pub db: Database,
    pub tags_collect: Collection<Document>,
    pub values_collect: Collection<Document>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Value {
    value: String,
    tags: Vec<String>,
}

impl Db {
    // /// init函数创建数据库与集合
    // /// 创建rtag数据库，同时创建tags集合和values集合
    // pub async fn init(addr: &str, app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    //     // 创建 MongoDB 的客户端连接
    //     let mut client_options = ClientOptions::parse(addr).await?;
    //     client_options.app_name = Some(app_name.to_string());
    //     let client = Client::with_options(client_options)?;

    //     // 获取要创建的数据库和集合名称
    //     // TODO: 后期可以修改成toml文件读取
    //     let database_name = "rtag";
    //     let collection_name = "tags";
    //     let collection_name2 = "values";

    //     // 创建数据库
    //     let database = client.database(database_name);

    //     // 创建集合（如果不存在）
    //     database.create_collection(collection_name, None).await?;
    //     database.create_collection(collection_name2, None).await?;

    //     info!("Database and collection created successfully.");

    //     Ok(())
    // }

    /// new函数连接mongodb数据库并返回Result<DataBase, Box<dyn std::error::Error>>
    /// DataBase中存储了client是和数据库的连接，通过此连接来读写数据库.
    /// 函数有两个参数，addr和app_name
    /// addr: &str;是连接数据库的uri地址，默认是"mongodb://localhost:27017"
    /// app_name: String;是数据库日志记录过程中的一个表示，方便调试
    pub async fn new(
        addr: &str,
        app_name: &str,
        tags: &str,
        values: &str,
    ) -> Result<Db, Box<dyn std::error::Error>> {
        let mut client_options = ClientOptions::parse(addr).await?;
        client_options.app_name = Some(app_name.to_string());

        // 建立与MongoDB的连接
        let c = match Client::with_options(client_options) {
            Err(x) => {
                error!("connect mongodb failed; error: {}", x);
                panic!("connect mongodb failed");
            }
            Ok(x) => x,
        };

        // 连接数据库，数据库名这是暂定是rtag
        // TODO: 数据库名，集合名传参
        let db = c.database(app_name);
        let tags_collection: mongodb::Collection<mongodb::bson::Document> = db.collection(tags);
        let values_collection = db.collection(values);
        let data_base = Db {
            client: c,
            db,
            tags: Db::find_tags(&tags_collection).await.unwrap(),
            tags_collect: tags_collection,
            values_collect: values_collection,
        };
        Ok(data_base)
    }

    /// find_tags是查询数据库有多少个tag，返回一个Vec<String>记录所有的tag
    pub async fn find_tags(
        collect: &Collection<Document>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let filter = doc! { "tag": { "$exists": true } };
        let mut cursor = collect.find(filter, None).await?;
        let mut tags: Vec<String> = Vec::new();

        while let Some(document) = cursor.try_next().await? {
            // 处理查询结果
            if let Some(tag) = document.get_str("tag").ok().map(|s| s.to_owned()) {
                tags.push(tag);
            }
        }

        Ok(tags)
    }

    /// 搜索多个tag都有的数据
    pub async fn search_tag(&self, tags: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut hashmap: HashMap<String, usize> = HashMap::new();

        for tag in tags {
            let query = doc! {"tag": tag};
            let result = self.tags_collect.find_one(query, None).await?;

            if let Some(document) = result {
                let value_array = match document.get_array("value") {
                    Ok(array) => array,
                    Err(_) => continue, // 忽略获取不到数组的情况，继续下一个tag
                };

                for ele in value_array.iter() {
                    let key = ele.to_string();
                    let count = hashmap.entry(key).or_insert(0);
                    *count += 1;
                }
            }
        }

        // 打印结果
        for (ele, &count) in hashmap.iter() {
            if count >= tags.len() {
                println!("{}", ele.replace("\"", "").red());
            }
        }

        Ok(())
    }

    /// 向数据库插入值，当这个tag中已经有这个值的时候就不会重复添加
    async fn add_value_in_tags(
        &self,
        tags: &Vec<String>,
        val: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for tag in tags {
            // let value_doc = mongodb::bson::to_document(&val)?;
            let update_doc = doc! {
                "$addToSet": {
                    "value": val
                }
            };

            let options = UpdateOptions::builder().upsert(false).build();
            let query = doc! {"tag": tag};
            self.tags_collect
                .update_one(query, update_doc, options)
                .await?;
        }

        info!("add tags: {:?} and value: {} success", tags, val);

        Ok(())
    }

    /// add_tag是数据库中tags集合中创建一个新的tag文档
    async fn add_tag(&self, tag: &str) -> Result<(), Box<dyn std::error::Error>> {
        let document = doc! {"tag": tag, "value": []};
        self.tags_collect.insert_one(document, None).await?;
        info!("insert new tag: {}", tag);
        Ok(())
    }

    /// add_value是在数据库中values集合中，如果value不存在创建一个新的values文档，
    /// 并插入val值和tags的值，如果value已经存在values集合，则更新原来的tags列表值
    /// 同时返回以前tags中应该删除的tag项
    /// 参数是value： &str和tags: & Vec<String>
    async fn update_value(
        &self,
        value: &str,
        tags: &Vec<String>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let query = doc! {"value": value};

        let options = FindOneOptions::default();
        let result = self.values_collect.find_one(query, options).await.unwrap();

        if let Some(docu) = result {
            warn!("[!] value: {} already exists", value);
            info!("[+] merge old tags and new tags");

            // 获取原本tags值，寻找应该删除的项
            let tags_diff = docu
                .get_array("tags")
                .ok()
                .unwrap()
                .iter()
                .filter_map(|bson| {
                    if let Bson::String(string_value) = bson {
                        if !tags.contains(string_value) {
                            Some(string_value.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();

            // 更新value的tags值
            let update = doc! { "$set": {"tags": tags}};
            self.values_collect.update_one(docu, update, None).await?;
            return Ok(tags_diff);
        } else {
            let query = doc! {"value": value, "tags": tags};
            self.values_collect.insert_one(query, None).await?;
            info!("[+] insert new value: {}", value);
        }

        Ok(Vec::new())
    }

    /// update_tag是更新tags集合中的值，当插入新的值或者新的tag时，都可以调用此函数
    /// 此函数会创建新的tag文档插入tags集合中或者将值插入已有tag文档中中，
    /// 同时还会创建一个value文档插入values集合中
    pub async fn update_tag(
        &mut self,
        tags: &Vec<String>,
        val: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for tag in tags {
            if !self.tags.contains(tag) {
                self.add_tag(tag).await?;
                self.tags.push(tag.clone());
            }
        }

        // 更新values集合中的值信息，没有则插入新的
        let diff_tag = self.update_value(val, tags).await?;
        self.delete_value_in_tag(&diff_tag, val).await?;
        self.add_value_in_tags(tags, val).await?;

        Ok(())
    }

    /// delete_value_in_tag函数是删除tags集合中指定tag列表中的值val
    pub async fn delete_value_in_tag(
        &self,
        tags: &Vec<String>,
        val: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 定义要删除的值
        let value_to_remove = Bson::String(val.to_owned());

        // 使用 $pull 操作符从数组字段中删除特定值
        let filter = doc! {"tag": {"$in": tags}};
        let update = doc! {"$pull": {"value": value_to_remove}};
        let update_options = UpdateOptions::default();
        self.tags_collect
            .update_many(filter, update, update_options)
            .await
            .unwrap();
    
        // 删除空的tag文档
        for tag in tags {
            let filter = doc! {
                "tag": tag,
                "value": {
                    "$eq": Bson::Array(Vec::new())
                }
            };
        
            self.tags_collect
                .delete_one(filter, None)
                .await?;
        }

        info!("[+] delete the diff tags");
        Ok(())
    }

    /// find_value查找values集合中,大小写不敏感
    pub async fn find_value(&self, val: &str) -> Result<(), Box<dyn std::error::Error>> {
        let filter = doc! { "value": { "$exists": true } };
        let mut cursor = self.values_collect.find(filter, None).await?;

        while let Some(document) = cursor.try_next().await? {
            // 处理查询结果
            if let Some(value) = document.get_str("value").ok() {
                let low_a = value.to_lowercase();
                let low_b = val.to_lowercase();

                if let Some(begin) = low_a.find(&low_b) {
                    let end = begin + low_b.len();
                    println!(
                        "value: {}{}{}",
                        &low_a[0..begin],
                        low_b.red(),
                        &low_a[end..]
                    );
                    print!("tags: ");
                    let tags = document.get_array("tags").ok().unwrap();
                    for tag in tags {
                        print!("{}, ", tag.to_string().replace("\"", "").green());
                    }
                    println!();
                }
            }
        }

        Ok(())
    }

    fn doc2val(docu: Document) -> Value {
        let value = docu.get_str("value").ok().unwrap().to_string();
        let tags: Vec<String> = docu
            .get_array("tags")
            .ok()
            .unwrap()
            .iter()
            .map(|bson| bson.to_string())
            .collect();
        return Value { value, tags };
    }

    /// export函数是用来导出mongodb数据变成json格式
    /// 导出数据只导出values集合中的，tagss集合中的内容可以通过values集合中的内容推导出来
    pub async fn export(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 从数据库中提取值，组成Vec<Tag>
        let filter = doc! { "value": { "$exists": true } };
        let mut cursor = self.values_collect.find(filter, None).await?;
        let mut tags = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            tags.push(Db::doc2val(document));
        }

        // 转换成json格式字符串
        let json_str = serde_json::to_string(&tags)?;

        // 创建文件并写入内容
        let mut file = File::create(path).expect("Failed to create file");
        file.write_all(json_str.as_bytes())
            .expect("Failed to write to file");
        info!("[+] export tag file as json {} ", path);

        Ok(())
    }

    /// import函数是导入json到数据库中
    pub async fn import(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut str_json = String::new();
        let mut file = File::open(path).expect("Failed to open file");
        file.read_to_string(&mut str_json)
            .expect("Failed to read file");

        let values: Vec<Value> = serde_json::from_str(&str_json)?;

        for value in values {
            self.update_tag(&value.tags, &value.value).await?
        }

        Ok(())
    }
}
