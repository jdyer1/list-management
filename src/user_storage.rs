use diesel::prelude::*;

use crate::common::{Account, AccountType, User};
use crate::db;
use crate::list_storage::all_account_types;
use crate::models::{AccountDb, UserAccountDb, UserDb};
use crate::schema::{account, user, user_account};

pub(crate) fn create_or_update_user(u1: User) -> User {
    let prior_val: Option<User> = if *(&u1.id.is_some()) {
        retrieve_user_by_id(&u1.id.unwrap())
    } else {
        None
    };
    let user_id: i32 = {
        let mut c = db::connection();

        let my_user_id = if prior_val.is_none() {
            diesel::insert_into(user::table)
                .values((
                    user::name.eq(&u1.name),
                    user::source.eq(&u1.source),
                    user::source_id.eq(&u1.source_id),
                ))
                .execute(&mut c)
                .expect("Could not insert user");
            user::table
                .select(UserDb::as_select())
                .filter(user::name.eq(&u1.name))
                .filter(user::source.eq(&u1.source))
                .filter(user::source_id.eq(&u1.source_id))
                .load(&mut c)
                .unwrap()[0]
                .id
        } else {
            let my_user_id_1 = prior_val.unwrap().id.unwrap() as i32;
            let _ = diesel::update(user::table)
                .filter(user::id.eq(my_user_id_1))
                .set((
                    user::name.eq(&u1.name),
                    user::source.eq(&u1.source),
                    user::source_id.eq(&u1.source_id),
                ))
                .execute(&mut c);
            let _ = diesel::delete(user_account::table)
                .filter(user_account::user_id.eq(&my_user_id_1))
                .execute(&mut c)
                .expect("could not delete old accounts");
            my_user_id_1
        };
        for acct in u1.user_accounts {
            let _ = diesel::insert_into(user_account::table)
                .values((
                    user_account::user_id.eq(&my_user_id),
                    user_account::account_id.eq(&(acct.id.unwrap() as i32)),
                ))
                .execute(&mut c)
                .expect("could not insert user account.");
        }
        my_user_id
    };
    retrieve_user_by_id(&(user_id as u64)).unwrap()
}

fn delete_user(user_id: &u64) -> bool {
    let mut c = db::connection();
    let uid: i32 = *user_id as i32;
    let _ = diesel::delete(user_account::table)
        .filter(user_account::user_id.eq(&uid))
        .execute(&mut c)
        .expect("Could not delete user account relation");
    let num = diesel::delete(user::table)
        .filter(user::id.eq(uid))
        .execute(&mut c)
        .expect("Could not delete user");
    num > 0
}

pub(crate) fn retrieve_user(source: &str, source_id: &str) -> Option<User> {
    let udb = {
        let mut c = db::connection();
        let udb_opt = user::table
            .select(UserDb::as_select())
            .filter(user::source.eq(source))
            .filter(user::source_id.eq(source_id))
            .get_result(&mut c)
            .optional()
            .unwrap();
        if udb_opt.is_none() {
            return None;
        }
        udb_opt.unwrap()
    };
    Some(udb_to_user(udb))
}

pub(crate) fn retrieve_user_by_id(id: &u64) -> Option<User> {
    let udb = {
        let mut c = db::connection();
        let udb_opt = user::table
            .select(UserDb::as_select())
            .filter(user::id.eq(*id as i32))
            .get_result(&mut c)
            .optional()
            .unwrap();
        if udb_opt.is_none() {
            return None;
        }
        udb_opt.unwrap()
    };
    Some(udb_to_user(udb))
}

fn udb_to_user(udb: UserDb) -> User {
    let account_types = all_account_types();
    let mut c = db::connection();
    let accounts: Vec<AccountDb> = UserAccountDb::belonging_to(&udb)
        .inner_join(account::table)
        .select(AccountDb::as_select())
        .load(&mut c)
        .unwrap();

    User {
        id: Some(udb.id as u64),
        name: udb.name.clone(),
        source: udb.source.clone(),
        source_id: udb.source_id.clone(),
        user_accounts: accounts
            .into_iter()
            .map(|a| Account {
                id: Some(a.id as u64),
                account_type: (account_types.get(&a.account_type_id).unwrap_or(&AccountType {
                    id: Some(a.account_type_id as u64),
                    name: "".to_string(),
                    source: "".to_string(),
                })).clone(),
                account_source_id: a.account_source_id,
            })
            .collect(),
    }
}


#[cfg(test)]
pub mod tests {
    use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
    use serial_test::serial;

    use crate::test_helpers::setup_accounts;

    use super::*;

    #[test]
    #[serial]
    fn test_user_storage() {
        let (a1_id, a2_id) = setup_accounts();
        let ac0 = AccountType {
            id: None,
            name: "".to_string(),
            source: "".to_string(),
        };
        let a1 = Account {
            id: Some(a1_id as u64),
            account_type: ac0.clone(),
            account_source_id: "".to_string(),
        };
        let a2 = Account {
            id: Some(a2_id as u64),
            account_type: ac0.clone(),
            account_source_id: "".to_string(),
        };
        let u1 = User {
            id: Some(0),
            name: "user One".to_string(),
            source: "s1".to_string(),
            source_id: "s1-1".to_string(),
            user_accounts: vec![a1.clone(), a2.clone()],
        };
        let u2 = User {
            id: Some(0),
            name: "user Two".to_string(),
            source: "s2".to_string(),
            source_id: "s2-1".to_string(),
            user_accounts: vec![a2.clone()],
        };
        let _ = create_or_update_user(u1);
        let _ = create_or_update_user(u2);

        let s1_11_opt = retrieve_user("s1", "s1-1");
        let s2_21_opt = retrieve_user("s2", "s2-1");
        let s2_11_opt = retrieve_user("s2", "s1-1");
        let s1_21_opt = retrieve_user("s1", "s2-1");

        assert!(s1_11_opt.is_some());
        assert!(s2_21_opt.is_some());
        assert!(s2_11_opt.is_none());
        assert!(s1_21_opt.is_none());

        let s1_11 = s1_11_opt.unwrap();
        assert!(s1_11.id > Some(0));
        assert_eq!("user One", s1_11.name);
        assert_eq!("s1", s1_11.source);
        assert_eq!("s1-1", s1_11.source_id);
        assert_eq!(2, s1_11.user_accounts.len());

        let s2_21 = s2_21_opt.unwrap();
        assert!(s2_21.id > Some(0));
        assert_eq!("user Two", s2_21.name);
        assert_eq!("s2", s2_21.source);
        assert_eq!("s2-1", s2_21.source_id);
        assert_eq!(2, s1_11.user_accounts.len());

        assert!(delete_user(&s1_11.id.unwrap()));
        assert!(!delete_user(&s1_11.id.unwrap()));
        assert!(delete_user(&s2_21.id.unwrap()));
        assert!(!delete_user(&s2_21.id.unwrap()));

        let c = &mut db::connection();
        let count = user_account::table
            .select(UserAccountDb::as_select())
            .load(c)
            .unwrap()
            .len();
        assert_eq!(0, count);
    }
}
