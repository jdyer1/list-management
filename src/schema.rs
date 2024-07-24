// @generated automatically by Diesel CLI.

diesel::table! {
    item_lists (id) {
        id -> Integer,
        created -> TimestamptzSqlite,
        deleted -> Bool,
        folder -> Text,
        access -> Text,
        list_type -> Text,
        name -> Text,
        modified -> TimestamptzSqlite,
    }
}
