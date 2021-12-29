use rorm::Entity;

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(table_name = "user")]
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

    // Check table name
    assert_eq!("user", User::TABLE_NAME);

    // Insert bob
    let bob = UserModel {
        name: Set("bob".into()),
        ..Default::default()
    };
    let bob_id = User::insert(&conn, bob).await?;
    assert_eq!(bob_id, 1);

    // Insert alice
    let alice = UserModel {
        name: Set("alice".into()),
        ..Default::default()
    };
    let alice_id = User::insert(&conn, alice).await?;
    assert_eq!(alice_id, 2);

    // Find bob by id
    let bob = User::find(&conn, 1, None).await?;
    assert_eq!(
        bob,
        User {
            id: 1,
            name: "bob".into()
        }
    );

    // Find alice by name
    let alice = User::find(
        &conn,
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
    let list = User::find_many(&conn, UserModel::default(), None).await?;
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
    User::delete(&conn, 1).await?;
    let list = User::find_many(&conn, UserModel::default(), None).await?;
    assert_eq!(
        list,
        vec![User {
            id: 2,
            name: "alice".into(),
        }]
    );

    // Update alice
    User::update(
        &conn,
        2,
        UserModel {
            name: Set("alex".into()),
            ..Default::default()
        },
    )
    .await?;
    let list = User::find_many(&conn, UserModel::default(), None).await?;
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
    User::insert_many(&conn, [carl, lee]).await?;
    let list = User::find_many(&conn, UserModel::default(), None).await?;
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
        &conn,
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
