use rorm::{Entity, Repository};
use rorm_test::run_async_test;

#[derive(Debug, Entity, PartialEq, Eq)]
struct User {
    #[rorm(primary_key, auto_increment)]
    pub id: u32,
    #[rorm(flatten)]
    pub info: Info,
}

#[derive(Debug, Entity, PartialEq, Eq)]
struct Info {
    pub name: String,
    pub age: u8,
}

#[tokio::test]
async fn test_1() {
    run_async_test!((repo: Repository<User>) => {
        let bob = UserModel {
            info: InfoModel {
                name: "bob".into(),
                age: 20.into(),
                ..Default::default()
            }.into(),
            ..Default::default()
        };

        let bob_id = repo.insert().model(bob).one().await.unwrap();

        let bob = repo.find().filter_model(bob_id).one().await.unwrap();

        assert_eq!(
            bob,
            User {
                id: bob_id,
                info: Info {
                    name: "bob".into(),
                    age: 20,
                }
            }
        );
    });
}
