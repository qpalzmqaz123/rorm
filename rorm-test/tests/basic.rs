use rorm::{Entity, Repository};
use rorm_test::run_async_test;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Entity)]
#[rorm(table_name = "user")]
#[rorm(index = [name])]
#[rorm(index = [email, address])]
struct User {
    #[rorm(primary_key, auto_increment)]
    pub id: u32,
    #[serde(rename = "user_name")]
    #[rorm(length = 20, default = "NONAME", unique)]
    pub name: String,
    pub email: Option<String>,
    #[rorm(serde = serde_json, length = 100)]
    pub address: Address,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
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
                    flatten_ref: None,
                },
                rorm::ColumnInfo {
                    name: "name",
                    ty: rorm::ColumnType::Str(20),
                    is_primary_key: false,
                    is_not_null: true,
                    is_auto_increment: false,
                    default: Some("'NONAME'"),
                    is_unique: true,
                    flatten_ref: None,
                },
                rorm::ColumnInfo {
                    name: "email",
                    ty: rorm::ColumnType::Str(65535),
                    is_primary_key: false,
                    is_not_null: false,
                    is_auto_increment: false,
                    default: None,
                    is_unique: false,
                    flatten_ref: None,
                },
                rorm::ColumnInfo {
                    name: "address",
                    ty: rorm::ColumnType::Str(100),
                    is_primary_key: false,
                    is_not_null: true,
                    is_auto_increment: false,
                    default: None,
                    is_unique: false,
                    flatten_ref: None,
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
        repo.insert().model(UserModel {
            name: "bob".into(),
            address: Address::new("a", "b").into(),
            ..Default::default()
        })
        .one()
        .await
        .unwrap();

        // Second insert
        let res = repo
            .insert().model(UserModel {
                name: "bob".into(),
                address: Address::new("a", "b").into(),
                ..Default::default()
            })
            .one()
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
            .insert().model(UserModel {
                address: Address::new("a", "b").into(),
                ..Default::default()
            })
            .one()
            .await
            .unwrap();

        let user = repo.find().filter_model(id).one().await.unwrap();

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
            .insert().model(UserModel {
                address: Address::new("a", "b").into(),
                email: Some("abc").into(),
                ..Default::default()
            })
            .one()
            .await
            .unwrap();

        let user = repo.find().filter_model(id).one().await.unwrap();

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

        let ids = repo.insert().models(users).all().await.unwrap();

        assert_eq!(ids, vec![1, 2]);

        let find_users = repo
            .find()
            .order_by("id", false)
            .all()
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

#[tokio::test]
async fn test_delete() {
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
            UserModel {
                name: "c".into(),
                address: Address::new("a", "b").into(),
                ..Default::default()
            },
            UserModel {
                name: "d".into(),
                address: Address::new("a", "b").into(),
                ..Default::default()
            },
        ];

        repo.insert().models(users).all().await.unwrap();

        // Delete one
        repo.delete().filter_model(UserModel {name: "a".into(), ..Default::default()}).one().await.unwrap();

        assert_eq!(
            repo.find().all().await.unwrap(),
            vec![
                User {
                    id: 2,
                    name: "b".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
                User {
                    id: 3,
                    name: "c".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
                User {
                    id: 4,
                    name: "d".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
            ]
        );

        // Delete limit
        repo.delete().order_by("id", false).limit(1, 0).await.unwrap();

        assert_eq!(
            repo.find().all().await.unwrap(),
            vec![
                User {
                    id: 2,
                    name: "b".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
                User {
                    id: 3,
                    name: "c".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
            ]
        );

        // Delete all
        repo.delete().all().await.unwrap();

        assert_eq!(
            repo.find().all().await.unwrap(),
            vec![]
        );
    });
}

#[tokio::test]
async fn test_update() {
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
            UserModel {
                name: "c".into(),
                address: Address::new("a", "b").into(),
                ..Default::default()
            },
            UserModel {
                name: "d".into(),
                address: Address::new("a", "b").into(),
                ..Default::default()
            },
        ];

        repo.insert().models(users).all().await.unwrap();

        // Update one
        repo
            .update()
            .set_model(UserModel {address: Address::new("c", "d").into(), ..Default::default()})
            .filter_model(UserModel {name: "a".into(), ..Default::default()})
            .one().await.unwrap();

        assert_eq!(
            repo.find().all().await.unwrap(),
            vec![
                User {
                    id: 1,
                    name: "a".into(),
                    email: None,
                    address: Address::new("c", "d"),
                },
                User {
                    id: 2,
                    name: "b".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
                User {
                    id: 3,
                    name: "c".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
                User {
                    id: 4,
                    name: "d".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
            ]
        );

        // Update limit
        repo
            .update()
            .set_model(UserModel {address: Address::new("c", "d").into(), ..Default::default()})
            .order_by("id", false)
            .limit(1, 1).await.unwrap();

        assert_eq!(
            repo.find().all().await.unwrap(),
            vec![
                User {
                    id: 1,
                    name: "a".into(),
                    email: None,
                    address: Address::new("c", "d"),
                },
                User {
                    id: 2,
                    name: "b".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
                User {
                    id: 3,
                    name: "c".into(),
                    email: None,
                    address: Address::new("c", "d"),
                },
                User {
                    id: 4,
                    name: "d".into(),
                    email: None,
                    address: Address::new("a", "b"),
                },
            ]
        );

        // Update all
        repo
            .update()
            .set_model(UserModel {address: Address::new("e", "f").into(), ..Default::default()})
            .all().await.unwrap();

        assert_eq!(
            repo.find().all().await.unwrap(),
            vec![
                User {
                    id: 1,
                    name: "a".into(),
                    email: None,
                    address: Address::new("e", "f"),
                },
                User {
                    id: 2,
                    name: "b".into(),
                    email: None,
                    address: Address::new("e", "f"),
                },
                User {
                    id: 3,
                    name: "c".into(),
                    email: None,
                    address: Address::new("e", "f"),
                },
                User {
                    id: 4,
                    name: "d".into(),
                    email: None,
                    address: Address::new("e", "f"),
                },
            ]
        );
    });
}
