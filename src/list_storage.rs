use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use regex::Regex;
use rust_decimal::Decimal;

use crate::common::{
    Account, AccountType, ItemList, ListAccess, ListAttribute, ListItem, ListStorage, ListType,
    Price, UserState,
};
use crate::db;
use crate::models::{
    AccountDb, AccountTypeDb, ItemListAccountDb, ItemListAttributeDb, ItemListDb,
    ListItemAttributeDb, ListItemDb,
};
use crate::schema::{
    account, account_type, item_list, item_list_attribute, list_item, list_item_attribute,
};
use crate::schema::item_list::owner_user_id;

pub struct DatabaseListStorage();

impl ListStorage for DatabaseListStorage {
    fn user_lists(&self, user_state: UserState) -> Vec<ItemList> {
        let mut lists: Vec<ItemListDb> = Vec::new();
        {
            let mut c = db::connection();
            let mut l: Vec<ItemListDb> = item_list::table
                .filter(owner_user_id.eq(user_state.user_id as i32))
                .select(ItemListDb::as_select())
                .order(item_list::id.asc())
                .load(&mut c)
                .unwrap();
            lists.append(&mut l);
        }
        get_lists(lists)
    }

    fn all_lists(&self) -> Vec<ItemList> {
        let mut lists: Vec<ItemListDb> = Vec::new();
        {
            let mut c = db::connection();
            let mut l: Vec<ItemListDb> = item_list::table
                .select(ItemListDb::as_select())
                .order(item_list::id.asc())
                .load(&mut c)
                .unwrap();
            lists.append(&mut l);
        }
        get_lists(lists)
    }
}

fn get_lists(lists: Vec<ItemListDb>) -> Vec<ItemList> {
    let account_types_by_id: HashMap<i32, AccountType> = all_account_types();
    let mut c = db::connection();

    let accounts: Vec<(ItemListAccountDb, AccountDb)> = ItemListAccountDb::belonging_to(&lists)
        .inner_join(account::table)
        .select((ItemListAccountDb::as_select(), AccountDb::as_select()))
        .load(&mut c)
        .unwrap();

    let accounts_per_lists: HashMap<i32, Vec<Account>> = accounts
        .grouped_by(&lists)
        .into_iter()
        .zip(&lists)
        .map(|(vec_of_iladb_adb, ildb)| {
            (
                ildb.id,
                vec_of_iladb_adb
                    .into_iter()
                    .map(|(_, adb)| Account {
                        id: adb.id as u64,
                        account_type: account_types_by_id
                            .get(&adb.account_type_id)
                            .unwrap_or(&AccountType {
                                id: adb.account_type_id as u64,
                                name: "".to_string(),
                                source: "".to_string(),
                            })
                            .clone(),
                        account_source_id: adb.account_source_id,
                    })
                    .collect(),
            )
        })
        .collect();

    let items: Vec<ListItemDb> = ListItemDb::belonging_to(&lists)
        .select(ListItemDb::as_select())
        .order(list_item::id.asc())
        .load(&mut c)
        .unwrap();

    let list_attributes: Vec<ItemListAttributeDb> = ItemListAttributeDb::belonging_to(&lists)
        .select(ItemListAttributeDb::as_select())
        .order(item_list_attribute::id.asc())
        .load(&mut c)
        .unwrap();

    let list_item_attributes: Vec<ListItemAttributeDb> = ListItemAttributeDb::belonging_to(&items)
        .select(ListItemAttributeDb::as_select())
        .order(list_item_attribute::id.asc())
        .load(&mut c)
        .unwrap();

    let mut list_item_attribute_map: HashMap<i32, HashMap<String, ListAttribute>> = HashMap::new();
    for liadb in list_item_attributes {
        let lia_type = liadb.attribute_type;
        let lia_attr: ListAttribute = ListAttribute::from_str(&lia_type)
            .unwrap_or_else(|_| ListAttribute::Text("".to_string()));
        let lia_attr: ListAttribute = match lia_attr {
            ListAttribute::Boolean(_) => ListAttribute::Boolean(liadb.bool_val.unwrap_or(false)),
            ListAttribute::DateTime(_) => ListAttribute::DateTime(NaiveDateTime::from(
                liadb
                    .timestamp_val
                    .unwrap_or(Utc::now().naive_utc()),
            )),
            ListAttribute::Float(_) => ListAttribute::Float(liadb.float_val.unwrap_or(0f32) as f64),
            ListAttribute::Integer(_) => {
                ListAttribute::Integer(liadb.integer_val.unwrap_or(0) as i64)
            }
            ListAttribute::Price(_) => {
                ListAttribute::Price(to_price(liadb.text_val.unwrap_or("".to_string())))
            }
            ListAttribute::Text(_) => ListAttribute::Text(liadb.text_val.unwrap_or("".to_string())),
        };
        list_item_attribute_map
            .entry(liadb.list_item_id)
            .or_default()
            .insert(liadb.name, lia_attr);
    }

    let mut list_attribute_map: HashMap<i32, HashMap<String, ListAttribute>> = HashMap::new();
    for iladb in list_attributes {
        let ila_type = iladb.attribute_type;
        let ila_attr: ListAttribute = ListAttribute::from_str(&ila_type)
            .unwrap_or_else(|_| ListAttribute::Text("".to_string()));
        let ila_attr: ListAttribute = match ila_attr {
            ListAttribute::Boolean(_) => ListAttribute::Boolean(iladb.bool_val.unwrap_or(false)),
            ListAttribute::DateTime(_) => ListAttribute::DateTime(NaiveDateTime::from(
                iladb
                    .timestamp_val
                    .unwrap_or(Utc::now().naive_utc()),
            )),
            ListAttribute::Float(_) => ListAttribute::Float(iladb.float_val.unwrap_or(0f32) as f64),
            ListAttribute::Integer(_) => {
                ListAttribute::Integer(iladb.integer_val.unwrap_or(0) as i64)
            }
            ListAttribute::Price(_) => {
                ListAttribute::Price(to_price(iladb.text_val.unwrap_or("".to_string())))
            }
            ListAttribute::Text(_) => ListAttribute::Text(iladb.text_val.unwrap_or("".to_string())),
        };
        list_attribute_map
            .entry(iladb.item_list_id)
            .or_default()
            .insert(iladb.name, ila_attr);
    }

    let items_per_list: Vec<(ItemListDb, Vec<ListItemDb>)> = items
        .grouped_by(&lists)
        .into_iter()
        .zip(lists)
        .map(|(i, l)| (l, i))
        .collect::<Vec<(ItemListDb, Vec<ListItemDb>)>>();

    items_per_list
        .iter()
        .map(|ildb| -> ItemList {
            let list_attr_map_opt: Option<&HashMap<String, ListAttribute>> =
                list_attribute_map.get(&ildb.0.id);
            let list_attr_map = match list_attr_map_opt {
                None => &HashMap::with_capacity(0),
                Some(_) => list_attr_map_opt.unwrap(),
            };

            let il_id: i32 = ildb.0.id;
            ItemList {
                id: il_id as u64,
                attributes: list_attr_map.to_owned(),
                created: NaiveDateTime::from(ildb.0.created),
                deleted: ildb.0.deleted,
                folder: ildb.0.folder.clone(),
                items: ildb
                    .1
                    .iter()
                    .map(|lidb| -> ListItem {
                        let item_list_attr_map_opt: Option<&HashMap<String, ListAttribute>> =
                            list_item_attribute_map.get(&lidb.id);
                        let item_attr_map = match item_list_attr_map_opt {
                            None => &HashMap::with_capacity(0),
                            Some(_) => item_list_attr_map_opt.unwrap(),
                        };
                        ListItem {
                            id: lidb.id as u64,
                            attributes: item_attr_map.to_owned(),
                            created: NaiveDateTime::from(lidb.created),
                            modified: NaiveDateTime::from(lidb.modified),
                            name: lidb.name.clone(),
                            source: lidb.source.clone(),
                        }
                    })
                    .collect(),
                list_access: ListAccess::from_str(&ildb.0.access).unwrap_or(ListAccess::Public),
                list_accounts: accounts_per_lists.get(&il_id).unwrap_or(&vec![]).to_owned(),
                list_type: ListType::from_str(&ildb.0.list_type).unwrap_or(ListType::Standard),
                modified: NaiveDateTime::from(ildb.0.modified),
                name: ildb.0.name.clone(),
                read_only: false,
            }
        })
        .collect()
}

impl Display for Price {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PRICE: _{} _{}", self.amount, self.source)
    }
}

fn to_price(str: String) -> Price {
    let re = Regex::new(r"^PRICE[:]\s_([^\s]+)\s_([^s]+)$").unwrap();
    let Some(caps) = re.captures(&str) else {
        return Price {
            amount: Decimal::from_str("0.00").unwrap(),
            source: "".to_string(),
        };
    };
    let amount_str = &caps[1];
    let source = &caps[2];
    Price {
        amount: Decimal::from_str(amount_str).unwrap(),
        source: source.to_string(),
    }
}

fn all_account_types() -> HashMap<i32, AccountType> {
    let mut c = db::connection();
    let mut m: HashMap<i32, AccountType> = HashMap::new();
    let v = account_type::table
        .select(AccountTypeDb::as_select())
        .load(&mut c)
        .unwrap();
    for atdb in v {
        let id = atdb.id;
        let at = AccountType {
            id: id as u64,
            name: atdb.name,
            source: atdb.source,
        };
        m.insert(id, at);
    }
    m
}

#[cfg(test)]
pub mod tests {
    use serial_test::serial;

    use crate::common::ListAttribute;
    use crate::db;
    use crate::models::UserDb;
    use crate::schema::user;
    use crate::test_helpers::{insert_user, setup_accounts, setup_db, setup_lists};

    use super::*;

    #[test]
    #[serial]
    fn test_all_lists() {
        setup();
        let a: Vec<ItemList> = DatabaseListStorage().all_lists();
        assert_eq!(2, a.len());
        assert_eq!("Item List One", a[0].name);
        assert_eq!(1, a[0].attributes.len());
        let foo_val = &a[0].attributes["Foo"];
        if let ListAttribute::Text(val) = foo_val {
            assert_eq!("Bar", val);
        } else {
            panic!("Should be a String, was {}", foo_val);
        }
        assert_eq!(2, a[0].list_accounts.len());

        assert_eq!(2, a[0].items.len());
        assert_eq!("IL1-1", a[0].items[0].name);
        assert_eq!("IL1-2", a[0].items[1].name);
        assert_eq!(1, a[0].items[0].attributes.len());
        assert_eq!(1, a[0].items[1].attributes.len());

        assert_eq!("Item List Two", a[1].name);
        assert_eq!(2, a[0].list_accounts.len());
        assert_eq!(
            "at source 1".to_string(),
            a[0].list_accounts[0].account_type.source
        );
        assert_eq!(
            "account type 1".to_string(),
            a[0].list_accounts[0].account_type.name
        );
        assert_eq!(
            "at source 1 one".to_string(),
            a[0].list_accounts[0].account_source_id
        );

        assert_eq!(2, a[1].items.len());
        assert_eq!("IL2-1", a[1].items[0].name);
        assert_eq!("IL2-2", a[1].items[1].name);
        assert_eq!(1, a[1].items[0].attributes.len());
        assert_eq!(1, a[1].items[0].attributes.len());
    }

    #[test]
    #[serial]
    fn test_user_lists() {
        setup();
        let user_ids = {
            let c = &mut db::connection();
            let users_vec = user::table
                .select(UserDb::as_select())
                .order(user::id.asc())
                .load(c)
                .unwrap();
            (users_vec[0].id, users_vec[1].id)
        };
        let us: UserState = UserState {
            active_user_accounts: vec![],
            user_id: user_ids.0 as u64,
        };
        let v = DatabaseListStorage().user_lists(us);
        assert_eq!(1, v.len());
        assert_eq!("Item List One", v[0].name);

        let us1: UserState = UserState {
            active_user_accounts: vec![],
            user_id: user_ids.1 as u64,
        };
        let v1 = DatabaseListStorage().user_lists(us1);
        assert_eq!(1, v1.len());
        assert_eq!("Item List Two", v1[0].name);

        let us2: UserState = UserState {
            active_user_accounts: vec![],
            user_id: (user_ids.0 + user_ids.1) as u64,
        };
        let v2 = DatabaseListStorage().user_lists(us2);
        assert_eq!(0, v2.len());
    }

    fn setup() {
        setup_db();
        let (a1_id, a2_id) = setup_accounts();
        let user_id_1 = insert_user("name", "source", "source-1");
        let user_id_2 = insert_user("name", "source", "source-2");
        setup_lists(vec![a1_id, a2_id], vec![a1_id], user_id_1, user_id_2);
    }

}
