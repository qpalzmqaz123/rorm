use rorm::{Connection, Entity};

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(table_name = "user")]
#[rorm(index = [id])]
struct User {
    pub id: String,
    pub name: String,
}

impl From<&str> for UserModel {
    fn from(id: &str) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let connection = Connection::connect("fcss://127.0.0.1:50051").await?;
    let user_repo = connection.repository::<User>();

    // Init table
    user_repo.init().await?;

    // Delete all
    user_repo.delete().all().await?;

    // Check table name
    assert_eq!("user", User::INFO.name);

    // Insert bob
    let bob = UserModel {
        id: "b".into(),
        name: "bob".into(),
    };
    user_repo.insert().model(bob).one().await?;

    // Insert alice
    let alice = UserModel {
        id: "a".into(),
        name: "alice".into(),
    };
    user_repo.insert().model(alice).one().await?;

    // Find bob by id
    let bob = user_repo.find().filter_model("b").one().await?;
    assert_eq!(
        bob,
        User {
            id: "b".into(),
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
            id: "a".into(),
            name: "alice".into(),
        }
    );

    // Find list
    let list = user_repo.find().all().await?;
    assert_eq!(
        list,
        vec![
            User {
                id: "a".into(),
                name: "alice".into(),
            },
            User {
                id: "b".into(),
                name: "bob".into()
            },
        ]
    );

    // Delete bob
    user_repo.delete().filter_model("b").all().await?;
    let list = user_repo.find().all().await?;
    assert_eq!(
        list,
        vec![User {
            id: "a".into(),
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
        .filter_model("a")
        .all()
        .await?;
    let list = user_repo.find().all().await?;
    assert_eq!(
        list,
        vec![User {
            id: "a".into(),
            name: "alex".into(),
        }]
    );

    // Insert many
    let carl = UserModel {
        id: "c".into(),
        name: "carl".into(),
    };
    let lee = UserModel {
        id: "l".into(),
        name: "lee".into(),
    };
    user_repo.insert().models([carl, lee]).all().await?;
    let list = user_repo.find().all().await?;
    assert_eq!(
        list,
        vec![
            User {
                id: "a".into(),
                name: "alex".into(),
            },
            User {
                id: "c".into(),
                name: "carl".into(),
            },
            User {
                id: "l".into(),
                name: "lee".into(),
            }
        ]
    );

    // Find limit 2
    let list = user_repo.find().limit(2, 1).await?;
    assert_eq!(
        list,
        vec![
            User {
                id: "c".into(),
                name: "carl".into(),
            },
            User {
                id: "l".into(),
                name: "lee".into(),
            },
        ]
    );

    Ok(())
}
