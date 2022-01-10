use rorm_conn::{Connection, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let conn = Connection::connect("sqlite://memory")?;

    // Init table
    {
        conn.execute_many(vec![("CREATE TABLE ta (a INTEGER)".into(), vec![])])
            .await?;
    }

    // Insert value
    let t1 = std::time::SystemTime::now();
    {
        let params: Vec<Vec<Value>> = (0..10000).map(|i| vec![Value::U32(i)]).collect();
        let ids = conn
            .execute_many(vec![("INSERT INTO ta (a) VALUES (?)".into(), params)])
            .await?;
        assert_eq!(ids, (1..10000 + 1).collect::<Vec<u64>>());
    }
    println!(
        "Diff time: {}ms",
        std::time::SystemTime::now().duration_since(t1)?.as_millis()
    );

    // Query value
    let res = conn
        .query_many_map("SELECT (a) FROM ta WHERE a < 5", vec![], |row| async move {
            Ok(row.get::<i32>("a")?)
        })
        .await?;
    println!("res: {:?}", res);

    Ok(())
}
