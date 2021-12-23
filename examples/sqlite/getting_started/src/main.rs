#[derive(Debug, rorm::Entity, PartialEq, Eq)]
struct User {
    #[rorm(primary_key)]
    pub id: u32,
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    use rorm::ModelColumn::Set;

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
    let bob = User::find_by_id(1, &conn).await?;
    assert_eq!(
        bob,
        User {
            id: 1,
            name: "bob".into()
        }
    );

    // Find alice by name
    let alice = User::find_one(
        UserModel {
            name: Set("alice".into()),
            ..Default::default()
        },
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
    let list = User::find(UserModel::default(), &conn).await?;
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

    Ok(())
}
