use crate::common::{ItemList, ItemListRollup, ListStorage, ListType, LMContext, PagingRequest, SortKey, SortRequest};

struct ListSelector {
    limit_show_read_only: bool,
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
        let mut include: bool = i>=start;
        include = include && (selector.limit_show_not_deleted || item_list.deleted);
        include = include && (selector.limit_show_deleted || !item_list.deleted);
        include = include && (selector.limit_show_read_only || !item_list.read_only);
        include = include && (selector.limit_in_folders.is_empty() || selector.limit_in_folders.contains(&item_list.folder));
        if include {
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
        let sort_request = sort(SortKey::ID, false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(3, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_name() {
        let sort_request = sort(SortKey::NAME, false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!("A3", results[0].list.name);
        assert_eq!("B1", results[1].list.name);
        assert_eq!("C2", results[2].list.name);
    }

    #[test]
    fn test_retrieve_all_lists_by_id_descending() {
        let sort_request = sort(SortKey::ID, true);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(3, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(1, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_name_descending() {
        let sort_request = sort(SortKey::NAME, true);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!("C2", results[0].list.name);
        assert_eq!("B1", results[1].list.name);
        assert_eq!("A3", results[2].list.name);
    }

    #[test]
    fn test_retrieve_all_lists_with_paging() {
        let results = retrieve_lists(context(item_lists()), selector(), paging(1, 1), sort(SortKey::ID, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_with_paging_beyond_end() {
        let results = retrieve_lists(context(item_lists()), selector(), paging(3, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(0, results.len());
    }

    #[test]
    fn test_retrieve_all_lists_with_no_rows_requested() {
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 0), sort(SortKey::ID, false), true, true);
        assert_eq!(0, results.len());
    }

    #[test]
    fn test_retrieve_not_deleted_lists_by_id() {
        let item_lists = vec![
            item_list_1(1, "B1".to_string(), "default".to_string(), true, true),
            item_list_1(2, "C2".to_string(), "archive".to_string(), false, true),
            item_list_1(3, "A3".to_string(), "default".to_string(), false, false),
        ];
        let mut selector = selector();
        selector.limit_show_deleted = false;
        let results = retrieve_lists(context(item_lists), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(2, results[0].list.id);
        assert_eq!(3, results[1].list.id);
    }

    #[test]
    fn test_retrieve_deleted_lists_by_id() {
        let item_lists = vec![
            item_list_1(1, "B1".to_string(), "default".to_string(), true, true),
            item_list_1(2, "C2".to_string(), "archive".to_string(), false, true),
            item_list_1(3, "A3".to_string(), "default".to_string(), false, false),
        ];
        let mut selector = selector();
        selector.limit_show_not_deleted = false;
        let results = retrieve_lists(context(item_lists), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(1, results[0].list.id);
    }

    #[test]
    fn test_retrieve_editable_lists_by_id() {
        let item_lists = vec![
            item_list_1(1, "B1".to_string(), "default".to_string(), true, false),
            item_list_1(2, "C2".to_string(), "archive".to_string(), false, false),
            item_list_1(3, "A3".to_string(), "default".to_string(), false, true),
        ];
        let mut selector = selector();
        selector.limit_show_read_only = false;
        let results = retrieve_lists(context(item_lists), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
    }

    #[test]
    fn test_retrieve_lists_in_archive_folder_by_id() {
        let mut selector = selector();
        selector.limit_in_folders = vec!["archive".to_string()];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].list.id);
    }

    fn item_list(id: u64, name: String, folder: String) -> ItemList {
        item_list_1(id, name, folder, false, false)
    }

    fn item_list_1(id: u64, name: String, folder: String, deleted: bool, read_only: bool) -> ItemList {
        ItemList {
            id,
            attributes: vec![],
            read_only,
            created: DateTime::parse_from_rfc3339("2024-07-19T00:00:00-00:00").unwrap(),
            deleted,
            folder,
            items: vec![],
            list_access: ListAccess::PUBLIC,
            list_type: ListType::STANDARD,
            modified: DateTime::parse_from_rfc3339("2024-07-19T00:00:00-00:00").unwrap(),
            name,
        }
    }

    fn context(item_lists: Vec<ItemList>) -> impl LMContext {
        struct LMC {
            lmc_al: Vec<ItemList>
        }
        struct LS {
            ls_al: Vec<ItemList>
        }

        impl LMContext for LMC {
            fn list_storage(&self) -> impl ListStorage {
                return LS {
                   ls_al: self.lmc_al.clone()
                };
            }
        }

        impl ListStorage for LS {
            fn all_lists(&self) -> Vec<ItemList> {
                self.ls_al.clone()
            }
        }
        LMC {
            lmc_al: item_lists,
        }
    }

    fn item_lists() -> Vec<ItemList> {
        vec![
            item_list(1, "B1".to_string(), "default".to_string()),
            item_list(3, "A3".to_string(), "default".to_string()),
            item_list(2, "C2".to_string(), "archive".to_string()),
        ]
    }

    fn selector() -> ListSelector {
        ListSelector {
            limit_show_read_only: true,
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
