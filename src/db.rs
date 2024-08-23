use std::env;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::r2d2::Pool;
use dotenvy::dotenv;
use lazy_static::lazy_static;

#[derive(diesel::MultiConnection)]
pub enum MultiConnection {
    Sqlite(diesel::SqliteConnection),
}

lazy_static! {
    static ref POOL: Pool<ConnectionManager<MultiConnection>> = get_connection_pool();
}

pub fn connection() -> PooledConnection<ConnectionManager<MultiConnection>> {
    POOL.get().unwrap()
}

fn get_connection_pool() -> Pool<ConnectionManager<MultiConnection>> {
    dotenv().ok();

    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool_size_str = env::var("DATABASE_POOL_SIZE").expect("DATABASE_POOL_SIZE must be set");
    let pool_size: u32 = pool_size_str
        .parse()
        .expect("DATABASE_POOL_SIZE must be a positive integer");

    let manager = ConnectionManager::<MultiConnection>::new(url);
    Pool::builder()
        .max_size(pool_size)
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

#[cfg(test)]
pub mod tests {
    use chrono::NaiveDate;
    use serial_test::serial;

    use crate::db;
    use crate::models::{
        ItemListAttributeDb, ItemListDb, ListItemAttributeDb, ListItemDb
        ,
    };
    use crate::schema::{
        item_list, item_list_attribute, list_item,
        list_item_attribute,
    };
    use crate::test_helpers::{insert_item_list, insert_list_item, insert_user, setup_db};

    use super::*;

    #[test]
    #[serial]
    fn test_item_lists() {
        setup_db();
        let user_id = insert_user("name","a", "b");
        let c = &mut db::connection();
        let item_list_id_1 = insert_item_list(c, user_id, "Item List One".to_string());
        let item_list_id_2 = insert_item_list(c, user_id, "Item List Two".to_string());
        let item_list_id_3 = insert_item_list(c, user_id, "Item List Three".to_string());

        let results: Vec<ItemListDb> = item_list::table
            .select(ItemListDb::as_select())
            .order(item_list::id.desc())
            .load(c)
            .unwrap();

        assert_eq!(3, results.len());
        assert!(!results[0].deleted);
        assert_eq!("default", results[0].folder);
        assert_eq!("Public", results[0].access);
        assert_eq!("Standard", results[0].list_type);
        assert_eq!(item_list_id_3, results[0].id);
        assert_eq!("Item List Three", results[0].name);
        assert_eq!(item_list_id_2, results[1].id);
        assert_eq!("Item List Two", results[1].name);
        assert_eq!(item_list_id_1, results[2].id);
        assert_eq!("Item List One", results[2].name);

        diesel::delete(item_list::table.filter(item_list::id.eq(2)))
            .execute(c)
            .unwrap();
        let results: Vec<ItemListDb> = item_list::table
            .select(ItemListDb::as_select())
            .order(item_list::id.desc())
            .load(c)
            .unwrap();
        assert_eq!(2, results.len());
        assert_eq!("Item List Three", results[0].name);
        assert_eq!("Item List One", results[1].name);
    }

    #[test]
    #[serial]
    fn test_list_items() {
        setup_db();
        let user_id = insert_user("name", "a", "b");
        let c = &mut db::connection();
        let item_list_id_1 = insert_item_list(c, user_id, "Item List One".to_string());
        let list_item_id_1 = insert_list_item(c, item_list_id_1, "List Item One".to_string());
        let list_item_id_2 = insert_list_item(c, item_list_id_1, "List Item Two".to_string());

        let results: Vec<ListItemDb> = list_item::table
            .select(ListItemDb::as_select())
            .order(list_item::id.desc())
            .load(c)
            .unwrap();

        assert_eq!(2, results.len());
        assert_eq!(list_item_id_2, results[0].id);
        assert_eq!(item_list_id_1, results[0].item_list_id);
        assert_eq!("My Source", results[0].source);
        assert_eq!("List Item Two", results[0].name);
        assert_eq!(list_item_id_1, results[1].id);
        assert_eq!("List Item One", results[1].name);
    }

    #[test]
    #[serial]
    fn test_list_item_attributes() {
        setup_db();
        let user_id = insert_user("name", "a", "b");
        let c = &mut db::connection();
        let item_list_id_1 = insert_item_list(c, user_id, "Item List One".to_string());
        let list_item_id_1 = insert_list_item(c, item_list_id_1, "List Item One".to_string());
        let july_19_2024 = NaiveDate::from_ymd_opt(2024, 7, 19).unwrap().and_hms_opt(0,0,0).unwrap();
        diesel::insert_into(list_item_attribute::table)
            .values((
                list_item_attribute::list_item_id.eq(&list_item_id_1),
                list_item_attribute::attribute_type.eq("Boolean"),
                list_item_attribute::name.eq("My Boolean"),
                list_item_attribute::bool_val.eq(true),
            ))
            .execute(c)
            .expect("Could not insert boolean");
        diesel::insert_into(list_item_attribute::table)
            .values((
                list_item_attribute::list_item_id.eq(&list_item_id_1),
                list_item_attribute::attribute_type.eq("DateTime"),
                list_item_attribute::name.eq("My DateTime"),
                list_item_attribute::timestamp_val.eq(july_19_2024),
            ))
            .execute(c)
            .expect("Could not insert DateTime");
        diesel::insert_into(list_item_attribute::table)
            .values((
                list_item_attribute::list_item_id.eq(&list_item_id_1),
                list_item_attribute::attribute_type.eq("Float"),
                list_item_attribute::name.eq("My Float"),
                list_item_attribute::float_val.eq(1.1f32),
            ))
            .execute(c)
            .expect("Could not insert float");
        diesel::insert_into(list_item_attribute::table)
            .values((
                list_item_attribute::list_item_id.eq(&list_item_id_1),
                list_item_attribute::attribute_type.eq("Integer"),
                list_item_attribute::name.eq("My Integer"),
                list_item_attribute::integer_val.eq(123),
            ))
            .execute(c)
            .expect("Could not insert integer");
        diesel::insert_into(list_item_attribute::table)
            .values((
                list_item_attribute::list_item_id.eq(&list_item_id_1),
                list_item_attribute::attribute_type.eq("Text"),
                list_item_attribute::name.eq("My Text"),
                list_item_attribute::text_val.eq("my text"),
            ))
            .execute(c)
            .expect("Could not insert text");

        let results: Vec<ListItemAttributeDb> = list_item_attribute::table
            .select(ListItemAttributeDb::as_select())
            .filter(list_item_attribute::list_item_id.eq(list_item_id_1))
            .order(list_item_attribute::id.desc())
            .load(c)
            .unwrap();

        assert_eq!(5, results.len());
        assert_eq!("Text", results[0].attribute_type);
        assert_eq!("My Text", results[0].name);
        assert_eq!("my text", results[0].text_val.clone().unwrap());
        assert_eq!("My Integer", results[1].name);
        assert_eq!(123, results[1].integer_val.unwrap());
        assert_eq!("My Float", results[2].name);
        assert!(results[2].float_val.unwrap() > 1.09 && results[2].float_val.unwrap() < 1.11);
        assert_eq!("My DateTime", results[3].name);
        assert_eq!(july_19_2024, results[3].timestamp_val.unwrap());
        assert_eq!("My Boolean", results[4].name);
        assert!(results[4].bool_val.unwrap());
    }

    #[test]
    #[serial]
    fn test_item_list_attributes() {
        setup_db();
        let user_id = insert_user("name", "a", "b");
        let c = &mut db::connection();

        let item_list_id_1 = insert_item_list(c, user_id, "Item List One".to_string());
        let july_20_2024 = NaiveDate::from_ymd_opt(2024, 7, 20).unwrap().and_hms_opt(0,0,0).unwrap();
        diesel::insert_into(item_list_attribute::table)
            .values((
                item_list_attribute::item_list_id.eq(&item_list_id_1),
                item_list_attribute::attribute_type.eq("Boolean"),
                item_list_attribute::name.eq("My Boolean"),
                item_list_attribute::bool_val.eq(true),
            ))
            .execute(c)
            .expect("Could not insert boolean");
        diesel::insert_into(item_list_attribute::table)
            .values((
                item_list_attribute::item_list_id.eq(&item_list_id_1),
                item_list_attribute::attribute_type.eq("DateTime"),
                item_list_attribute::name.eq("My DateTime"),
                item_list_attribute::timestamp_val.eq(july_20_2024),
            ))
            .execute(c)
            .expect("Could not insert DateTime");
        diesel::insert_into(item_list_attribute::table)
            .values((
                item_list_attribute::item_list_id.eq(&item_list_id_1),
                item_list_attribute::attribute_type.eq("Float"),
                item_list_attribute::name.eq("My Float"),
                item_list_attribute::float_val.eq(1.1f32),
            ))
            .execute(c)
            .expect("Could not insert float");
        diesel::insert_into(item_list_attribute::table)
            .values((
                item_list_attribute::item_list_id.eq(&item_list_id_1),
                item_list_attribute::attribute_type.eq("Integer"),
                item_list_attribute::name.eq("My Integer"),
                item_list_attribute::integer_val.eq(123),
            ))
            .execute(c)
            .expect("Could not insert integer");
        diesel::insert_into(item_list_attribute::table)
            .values((
                item_list_attribute::item_list_id.eq(&item_list_id_1),
                item_list_attribute::attribute_type.eq("Text"),
                item_list_attribute::name.eq("My Text"),
                item_list_attribute::text_val.eq("my text"),
            ))
            .execute(c)
            .expect("Could not insert text");

        let results: Vec<ItemListAttributeDb> = item_list_attribute::table
            .select(ItemListAttributeDb::as_select())
            .filter(item_list_attribute::item_list_id.eq(item_list_id_1))
            .order(item_list_attribute::id.desc())
            .load(c)
            .unwrap();
        assert_eq!(5, results.len());
        assert_eq!("Text", results[0].attribute_type);
        assert_eq!("My Text", results[0].name);
        assert_eq!("my text", results[0].text_val.clone().unwrap());
        assert_eq!("My Integer", results[1].name);
        assert_eq!(123, results[1].integer_val.unwrap());
        assert_eq!("My Float", results[2].name);
        assert!(results[2].float_val.unwrap() > 1.09 && results[2].float_val.unwrap() < 1.11);
        assert_eq!("My DateTime", results[3].name);
        assert_eq!(july_20_2024, results[3].timestamp_val.unwrap());
        assert_eq!("My Boolean", results[4].name);
        assert!(results[4].bool_val.unwrap());
    }


}
