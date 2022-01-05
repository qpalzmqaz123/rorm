use rorm::{Entity, Repository};
use rorm_test::run_async_test;

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(table_name = "user")]
struct User {
    #[rorm(primary_key, auto_increment)]
    pub id: u32,
    #[rorm(length = 20, default = "NONAME")]
    pub name: String,
    #[rorm(relation = id > user_id)]
    pub avatar: Avatar,
    #[rorm(relation = id > user_id)]
    pub address: Vec<Address>,
    #[rorm(relation = id > user_id)]
    pub extra: Option<Extra>,
}

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(index = [user_id])]
struct Avatar {
    #[rorm(primary_key)]
    pub id: u32,
    pub user_id: u32,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(index = [user_id])]
struct Address {
    #[rorm(primary_key)]
    pub id: u32,
    pub user_id: u32,
    pub city: String,
    pub street: String,
}

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(index = [user_id])]
struct Extra {
    #[rorm(primary_key)]
    pub id: u32,
    pub user_id: u32,
    pub height: u32,
}

#[tokio::test]
async fn test_normal() {
    run_async_test!((user_repo: Repository<User>, avatar_repo: Repository<Avatar>, _address_repo: Repository<Address>, _extra_repo: Repository<Extra>) => {
        let user_id = user_repo
            .insert(UserModel {
                name: "user1".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let avatar_id = avatar_repo
            .insert(AvatarModel {
                user_id: user_id.into(),
                url: "xxx".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let user = user_repo.find(user_id, None).await.unwrap();
        assert_eq!(
            user,
            User {
                id: 1,
                name: "user1".into(),
                avatar: Avatar {
                    id: avatar_id,
                    user_id: 1,
                    url: "xxx".into(),
                },
                address: vec![],
                extra: None,
            }
        );

        avatar_repo.delete(avatar_id).await.unwrap();

        let res = user_repo.find(user_id, None).await;
        assert!(res.is_err());
    });
}

#[tokio::test]
async fn test_option() {
    run_async_test!((user_repo: Repository<User>, avatar_repo: Repository<Avatar>, _address_repo: Repository<Address>, extra_repo: Repository<Extra>) => {
        let user_id = user_repo
            .insert(UserModel {
                name: "user1".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let avatar_id = avatar_repo
            .insert(AvatarModel {
                user_id: user_id.into(),
                url: "xxx".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let extra_id = extra_repo.insert(ExtraModel {
            user_id: user_id.into(),
            height: 100.into(),
            ..Default::default()
        }).await.unwrap();

        let user = user_repo.find(user_id, None).await.unwrap();
        assert_eq!(
            user,
            User {
                id: 1,
                name: "user1".into(),
                avatar: Avatar {
                    id: avatar_id,
                    user_id: 1,
                    url: "xxx".into(),
                },
                address: vec![],
                extra: Some(Extra {
                    id: extra_id,
                    user_id,
                    height: 100,
                }),
            }
        );

        extra_repo.delete(extra_id).await.unwrap();

        let user = user_repo.find(user_id, None).await.unwrap();
        assert_eq!(
            user,
            User {
                id: 1,
                name: "user1".into(),
                avatar: Avatar {
                    id: avatar_id,
                    user_id: 1,
                    url: "xxx".into(),
                },
                address: vec![],
                extra: None,
            }
        );
    });
}

#[tokio::test]
async fn test_vec() {
    run_async_test!((user_repo: Repository<User>, avatar_repo: Repository<Avatar>, address_repo: Repository<Address>, _extra_repo: Repository<Extra>) => {
        let user_id = user_repo
            .insert(UserModel {
                name: "user1".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let avatar_id = avatar_repo
            .insert(AvatarModel {
                user_id: user_id.into(),
                url: "xxx".into(),
                ..Default::default()
            })
            .await
            .unwrap();

        let address_ids = address_repo.insert_many([
            AddressModel {
                user_id: user_id.into(),
                city: "a".into(),
                street: "a".into(),
                ..Default::default()
            },
            AddressModel {
                user_id: user_id.into(),
                city: "b".into(),
                street: "b".into(),
                ..Default::default()
            },
        ]).await.unwrap();

        let user = user_repo.find(user_id, None).await.unwrap();
        assert_eq!(
            user,
            User {
                id: 1,
                name: "user1".into(),
                avatar: Avatar {
                    id: avatar_id,
                    user_id: 1,
                    url: "xxx".into(),
                },
                address: vec![
                    Address {
                        id: address_ids[0],
                        user_id,
                        city: "a".into(),
                        street: "a".into(),
                    },
                    Address {
                        id: address_ids[1],
                        user_id,
                        city: "b".into(),
                        street: "b".into(),
                    },
                ],
                extra: None,
            }
        );

        address_repo.delete(address_ids[0]).await.unwrap();

        let user = user_repo.find(user_id, None).await.unwrap();
        assert_eq!(
            user,
            User {
                id: 1,
                name: "user1".into(),
                avatar: Avatar {
                    id: avatar_id,
                    user_id: 1,
                    url: "xxx".into(),
                },
                address: vec![
                    Address {
                        id: address_ids[1],
                        user_id,
                        city: "b".into(),
                        street: "b".into(),
                    },
                ],
                extra: None,
            }
        );

        address_repo.delete(address_ids[1]).await.unwrap();

        let user = user_repo.find(user_id, None).await.unwrap();
        assert_eq!(
            user,
            User {
                id: 1,
                name: "user1".into(),
                avatar: Avatar {
                    id: avatar_id,
                    user_id: 1,
                    url: "xxx".into(),
                },
                address: vec![],
                extra: None,
            }
        );
    });
}
