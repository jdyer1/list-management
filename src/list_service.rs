use crate::common::{ItemList, ListManagementError, LMContext, PagingRequest, SortKey, SortRequest};
use crate::list_of_lists_service::{ListSelector,ListProvider};

pub fn retrieve_list(context: &impl LMContext, id: u64) -> Result<ItemList, ListManagementError> {
    let selector = ListSelector {
        limit_show_read_only: false,
        limit_list_types: vec![],
        limit_list_access: vec![],
        limit_show_deleted: false,
        limit_show_not_deleted: true,
        limit_in_folders: vec![],
        limit_name_keywords: None,
        limit_list_ids: vec![id],
    };
    let paging = PagingRequest {
        start: 0,
        rows: 1,
    };
    let sort = SortRequest {
        descending: false,
        key: SortKey::Id,
    };
    let mut lists = context.list_provider().retrieve_lists(context, selector, paging, sort, true, true);
    if lists.is_empty() {
        return Err(ListManagementError::NotFound(id.to_string()));
    }
    Ok(lists.remove(0))
}

#[cfg(test)]
mod tests {
    use crate::common::{ListAccess, ListType, LMContext, PagingRequest, SortRequest};
    use crate::common::tests::{context, context_with_lists};
    use crate::common::tests::state;
    use crate::common::tests::user;

    use super::*;

    #[test]
    pub fn test_retrieve_list() {
        let lists = vec![il(1, "one".to_string()), il(2, "two".to_string())];
        let context = &context_with_lists(user(), state(), lists);
        assert_eq!("one", retrieve_list(context, 1).unwrap().name);

        let context = &context_with_lists(user(), state(), vec![]);
        assert!(retrieve_list(context, 3).is_err());
    }

    fn il(id: u64, name: String) -> ItemList {
        ItemList {
            id: Some(id),
            attributes: Default::default(),
            created: Default::default(),
            deleted: false,
            folder: "".to_string(),
            items: None,
            list_access: ListAccess::Private,
            list_accounts: vec![],
            list_type: ListType::Standard,
            modified: Default::default(),
            name: name,
            read_only: false,
            rollups: None,
        }
    }
}