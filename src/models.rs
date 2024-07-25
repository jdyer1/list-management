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


