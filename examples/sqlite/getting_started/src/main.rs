use rorm::{Connection, Entity, Repository};

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
    env_logger::init();

    let connection = Connection::connect("sqlite://memory").await?;
    let user_repo = Repository::<User>::new(connection.clone());

    // Init table
    user_repo.init().await?;

    // Check table name
    assert_eq!("user", User::INFO.name);

    // Insert bob
    let bob = UserModel {
        name: "bob".into(),
        ..Default::default()
    };
    let bob_id = user_repo.insert().model(bob).one().await?;
    assert_eq!(bob_id, 1);

    // Insert alice
    let alice = UserModel {
        name: "alice".into(),
        ..Default::default()
    };
    let alice_id = user_repo.insert().model(alice).one().await?;
    assert_eq!(alice_id, 2);

    // Find bob by id
    let bob = user_repo.find().filter_model(1).one().await?;
    assert_eq!(
        bob,
        User {
            id: 1,
            name: "bob".into()
        }
    );

    // Find alice by name
    let alice = user_repo
        .find()
        .filter_model(UserModel {
            name: "alice".into(),
            ..Default::default()
        })
        .one()
        .await?;
    assert_eq!(
        alice,
        User {
            id: 2,
            name: "alice".into(),
        }
    );

    // Find list
    let list = user_repo.find().all().await?;
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
    user_repo.delete().filter_model(1).all().await?;
    let list = user_repo.find().all().await?;
    assert_eq!(
        list,
        vec![User {
            id: 2,
            name: "alice".into(),
        }]
    );

    // Update alice
    user_repo
        .update()
        .set_model(UserModel {
            name: "alex".into(),
            ..Default::default()
        })
        .filter_model(2)
        .all()
        .await?;
    let list = user_repo.find().all().await?;
    assert_eq!(
        list,
        vec![User {
            id: 2,
            name: "alex".into(),
        }]
    );

    // Insert many
    let carl = UserModel {
        name: "carl".into(),
        ..Default::default()
    };
    let lee = UserModel {
        name: "lee".into(),
        ..Default::default()
    };
    user_repo.insert().models([carl, lee]).all().await?;
    let list = user_repo.find().all().await?;
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
    let list = user_repo.find().order_by("id", false).limit(2, 1).await?;
    assert_eq!(
        list,
        vec![
            User {
                id: 3,
                name: "carl".into(),
            },
            User {
                id: 2,
                name: "alex".into(),
            },
        ]
    );

    Ok(())
}
