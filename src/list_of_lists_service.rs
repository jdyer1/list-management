use crate::common::{ItemList, ItemListRollup, ListStorage, ListType, LMContext, PagingRequest, SortKey, SortRequest};

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
    let mut a = context.list_storage().all_lists();
    let start = paging.start as usize;
    let mut end = (paging.start + paging.rows) as usize;
    if end > a.len() {
        end = a.len();
    }

    if paging.rows == 0 || paging.start >= a.len() as u64 {
        return vec![];
    }
    a.sort_by(|a, b| {
        let (one, two) = if sort.descending { (b, a) } else { (a, b) };
        match sort.key {
            SortKey::ID => one.id.cmp(&two.id),
            SortKey::NAME => one.name.cmp(&two.name),
        }
    });
    let mut i:usize = 0;
    let mut a1:Vec<ListResult> = Vec::new();
    for item_list in a {
        if i>=start {
            a1.push(ListResult {
                list: item_list,
                rollups: vec![],
            });
        }
        i = i + 1;
        if i==end {
          break;
        }
    }
    a1
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::common::*;

    use super::*;

    #[test]
    fn test_retrieve_all_lists_by_id() {
        let sortReq = sort(SortKey::ID, false);
        let results = retrieve_lists(context(), selector(), paging(0, 10), sortReq, true, true);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(3, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_name() {
        let sortReq = sort(SortKey::NAME, false);
        let results = retrieve_lists(context(), selector(), paging(0, 10), sortReq, true, true);
        assert_eq!(3, results.len());
        assert_eq!("A3", results[0].list.name);
        assert_eq!("B1", results[1].list.name);
        assert_eq!("C2", results[2].list.name);
    }

    #[test]
    fn test_retrieve_all_lists_by_id_descending() {
        let sortReq = sort(SortKey::ID, true);
        let results = retrieve_lists(context(), selector(), paging(0, 10), sortReq, true, true);
        assert_eq!(3, results.len());
        assert_eq!(3, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(1, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_name_descending() {
        let sortReq = sort(SortKey::NAME, true);
        let results = retrieve_lists(context(), selector(), paging(0, 10), sortReq, true, true);
        assert_eq!(3, results.len());
        assert_eq!("C2", results[0].list.name);
        assert_eq!("B1", results[1].list.name);
        assert_eq!("A3", results[2].list.name);
    }

    #[test]
    fn test_retrieve_all_lists_with_paging() {
        let sortReq = sort(SortKey::ID, false);
        let results = retrieve_lists(context(), selector(), paging(1, 1), sortReq, true, true);
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_with_paging_beyond_end() {
        let sortReq = sort(SortKey::ID, false);
        let results = retrieve_lists(context(), selector(), paging(3, 10), sortReq, true, true);
        assert_eq!(0, results.len());
    }

    #[test]
    fn test_retrieve_all_lists_with_no_rows_requested() {
        let sortReq = sort(SortKey::ID, false);
        let results = retrieve_lists(context(), selector(), paging(0, 0), sortReq, true, true);
        assert_eq!(0, results.len());
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
                    item_list(1, "B1".to_string(), "default".to_string()),
                    item_list(3, "A3".to_string(), "default".to_string()),
                    item_list(2, "C2".to_string(), "archive".to_string()),
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

    fn sort(key: SortKey, descending: bool) -> SortRequest {
        SortRequest {
            descending,
            key,
        }
    }
}
