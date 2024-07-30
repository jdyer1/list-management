use std::hash::{Hash, Hasher};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Hash, Debug)]
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



