use std::hash::{Hash, Hasher};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Hash, Debug)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::account)]
pub struct AccountDb {
    pub id: i32,
    pub account_type_id: i32,
    pub account_source_id: String
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Hash, Debug)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::account_type)]
pub struct AccountTypeDb {
    pub id: i32,
    pub name: String,
    pub source: String
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Hash, Debug)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::item_list)]
pub struct ItemListDb {
    pub id: i32,
    pub access: String,
    pub created: DateTime<Utc>,
    pub deleted: bool,
    pub folder: String,
    pub list_type: String,
    pub name: String,
    pub modified: DateTime<Utc>,
    pub owner_user_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::item_list)]
pub struct ItemListDbInsert<'a> {
    pub access: &'a String,
    pub deleted: &'a bool,
    pub folder: &'a String,
    pub list_type: &'a String,
    pub name: &'a String,
    pub owner_user_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(ItemListDb, foreign_key = item_list_id))]
#[diesel(belongs_to(AccountDb, foreign_key = account_id))]
#[diesel(table_name = crate::schema::item_list_account)]
#[diesel(primary_key(item_list_id, account_id))]
pub struct ItemListAccountDb {
    pub item_list_id: i32,
    pub account_id: i32,
}

#[derive(Queryable, Selectable, Identifiable, Associations, PartialEq, Debug)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::item_list_attribute)]
#[diesel(belongs_to(ItemListDb, foreign_key = item_list_id))]
pub struct ItemListAttributeDb {
    pub id: i32,
    pub item_list_id: i32,
    pub name: String,
    pub attribute_type: String,
    pub bool_val: Option<bool>,
    pub timestamp_val: Option<DateTime<Utc>>,
    pub float_val: Option<f32>,
    pub integer_val: Option<i32>,
    pub text_val: Option<String>,
}

impl Eq for ItemListAttributeDb {}

impl Hash for ItemListAttributeDb {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Queryable, Selectable, Identifiable, Associations, PartialEq, Eq, Hash, Debug)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::list_item)]
#[diesel(belongs_to(ItemListDb, foreign_key = item_list_id))]
pub struct ListItemDb {
    pub id: i32,
    pub item_list_id: i32,
    pub created: DateTime<Utc>,
    pub name: String,
    pub modified: DateTime<Utc>,
    pub source: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::list_item)]
pub struct ListItemDbInsert<'a> {
    pub item_list_id: &'a i32,
    pub name: &'a String,
    pub source: &'a String,
}

#[derive(Queryable, Selectable, Identifiable, Associations, PartialEq, Debug)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::list_item_attribute)]
#[diesel(belongs_to(ListItemDb, foreign_key = list_item_id))]
pub struct ListItemAttributeDb {
    pub id: i32,
    pub list_item_id: i32,
    pub name: String,
    pub attribute_type: String,
    pub bool_val: Option<bool>,
    pub timestamp_val: Option<DateTime<Utc>>,
    pub float_val: Option<f32>,
    pub integer_val: Option<i32>,
    pub text_val: Option<String>,
}

impl Eq for ListItemAttributeDb {}

impl Hash for ListItemAttributeDb {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::user)]
pub struct UserDb {
    pub id: i32,
    pub name: String,
    pub source: String,
    pub source_id: String
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(UserDb, foreign_key = user_id))]
#[diesel(belongs_to(AccountDb, foreign_key = account_id))]
#[diesel(table_name = crate::schema::user_account)]
#[diesel(primary_key(user_id, account_id))]
pub struct UserAccountDb {
    pub user_id: i32,
    pub account_id: i32,
}


