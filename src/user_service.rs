use crate::common::{LMContext, User};

pub fn create_user(user: User) -> User {
    crate::user_storage::create_or_update_user(user)
}

pub fn retrieve_user(context: impl LMContext, source: &str, source_id: &str) -> Option<User> {
    crate::user_storage::retrieve_user(source, source_id)
}

#[cfg(test)]
mod tests {
    use diesel::{RunQueryDsl, sql_query};
    use serial_test::serial;

    use crate::common::*;
    use crate::common::tests::context;
    use crate::db;

    use super::*;

    #[test]
    #[serial]
    fn test_retrieve_user() {
        {
            let mut c = db::connection();
            let _ = sql_query(r#"
        insert into account_type (id, name, source) values (1000, 'AT1', 'AT1 SOURCE')
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into account_type (id, name, source) values (1001, 'AT2', 'AT2 SOURCE')
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into account (id, account_type_id, account_source_id) values (100, 1000, 'AT1-ZERO')
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into account (id, account_type_id, account_source_id) values (101, 1001, 'AT2-ONE')
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into account (id, account_type_id, account_source_id) values (102, 1000, 'AT1-TWO')
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into USER (id, name, source, source_id) values (5, 'User Five', 'source', 'source-id-5')
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into USER (id, name, source, source_id) values (6, 'User Six', 'source', 'source-id-6')
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into USER_ACCOUNT (user_id, account_id) values (5, 100)
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into USER_ACCOUNT (user_id, account_id) values (5, 101)
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into USER_ACCOUNT (user_id, account_id) values (5, 102)
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into USER_ACCOUNT (user_id, account_id) values (6, 100)
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into USER_ACCOUNT (user_id, account_id) values (6, 102)
            "#).execute(&mut c);
        }

        let u5 = retrieve_user(us_context(User {
            id: Some(5),
            name: "".to_string(),
            source: "".to_string(),
            source_id: "".to_string(),
            user_accounts: vec![],
        }), "source", "source-id-5").unwrap();
        assert_eq!(5, u5.id.unwrap());
        assert_eq!("User Five", u5.name);
        assert_eq!(3, u5.user_accounts.len());
        assert_eq!(
            "AT1-ZERO".to_string(),
            u5.user_accounts[0].account_source_id
        );
        assert_eq!("AT2".to_string(), u5.user_accounts[1].account_type.name);
        assert_eq!(
            "AT1 SOURCE".to_string(),
            u5.user_accounts[2].account_type.source
        );

        let u6 = retrieve_user(us_context(User {
            id: Some(6),
            name: "".to_string(),
            source: "".to_string(),
            source_id: "".to_string(),
            user_accounts: vec![],
        }), "source", "source-id-6").unwrap();
        assert_eq!(6, u6.id.unwrap());
        assert_eq!("User Six", u6.name);
        assert_eq!(2, u6.user_accounts.len());
        assert_eq!(
            "AT1-ZERO".to_string(),
            u6.user_accounts[0].account_source_id
        );
        assert_eq!(
            "AT1-TWO".to_string(),
            u6.user_accounts[1].account_source_id
        );
    }

    #[test]
    #[serial]
    fn test_create_user() {
        {
            let mut c = db::connection();
            let _ = sql_query(r#"
        insert into account_type (id, name, source) values (1000, 'AT1', 'AT1 SOURCE')
            "#).execute(&mut c);
            let _ = sql_query(r#"
        insert into account (id, account_type_id, account_source_id) values (100, 1000, 'AT1-ZERO')
            "#).execute(&mut c);
        }
        let u1 = User {
            id: None,
            name: "My Name".to_string(),
            source: "My Source".to_string(),
            source_id: "My Id".to_string(),
            user_accounts: vec![Account {
                id: Some(100),
                account_type: AccountType {
                    id: None,
                    name: "".to_string(),
                    source: "".to_string(),
                },
                account_source_id: "".to_string(),
            }],
        };
        let u1 = create_user(u1);
        assert!(u1.id.is_some());
        assert_eq!("My Name", u1.name);
        assert_eq!("My Source", u1.source);
        assert_eq!("My Id", u1.source_id);
        assert_eq!(1, u1.user_accounts.len());
        assert_eq!("AT1-ZERO", u1.user_accounts[0].account_source_id);
        assert_eq!("AT1", u1.user_accounts[0].account_type.name);
        assert_eq!("AT1 SOURCE", u1.user_accounts[0].account_type.source);

    }


    fn us_context(user: User) -> impl LMContext {
        let current_user = user;
        let user_state = user_state(current_user.id.unwrap().clone(), current_user.user_accounts.clone());
        context(current_user, user_state)
    }

    fn user_state(id: u64, active_accounts: Vec<Account>) -> UserState {
        UserState {
            user_id: id,
            active_user_accounts: active_accounts,
        }
    }
}
