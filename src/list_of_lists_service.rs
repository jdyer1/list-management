use crate::common::{ItemList, ItemListRollup, ListStorage, ListType, LMContext, PagingRequest, SortRequest};

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


pub fn retrieve_lists(context: impl LMContext,
                      selector: ListSelector,
                      paging: PagingRequest,
                      sort: SortRequest,
                      return_attributes: bool,
                      return_rollups: bool) -> Vec<ListResult> {
    context.list_storage().all_lists().into_iter().map(|il| ListResult {
        list: il,
        rollups: vec![],
    }).collect()
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::common::*;

    use super::*;

    #[test]
    fn test_retrieve_all_lists() {
        let results = retrieve_lists(context(), selector(), paging(0, 10), sort(SortKey::ID, false, false), true, true);
        assert_eq!(3, results.len());
    }

    fn item_list(id: u64, name: String, folder: String) -> ItemList {
        ItemList {
            id,
            attributes: vec![],
            created: DateTime::parse_from_rfc3339("2024-07-19T00:00:00-00:00").unwrap(),
            deleted: false,
            folder,
            items: vec![],
            list_access: ListAccess::PUBLIC,
            list_type: ListType::STANDARD,
            modified: DateTime::parse_from_rfc3339("2024-07-19T00:00:00-00:00").unwrap(),
            name,
        }
    }

    fn context() -> impl LMContext {
        struct LMC;
        struct LS;

        impl LMContext for LMC {
            fn list_storage(&self) -> impl ListStorage {
                return LS;
            }
        }

        impl ListStorage for LS {
            fn all_lists(&self) -> Vec<ItemList> {
                vec![
                    item_list(1, "default".to_string(), "List One".to_string()),
                    item_list(3, "default".to_string(), "List Three".to_string()),
                    item_list(2, "default".to_string(), "List Two".to_string()),
                ]
            }
        }
        LMC
    }

    fn selector() -> ListSelector {
        ListSelector {
            limit_can_edit: false,
            limit_list_types: vec![],
            limit_show_deleted: true,
            limit_show_not_deleted: true,
            limit_in_folders: vec![],
            limit_name_keywords: None,
        }
    }

    fn paging(start: u64, rows: u64) -> PagingRequest {
        PagingRequest {
            start,
            rows,
        }
    }

    fn sort(key: SortKey, descending: bool, sort_missing_last: bool) -> SortRequest {
        SortRequest {
            descending,
            key,
            sort_missing_last,
        }
    }
}
