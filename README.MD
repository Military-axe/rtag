# rtag

一个tag管理的系统，为了方便使用tag查找文件.使用tag记录对应的值，找个值是文件路径/单纯的值.

使用mongodb数据库存储数据，rust编程

## 数据库

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

### 添加值和对应的tag

将`babyre`打上`rc4`,`base64`两个不同的tag

```sh
rtag -t rc4 base64 -v babyre
```

### 查看tag下所有的值

查看有`rc4`,`base64`两个tag的值

```sh
rtag -t rc4 base64
```

### 模糊搜索值以及对应tag

搜索包含`re`两个字符的值以及对应的tag

```sh
rtag -v re
```