use crate::common::{ItemList, ItemListRollup, ListAccess, ListStorage, ListType, LMContext, PagingRequest, SortKey, SortRequest};

struct ListSelector {
    limit_show_read_only: bool,
    limit_list_types: Vec<ListType>,
    limit_list_access: Vec<ListAccess>,
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
            SortKey::ATTRIBUTE(_) => { todo!() }
            SortKey::CREATED_DATE => { todo!() }
            SortKey::ID => one.id.cmp(&two.id),
            SortKey::MODIFIED_DATE => { todo!() }
            SortKey::NAME => one.name.cmp(&two.name),
        }
    });
    let mut i: usize = 0;
    let mut a1: Vec<ListResult> = Vec::new();
    for item_list in a {
        let mut include: bool = i >= start;
        include = include && (selector.limit_show_not_deleted || item_list.deleted);
        include = include && (selector.limit_show_deleted || !item_list.deleted);
        include = include && (selector.limit_show_read_only || !item_list.read_only);
        include = include && (selector.limit_list_access.is_empty() || selector.limit_list_access.contains(&item_list.list_access));
        include = include && (selector.limit_list_types.is_empty() || selector.limit_list_types.contains(&item_list.list_type));
        include = include && (selector.limit_in_folders.is_empty() || selector.limit_in_folders.contains(&item_list.folder));
        if include && selector.limit_name_keywords.is_some() {
            let name_tokens : Vec<String>  = item_list.name.split_whitespace().map(|a| a.to_ascii_lowercase()).collect();
            for kw in selector.limit_name_keywords.as_ref().unwrap().split_whitespace().map(|a| a.to_ascii_lowercase()) {
                let mut found: bool = false;
                if kw.len() > 1 && kw.ends_with("*") {
                    let mut kw_no_star = kw;
                    kw_no_star.pop();
                    for name_token in &name_tokens {
                        if name_token.starts_with(&kw_no_star) {
                            found = true;
                            break;
                        }
                    }
                } else {
                    found = name_tokens.contains(&kw);
                }
                if !found {
                    include = false;
                    break;
                }
            }
        }

        if include {
            a1.push(ListResult {
                list: item_list,
                rollups: vec![],
            });
        }
        i = i + 1;
        if i == end {
            break;
        }
    }
    a1
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset};

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
        assert_eq!("A3 Naming", results[0].list.name);
        assert_eq!("B1 My Name", results[1].list.name);
        assert_eq!("C2 Your Name", results[2].list.name);
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
        assert_eq!("C2 Your Name", results[0].list.name);
        assert_eq!("B1 My Name", results[1].list.name);
        assert_eq!("A3 Naming", results[2].list.name);
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
        let mut selector = selector();
        selector.limit_show_deleted = false;
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(2, results[0].list.id);
        assert_eq!(3, results[1].list.id);
    }

    #[test]
    fn test_retrieve_deleted_lists_by_id() {
        let mut selector = selector();
        selector.limit_show_not_deleted = false;
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(1, results[0].list.id);
    }

    #[test]
    fn test_retrieve_editable_lists_by_id() {
        let mut selector = selector();
        selector.limit_show_read_only = false;
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
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

    #[test]
    fn test_retrieve_private_lists_by_id() {
        let mut selector = selector();
        selector.limit_list_access = vec![ListAccess::PRIVATE];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].list.id);
    }

    #[test]
    fn test_retrieve_public_or_shared_lists_by_id() {
        let mut selector = selector();
        selector.limit_list_access = vec![ListAccess::PUBLIC, ListAccess::SHARED];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(3, results[1].list.id);
    }

    #[test]
    fn test_retrieve_transient_lists_by_id() {
        let mut selector = selector();
        selector.limit_list_types = vec![ListType::TRANSIENT];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(3, results[0].list.id);
    }

    #[test]
    fn test_retrieve_standard_or_program_lists_by_id() {
        let mut selector = selector();
        selector.limit_list_types = vec![ListType::STANDARD, ListType::PROGRAM];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
    }

    #[test]
    fn test_retrieve_lists_with_keyword_by_id() {
        let mut selector = selector();
        selector.limit_name_keywords = Some("name".to_string());
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
    }

    #[test]
    fn test_retrieve_lists_with_wildcard_keyword_by_id() {
        let mut selector = selector();
        selector.limit_name_keywords = Some("nam*".to_string());
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(3, results[2].list.id);
    }

    #[test]
    fn test_retrieve_lists_with_multiple_keyword_by_id() {
        let mut selector = selector();
        selector.limit_name_keywords = Some("Nam* c2".to_string());
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::ID, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].list.id);
    }

    fn context(item_lists: Vec<ItemList>) -> impl LMContext {
        struct LMC {
            lmc_al: Vec<ItemList>,
        }
        struct LS {
            ls_al: Vec<ItemList>,
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
        let d1 = DateTime::parse_from_rfc3339("2024-07-19T00:00:00-00:00").unwrap();
        let d2 = DateTime::parse_from_rfc3339("2024-07-20T00:00:00-00:00").unwrap();
        let d3 = DateTime::parse_from_rfc3339("2024-07-21T00:00:00-00:00").unwrap();

        vec![
            item_list(1, "B1 My Name".to_string(), "default".to_string(), true, false, ListAccess::PUBLIC, ListType::STANDARD, d1, d3),
            item_list(3, "A3 Naming".to_string(), "default".to_string(), false, true, ListAccess::SHARED, ListType::TRANSIENT, d2, d2),
            item_list(2, "C2 Your Name".to_string(), "archive".to_string(), false, false, ListAccess::PRIVATE, ListType::PROGRAM, d3, d1),
        ]
    }

    fn item_list(id: u64, name: String, folder: String, deleted: bool, read_only: bool,
                 list_access: ListAccess, list_type: ListType, created: DateTime<FixedOffset>, modified: DateTime<FixedOffset>) -> ItemList {
        ItemList {
            id,
            attributes: vec![],
            read_only,
            created: created,
            deleted,
            folder,
            items: vec![],
            list_access: list_access,
            list_type: list_type,
            modified: modified,
            name,
        }
    }

    fn selector() -> ListSelector {
        ListSelector {
            limit_show_read_only: true,
            limit_list_access: vec![],
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
