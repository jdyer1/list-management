diesel::table! {
    item_list (id) {
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
    list_item (id) {
        id -> Integer,
        item_list_id -> Integer,
        created -> TimestamptzSqlite,
        name -> Text,
        modified -> TimestamptzSqlite,
        source -> Text,
    }
}
