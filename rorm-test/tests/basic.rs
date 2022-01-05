use rorm::{Entity, FindOption, Repository};
use rorm_test::run_async_test;

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(table_name = "user")]
#[rorm(index = [name])]
#[rorm(index = [email, address])]
struct User {
    #[rorm(primary_key, auto_increment)]
    pub id: u32,
    #[rorm(length = 20, default = "NONAME", unique)]
    pub name: String,
    pub email: Option<String>,
    #[rorm(sql_type = String, length = 100)]
    pub address: Address,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Address {
    pub city: String,
    pub street: String,
}

impl Address {
    fn new(city: &str, street: &str) -> Self {
        Self {
            city: city.into(),
            street: street.into(),
        }
    }
}

impl rorm::pool::ToValue for Address {
    fn to_value(&self) -> rorm::pool::Value {
        rorm::pool::Value::Str(format!("{}#{}", self.city, self.street))
    }
}

impl rorm::pool::FromValue for Address {
    type Output = Address;

    fn from_value(v: &rorm::pool::Value) -> rorm::error::Result<Self::Output> {
        if let rorm::pool::Value::Str(s) = v {
            let mut arr = s.split("#");
            let city = arr
                .next()
                .ok_or(rorm::error::from_value!("Invalid address string"))?;
            let street = arr
                .next()
                .ok_or(rorm::error::from_value!("Invalid address string"))?;

            return Ok(Self {
                city: city.into(),
                street: street.into(),
            });
        }

        Err(rorm::error::from_value!("Address type must be string"))
    }
}

#[tokio::test]
async fn test_info() {
    assert_eq!(
        User::INFO,
        rorm::TableInfo {
            name: "user",
            columns: &[
                rorm::ColumnInfo {
                    name: "id",
                    ty: rorm::ColumnType::U32,
                    is_primary_key: true,
                    is_not_null: true,
                    is_auto_increment: true,
                    default: None,
                    is_unique: false,
                },
                rorm::ColumnInfo {
                    name: "name",
                    ty: rorm::ColumnType::Str(20),
                    is_primary_key: false,
                    is_not_null: true,
                    is_auto_increment: false,
                    default: Some("'NONAME'"),
                    is_unique: true,
                },
                rorm::ColumnInfo {
                    name: "email",
                    ty: rorm::ColumnType::Str(65535),
                    is_primary_key: false,
                    is_not_null: false,
                    is_auto_increment: false,
                    default: None,
                    is_unique: false,
                },
                rorm::ColumnInfo {
                    name: "address",
                    ty: rorm::ColumnType::Str(100),
                    is_primary_key: false,
                    is_not_null: true,
                    is_auto_increment: false,
                    default: None,
                    is_unique: false,
                },
            ],
            indexes: &[
                rorm::IndexInfo {
                    name: "user_index_name",
                    keys: &[rorm::IndexKeyInfo {
                        column_name: "name",
                    }],
                },
                rorm::IndexInfo {
                    name: "user_index_email_address",
                    keys: &[
                        rorm::IndexKeyInfo {
                            column_name: "email",
                        },
                        rorm::IndexKeyInfo {
                            column_name: "address",
                        }
                    ]
                }
            ],
        }
    );
}

#[tokio::test]
async fn test_unique() {
    run_async_test!((repo: Repository<User>) => {
        // First insert
        repo.insert(UserModel {
            name: "bob".into(),
            address: Address::new("a", "b").into(),
            ..Default::default()
        })
        .await
        .unwrap();

        // Second insert
        let res = repo
            .insert(UserModel {
                name: "bob".into(),
                address: Address::new("a", "b").into(),
                ..Default::default()
            })
            .await;

        // Assert error
        assert!(res.is_err());
    });
}

#[tokio::test]
async fn test_default() {
    run_async_test!((repo: Repository<User>) => {
        // Insert without name
        let id = repo
            .insert(UserModel {
                address: Address::new("a", "b").into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let user = repo.find(id, None).await.unwrap();

        assert_eq!(
            user,
            User {
                id,
                name: "NONAME".into(),
                email: None,
                address: Address::new("a", "b"),
            }
        );
    });
}

#[tokio::test]
async fn test_option() {
    run_async_test!((repo: Repository<User>) => {
        // Insert without name
        let id = repo
            .insert(UserModel {
                address: Address::new("a", "b").into(),
                email: Some("abc").into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let user = repo.find(id, None).await.unwrap();

        assert_eq!(
            user,
            User {
                id,
                name: "NONAME".into(),
                email: Some("abc".into()),
                address: Address::new("a", "b"),
            }
        );
    });
}

#[tokio::test]
async fn test_insert_many() {
    run_async_test!((repo: Repository<User>) => {
        let users = [
            UserModel {
                name: "a".into(),
                address: Address::new("a", "b").into(),
                ..Default::default()
            },
            UserModel {
                name: "b".into(),
                address: Address::new("a", "b").into(),
                ..Default::default()
            },
        ];

        let ids = repo.insert_many(users).await.unwrap();

        assert_eq!(ids, vec![1, 2]);

        let find_users = repo
            .find_many(
                UserModel::default(),
                Some(FindOption {
                    orders: vec![("id".into(), false)],
                    ..Default::default()
                }),
            )
            .await
            .unwrap();

        assert_eq!(
            find_users,
            vec![
                User {
                    id: 2,
                    name: "b".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
                User {
                    id: 1,
                    name: "a".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
            ]
        )
    });
}
