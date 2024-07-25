use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::item_list)]
pub struct ItemListDb {
    pub id: i32,
    pub created: DateTime<Utc>,
    pub deleted: bool,
    pub folder: String,
    pub access: String,
    pub list_type: String,
    pub name: String,
    pub modified: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::item_list)]
pub struct ItemListDbInsert<'a> {
    pub deleted: &'a bool,
    pub folder: &'a String,
    pub access: &'a String,
    pub list_type: &'a String,
    pub name: &'a String,
}

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::item_list_attribute)]
pub struct ItemListAttributeDb {
    pub id: i32,
    pub item_list_id: i32,
    pub name: String,
    pub bool_val: Option<bool>,
    pub timestamp_val: Option<DateTime<Utc>>,
    pub float_val: Option<f32>,
    pub integer_val: Option<i32>,
    pub text_val: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::item_list_attribute)]
pub struct ItemListAttributeDbInsert<'a> {
    pub item_list_id: &'a i32,
    pub name: &'a String,
    pub bool_val: &'a bool,
    pub timestamp_val: &'a DateTime<Utc>,
    pub float_val: &'a f32,
    pub integer_val:&'a  i32,
    pub text_val: &'a String,
}

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::list_item)]
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

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = crate::schema::list_item_attribute)]
pub struct ListItemAttributeDb {
    pub id: i32,
    pub list_item_id: i32,
    pub name: String,
    pub bool_val: Option<bool>,
    pub timestamp_val: Option<DateTime<Utc>>,
    pub float_val: Option<f32>,
    pub integer_val: Option<i32>,
    pub text_val: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::list_item_attribute)]
pub struct ListItemAttributeDbInsert<'a> {
    pub list_item_id: &'a i32,
    pub name: &'a String,
    pub bool_val: &'a bool,
    pub timestamp_val: &'a DateTime<Utc>,
    pub float_val: &'a f32,
    pub integer_val:&'a  i32,
    pub text_val: &'a String,
}


