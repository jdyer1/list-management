use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::DateTime;
use diesel::prelude::*;

use crate::common::{ItemList, ListAccess, ListStorage, ListType, Price};
use crate::db;
use crate::models::ItemListDb;
use crate::schema::item_list;

pub struct DatabaseListStorage();

impl ListStorage for DatabaseListStorage {
    fn all_lists(&self) -> Vec<ItemList> {
        let mut c = db::connection();
        let results: Vec<ItemListDb> = item_list::table
            .select(ItemListDb::as_select()).order(item_list::id.desc())
            .load(&mut c).unwrap();
        results.iter().map(|ildb| -> ItemList {
            ItemList {
                id: ildb.id as u64,
                attributes: Default::default(),
                created: DateTime::from(ildb.created),
                deleted: ildb.deleted,
                folder: ildb.folder.clone(),
                items: vec![],
                list_access: ListAccess::from_str(&ildb.access).unwrap_or_else(|_| { ListAccess::Public }),
                list_type: ListType::from_str(&ildb.list_type).unwrap_or_else(|_| { ListType::Standard }),
                modified: DateTime::from(ildb.modified),
                name: ildb.name.clone(),
                read_only: false,
            }
        }).collect()
    }
}

impl Display for Price {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PRICE: _{} _{}", self.amount.value(), self.source)
    }
}

#[cfg(test)]
mod tests {
    use currency_rs::Currency;
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::SqliteConnection;

    use crate::common::ListAttribute;
    use crate::db;

    use super::*;

    #[test]
    fn test_all_lists() {
        {
            let c = &mut db::tests::setup_db();
            let item_list_1_id = db::tests::insert_item_list(c, "Item List One".to_string());
            insert_item_list_attribute(c, item_list_1_id, "Foo".to_string(), ListAttribute::Text("Bar".to_string()));

            let list_item_1_1_id = db::tests::insert_list_item(c, item_list_1_id, "IL1-1".to_string());
            insert_list_item_attribute(c, list_item_1_1_id, "USD_US".to_string(), ListAttribute::Price(Price {
                amount: Currency::new_string("1.23", None).unwrap(),
                source: "KAU".to_string(),
            }));
            let list_item_1_2_id = db::tests::insert_list_item(c, item_list_1_id, "IL1-2".to_string());
            insert_list_item_attribute(c, list_item_1_1_id, "USD_US".to_string(), ListAttribute::Price(Price {
                amount: Currency::new_string("3.45", None).unwrap(),
                source: "KAU".to_string(),
            }));

            let item_list_2_id = db::tests::insert_item_list(c, "Item List Two".to_string());
            insert_item_list_attribute(c, item_list_2_id, "Number".to_string(), ListAttribute::Integer(1));

            let list_item_2_1_id = db::tests::insert_list_item(c, item_list_2_id, "IL2-1".to_string());
            insert_list_item_attribute(c, list_item_1_1_id, "Priceless".to_string(), ListAttribute::Boolean(true));
            let list_item_2_2_id = db::tests::insert_list_item(c, item_list_2_id, "IL2-2".to_string());
            insert_list_item_attribute(c, list_item_1_1_id, "Length".to_string(), ListAttribute::Float(7.65));
        }
        let a: Vec<ItemList> = DatabaseListStorage().all_lists();
        assert_eq!(2, a.len());
    }

    fn insert_list_item_attribute(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>, list_item_id: i32, name: String, attr: ListAttribute) {
        /*match(attr) {
            ListAttribute::Boolean(b) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::name.eq(name), list_item_attribute::bool_val.eq(b)))
                    .execute(c).expect("Could not insert boolean");
            }
            ListAttribute::DateTime(dt) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id),list_item_attribute:: name.eq(name), list_item_attribute::timestamp_val.eq(dt)))
                    .execute(c).expect("Could not insert DateTime");

            }
            ListAttribute::Float(f) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::name.eq(name), list_item_attribute::float_val.eq(f)))
                    .execute(c).expect("Could not insert float");

            }
            ListAttribute::Integer(i) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::name.eq(name), list_item_attribute::integer_val.eq(i)))
                    .execute(c).expect("Could not insert integer");
            }
            ListAttribute::Price(p) => {
                let str = format!("{}", p);
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id),list_item_attribute:: name.eq(name), list_item_attribute::text_val.eq(str)))
                    .execute(c).expect("Could not insert price");
            }
            ListAttribute::Text(s) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id),list_item_attribute:: name.eq(name), list_item_attribute::text_val.eq(s)))
                    .execute(c).expect("Could not insert text");

            }
        }*/
    }

    fn insert_item_list_attribute(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>, item_list_id: i32, name: String, attr: ListAttribute) {
        /* match(attr) {
             ListAttribute::Boolean(b) => {
                 diesel::insert_into(item_list_attribute::table)
                     .values((item_list_attribute::item_list_id.eq(&item_list_id), list_item_attribute::name.eq(name), list_item_attribute::bool_val.eq(b)))
                     .execute(c).expect("Could not insert boolean");
             }
             ListAttribute::DateTime(dt) => {
                 diesel::insert_into(item_list_attribute::table)
                     .values((item_list_attribute::item_list_id.eq(&item_list_id),list_item_attribute:: name.eq(name), list_item_attribute::timestamp_val.eq(dt)))
                     .execute(c).expect("Could not insert DateTime");

             }
             ListAttribute::Float(f) => {
                 diesel::insert_into(item_list_attribute::table)
                     .values((item_list_attribute::item_list_id.eq(&item_list_id), list_item_attribute::name.eq(name), list_item_attribute::float_val.eq(f)))
                     .execute(c).expect("Could not insert float");

             }
             ListAttribute::Integer(i) => {
                 diesel::insert_into(item_list_attribute::table)
                     .values((item_list_attribute::item_list_id.eq(&item_list_id), list_item_attribute::name.eq(name), list_item_attribute::integer_val.eq(i)))
                     .execute(c).expect("Could not insert integer");
             }
             ListAttribute::Price(p) => {
                 let str = format!("{}", p);
                 diesel::insert_into(item_list_attribute::table)
                     .values((item_list_attribute::item_list_id.eq(&item_list_id),list_item_attribute:: name.eq(name), list_item_attribute::text_val.eq(str)))
                     .execute(c).expect("Could not insert price");
             }
             ListAttribute::Text(s) => {
                 diesel::insert_into(item_list_attribute::table)
                     .values((item_list_attribute::item_list_id.eq(&item_list_id),list_item_attribute:: name.eq(name), list_item_attribute::text_val.eq(s)))
                     .execute(c).expect("Could not insert text");

             }
         }*/
    }
}