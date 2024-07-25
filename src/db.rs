use std::env;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::r2d2::Pool;
use dotenvy::dotenv;
use lazy_static::lazy_static;

lazy_static! {
    static ref POOL: Pool<ConnectionManager<SqliteConnection>> = {
        get_connection_pool()
    };
}


pub fn connection() -> PooledConnection<ConnectionManager<SqliteConnection>> {
    POOL.get().unwrap()
}

fn get_connection_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    dotenv().ok();

    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool_size_str = env::var("DATABASE_POOL_SIZE").expect("DATABASE_POOL_SIZE must be set");
    let pool_size: u32 = pool_size_str.parse().expect("DATABASE_POOL_SIZE must be a positive integer");

    let manager = ConnectionManager::<SqliteConnection>::new(url);
    Pool::builder()
        .max_size(pool_size)
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use diesel::r2d2::PooledConnection;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    use crate::models::{ItemListDb, ItemListDbInsert, ListItemDb, ListItemDbInsert};
    use crate::schema::{item_list, list_item};

    use super::*;

    #[test]
    fn test_item_lists() {
        let c = &mut setup_db();
        let item_list_id_1 = insert_item_list(c, "Item List One".to_string());
        let item_list_id_2 = insert_item_list(c, "Item List Two".to_string());
        let item_list_id_3 = insert_item_list(c, "Item List Three".to_string());

        let results: Vec<ItemListDb> = item_list::table
            .select(ItemListDb::as_select()).order(item_list::id.desc())
            .load(c).unwrap();

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

        diesel::delete(item_list::table.filter(item_list::id.eq(2))).execute(c).unwrap();
        let results: Vec<ItemListDb> = item_list::table
            .select(ItemListDb::as_select()).order(item_list::id.desc())
            .load(c).unwrap();
        assert_eq!(2, results.len());
        assert_eq!("Item List Three", results[0].name);
        assert_eq!("Item List One", results[1].name);

        cleanup_db(c);
    }

    #[test]
    fn test_list_items() {
        let c = &mut setup_db();
        let item_list_id = insert_item_list(c, "Item List One".to_string());
        let list_item_id_1 = insert_list_item(c, item_list_id, "List Item One".to_string());
        let list_item_id_2 = insert_list_item(c, item_list_id, "List Item Two".to_string());

        let results: Vec<ListItemDb> = list_item::table
            .select(ListItemDb::as_select()).order(list_item::id.desc())
            .load(c).unwrap();

        assert_eq!(2, results.len());
        assert_eq!(list_item_id_2, results[0].id);
        assert_eq!(item_list_id, results[0].item_list_id);
        assert_eq!("My Source", results[0].source);
        assert_eq!("List Item Two", results[0].name);
        assert_eq!(list_item_id_1, results[1].id);
        assert_eq!("List Item One", results[1].name);

        cleanup_db(c);
    }

    #[test]
    fn test_attributes() {
        let c = &mut setup_db();
        let item_list_id = insert_item_list(c, "Item List One".to_string());
        let list_item_id = insert_list_item(c, item_list_id, "List Item One".to_string());
        cleanup_db(c);
    }

    fn insert_item_list(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>, name: String) -> i32 {
        let item_list = ItemListDbInsert {
            deleted: &false,
            folder: &"default".to_string(),
            access: &"Public".to_string(),
            list_type: &"Standard".to_string(),
            name: &name,
        };
        diesel::insert_into(item_list::table).values(&item_list).execute(c).unwrap();
        item_list::table
            .select(ItemListDb::as_select()).order(item_list::id.desc())
            .load(c).unwrap()[0].id
    }

    fn insert_list_item(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>, item_list_id: i32, name: String) -> i32 {
        let list_item = ListItemDbInsert {
            item_list_id: &item_list_id,
            name: &name,
            source: &"My Source".to_string(),
        };
        diesel::insert_into(list_item::table).values(&list_item).execute(c).unwrap();
        list_item::table
            .select(ListItemDb::as_select()).order(list_item::id.desc())
            .load(c).unwrap()[0].id
    }

    fn cleanup_db(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>) {
        diesel::delete(item_list::table).execute(c).unwrap();
        diesel::delete(list_item::table).execute(c).unwrap();
    }

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    fn setup_db() -> PooledConnection<ConnectionManager<SqliteConnection>> {
        fs::remove_file("./sqlite.db").unwrap_or_default();
        let mut c = connection();
        c.run_pending_migrations(MIGRATIONS).expect("Could not run migrations");
        return c;
    }
}