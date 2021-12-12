use rorm::Entity;

#[derive(Debug, Entity)]
struct User {
    pub id: u32,
    pub name: String,
}

fn main() {
    let bob = User {
        id: 1,
        name: "bob".into(),
    };

    println!("{:?}", bob);
}
