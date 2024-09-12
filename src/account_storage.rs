use diesel::prelude::*;

use crate::common::ListManagementError;
use crate::db;
use crate::models::{AccountDb, AccountTypeDb};
use crate::schema::{account, account_type};

pub fn insert_account_type(name: &str, source: &str) -> Result<AccountTypeDb, ListManagementError> {
    let mut c = db::connection();
    let at: AccountTypeDb = diesel::insert_into(account_type::table)
        .values((account_type::name.eq(name),
                 account_type::source.eq(source)))
        .get_result(&mut c)?;
    Ok(at)
}

pub fn insert_account(account_type_id: i32, account_source_id: &str) -> Result<AccountDb, ListManagementError> {
    let mut c = db::connection();
    let acct: AccountDb = diesel::insert_into(account::table)
        .values((account::account_type_id.eq(account_type_id),
                 account::account_source_id.eq(account_source_id)))
        .get_result(&mut c)?;
    Ok(acct)
}


#[cfg(test)]
pub mod tests {
    use serial_test::serial;

    use crate::test_helpers::setup_db;

    use super::*;

    #[test]
    #[serial]
    fn test_insert_account_type() {
        setup_db();
        let at_result = insert_account_type("my name", "my source");
        assert!(at_result.is_ok());
        let at_result_1 = at_result.unwrap();
        assert!(at_result_1.id > 0);
        assert_eq!("my source", at_result_1.source);
        assert_eq!("my name", at_result_1.name);

        let at_result = insert_account_type("my name", "my source");
        assert!(at_result.is_err());

        let at_result = insert_account_type("my second name", "my source");
        assert!(at_result.is_ok());
        let at_result_2 = at_result.unwrap();
        assert!(at_result_2.id > 0);
        assert_ne!(at_result_1, at_result_2);

        let at_result = insert_account_type("my name", "my second source");
        assert!(at_result.is_ok());
        let at_result_3 = at_result.unwrap();
        assert!(at_result_3.id > 0);
        assert_ne!(at_result_1, at_result_3);
        assert_ne!(at_result_2, at_result_3);
    }

    #[test]
    #[serial]
    fn test_insert_account() {
        setup_db();
        let at = insert_account_type("my name", "my source").unwrap();
        let acct_result = insert_account(at.id, "my-source-123");
        assert!(acct_result.is_ok());
        let acct = acct_result.unwrap();
        assert!(acct.id > 0);
        assert_eq!(at.id, acct.account_type_id);
        assert_eq!("my-source-123", acct.account_source_id);

        let acct_result = insert_account(at.id, "my-source-123");
        assert!(acct_result.is_err());

        let acct_result = insert_account(at.id, "my-source-456");
        assert!(acct_result.is_ok());
        let acct2 = acct_result.unwrap();
        assert!(acct2.id > 0);
        assert_ne!(acct2.id, acct.id);
    }
}