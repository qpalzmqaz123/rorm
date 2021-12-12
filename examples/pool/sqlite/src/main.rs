use rorm_pool::{sqlite, Driver, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let conn = sqlite::Builder::memory().build()?;

    // Init table
    {
        conn.execute("CREATE TABLE ta (a INTEGER)", vec![]).await?;
    }

    // Insert value
    let t1 = std::time::SystemTime::now();
    {
        let params: Vec<Vec<Value>> = (0..10000).map(|i| vec![Value::U32(i)]).collect();
        conn.execute_many("INSERT INTO ta (a) VALUES (?)", params)
            .await?;
    }
    println!(
        "Diff time: {}ms",
        std::time::SystemTime::now().duration_since(t1)?.as_millis()
    );

    // Query value
    let res = conn
        .query_map("SELECT (a) FROM ta WHERE a < 5", vec![], |row| {
            row.get::<i32>(0)
        })
        .await?;
    println!("res: {:?}", res);

    Ok(())
}
