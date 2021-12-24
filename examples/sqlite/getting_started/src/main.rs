#[derive(Debug, rorm::Entity, PartialEq, Eq)]
struct User {
    #[rorm(primary_key)]
    pub id: u32,
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rorm::{FindOption, ModelColumn::Set};

    env_logger::init();

    let conn = rorm::pool::sqlite::Builder::memory().connect()?;

    // Create table
    conn.execute_one(
        "CREATE TABLE user (id INTEGER PRIMARY KEY AUTOINCREMENT, name VARCHAR NOT NULL)",
        vec![],
    )
    .await?;

    // Insert bob
    let bob = UserModel {
        name: Set("bob".into()),
        ..Default::default()
    };
    let bob_id = User::insert(bob, &conn).await?;
    assert_eq!(bob_id, 1);

    // Insert alice
    let alice = UserModel {
        name: Set("alice".into()),
        ..Default::default()
    };
    let alice_id = User::insert(alice, &conn).await?;
    assert_eq!(alice_id, 2);

    // Find bob by id
    let bob = User::find(1, None, &conn).await?;
    assert_eq!(
        bob,
        User {
            id: 1,
            name: "bob".into()
        }
    );

    // Find alice by name
    let alice = User::find(
        UserModel {
            name: Set("alice".into()),
            ..Default::default()
        },
        None,
        &conn,
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
    let list = User::find_many(UserModel::default(), None, &conn).await?;
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
    User::delete(1, &conn).await?;
    let list = User::find_many(UserModel::default(), None, &conn).await?;
    assert_eq!(
        list,
        vec![User {
            id: 2,
            name: "alice".into(),
        }]
    );

    // Update alice
    User::update(
        2,
        UserModel {
            name: Set("alex".into()),
            ..Default::default()
        },
        &conn,
    )
    .await?;
    let list = User::find_many(UserModel::default(), None, &conn).await?;
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
    User::insert_many([carl, lee], &conn).await?;
    let list = User::find_many(UserModel::default(), None, &conn).await?;
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
    let list = User::find_many(
        UserModel::default(),
        Some(FindOption {
            orders: vec![("id".into(), false)],
            limit: Some((2, 0)),
            ..Default::default()
        }),
        &conn,
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
