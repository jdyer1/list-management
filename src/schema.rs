diesel::table! {
    account (id) {
        id -> Integer,
        account_type_id -> Integer,
        account_source_id -> Text,
    }
}

diesel::table! {
    account_type (id) {
        id -> Integer,
        name -> Text,
        source -> Text,
    }
}

diesel::table! {
    item_list (id) {
        id -> Integer,
        owner_user_id -> Integer,
        created -> Timestamp,
        deleted -> Bool,
        folder -> Text,
        access -> Text,
        list_type -> Text,
        name -> Text,
        modified -> Timestamp,
    }
}

diesel::table! {
    item_list_account (item_list_id, account_id) {
        item_list_id -> Integer,
        account_id -> Integer,
    }
}

diesel::table! {
    item_list_attribute (id) {
        id -> Integer,
        item_list_id -> Integer,
        name -> Text,
        #[sql_name = "type"] attribute_type -> Text,
        bool_val -> Nullable<Bool>,
        timestamp_val -> Nullable<Timestamp>,
        float_val -> Nullable<Float>,
        integer_val -> Nullable<Integer>,
        text_val -> Nullable<Text>,
    }
}

diesel::table! {
    list_item (id) {
        id -> Integer,
        item_list_id -> Integer,
        created -> Timestamp,
        name -> Text,
        modified -> Timestamp,
        source -> Text,
    }
}

diesel::table! {
    list_item_attribute (id) {
        id -> Integer,
        list_item_id -> Integer,
        name -> Text,
        #[sql_name = "type"] attribute_type -> Text,
        bool_val -> Nullable<Bool>,
        timestamp_val -> Nullable<Timestamp>,
        float_val -> Nullable<Float>,
        integer_val -> Nullable<Integer>,
        text_val -> Nullable<Text>,
    }
}

diesel::table! {
    user (id) {
        id -> Integer,
        name -> Text,
        source -> Text,
        source_id -> Text
    }
}

diesel::table! {
    user_account (user_id, account_id) {
        user_id -> Integer,
        account_id -> Integer,
    }
}

diesel::joinable!(user_account -> user (user_id));
diesel::joinable!(user_account -> account (account_id));
diesel::joinable!(account -> account_type (account_type_id));
diesel::joinable!(item_list -> user (owner_user_id));
diesel::joinable!(item_list_account -> item_list (item_list_id));
diesel::joinable!(item_list_account -> account (account_id));
diesel::joinable!(item_list_attribute -> item_list (item_list_id));
diesel::joinable!(list_item -> item_list (item_list_id));
diesel::joinable!(list_item_attribute -> list_item (list_item_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    account_type,
    item_list,
    item_list_account,
    item_list_attribute,
    list_item,
    list_item_attribute,
    user,
    user_account,
);
