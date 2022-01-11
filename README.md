# RORM

## 特性

1. 支持异步
2. 支持 sql-builder，动态生成 sql
3. 支持关联，将列与表关联起来，查询时自动填充关联的列

## 入门

### 定义表

```rust
use rorm::Entity;

#[derive(Entity)]
struct User {
    #[rorm(primary_key, auto_increment)]
    pub id: u32,
    pub name: String,
    pub nickname: Option<String>,
}
```

### 初始化

#### 创建连接

根据不同数据库创建 sqlite, mysql (预留), postgresql (预留) 连接

```rust
let connection = rorm::Connection::connect("sqlite://memory")?;
```

#### 创建 repository

使用 repository 便于管理，虽然也可以不用 repository，直接调用结构体的方法，但这样每次调用都需要传入连接

```rust
let user_repo = connection.repository::<User>();
```

#### 初始化表

通过 init 方法，可以自动创建对应的表（若表已存在则会忽略）

```rust
user_repo.init().await?;
```

### 插入

由于插入修改等操作时可以只操作部分列，如果使用原始的结构体作为数据，则必须填充所有字段，与实际应用不符，所以针对所有表定义的结构体，会生成名为 Model 后缀的新结构体，作为调用时的参数。例如，User 会生成 UserModel 的结构体

同时，如果表中定义了 primary_key，则会自动生成 primary_key 类型到 Model 类型的转换代码，所以 primary_key 类型的数据可以直接传入。当然，表到 Model 的转换也自动生成了。例如，对于上述 User 定义，自动实现了 u32 和 User 到 UserModel 的转换

生成的 Model 内部字段名称与原始结构体相同（关联字段除外），但是每个字段都是 ModelColumn 枚举，分为 Set 与 NotSet，用于表示操作时是否设置相应列，不想设置的在最后加上 `..Default::default()` 即可

另外，ModelColumn  实现了所有数据到自身 Set 的转换，所以设置时不需要使用 Set 声明，直接用 into() 即可

```rust
let bob = UserModel {
    name: "bob".into(),
    ..Default::default()
};

let bob_id = user_repo.insert().model(bob).one().await?;
```

### 更新

```rust
let new_bob = UserModel {
    nickname: Some("bbb").into(),
    ..Default::default()
};

user_repo.update().set_model(new_bob).filter_model(bob_id).one().await?;
```

### 查找

```rust
let bob = user_repo.find().filter_model(bob_id).one().await?;

let users = user_repo.find().order_by("id").limit(10, 0).all().await?; // limit 10, offset 0
```

### 删除

```rust
user_repo.delete(bob_id).await?;

user_repo.delete().filter_model(UserModel {
    nickname: Some("bbb").into(),
    ..Default::default()
}).all().await?;
```

## 事务

使用 connection 可创建事务，目前事务为纯上层实现，所以插入时获取不到 id

```rust
let mut tx = connection.transaction();
let mut tx_user_repo = tx.repository::<User>();

tx_user_repo.delete().filter_model(1).all().await?;
tx_user_repo.delete().filter_model(2).all().await?;

tx.commit().await?;
```

## 宏

宏里面可以定义表相关信息，格式为 `#[rorm(key [= value], ...)]`

支持字段如下：

1. primary_key

   设置列尾主键

2. auto_increment

   设置自增列

3. unique

   列为唯一

4. flatten

   类似于 serde(flatten)，将对应结构体的所有列展开到当前表

5. table_name = "NAME"

   重命名表

6. length = NUMBER

   定义字段长度，主要用于 mysql 与 psql 中，例如对于 String 类型若长度为 30，则表中类型定义为 VARCHAR(30)。如果不填，默认 String 长度为 65535

7. serde = (serde_json | serde_bson | serde_yaml | ...)

   设置列的序列化方式，暂时只支持 serde_json，没设置 serde 时会使用列的 ToValue / FromValue trait 将 rust 类型与 sql 类型互转，设置 serde 后会使用 serde 对列序列化和反序列化

8. index = [col1, col2, ...]

   定义索引

9. default = (NUMBER | STR)

   设置字段默认值

10. relation = SELF_COLUMN > REFER_COLUMN

    声明列的关联性，用于表一对多的情况，SELF_COLUMN 为自身结构体中的字段名称，REFER_COLUMN 为关联的结构体中字段名称，例如，对于如下两张表：

    ```sql
    CREATE TABLE User {
        id INTEGER,
        name VARCHAR,
    }
    
    CREATE TABLE Address {
        id INTEGER,
        user_id INTEGER,
        city VARCHAR,
        street VARCHAR,
    }
    ```

    其中 user 可以拥有多个地址，在 rust 中这样定义：

    ```rust
    #[derive(Entity)]
    struct User {
        pub id: u32,
        pub name: String,
        #[rorm(relation = id > user_id)] // 将自己的 id 与 address 的 user_id 关联
        pub addresses: Vec<Address>,
    }
    
    #[derive(Entity)]
    struct Address {
        pub id: u32,
        pub user_id: u32,
        pub city: String,
        pub street: String,
    }
    ```

    在数据库中，user 实际没有 addresses 字段，但只要定义 relation 后，在 rust 中查询 user 时，会自动去 address 表中查询匹配 user_id 的数据，并填充到 addresses  中一并返回。
