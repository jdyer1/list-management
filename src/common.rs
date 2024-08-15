use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use currency_rs::Currency;
use strum_macros::{Display, EnumString};

pub static ATTRIBUTE_QUANTITY: &str = "quantity";

#[derive(Clone, Debug, PartialEq)]
pub struct AccountType {
    pub id: u64,
    //
    pub name: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub id: u64,
    //
    pub account_type: AccountType,
    pub account_source_id: String,
}

#[derive(Clone, Debug)]
pub struct ItemList {
    pub id: u64,
    //
    pub attributes: HashMap<String, ListAttribute>,
    pub created: DateTime<FixedOffset>,
    pub deleted: bool,
    pub folder: String,
    pub items: Vec<ListItem>,
    pub list_access: ListAccess,
    pub list_accounts: Vec<Account>,
    pub list_type: ListType,
    pub modified: DateTime<FixedOffset>,
    pub name: String,
    pub read_only: bool,
}

#[derive(Clone, Debug)]
pub struct ItemListRollup {
    pub total_amount: Price,
    pub total_lines: u64,
    pub total_units: u64,
}

#[derive(Clone, Debug, EnumString, PartialEq)]
pub enum ListAccess {
    Private,
    Public,
    Shared,
}

#[derive(Clone, Debug, Display, EnumString, PartialEq)]
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

    fn user_lists(&self, user_state: UserState) -> Vec<ItemList>;
}

#[derive(Clone, Debug, EnumString, PartialEq)]
pub enum ListType {
    Standard,
    System,
    Transient,
}

pub trait LMContext {
    fn list_storage(self) -> impl ListStorage;
    fn user_storage(self) -> impl UserStorage;
    fn current_user(self) -> (User, Self);
    fn current_user_state(self) -> (UserState, Self);
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

impl Default for Price {
    fn default() -> Self {
        Price {
            amount: Currency::new_string("0.00", None).unwrap(),
            source: "".to_string(),
        }
    }
}

impl PartialEq<Self> for Price {
    fn eq(&self, other: &Self) -> bool {
        self.amount.value() == other.amount.value()
    }
}

#[derive(Clone, Debug, EnumString)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: u64,
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

pub trait UserStorage {
    fn create_or_update_user(&mut self, user: User) -> User;
    fn delete_user(&mut self, user_id: &u64) -> bool;
    fn retrieve_user(&self, source: &str, source_id: &str) -> Option<User>;
    fn retrieve_user_by_id(&self, id: &u64) -> Option<(usize, User)>;
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::common::{ItemList, LMContext, ListStorage, User, UserState, UserStorage};

    pub fn context(
        item_lists: Vec<ItemList>,
        users: Vec<User>,
        user: User,
        state: UserState,
    ) -> impl LMContext {
        let my_list_storage = LS {
            ls_all_item_lists: item_lists,
        };

        let my_user_storage = US {
            us_all_users: users,
        };

        LMC {
            all_item_lists: my_list_storage,
            us_user_storage: my_user_storage,
            current_user: user,
            current_user_state: state,
        }
    }

    struct LMC {
        all_item_lists: LS,
        us_user_storage: US,
        current_user: User,
        current_user_state: UserState,
    }

    impl LMContext for LMC {
        fn list_storage(self) -> impl ListStorage {
            self.all_item_lists
        }

        fn user_storage(self) -> impl UserStorage {
            self.us_user_storage
        }

        fn current_user(self) -> (User, Self) {
            (self.current_user.clone(), self)
        }

        fn current_user_state(self) -> (UserState, Self) {
            (self.current_user_state.clone(), self)
        }
    }

    pub struct LS {
        ls_all_item_lists: Vec<ItemList>,
    }

    impl ListStorage for LS {
        fn all_lists(&self) -> Vec<ItemList> {
            self.ls_all_item_lists.clone()
        }

        fn user_lists(&self, _user_state: UserState) -> Vec<ItemList> {
            self.ls_all_item_lists.clone()
        }
    }

    pub struct US {
        pub us_all_users: Vec<User>,
    }

    impl UserStorage for US {
        fn create_or_update_user(&mut self, user: User) -> User {
            let prior_val = self.retrieve_user_by_id(&user.id);
            if prior_val.is_none() {
                let max_id = self
                    .us_all_users
                    .iter()
                    .map(|a| a.id)
                    .max()
                    .unwrap_or_else(|| 0);
                let new_id = max_id + 1;
                let mut new_obj = user.clone();
                new_obj.id = new_id;
                let return_obj = new_obj.clone();
                self.us_all_users.push(new_obj);
                return return_obj;
            }
            let index = prior_val.unwrap().0;
            let new_obj = user.clone();
            self.us_all_users[index] = new_obj;
            return user;
        }

        fn delete_user(&mut self, user_id: &u64) -> bool {
            let o: Option<(usize, User)> = self.retrieve_user_by_id(user_id);
            if o.is_none() {
                return false;
            }
            self.us_all_users.remove(o.unwrap().0);
            true
        }

        fn retrieve_user(&self, source: &str, source_id: &str) -> Option<User> {
            for a in &self.us_all_users {
                if a.source == source && a.source_id == source_id {
                    let return_a_copy = a.clone();
                    return Some(return_a_copy);
                }
            }
            None
        }

        fn retrieve_user_by_id(&self, id: &u64) -> Option<(usize, User)> {
            let mut i: usize = 0;
            for a in &self.us_all_users {
                if a.id == *id {
                    return Some((i, a.clone()));
                }
                i += 1;
            }
            None
        }
    }
}
