use rorm::Entity;

#[derive(Debug, PartialEq, Eq, Entity)]
#[rorm(table_name = "user")]
struct User {
    #[rorm(primary_key, auto_increment)]
    pub id: u32,
    #[rorm(length = 20)]
    pub name: String,
    pub email: Option<String>,
    #[rorm(sql_type = "String", length = 100)]
    pub address: Address,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Address {
    pub city: String,
    pub street: String,
}

impl rorm::pool::ToValue for Address {
    fn to_value(&self) -> rorm::pool::Value {
        rorm::pool::Value::Str(format!("{}#{}", self.city, self.street))
    }
}

impl rorm::pool::FromValue for Address {
    type Output = Address;

    fn from_value(v: &rorm::pool::Value) -> rorm::error::Result<Self::Output> {
        if let rorm::pool::Value::Str(s) = v {
            let mut arr = s.split("#");
            let city = arr
                .next()
                .ok_or(rorm::error::from_value!("Invalid address string"))?;
            let street = arr
                .next()
                .ok_or(rorm::error::from_value!("Invalid address string"))?;

            return Ok(Self {
                city: city.into(),
                street: street.into(),
            });
        }

        Err(rorm::error::from_value!("Address type must be string"))
    }
}

#[tokio::test]
async fn test_info() {
    assert_eq!(
        User::INFO,
        rorm::TableInfo {
            name: "user",
            columns: &[
                rorm::ColumnInfo {
                    name: "id",
                    ty: rorm::ColumnType::U32,
                    is_primary_key: true,
                    is_not_null: true,
                    is_auto_increment: true,
                    default: None,
                },
                rorm::ColumnInfo {
                    name: "name",
                    ty: rorm::ColumnType::Str(20),
                    is_primary_key: false,
                    is_not_null: true,
                    is_auto_increment: false,
                    default: None,
                },
                rorm::ColumnInfo {
                    name: "email",
                    ty: rorm::ColumnType::Str(65535),
                    is_primary_key: false,
                    is_not_null: false,
                    is_auto_increment: false,
                    default: None,
                },
                rorm::ColumnInfo {
                    name: "address",
                    ty: rorm::ColumnType::Str(100),
                    is_primary_key: false,
                    is_not_null: true,
                    is_auto_increment: false,
                    default: None,
                },
            ],
            indexes: &[],
        }
    );
}
