use rorm::{Entity, Repository};

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(table_name = "user")]
#[rorm(index = [id, name])]
struct User {
    #[rorm(primary_key, auto_increment)]
    pub id: u32,
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rorm::{FindOption, ModelColumn::Set};

    env_logger::init();

    let connection = rorm::pool::sqlite::Builder::memory().connect()?;
    let user_repo = Repository::<User>::new(connection.clone());

    // Init table
    user_repo.init().await?;

    // Check table name
    assert_eq!("user", User::TABLE_NAME);

    // Insert bob
    let bob = UserModel {
        name: Set("bob".into()),
        ..Default::default()
    };
    let bob_id = user_repo.insert(bob).await?;
    assert_eq!(bob_id, 1);

    // Insert alice
    let alice = UserModel {
        name: Set("alice".into()),
        ..Default::default()
    };
    let alice_id = user_repo.insert(alice).await?;
    assert_eq!(alice_id, 2);

    // Find bob by id
    let bob = user_repo.find(1, None).await?;
    assert_eq!(
        bob,
        User {
            id: 1,
            name: "bob".into()
        }
    );

    // Find alice by name
    let alice = user_repo
        .find(
            UserModel {
                name: Set("alice".into()),
                ..Default::default()
            },
            None,
        )
        .await?;
    assert_eq!(
        alice,
        User {
            id: 2,
            name: "alice".into(),
        }
    );

    // Find list
    let list = user_repo.find_many(UserModel::default(), None).await?;
    assert_eq!(
        list,
        vec![
            User {
                id: 1,
                name: "bob".into()
            },
            User {
                id: 2,
                name: "alice".into(),
            }
        ]
    );

    // Delete bob
    user_repo.delete(1).await?;
    let list = user_repo.find_many(UserModel::default(), None).await?;
    assert_eq!(
        list,
        vec![User {
            id: 2,
            name: "alice".into(),
        }]
    );

    // Update alice
    user_repo
        .update(
            2,
            UserModel {
                name: Set("alex".into()),
                ..Default::default()
            },
        )
        .await?;
    let list = user_repo.find_many(UserModel::default(), None).await?;
    assert_eq!(
        list,
        vec![User {
            id: 2,
            name: "alex".into(),
        }]
    );

    // Insert many
    let carl = UserModel {
        name: Set("carl".into()),
        ..Default::default()
    };
    let lee = UserModel {
        name: Set("lee".into()),
        ..Default::default()
    };
    user_repo.insert_many([carl, lee]).await?;
    let list = user_repo.find_many(UserModel::default(), None).await?;
    assert_eq!(
        list,
        vec![
            User {
                id: 2,
                name: "alex".into(),
            },
            User {
                id: 3,
                name: "carl".into(),
            },
            User {
                id: 4,
                name: "lee".into(),
            }
        ]
    );

    // Find order by id desc limit 2
    let list = user_repo
        .find_many(
            UserModel::default(),
            Some(FindOption {
                orders: vec![("id".into(), false)],
                limit: Some((2, 0)),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        list,
        vec![
            User {
                id: 4,
                name: "lee".into(),
            },
            User {
                id: 3,
                name: "carl".into(),
            },
        ]
    );

    Ok(())
}
