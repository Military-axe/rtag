use futures::stream::TryStreamExt;
use log::{error, info};
use mongodb::bson::{doc, Document};
use mongodb::{options::ClientOptions, options::UpdateOptions, Client, Collection, Database};
use std::collections::HashMap;

pub struct Db {
    pub client: Client,
    pub db: Database,
    pub tags_collect: Collection<Document>,
    pub values_collect: Collection<Document>,
    pub tags: Vec<String>
}

impl Db {
    // init函数创建数据库与集合
    pub async fn init(addr: &str, app_name: String) -> Result<(), Box<dyn std::error::Error>> {
        // 创建 MongoDB 的客户端连接
        let mut client_options = ClientOptions::parse(addr).await?;
        client_options.app_name = Some(app_name);
        let client = Client::with_options(client_options)?;

        // 获取要创建的数据库和集合名称
        // TODO: 后期可以修改成toml文件读取
        let database_name = "rtag";
        let collection_name = "tags";
        let collection_name2 = "values";

        // 创建数据库
        let database = client.database(database_name);

        // 创建集合（如果不存在）
        database.create_collection(collection_name, None).await?;
        database.create_collection(collection_name2, None).await?;

        info!("Database and collection created successfully.");

        Ok(())
    }

    /// new函数连接mongodb数据库并返回Result<DataBase, Box<dyn std::error::Error>>
    /// DataBase中存储了client是和数据库的连接，通过此连接来读写数据库.
    /// 函数有两个参数，addr和app_name
    /// addr: &str;是连接数据库的uri地址，默认是"mongodb://localhost:27017"
    /// app_name: String;是数据库日志记录过程中的一个表示，方便调试
    pub async fn new(addr: &str, app_name: String) -> Result<Db, Box<dyn std::error::Error>> {
        let mut client_options = ClientOptions::parse(addr).await?;
        client_options.app_name = Some(app_name);

        // 建立与MongoDB的连接
        let c = match Client::with_options(client_options) {
            Err(x) => {
                error!("connect mongodb failed; error: {}", x);
                panic!("connect mongodb failed");
            }
            Ok(x) => x,
        };

        // 连接数据库，数据库名这是暂定是rtag_data.集合名为test
        // TODO: 数据库名，集合名传参
        let db = c.database("rtag_data");
        let tags_collection: mongodb::Collection<mongodb::bson::Document> = db.collection("tags");
        let values_collection = db.collection("values");
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
    pub async fn find_tags(collect: &Collection<Document>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let filter = doc! { "tag": { "$exists": true } };
        let mut cursor =  collect.find(filter, None).await?;
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
    /// TODO: 优化最后的打印部分
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
        for (ele, count) in hashmap.iter() {
            println!("Value: {}, Count: {}", ele, count);
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
                "$push": {
                    "value": val
                }
            };

            let options = UpdateOptions::builder().upsert(false).build();
            let query = doc! {"tag": tag};
            self.tags_collect.update_one(query, update_doc, options).await?;
        }

        Ok(())
    }

    /// add_tag是数据库中tags集合中创建一个新的tag文档
    async fn add_tag(&self, tag: &str) -> Result<(), Box<dyn std::error::Error>> {
        let document = doc! {"tag": tag, "value": []};
        self.tags_collect.insert_one(document, None).await?;
        info!("insert new tag: {}", tag);
        Ok(())
    }

    /// add_value是在数据库中values集合中，创建一个新的values文档，并插入
    /// val值和tags的值
    async fn add_value(&self, value: &str, tags: &Vec<String>,) -> Result<(), Box<dyn std::error::Error>> {
        let document = doc! {"value": value, "tags": tags};
        self.tags_collect.insert_one(document, None).await?;
        info!("insert new value: {}", value);
        Ok(())
    }

    /// update_tag是更新tags集合中的值，当插入新的值或者新的tag时，都可以调用此函数
    /// 此函数会创建新的tag或者将值插入已有tag中
    pub async fn update_tag(
        &mut self,
        tags: &Vec<String>,
        val: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for tag in tags{
            if !self.tags.contains(tag){
                self.add_tag(tag).await?;
                self.tags.push(tag.clone());
            }
        }

        // 没有对比values文档，直接创建新的values文档，默认
        // 插入新的值就是之前没有过的值
        self.add_value(val, tags).await?;

        self.add_value_in_tags(tags, val).await?;

        Ok(())
    }

    /// find_value查找values集合中
    /// TODO: 继续开发
    pub async fn find_value(&self, val: &str) -> Result<(), Box<dyn std::error::Error>> {
        let filter = doc! { "tag": { "$exists": true } };
        let mut cursor =  self.values_collect.find(filter, None).await?;

        Ok(())
    }

}
