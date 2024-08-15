use crate::common::{LMContext, User, UserStorage};

pub fn retrieve_user(context: impl LMContext, source: &str, source_id: &str) -> Option<User> {
    context.user_storage().retrieve_user(source, source_id)
}

#[cfg(test)]
mod tests {
    use crate::common::tests::context;
    use crate::common::*;

    use super::*;

    #[test]
    fn test_retrieve_user() {
        let u5 = retrieve_user(us_context(), "source", "source-id-5").unwrap();
        assert_eq!(5, u5.id);
        assert_eq!("User Five", u5.name);
        assert_eq!(3, u5.user_accounts.len());
        assert_eq!(
            "AT1-FIVE-SIX".to_string(),
            u5.user_accounts[0].account_source_id
        );
        assert_eq!("AT1A".to_string(), u5.user_accounts[1].account_type.name);
        assert_eq!(
            "AT2 SOURCE".to_string(),
            u5.user_accounts[2].account_type.source
        );

        let u6 = retrieve_user(us_context(), "source", "source-id-6").unwrap();
        assert_eq!(6, u6.id);
        assert_eq!("User Six", u6.name);
        assert_eq!(3, u6.user_accounts.len());
        assert_eq!(
            "AT1-FIVE-SIX".to_string(),
            u6.user_accounts[0].account_source_id
        );
        assert_eq!(
            "AT1A-FIVE-SIX".to_string(),
            u6.user_accounts[1].account_source_id
        );
        assert_eq!("AT2-SIX".to_string(), u6.user_accounts[2].account_source_id);
    }

    fn us_context() -> impl LMContext {
        let all_users = users();
        let current_user = all_users[0].clone();
        let user_state = user_state(current_user.id.clone(), current_user.user_accounts.clone());
        context(vec![], all_users, current_user, user_state)
    }

    fn user(
        id: u64,
        name: String,
        source: String,
        source_id: String,
        user_accounts: Vec<Account>,
    ) -> User {
        User {
            id,
            name,
            source,
            source_id,
            user_accounts,
        }
    }

    fn users() -> Vec<User> {
        let u5_accounts = vec![
            user_account(55, at1(), "AT1-FIVE-SIX".to_string()),
            user_account(56, at1a(), "AT1A-FIVE-SIX".to_string()),
            user_account(57, at2(), "AT2-FIVE".to_string()),
        ];
        let u6_accounts = vec![
            user_account(65, at1(), "AT1-FIVE-SIX".to_string()),
            user_account(66, at1a(), "AT1A-FIVE-SIX".to_string()),
            user_account(67, at2(), "AT2-SIX".to_string()),
        ];
        vec![
            user(
                5,
                "User Five".to_string(),
                "source".to_string(),
                "source-id-5".to_string(),
                u5_accounts,
            ),
            user(
                6,
                "User Six".to_string(),
                "source".to_string(),
                "source-id-6".to_string(),
                u6_accounts,
            ),
        ]
    }

    fn user_state(id: u64, active_accounts: Vec<Account>) -> UserState {
        UserState {
            user_id: id,
            active_user_accounts: active_accounts,
        }
    }

    fn user_account(id: u64, account_type: AccountType, source_id: String) -> Account {
        Account {
            id,
            account_type,
            account_source_id: source_id,
        }
    }

    fn at1() -> AccountType {
        AccountType {
            id: 100,
            name: String::from("AT1"),
            source: String::from("AT1 SOURCE"),
        }
    }

    fn at1a() -> AccountType {
        AccountType {
            id: 101,
            name: String::from("AT1A".to_string()),
            source: String::from("AT1 SOURCE"),
        }
    }

    fn at2() -> AccountType {
        AccountType {
            id: 102,
            name: String::from("AT2"),
            source: String::from("AT2 SOURCE"),
        }
    }
}
