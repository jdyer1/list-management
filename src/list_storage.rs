use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::DateTime;
use currency_rs::Currency;
use diesel::prelude::*;
use regex::Regex;

use crate::common::{ItemList, ListAccess, ListAttribute, ListItem, ListStorage, ListType, Price};
use crate::db;
use crate::models::{ItemListAttributeDb, ItemListDb, ListItemAttributeDb, ListItemDb};
use crate::schema::{item_list, item_list_attribute, list_item, list_item_attribute};

pub struct DatabaseListStorage();

impl ListStorage for DatabaseListStorage {
    fn all_lists(&self) -> Vec<ItemList> {
        let mut c = db::connection();
        let lists: Vec<ItemListDb> = item_list::table
            .select(ItemListDb::as_select())
            .order(item_list::id.asc())
            .load(&mut c).unwrap();

        let items = ListItemDb::belonging_to(&lists)
            .select(ListItemDb::as_select())
            .order(list_item::id.asc())
            .load(&mut c).unwrap();

        let list_attributes = ItemListAttributeDb::belonging_to(&lists)
            .select(ItemListAttributeDb::as_select())
            .order(item_list_attribute::id.asc())
            .load(&mut c).unwrap();

        let list_item_attributes = ListItemAttributeDb::belonging_to(&items)
            .select(ListItemAttributeDb::as_select())
            .order(list_item_attribute::id.asc())
            .load(&mut c).unwrap();

        let mut list_item_attribute_map: HashMap<i32, HashMap<String, ListAttribute>> = HashMap::new();
        for liadb in list_item_attributes {
            let lia_type = liadb.attribute_type;
            let lia_attr: ListAttribute = ListAttribute::from_str(&lia_type).unwrap_or_else(|_| { ListAttribute::Text("".to_string()) });
            let lia_attr: ListAttribute = match lia_attr {
                ListAttribute::Boolean(_) => { ListAttribute::Boolean(liadb.bool_val.unwrap_or_else(|| false)) }
                ListAttribute::DateTime(_) => { ListAttribute::DateTime(DateTime::from(liadb.timestamp_val.unwrap_or_else(|| DateTime::from(chrono::offset::Local::now())))) }
                ListAttribute::Float(_) => { ListAttribute::Float(liadb.float_val.unwrap_or_else(|| 0f32) as f64) }
                ListAttribute::Integer(_) => { ListAttribute::Integer(liadb.integer_val.unwrap_or_else(|| 0) as i64) }
                ListAttribute::Price(_) => { ListAttribute::Price(to_price(liadb.text_val.unwrap_or_else(|| "".to_string()))) }
                ListAttribute::Text(_) => { ListAttribute::Text(liadb.text_val.unwrap_or_else(|| "".to_string())) }
            };
            list_item_attribute_map.entry(liadb.list_item_id)
                .or_default()
                .insert(liadb.name, lia_attr);
        }

        let mut list_attribute_map: HashMap<i32, HashMap<String, ListAttribute>> = HashMap::new();
        for iladb in list_attributes {
            let ila_type = iladb.attribute_type;
            let ila_attr: ListAttribute = ListAttribute::from_str(&ila_type).unwrap_or_else(|_| { ListAttribute::Text("".to_string()) });
            let ila_attr: ListAttribute = match ila_attr {
                ListAttribute::Boolean(_) => { ListAttribute::Boolean(iladb.bool_val.unwrap_or_else(|| false)) }
                ListAttribute::DateTime(_) => { ListAttribute::DateTime(DateTime::from(iladb.timestamp_val.unwrap_or_else(|| DateTime::from(chrono::offset::Local::now())))) }
                ListAttribute::Float(_) => { ListAttribute::Float(iladb.float_val.unwrap_or_else(|| 0f32) as f64) }
                ListAttribute::Integer(_) => { ListAttribute::Integer(iladb.integer_val.unwrap_or_else(|| 0) as i64) }
                ListAttribute::Price(_) => { ListAttribute::Price(to_price(iladb.text_val.unwrap_or_else(|| "".to_string()))) }
                ListAttribute::Text(_) => { ListAttribute::Text(iladb.text_val.unwrap_or_else(|| "".to_string())) }
            };
            list_attribute_map.entry(iladb.item_list_id)
                .or_default()
                .insert(iladb.name, ila_attr);
        }

        let items_per_list = items
            .grouped_by(&lists)
            .into_iter()
            .zip(lists)
            .map(|(i, l)| (l, i))
            .collect::<Vec<(ItemListDb, Vec<ListItemDb>)>>();


        items_per_list.iter().map(|ildb| -> ItemList {
            let list_attr_map_opt : Option<&HashMap<String, ListAttribute>> = list_attribute_map.get(&ildb.0.id);
            let list_attr_map = match list_attr_map_opt {
                None => { &HashMap::with_capacity(0)}
                Some(_) => {list_attr_map_opt.unwrap()}
            };

            ItemList {
                id: ildb.0.id as u64,
                attributes: list_attr_map.to_owned(),
                created: DateTime::from(ildb.0.created),
                deleted: ildb.0.deleted,
                folder: ildb.0.folder.clone(),
                items: ildb.1.iter().map(|lidb| -> ListItem {
                    let item_list_attr_map_opt : Option<&HashMap<String, ListAttribute>> = list_item_attribute_map.get(&lidb.id);
                    let item_attr_map = match item_list_attr_map_opt {
                        None => {&HashMap::with_capacity(0)}
                        Some(_) => {item_list_attr_map_opt.unwrap()}
                    };

                    ListItem {
                        id: lidb.id as u64,
                        attributes: item_attr_map.to_owned(),
                        created: DateTime::from(lidb.created),
                        modified: DateTime::from(lidb.modified),
                        name: lidb.name.clone(),
                        source: lidb.source.clone(),
                    }
                }).collect(),
                list_access: ListAccess::from_str(&ildb.0.access).unwrap_or_else(|_| { ListAccess::Public }),
                list_type: ListType::from_str(&ildb.0.list_type).unwrap_or_else(|_| { ListType::Standard }),
                modified: DateTime::from(ildb.0.modified),
                name: ildb.0.name.clone(),
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

fn to_price(str: String) -> Price {
    let re = Regex::new(r"^PRICE[:]\s_([^\s]+)\s_([^s]+)$").unwrap();
    let Some(caps) = re.captures(&str) else {
        return Price {
            amount: Currency::new_string("0.00", None).unwrap(),
            source: "".to_string(),
        };
    };
    let amount_str = &caps[1];
    let source = &caps[2];
    Price {
        amount: Currency::new_string(amount_str, None).unwrap(),
        source: source.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use currency_rs::Currency;
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::SqliteConnection;

    use crate::common::ListAttribute;
    use crate::db;
    use crate::schema::{item_list_attribute, list_item_attribute};

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
            insert_list_item_attribute(c, list_item_1_2_id, "USD_US".to_string(), ListAttribute::Price(Price {
                amount: Currency::new_string("3.45", None).unwrap(),
                source: "KAU".to_string(),
            }));

            let item_list_2_id = db::tests::insert_item_list(c, "Item List Two".to_string());
            insert_item_list_attribute(c, item_list_2_id, "Number".to_string(), ListAttribute::Integer(1));

            let list_item_2_1_id = db::tests::insert_list_item(c, item_list_2_id, "IL2-1".to_string());
            insert_list_item_attribute(c, list_item_2_1_id, "Priceless".to_string(), ListAttribute::Boolean(true));
            let list_item_2_2_id = db::tests::insert_list_item(c, item_list_2_id, "IL2-2".to_string());
            insert_list_item_attribute(c, list_item_2_2_id, "Length".to_string(), ListAttribute::Float(7.65));
        }
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

        //assert_eq!("Foo", a[0].attributes[0]);
        assert_eq!(2, a[0].items.len());
        assert_eq!("IL1-1", a[0].items[0].name);
        assert_eq!("IL1-2", a[0].items[1].name);
        assert_eq!(1, a[0].items[0].attributes.len());
        assert_eq!(1, a[0].items[1].attributes.len());


        assert_eq!("Item List Two", a[1].name);
        assert_eq!(2, a[1].items.len());
        assert_eq!("IL2-1", a[1].items[0].name);
        assert_eq!("IL2-2", a[1].items[1].name);
        assert_eq!(1, a[1].items[0].attributes.len());
        assert_eq!(1, a[1].items[0].attributes.len());
    }

    fn insert_list_item_attribute(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>, list_item_id: i32, name: String, attr: ListAttribute) {
        match attr {
            ListAttribute::Boolean(b) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::attribute_type.eq("Boolean"), list_item_attribute::name.eq(name), list_item_attribute::bool_val.eq(b)))
                    .execute(c).expect("Could not insert boolean");
            }
            ListAttribute::DateTime(dt) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::attribute_type.eq("DateTime"), list_item_attribute::name.eq(name), list_item_attribute::timestamp_val.eq(dt)))
                    .execute(c).expect("Could not insert DateTime");
            }
            ListAttribute::Float(f) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::attribute_type.eq("Float"), list_item_attribute::name.eq(name), list_item_attribute::float_val.eq(f as f32)))
                    .execute(c).expect("Could not insert float");
            }
            ListAttribute::Integer(i) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::attribute_type.eq("Integer"), list_item_attribute::name.eq(name), list_item_attribute::integer_val.eq(i as i32)))
                    .execute(c).expect("Could not insert integer");
            }
            ListAttribute::Price(p) => {
                let str = format!("{}", p);
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::attribute_type.eq("Price"), list_item_attribute::name.eq(name), list_item_attribute::text_val.eq(str)))
                    .execute(c).expect("Could not insert price");
            }
            ListAttribute::Text(s) => {
                diesel::insert_into(list_item_attribute::table)
                    .values((list_item_attribute::list_item_id.eq(&list_item_id), list_item_attribute::attribute_type.eq("Text"), list_item_attribute::name.eq(name), list_item_attribute::text_val.eq(s)))
                    .execute(c).expect("Could not insert text");
            }
        }
    }

    fn insert_item_list_attribute(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>, item_list_id: i32, name: String, attr: ListAttribute) {
        match attr {
            ListAttribute::Boolean(b) => {
                diesel::insert_into(item_list_attribute::table)
                    .values((item_list_attribute::item_list_id.eq(&item_list_id), item_list_attribute::attribute_type.eq("Boolean"), item_list_attribute::name.eq(name), item_list_attribute::bool_val.eq(b)))
                    .execute(c).expect("Could not insert boolean");
            }
            ListAttribute::DateTime(dt) => {
                diesel::insert_into(item_list_attribute::table)
                    .values((item_list_attribute::item_list_id.eq(&item_list_id), item_list_attribute::attribute_type.eq("DateTime"), item_list_attribute::name.eq(name), item_list_attribute::timestamp_val.eq(dt)))
                    .execute(c).expect("Could not insert DateTime");
            }
            ListAttribute::Float(f) => {
                diesel::insert_into(item_list_attribute::table)
                    .values((item_list_attribute::item_list_id.eq(&item_list_id), item_list_attribute::attribute_type.eq("Float"), item_list_attribute::name.eq(name), item_list_attribute::float_val.eq(f as f32)))
                    .execute(c).expect("Could not insert float");
            }
            ListAttribute::Integer(i) => {
                diesel::insert_into(item_list_attribute::table)
                    .values((item_list_attribute::item_list_id.eq(&item_list_id), item_list_attribute::attribute_type.eq("Integer"), item_list_attribute::name.eq(name), item_list_attribute::integer_val.eq(i as i32)))
                    .execute(c).expect("Could not insert integer");
            }
            ListAttribute::Price(p) => {
                let str = format!("{}", p);
                diesel::insert_into(item_list_attribute::table)
                    .values((item_list_attribute::item_list_id.eq(&item_list_id), item_list_attribute::attribute_type.eq("Price"), item_list_attribute::name.eq(name), item_list_attribute::text_val.eq(str)))
                    .execute(c).expect("Could not insert price");
            }
            ListAttribute::Text(s) => {
                diesel::insert_into(item_list_attribute::table)
                    .values((item_list_attribute::item_list_id.eq(&item_list_id), item_list_attribute::attribute_type.eq("Text"), item_list_attribute::name.eq(name), item_list_attribute::text_val.eq(s)))
                    .execute(c).expect("Could not insert text");
            }
        }
    }
}