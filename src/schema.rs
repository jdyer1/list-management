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

diesel::table! {
    list_items (id) {
        id -> Integer,
        item_lists_id -> Integer,
        created -> TimestamptzSqlite,
        name -> Text,
        modified -> TimestamptzSqlite,
        source -> Text,
    }
}
