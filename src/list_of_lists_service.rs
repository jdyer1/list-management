use crate::common::{ItemList, ItemListRollup, ListStorage, ListType, PagingRequest, SortRequest};

struct ListSelector {
    limit_can_edit: bool,
    limit_list_types: Vec<ListType>,
    limit_show_deleted: bool,
    limit_show_not_deleted: bool,
    limit_in_folders: Vec<String>,
    limit_name_keywords: Option<String>,
}

struct ListResult {
    list: ItemList,
    rollups: Vec<ItemListRollup>,
}


pub fn retrieve_lists(lists: ListStorage,
                      selector: ListSelector,
                      paging: PagingRequest,
                      sort: SortRequest,
                      return_attributes: bool,
                      return_rollups: bool) -> Vec<ListResult> {
    lists.lists.into_iter().map(|il| ListResult {
        list: il,
        rollups: vec![],
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::*;
    use chrono::{DateTime, FixedOffset};

    #[test]
    fn test_retrieve_all_lists() {
        let l1 = ItemList {
            id: 1,
            attributes: vec![],
            created: DateTime::parse_from_rfc3339("2024-07-19T00:00:00-00:00").unwrap(),
            deleted: false,
            folder: "default".to_string(),
            items: vec![],
            list_access: ListAccess::PUBLIC,
            list_type: ListType::STANDARD,
            modified: DateTime::parse_from_rfc3339("2024-07-19T00:00:00-00:00").unwrap(),
            name: "List One".to_string(),
        };
        let l2 = ItemList {
            id: 2,
            attributes: vec![],
            created: DateTime::parse_from_rfc3339("2024-07-20T00:00:00-00:00").unwrap(),
            deleted: false,
            folder: "default".to_string(),
            items: vec![],
            list_access: ListAccess::PRIVATE,
            list_type: ListType::CART,
            modified: DateTime::parse_from_rfc3339("2024-07-20T00:00:00-00:00").unwrap(),
            name: "List Two".to_string(),
        };
        let l3 = ItemList {
            id: 3,
            attributes: vec![],
            created: DateTime::parse_from_rfc3339("2024-07-21T00:00:00-00:00").unwrap(),
            deleted: true,
            folder: "default".to_string(),
            items: vec![],
            list_access: ListAccess::SHARED,
            list_type: ListType::PROGRAM,
            modified: DateTime::parse_from_rfc3339("2024-07-21T00:00:00-00:00").unwrap(),
            name: "List Three".to_string(),
        };
        let storage = ListStorage {
            lists: vec![l1, l2, l3],
        };
        let selector = ListSelector {
            limit_can_edit: false,
            limit_list_types: vec![],
            limit_show_deleted: true,
            limit_show_not_deleted: true,
            limit_in_folders: vec![],
            limit_name_keywords: None,
        };
        let paging = PagingRequest {
            start: 0,
            rows: 10,
        };
        let sort= SortRequest {
            descending: false,
            key: SortKey::ID,
        };
        let results  = retrieve_lists(storage, selector, paging, sort, true, true);
        assert_eq!(3, results.len());
    }
}
