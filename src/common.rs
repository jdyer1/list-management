use std::collections::HashMap;

use chrono::{NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub static ATTRIBUTE_QUANTITY: &str = "quantity";

#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Account {
    pub id: Option<u64>,
    //
    pub account_type: AccountType,
    pub account_source_id: String,
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct AccountType {
    pub id: Option<u64>,
    //
    pub name: String,
    pub source: String,
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct ItemList {
    pub id: Option<u64>,
    //
    pub attributes: HashMap<String, ListAttribute>,
    pub created: NaiveDateTime,
    pub deleted: bool,
    pub folder: String,
    pub items: Option<Vec<ListItem>>,
    pub list_access: ListAccess,
    pub list_accounts: Vec<Account>,
    pub list_type: ListType,
    pub modified: NaiveDateTime,
    pub name: String,
    pub read_only: bool,
    pub rollups: Option<HashMap<String, ItemListRollup>>,
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct ItemListRollup {
    pub total_amount: Price,
    pub total_lines: u64,
    pub total_units: u64,
}

#[derive(Clone, Debug, EnumString, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ListAccess {
    Private,
    Public,
    Shared,
}

#[derive(Clone, Debug, Display, EnumString, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ListAttribute {
    Boolean(bool),
    DateTime(NaiveDateTime),
    Float(f64),
    Integer(i64),
    Price(Price),
    Text(String),
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct ListItem {
    pub id: Option<u64>,
    //
    pub attributes: HashMap<String, ListAttribute>,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub name: String,
    pub source: String,
}

#[derive(Clone, Debug, EnumString, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ListType {
    Standard,
    System,
    Transient,
}

pub trait LMContext {
    fn current_user(self) -> (User, Self);
    fn current_user_state(self) -> (UserState, Self);
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct PagingRequest {
    pub start: u64,
    pub rows: u64,
}


#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Price {
    pub amount: Decimal,
    pub source: String,
}


#[derive(Clone, Debug, EnumString)]
#[derive(Serialize, Deserialize)]
pub enum SortKey {
    Attribute(String),
    CreatedDate,
    Id,
    ModifiedDate,
    Name,
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct SortRequest {
    pub descending: bool,
    pub key: SortKey,
}

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: Option<u64>,
    //
    pub name: String,
    pub source: String,
    pub source_id: String,
    pub user_accounts: Vec<Account>,
}

#[derive(Clone, Debug)]
pub struct UserState {
    pub active_user_accounts: Vec<Account>,
    pub user_id: u64,
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::common::{ItemList, LMContext, User, UserState};

    pub fn context(
        user: User,
        state: UserState,
    ) -> impl LMContext {
        LMC {
            current_user: user,
            current_user_state: state,
        }
    }

    struct LMC {
        current_user: User,
        current_user_state: UserState,
    }

    impl LMContext for LMC {
        fn current_user(self) -> (User, Self) {
            (self.current_user.clone(), self)
        }

        fn current_user_state(self) -> (UserState, Self) {
            (self.current_user_state.clone(), self)
        }
    }
}
