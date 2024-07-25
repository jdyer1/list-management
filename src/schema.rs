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
    item_list_attribute (id) {
        id -> Integer,
        item_list_id -> Integer,
        name -> Text,
        bool_val -> Nullable<Bool>,
        timestamp_val -> Nullable<TimestamptzSqlite>,
        float_val -> Nullable<Float>,
        integer_val -> Nullable<Integer>,
        text_val -> Nullable<Text>,
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

diesel::table! {
    list_item_attribute (id) {
        id -> Integer,
        list_item_id -> Integer,
        name -> Text,
        bool_val -> Nullable<Bool>,
        timestamp_val -> Nullable<TimestamptzSqlite>,
        float_val -> Nullable<Float>,
        integer_val -> Nullable<Integer>,
        text_val -> Nullable<Text>,
    }
}

