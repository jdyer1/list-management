use std::collections::HashMap;
use std::string::ToString;

use chrono::{DateTime, FixedOffset};
use currency_rs::Currency;

pub static ATTRIBUTE_QUANTITY: &str = "quantity";

#[derive(Clone, Debug)]
pub struct ItemList {
    pub id: u64,
    //
    pub attributes: HashMap<String, ListAttribute>,
    pub read_only: bool,
    pub created: DateTime<FixedOffset>,
    pub deleted: bool,
    pub folder: String,
    pub items: Vec<ListItem>,
    pub list_access: ListAccess,
    pub list_type: ListType,
    pub modified: DateTime<FixedOffset>,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct ItemListRollup {
    pub total_lines: u64,
    pub total_units: u64,
    pub total_amount: Price,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ListAccess {
    Private,
    Public,
    Shared,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ListAttribute {
    Boolean(bool),
    DateTime(DateTime<FixedOffset>),
    Float(f64),
    Integer(i64),
    Price(Price),
    Text(String),
}

#[derive(Clone, Debug)]
pub struct ListItem {
    pub id: u64,
    //
    pub attributes: HashMap<String, ListAttribute>,
    pub created: DateTime<FixedOffset>,
    pub modified: DateTime<FixedOffset>,
    pub name: String,
    pub source: String,
}


pub trait ListStorage {
    fn all_lists(&self) -> Vec<ItemList>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum ListType {
    Standard,
    System,
    Transient,

}

pub trait LMContext {
    fn list_storage(&self) -> impl ListStorage;
}

#[derive(Clone, Debug)]
pub struct PagingRequest {
    pub start: u64,
    pub rows: u64,
}

#[derive(Clone, Debug)]
pub struct Price {
    pub amount: Currency,
    pub source: String,
}

impl PartialEq<Self> for Price {
    fn eq(&self, other: &Self) -> bool {
        &self.amount.value() == &other.amount.value()
    }
}


#[derive(Clone, Debug)]
pub enum SortKey {
    Attribute(String),
    CreatedDate,
    Id,
    ModifiedDate,
    Name,
}

#[derive(Clone, Debug)]
pub struct SortRequest {
    pub descending: bool,
    pub key: SortKey,
}

