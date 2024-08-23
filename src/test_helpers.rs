use std::env;
use std::str::FromStr;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use rust_decimal::Decimal;
use tracing::dispatcher::set_global_default;
use tracing_log::LogTracer;

use crate::common::{ListAttribute, Price, User, UserStorage};
use crate::db;
use crate::db::{connection, MultiConnection};
use crate::helpers::tracing_subscriber;
use crate::models::{AccountDb, AccountTypeDb, ItemListDb, ItemListDbInsert, ListItemDb, ListItemDbInsert};
use crate::schema::{account, account_type, item_list, item_list_account, item_list_attribute, list_item, list_item_attribute, user, user_account};
use crate::user_storage::DatabaseUserStorage;

pub fn setup_logging() {
    match LogTracer::init() {
        Ok(_) => {
            dotenv().ok();
            let log_level = env::var("LOG_LEVEL").unwrap_or("debug".to_string());
            set_global_default(tracing_subscriber(log_level, std::io::stdout).into()).expect("Failed to set subscriber");
        }
        Err(_) => {
            // ignore, log tracer was already initalized.
        }
    }
}

fn cleanup_db(c: &mut PooledConnection<ConnectionManager<MultiConnection>>) {
    diesel::delete(list_item_attribute::table)
        .execute(c)
        .unwrap();
    diesel::delete(list_item::table).execute(c).unwrap();
    diesel::delete(item_list_attribute::table)
        .execute(c)
        .unwrap();
    diesel::delete(item_list_account::table).execute(c).unwrap();
    diesel::delete(item_list::table).execute(c).unwrap();
    diesel::delete(user_account::table).execute(c).unwrap();
    diesel::delete(account_type::table).execute(c).unwrap();
    diesel::delete(account::table).execute(c).unwrap();
    diesel::delete(user::table).execute(c).unwrap();
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn setup_db() {
    let mut c = connection();
    c.run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");
    cleanup_db(&mut c);
}

pub fn setup_accounts() -> (i32, i32) {
    setup_db();
    let at1_id = insert_account_type(
        "account type 1".to_string(),
        "at source 1".to_string(),
    );
    let at2_id = insert_account_type(
        "account type 2".to_string(),
        "at source 2".to_string(),
    );
    let my_a1_id = insert_account(at1_id, "at source 1 one".to_string());
    let my_a2_id = insert_account(at2_id, "at source 2 one".to_string());
    (my_a1_id, my_a2_id)
}

pub fn insert_account_type(
    name: String,
    source: String,
) -> i32 {
    let mut c = connection();
    diesel::insert_into(account_type::table)
        .values((
            account_type::name.eq(&name),
            account_type::source.eq(&source),
        ))
        .execute(&mut c)
        .expect("Could not insert account_type");
    account_type::table
        .select(AccountTypeDb::as_select())
        .filter(account_type::name.eq(&name))
        .filter(account_type::source.eq(&source))
        .load(&mut c)
        .unwrap()[0]
        .id
}

pub fn insert_account(
    account_type_id: i32,
    source_id: String,
) -> i32 {
    let mut c = connection();
    diesel::insert_into(account::table)
        .values((
            account::account_type_id.eq(&account_type_id),
            account::account_source_id.eq(&source_id),
        ))
        .execute(&mut c)
        .expect("Could not insert account");
    account::table
        .select(AccountDb::as_select())
        .filter(account::account_type_id.eq(&account_type_id))
        .filter(account::account_source_id.eq(&source_id))
        .load(&mut c)
        .unwrap()[0]
        .id
}

pub fn setup_lists(item_list_1_account_ids: Vec<i32>, item_list_2_account_ids: Vec<i32>, user_id_1: i32, user_id_2: i32) {
    let c = &mut db::connection();

    let item_list_1_id = insert_item_list(c, user_id_1, "Item List One".to_string());
    insert_item_list_attribute(
        c,
        item_list_1_id,
        "Foo".to_string(),
        ListAttribute::Text("Bar".to_string()),
    );

    for account_id in item_list_1_account_ids {
        insert_item_list_account(c, item_list_1_id, account_id);
    }

    let list_item_1_1_id = insert_list_item(c, item_list_1_id, "IL1-1".to_string());
    insert_list_item_attribute(
        c,
        list_item_1_1_id,
        "USD_US".to_string(),
        ListAttribute::Price(Price {
            amount: Decimal::from_str("1.23").unwrap(),
            source: "KAU".to_string(),
        }),
    );
    let list_item_1_2_id = insert_list_item(c, item_list_1_id, "IL1-2".to_string());
    insert_list_item_attribute(
        c,
        list_item_1_2_id,
        "USD_US".to_string(),
        ListAttribute::Price(Price {
            amount: Decimal::from_str("3.45").unwrap(),
            source: "KAU".to_string(),
        }),
    );

    let item_list_2_id = insert_item_list(c, user_id_2, "Item List Two".to_string());
    insert_item_list_attribute(
        c,
        item_list_2_id,
        "Number".to_string(),
        ListAttribute::Integer(1),
    );

    for account_id in item_list_2_account_ids {
        insert_item_list_account(c, item_list_2_id, account_id);
    }

    let list_item_2_1_id = insert_list_item(c, item_list_2_id, "IL2-1".to_string());
    insert_list_item_attribute(
        c,
        list_item_2_1_id,
        "Priceless".to_string(),
        ListAttribute::Boolean(true),
    );
    let list_item_2_2_id = insert_list_item(c, item_list_2_id, "IL2-2".to_string());
    insert_list_item_attribute(
        c,
        list_item_2_2_id,
        "Length".to_string(),
        ListAttribute::Float(7.65),
    );
}

fn insert_list_item_attribute(
    c: &mut PooledConnection<ConnectionManager<MultiConnection>>,
    list_item_id: i32,
    name: String,
    attr: ListAttribute,
) {
    match attr {
        ListAttribute::Boolean(b) => {
            diesel::insert_into(list_item_attribute::table)
                .values((
                    list_item_attribute::list_item_id.eq(&list_item_id),
                    list_item_attribute::attribute_type.eq("Boolean"),
                    list_item_attribute::name.eq(name),
                    list_item_attribute::bool_val.eq(b),
                ))
                .execute(c)
                .expect("Could not insert boolean");
        }
        ListAttribute::DateTime(dt) => {
            diesel::insert_into(list_item_attribute::table)
                .values((
                    list_item_attribute::list_item_id.eq(&list_item_id),
                    list_item_attribute::attribute_type.eq("DateTime"),
                    list_item_attribute::name.eq(name),
                    list_item_attribute::timestamp_val.eq(dt),
                ))
                .execute(c)
                .expect("Could not insert DateTime");
        }
        ListAttribute::Float(f) => {
            diesel::insert_into(list_item_attribute::table)
                .values((
                    list_item_attribute::list_item_id.eq(&list_item_id),
                    list_item_attribute::attribute_type.eq("Float"),
                    list_item_attribute::name.eq(name),
                    list_item_attribute::float_val.eq(f as f32),
                ))
                .execute(c)
                .expect("Could not insert float");
        }
        ListAttribute::Integer(i) => {
            diesel::insert_into(list_item_attribute::table)
                .values((
                    list_item_attribute::list_item_id.eq(&list_item_id),
                    list_item_attribute::attribute_type.eq("Integer"),
                    list_item_attribute::name.eq(name),
                    list_item_attribute::integer_val.eq(i as i32),
                ))
                .execute(c)
                .expect("Could not insert integer");
        }
        ListAttribute::Price(p) => {
            let str = format!("{}", p);
            diesel::insert_into(list_item_attribute::table)
                .values((
                    list_item_attribute::list_item_id.eq(&list_item_id),
                    list_item_attribute::attribute_type.eq("Price"),
                    list_item_attribute::name.eq(name),
                    list_item_attribute::text_val.eq(str),
                ))
                .execute(c)
                .expect("Could not insert price");
        }
        ListAttribute::Text(s) => {
            diesel::insert_into(list_item_attribute::table)
                .values((
                    list_item_attribute::list_item_id.eq(&list_item_id),
                    list_item_attribute::attribute_type.eq("Text"),
                    list_item_attribute::name.eq(name),
                    list_item_attribute::text_val.eq(s),
                ))
                .execute(c)
                .expect("Could not insert text");
        }
    }
}

fn insert_item_list_attribute(
    c: &mut PooledConnection<ConnectionManager<MultiConnection>>,
    item_list_id: i32,
    name: String,
    attr: ListAttribute,
) {
    match attr {
        ListAttribute::Boolean(b) => {
            diesel::insert_into(item_list_attribute::table)
                .values((
                    item_list_attribute::item_list_id.eq(&item_list_id),
                    item_list_attribute::attribute_type.eq("Boolean"),
                    item_list_attribute::name.eq(name),
                    item_list_attribute::bool_val.eq(b),
                ))
                .execute(c)
                .expect("Could not insert boolean");
        }
        ListAttribute::DateTime(dt) => {
            diesel::insert_into(item_list_attribute::table)
                .values((
                    item_list_attribute::item_list_id.eq(&item_list_id),
                    item_list_attribute::attribute_type.eq("DateTime"),
                    item_list_attribute::name.eq(name),
                    item_list_attribute::timestamp_val.eq(dt),
                ))
                .execute(c)
                .expect("Could not insert DateTime");
        }
        ListAttribute::Float(f) => {
            diesel::insert_into(item_list_attribute::table)
                .values((
                    item_list_attribute::item_list_id.eq(&item_list_id),
                    item_list_attribute::attribute_type.eq("Float"),
                    item_list_attribute::name.eq(name),
                    item_list_attribute::float_val.eq(f as f32),
                ))
                .execute(c)
                .expect("Could not insert float");
        }
        ListAttribute::Integer(i) => {
            diesel::insert_into(item_list_attribute::table)
                .values((
                    item_list_attribute::item_list_id.eq(&item_list_id),
                    item_list_attribute::attribute_type.eq("Integer"),
                    item_list_attribute::name.eq(name),
                    item_list_attribute::integer_val.eq(i as i32),
                ))
                .execute(c)
                .expect("Could not insert integer");
        }
        ListAttribute::Price(p) => {
            let str = format!("{}", p);
            diesel::insert_into(item_list_attribute::table)
                .values((
                    item_list_attribute::item_list_id.eq(&item_list_id),
                    item_list_attribute::attribute_type.eq("Price"),
                    item_list_attribute::name.eq(name),
                    item_list_attribute::text_val.eq(str),
                ))
                .execute(c)
                .expect("Could not insert price");
        }
        ListAttribute::Text(s) => {
            diesel::insert_into(item_list_attribute::table)
                .values((
                    item_list_attribute::item_list_id.eq(&item_list_id),
                    item_list_attribute::attribute_type.eq("Text"),
                    item_list_attribute::name.eq(name),
                    item_list_attribute::text_val.eq(s),
                ))
                .execute(c)
                .expect("Could not insert text");
        }
    }
}

fn insert_item_list_account(
    c: &mut PooledConnection<ConnectionManager<MultiConnection>>,
    item_list_id: i32,
    account_id: i32,
) {
    diesel::insert_into(item_list_account::table)
        .values((
            item_list_account::item_list_id.eq(&item_list_id),
            item_list_account::account_id.eq(&account_id),
        ))
        .execute(c)
        .expect("Could not insert item_list_account");
}

pub fn insert_user(name: &str, source: &str, source_id: &str) -> i32 {
    DatabaseUserStorage()
        .create_or_update_user(User {
            id: 0,
            name: name.to_string(),
            source: source.to_string(),
            source_id: source_id.to_string(),
            user_accounts: vec![],
        })
        .id as i32
}

pub fn insert_item_list(
    c: &mut PooledConnection<ConnectionManager<MultiConnection>>,
    user_id: i32,
    name1: String,
) -> i32 {
    let item_list = ItemListDbInsert {
        deleted: &false,
        folder: &"default".to_string(),
        access: &"Public".to_string(),
        list_type: &"Standard".to_string(),
        name: &name1,
        owner_user_id: user_id,
    };
    diesel::insert_into(item_list::table)
        .values(&item_list)
        .execute(c)
        .unwrap();
    item_list::table
        .select(ItemListDb::as_select())
        .order(item_list::id.desc())
        .load(c)
        .unwrap()[0]
        .id
}

pub fn insert_list_item(
    c: &mut PooledConnection<ConnectionManager<MultiConnection>>,
    item_list_id: i32,
    name1: String,
) -> i32 {
    let list_item = ListItemDbInsert {
        item_list_id: &item_list_id,
        name: &name1,
        source: &"My Source".to_string(),
    };
    diesel::insert_into(list_item::table)
        .values(&list_item)
        .execute(c)
        .unwrap();
    list_item::table
        .select(ListItemDb::as_select())
        .order(list_item::id.desc())
        .load(c)
        .unwrap()[0]
        .id
}