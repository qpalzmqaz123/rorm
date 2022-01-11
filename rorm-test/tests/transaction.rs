use rorm::{Entity, Repository};
use rorm_test::run_async_test;

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(table_name = "user")]
struct User {
    #[rorm(primary_key, auto_increment)]
    pub id: u32,
    #[rorm(length = 20, default = "NONAME", unique)]
    pub name: String,
}

fn user_model(name: &str) -> UserModel {
    UserModel {
        name: name.into(),
        ..Default::default()
    }
}

#[tokio::test]
async fn test_unique() {
    run_async_test!((repo: Repository<User>) => {
        let mut tx = repo.conn.transaction();
        let mut tx_repo = tx.repository::<User>();

        tx_repo.insert().model(user_model("bob")).one().await.unwrap();
        tx_repo.insert().model(user_model("alice")).one().await.unwrap();
        tx_repo.delete().filter_model(user_model("bob")).one().await.unwrap();
        tx_repo.update().filter_model(user_model("alice")).set_model(user_model("frank")).one().await.unwrap();

        tx.commit().await.unwrap();

        assert_eq!(
            repo.find().all().await.unwrap(),
            vec![
                User {
                    id: 2,
                    name: "frank".into(),
                },
            ]
        );
    });
}
