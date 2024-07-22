use chrono::{DateTime, FixedOffset};
use currency_rs::Currency;

pub struct ItemList {
    pub id: u64,
    //
    pub attributes: Vec<ListAttribute>,
    pub created: DateTime<FixedOffset>,
    pub deleted: bool,
    pub folder: String,
    pub items: Vec<ListItem>,
    pub list_access: ListAccess,
    pub list_type: ListType,
    pub modified: DateTime<FixedOffset>,
    pub name: String,
}

pub struct ItemListRollup {
    pub total_lines: u64,
    pub total_units: u64,
    pub total_amount: Price,
}

pub enum ListAccess {
    PRIVATE,
    PUBLIC,
    SHARED,
}

pub enum ListAttribute {
    Boolean(bool),
    DateTime(DateTime<FixedOffset>),
    Float(f64),
    Integer(i64),
    Price(Price),
    Text(String),
}

pub struct ListItem {
    pub id: u64,
    //
    pub attributes: Vec<ListAttribute>,
    pub created: DateTime<FixedOffset>,
    pub modified: DateTime<FixedOffset>,
    pub name: String,
    pub source: String,
}


pub trait ListStorage {
    fn all_lists(&self) -> Vec<ItemList>;
}

pub enum ListType {
    CART,
    STANDARD,
    PROGRAM,
}

pub trait LMContext {
    fn list_storage(&self) -> impl ListStorage;
}

pub struct PagingRequest {
    pub start: i64,
    pub rows: i64,
}

pub struct Price {
    pub amount: Currency,
    pub source: String,
}

pub enum SortKey {
    ID,
    NAME,
}

pub struct SortRequest {
    pub descending: bool,
    pub key: SortKey,
}

