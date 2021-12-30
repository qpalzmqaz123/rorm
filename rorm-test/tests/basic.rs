use rorm::Entity;

#[derive(Debug, Entity)]
struct User {
    #[rorm(primary_key)]
    id: u32,
    name: String,
    email: Option<String>,
}

#[tokio::test]
async fn test_info() {
    assert_eq!(
        User::INFO,
        rorm::TableInfo {
            name: "User",
            columns: &[
                rorm::ColumnInfo {
                    name: "id",
                    ty: rorm::ColumnType::U32,
                    is_primary_key: true,
                    is_not_null: true,
                    is_auto_increment: false,
                    default: None,
                },
                rorm::ColumnInfo {
                    name: "name",
                    ty: rorm::ColumnType::Str(65536),
                    is_primary_key: false,
                    is_not_null: true,
                    is_auto_increment: false,
                    default: None,
                },
                rorm::ColumnInfo {
                    name: "email",
                    ty: rorm::ColumnType::Str(65536),
                    is_primary_key: false,
                    is_not_null: false,
                    is_auto_increment: false,
                    default: None,
                },
            ],
            indexes: &[],
        }
    );
}
