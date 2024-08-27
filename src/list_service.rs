use crate::common::{ItemList, ListManagementError, LMContext};

fn create_or_update_item_list(context: &impl LMContext, mut item_list: &ItemList) -> Result<u64, ListManagementError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use chrono::NaiveDate;
    use crate::common::*;
    use crate::common::tests::{context, state, user};

    use super::*;

    #[test]
    fn test_create_or_update_item_list() {
        let throw_away_created_date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let throw_away_modified_date = NaiveDate::from_ymd_opt(2024, 3, 4).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let mut il1 = ItemList {
            id: 0,
            attributes: HashMap::with_capacity(0),
            created: throw_away_created_date.clone(),
            deleted: false,
            folder: "My FOlder".to_string(),
            items: vec![],
            list_access: ListAccess::Public,
            list_accounts: vec![],
            list_type: ListType::Standard,
            modified: throw_away_modified_date.clone(),
            name: "LI #1".to_string(),
            read_only: false
        };
        let lists = vec![];
        let users = vec![user()];
        let u1 = user();
        let us1 = state();
        let context = context(lists,users,u1,us1);

        let il1_id = create_or_update_item_list(&context, &mut il1).unwrap();
        assert!(il1_id > 0);
        assert_eq!(il1_id, il1.id);
        assert_ne!(throw_away_created_date, il1.created);
        assert_ne!(throw_away_modified_date, il1.modified);

        let lists = context.list_storage().all_lists();
        assert_eq!(1, lists.len());
        assert_eq!(il1_id, lists[0].id);
    }
}