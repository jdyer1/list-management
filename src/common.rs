use std::collections::HashMap;

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use thiserror::Error;

use crate::list_of_lists_service::{ListOfListsService, ListProvider};

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

#[derive(Error, Debug)]
pub enum ListManagementError {
    #[error("database error")]
    Database(#[from] diesel::result::Error),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("other error")]
    Other,
}

#[derive(Clone, Debug, EnumString, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ListType {
    Standard,
    System,
    Transient,
}

pub trait LMContext {
    fn current_user(&self) -> User;
    fn current_user_state(&self) -> UserState;
    fn list_provider(&self) -> impl ListProvider {
        ListOfListsService()
    }
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
    use crate::common::{ItemList, LMContext, PagingRequest, SortRequest, User, UserState};
    use crate::list_of_lists_service::{ListProvider, ListSelector};

    pub fn context(
        user: User,
        state: UserState,
    ) -> impl LMContext {
        LMC {
            current_user: user,
            current_user_state: state,
            list_provider: mock_list_provider(vec![]),
        }
    }

    pub fn context_with_lists(
        user: User,
        state: UserState,
        lists: Vec<ItemList>,

    ) -> impl LMContext {
        LMC {
            current_user: user,
            current_user_state: state,
            list_provider: mock_list_provider(lists),
        }
    }

    pub struct LMC {
        pub current_user: User,
        pub current_user_state: UserState,
        pub list_provider: MockListProvider,
    }

    impl LMContext for LMC {
        fn current_user(&self) -> User {
            self.current_user.clone()
        }

        fn current_user_state(&self) -> UserState {
            self.current_user_state.clone()
        }

        fn list_provider(&self) -> impl ListProvider {
            self.list_provider.clone()
        }
    }

    pub fn state() -> UserState {
        UserState {
            active_user_accounts: user().user_accounts,
            user_id: user().id.unwrap(),
        }
    }

    pub fn user() -> User {
        User {
            id: Some(1),
            name: "One Name".to_string(),
            source: "user-source".to_string(),
            source_id: "ONE-ID".to_string(),
            user_accounts: vec![],
        }
    }

    #[derive(Clone)]
    pub struct MockListProvider {
        last_selector: Option<ListSelector>,
        last_paging: Option<PagingRequest>,
        last_sort: Option<SortRequest>,
        last_return_attributes: Option<bool>,
        last_return_rollups: Option<bool>,
        //
        lists: Vec<ItemList>,
    }

    impl ListProvider for MockListProvider {
        fn retrieve_lists(&mut self,
                          _context: &impl LMContext,
                          selector: ListSelector,
                          paging: PagingRequest,
                          sort: SortRequest,
                          return_attributes: bool,
                          return_rollups: bool) -> Vec<ItemList> {
            self.last_selector = Some(selector);
            self.last_paging = Some(paging);
            self.last_sort = Some(sort);
            self.last_return_attributes = Some(return_attributes);
            self.last_return_rollups = Some(return_rollups);

            self.lists.clone()
        }
    }

    pub fn mock_list_provider(lists: Vec<ItemList>) -> MockListProvider {
        MockListProvider {
            last_selector: None,
            last_paging: None,
            last_sort: None,
            last_return_attributes: None,
            last_return_rollups: None,
            lists,
        }
    }
}


