use diesel::prelude::*;

use crate::common::{Account, AccountType, User, UserStorage};
use crate::db;
use crate::models::{AccountDb, UserAccountDb, UserDb};
use crate::schema::{account, user, user_account};

pub struct DatabaseUserStorage();

impl UserStorage for DatabaseUserStorage {
    fn create_or_update_user(&mut self, u1: User) -> User {
        let prior_val = self.retrieve_user_by_id(&u1.id);
        let user_id: i32 = {
            let mut c = db::connection();

            let my_user_id = if prior_val.is_none() {
                diesel::insert_into(user::table)
                    .values((user::name.eq(&u1.name), user::source.eq(&u1.source), user::source_id.eq(&u1.source_id)))
                    .execute(&mut c).expect("Could not insert user");
                let my_user_id_1 = user::table.select(UserDb::as_select())
                    .filter(user::name.eq(&u1.name))
                    .filter(user::source.eq(&u1.source))
                    .filter(user::source_id.eq(&u1.source_id))
                    .load(&mut c).unwrap()[0].id;
                my_user_id_1
            } else {
                let my_user_id_1 = prior_val.unwrap().1.id as i32;
                let _ = diesel::update(user::table)
                    .filter(user::id.eq(my_user_id_1))
                    .set((user::name.eq(&u1.name), user::source.eq(&u1.source), user::source_id.eq(&u1.source_id)))
                    .execute(&mut c);
                let _ = diesel::delete(user_account::table)
                    .filter(user_account::user_id.eq(&my_user_id_1))
                    .execute(&mut c).expect("could not delete old accounts");
                my_user_id_1
            };
            for acct in u1.user_accounts {
                let _ = diesel::insert_into(user_account::table).values((user_account::user_id.eq(&my_user_id), user_account::account_id.eq(&(acct.id as i32))))
                    .execute(&mut c).expect("could not insert user account.");
            }
            my_user_id
        };
        let udb = self.retrieve_user_by_id(&(user_id as u64)).unwrap().1;
        User {
            id: udb.id,
            name: udb.name.clone(),
            source: udb.source.clone(),
            source_id: udb.source_id.clone(),
            user_accounts: vec![],
        }
    }

    fn delete_user(&mut self, user_id: &u64) -> bool {
        let mut c = db::connection();
        let uid: i32 = user_id.clone() as i32;
        let _ = diesel::delete(user_account::table).filter(user_account::user_id.eq(&uid)).execute(&mut c).expect("Could not delete user account relation");
        let num = diesel::delete(user::table).filter(user::id.eq(uid)).execute(&mut c).expect("Could not delete user");
        num > 0
    }

    fn retrieve_user(&self, source: &str, source_id: &str) -> Option<User> {
        let mut c = db::connection();
        let udb_opt = user::table.select(UserDb::as_select())
            .filter(user::source.eq(source))
            .filter(user::source_id.eq(source_id))
            .get_result(&mut c).optional().unwrap();
        if udb_opt.is_none() {
            return None;
        }
        let udb = udb_opt.unwrap();

        let accounts: Vec<AccountDb> = UserAccountDb::belonging_to(&udb)
            .inner_join(account::table)
            .select(AccountDb::as_select())
            .load(&mut c).unwrap();

        let u = User {
            id: udb.id as u64,
            name: udb.name.clone(),
            source: udb.source.clone(),
            source_id: udb.source_id.clone(),
            user_accounts: accounts.into_iter().map(|a| Account {
                id: a.id as u64,
                account_type: AccountType {
                    id: a.account_type_id as u64,
                    name: "".to_string(),
                    source: "".to_string(),
                },
                account_source_id: a.account_source_id,
            }).collect(),
        };
        Some(u)
    }

    fn retrieve_user_by_id(&self, id: &u64) -> Option<(usize, User)> {
        let mut c = db::connection();
        let v = user::table.select(UserDb::as_select())
            .filter(user::id.eq(*id as i32))
            .load(&mut c).unwrap();
        if v.is_empty() {
            return None;
        }
        let udb = &v[0];
        let u = User {
            id: udb.id as u64,
            name: udb.name.clone(),
            source: udb.source.clone(),
            source_id: udb.source_id.clone(),
            user_accounts: vec![],
        };
        let u1 = (0, u);
        Some(u1)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use serial_test::serial;

    use crate::models::AccountTypeDb;
    use crate::schema::account_type;

    use super::*;

    #[test]
    #[serial]
    fn test_user_storage() {
        let (a1_id, a2_id) = setup_accounts();
        let ac0 = AccountType {
            id: 0,
            name: "".to_string(),
            source: "".to_string(),
        };
        let a1 = Account {
            id: a1_id as u64,
            account_type: ac0.clone(),
            account_source_id: "".to_string(),
        };
        let a2 = Account {
            id: a2_id as u64,
            account_type: ac0.clone(),
            account_source_id: "".to_string(),
        };
        let u1 = User {
            id: 0,
            name: "user One".to_string(),
            source: "s1".to_string(),
            source_id: "s1-1".to_string(),
            user_accounts: vec![a1.clone(), a2.clone()],
        };
        let u2 = User {
            id: 0,
            name: "user Two".to_string(),
            source: "s2".to_string(),
            source_id: "s2-1".to_string(),
            user_accounts: vec![a2.clone()],
        };
        let _ = DatabaseUserStorage().create_or_update_user(u1);
        let _ = DatabaseUserStorage().create_or_update_user(u2);

        let s1_11_opt = DatabaseUserStorage().retrieve_user("s1", "s1-1");
        let s2_21_opt = DatabaseUserStorage().retrieve_user("s2", "s2-1");
        let s2_11_opt = DatabaseUserStorage().retrieve_user("s2", "s1-1");
        let s1_21_opt = DatabaseUserStorage().retrieve_user("s1", "s2-1");


        assert!(s1_11_opt.is_some());
        assert!(s2_21_opt.is_some());
        assert!(s2_11_opt.is_none());
        assert!(s1_21_opt.is_none());

        let s1_11 = s1_11_opt.unwrap();
        assert!(s1_11.id > 0);
        assert_eq!("user One", s1_11.name);
        assert_eq!("s1", s1_11.source);
        assert_eq!("s1-1", s1_11.source_id);
        assert_eq!(2, s1_11.user_accounts.len());

        let s2_21 = s2_21_opt.unwrap();
        assert!(s2_21.id > 0);
        assert_eq!("user Two", s2_21.name);
        assert_eq!("s2", s2_21.source);
        assert_eq!("s2-1", s2_21.source_id);
        assert_eq!(2, s1_11.user_accounts.len());

        assert!(DatabaseUserStorage().delete_user(&s1_11.id));
        assert!(!DatabaseUserStorage().delete_user(&s1_11.id));
        assert!(DatabaseUserStorage().delete_user(&s2_21.id));
        assert!(!DatabaseUserStorage().delete_user(&s2_21.id));

        let c = &mut db::connection();
        let count = user_account::table.select(UserAccountDb::as_select()).load(c).unwrap().len();
        assert_eq!(0, count);
    }

    pub fn setup_accounts() -> (i32, i32) {
        db::tests::setup_db();
        let mut c = &mut db::connection();
        let at1_id = insert_account_type(&mut c, "account type 1".to_string(), "at source 1".to_string());
        let at2_id = insert_account_type(&mut c, "account type 2".to_string(), "at source 2".to_string());
        let my_a1_id = insert_account(&mut c, at1_id, "at source 1 one".to_string());
        let my_a2_id = insert_account(&mut c, at2_id, "at source 2 one".to_string());
        (my_a1_id, my_a2_id)
    }

    fn insert_account_type(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>, name: String, source: String) -> i32 {
        diesel::insert_into(account_type::table)
            .values((account_type::name.eq(&name), account_type::source.eq(&source)))
            .execute(c).expect("Could not insert account_type");
        account_type::table.select(AccountTypeDb::as_select())
            .filter(account_type::name.eq(&name))
            .filter(account_type::source.eq(&source))
            .load(c).unwrap()[0].id
    }

    fn insert_account(c: &mut PooledConnection<ConnectionManager<SqliteConnection>>, account_type_id: i32, source_id: String) -> i32 {
        diesel::insert_into(account::table)
            .values((account::account_type_id.eq(&account_type_id), account::account_source_id.eq(&source_id)))
            .execute(c).expect("Could not insert account");
        account::table.select(AccountDb::as_select())
            .filter(account::account_type_id.eq(&account_type_id))
            .filter(account::account_source_id.eq(&source_id))
            .load(c).unwrap()[0].id
    }
}


