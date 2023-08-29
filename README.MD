# rtag

一个tag管理的系统，为了方便使用tag查找文件.使用tag记录对应的值，找个值是文件路径/单纯的值.

使用mongodb数据库存储数据，rust编程

## 数据库

以tag为主来存储，每个tag中有一个列表，列表中是对应所有这个tag的值

### 集合tags

存在多个文档，一个文档代表一个tag，每个文档中的值如下

```json
{
  "_id": {
    "$oid": "64e714b28054de22d73432e0"
  },
  "tag": "test", // tag名称
  "value": [     // 包含此tag的值
    "text0",
    "text1"
  ]
}
```

### 集合values

以values为主的集合，一个values一个文档

```json
{
    "_id": {
    "$oid": "64e714b28054de22d73432e0"
  },
  "value": "test", // 值
  "tag": [         // tag
    "v1",
    "v2"
  ]
}
```


## 命令行参数

+ -v/--value: 目标value
+ -t/--tag: 目标tag

value + tag : 插入值到对应tag中
value: 所有包含字符串的值以及对应的tag
tag: 展示有一个tag或者多个tag的值