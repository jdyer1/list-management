use std::env;

use diesel::prelude::*;
use dotenvy::dotenv;

pub fn connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url).unwrap()
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use diesel::query_dsl::methods::OrderDsl;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use crate::models::{ItemListDb, ItemListDbInsert};
    use crate::schema::item_lists;
    use crate::schema::item_lists::id;
    use super::*;
    #[test]
    fn test_item_lists() {
        let c = &mut setup_db();

        let mut item_list = ItemListDbInsert {
            deleted: &false,
            folder: &"default".to_string(),
            access: &"Public".to_string(),
            list_type: &"Standard".to_string(),
            name: &"Item List One".to_string(),
        };
        diesel::insert_into(item_lists::table).values(&item_list).execute(c);

        let binding = "Item List Two".to_string();
        item_list.name = &binding;
        diesel::insert_into(item_lists::table).values(&item_list).execute(c);

        let binding = "Item List Three".to_string();
        item_list.name = &binding;
        diesel::insert_into(item_lists::table).values(&item_list).execute(c);


        let results: Vec<ItemListDb> = item_lists::table
            .select(ItemListDb::as_select())
            .load(c).unwrap();

        assert_eq!(3, results.len());
        assert!(!results[0].deleted);
        assert_eq!("default", results[0].folder);
        assert_eq!("Public", results[0].access);
        assert_eq!("Standard", results[0].list_type);
        assert_eq!("Item List One", results[0].name);


    }

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    type DB = diesel::sqlite::Sqlite;

    fn setup_db() -> SqliteConnection {
        fs::remove_file("./sqlite.db").unwrap_or_default();
        let mut c = connection();
        c.run_pending_migrations(MIGRATIONS).expect("Could not run migrations");
        return c;
    }
}