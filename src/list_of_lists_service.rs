use std::cmp::Ordering;
use std::collections::HashMap;

use rust_decimal::Decimal;
use tracing::info;

use crate::common::{ATTRIBUTE_QUANTITY, ItemList, ItemListRollup, ListAccess, ListAttribute, ListItem, ListType, LMContext, PagingRequest, Price, SortKey, SortRequest};
use crate::common::ListAttribute::DateTime;

pub trait ListProvider {
    fn retrieve_lists(
       &self,
        context: impl LMContext,
        selector: ListSelector,
        paging: PagingRequest,
        sort: SortRequest,
        return_attributes: bool,
        return_rollups: bool,
    ) -> Vec<ItemList>;
}

#[derive(Debug)]
pub struct ListSelector {
    pub limit_show_read_only: bool,
    pub limit_list_types: Vec<ListType>,
    pub limit_list_access: Vec<ListAccess>,
    pub limit_show_deleted: bool,
    pub limit_show_not_deleted: bool,
    pub limit_in_folders: Vec<String>,
    pub limit_name_keywords: Option<String>,
    pub limit_list_ids: Vec<u64>,
}

pub struct ListOfListsService();

impl ListProvider for ListOfListsService {
    fn retrieve_lists(
        &self,
        context: impl LMContext,
        selector: ListSelector,
        paging: PagingRequest,
        sort: SortRequest,
        return_attributes: bool,
        return_rollups: bool,
    ) -> Vec<ItemList> {
        let user_state = context.current_user_state();
        let a = crate::list_storage::user_lists(user_state);
        let start = paging.start as usize;
        let mut end = (paging.start + paging.rows) as usize;
        if end > a.len() {
            end = a.len();
        }

        if paging.rows == 0 || paging.start >= a.len() as u64 {
            return vec![];
        }
        let a = sort_list_of_lists(a, sort);
        let mut i: usize = 0;
        let mut a1: Vec<ItemList> = Vec::new();
        for mut item_list in a {
            let mut include: bool = i >= start;
            include = include && (selector.limit_show_not_deleted || item_list.deleted);
            include = include && (selector.limit_show_deleted || !item_list.deleted);
            include = include && (selector.limit_show_read_only || !item_list.read_only);
            include = include
                && (selector.limit_list_access.is_empty()
                || selector.limit_list_access.contains(&item_list.list_access));
            include = include
                && (selector.limit_list_types.is_empty()
                || selector.limit_list_types.contains(&item_list.list_type));
            include = include
                && (selector.limit_in_folders.is_empty()
                || selector.limit_in_folders.contains(&item_list.folder));
            include = include
                && (selector.limit_list_ids.is_empty()
                || (item_list.id.is_some() && selector.limit_list_ids.contains(&item_list.id.unwrap())));
            if include && selector.limit_name_keywords.is_some() {
                let name_tokens: Vec<String> = item_list
                    .name
                    .split_whitespace()
                    .map(|a| a.to_ascii_lowercase())
                    .collect();
                for kw in selector
                    .limit_name_keywords
                    .as_ref()
                    .unwrap()
                    .split_whitespace()
                    .map(|a| a.to_ascii_lowercase())
                {
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
                if !return_attributes {
                    item_list.attributes = HashMap::with_capacity(0);
                }
                let il_i = &item_list.items.as_ref().unwrap();
                item_list.rollups = compute_rollup_values(return_rollups, &il_i);
                a1.push(item_list);
            }
            i += 1;
            if i == end {
                break;
            }
        }
        info!("Returning {} list results for {:?} with {:?}", a1.len(), selector, paging);
        a1
    }
}

fn sort_list_of_lists(mut a: Vec<ItemList>, sort: SortRequest) -> Vec<ItemList> {
    a.sort_by(|a, b| {
        let (one, two) = if sort.descending { (b, a) } else { (a, b) };
        match &sort.key {
            SortKey::Attribute(attribute_name) => {
                let one_attribute_opt = one.attributes.get(attribute_name);
                let two_attribute_opt = two.attributes.get(attribute_name);
                if one_attribute_opt.is_some() && two_attribute_opt.is_some() {
                    let one_attribute_value = one_attribute_opt.unwrap();
                    let two_attribute_value = two_attribute_opt.unwrap();
                    let mut ordering: Option<Ordering> = None;
                    match one_attribute_value {
                        ListAttribute::Boolean(v1) => {
                            if let ListAttribute::Boolean(v2) = two_attribute_value {
                                ordering = Some(ordering_with_tiebreaker(v1.cmp(v2), one, two));
                            }
                        }
                        DateTime(v1) => {
                            if let DateTime(v2) = two_attribute_value {
                                ordering = Some(ordering_with_tiebreaker(v1.cmp(v2), one, two));
                            }
                        }
                        ListAttribute::Float(v1) => {
                            if let ListAttribute::Float(v2) = two_attribute_value {
                                if v1.min(*v2) == *v1 {
                                    return Ordering::Less;
                                }
                                return Ordering::Greater;
                            }
                        }
                        ListAttribute::Integer(v1) => {
                            if let ListAttribute::Integer(v2) = two_attribute_value {
                                ordering = Some(ordering_with_tiebreaker(v1.cmp(v2), one, two));
                            }
                        }
                        ListAttribute::Price(v1) => {
                            if let ListAttribute::Price(v2) = two_attribute_value {
                                let value1 = v1.amount;
                                let value2 = v2.amount;
                                if value1.min(value2) == value1 {
                                    return Ordering::Less;
                                }
                                return Ordering::Greater;
                            }
                        }
                        ListAttribute::Text(v1) => {
                            if let ListAttribute::Text(v2) = two_attribute_value {
                                ordering = Some(ordering_with_tiebreaker(v1.cmp(v2), one, two));
                            }
                        }
                    }
                    if let Some(ret_val) = ordering {
                        return ret_val;
                    }
                }
                ordering_by_id(one, two)
            }
            SortKey::CreatedDate => one.created.cmp(&two.created),
            SortKey::Id => ordering_by_id(one, two),
            SortKey::ModifiedDate => one.modified.cmp(&two.modified),
            SortKey::Name => one.name.cmp(&two.name),
        }
    });
    a
}

fn ordering_by_id(one: &ItemList, two: &ItemList) -> Ordering {
    one.id.cmp(&two.id)
}

fn ordering_with_tiebreaker(o: Ordering, one: &ItemList, two: &ItemList) -> Ordering {
    if o == Ordering::Equal {
        ordering_by_id(one, two)
    } else {
        o
    }
}


fn compute_rollup_values(
    return_rollups: bool,
    items: &Vec<ListItem>,
) -> Option<HashMap<String, ItemListRollup>> {
    if return_rollups {
        let mut rollups: HashMap<String, ItemListRollup> = HashMap::new();
        for item in items {
            let qty_o = item.attributes.get(ATTRIBUTE_QUANTITY);
            let mut qty: u64 = 0;
            if qty_o.is_some() {
                if let ListAttribute::Integer(qty1) = qty_o.unwrap() {
                    qty = *qty1 as u64;
                }
            }
            for (k, v) in &item.attributes {
                if let ListAttribute::Price(price) = v {
                    let ilr_price = Price {
                        amount: price.amount * Decimal::from(qty),
                        source: price.source.clone(),
                    };

                    let ilr_o = rollups.get(k);
                    if ilr_o.is_none() {
                        let ilr = ItemListRollup {
                            total_lines: 1,
                            total_units: qty,
                            total_amount: ilr_price,
                        };
                        rollups.insert(k.clone(), ilr);
                    } else {
                        let ilr = ilr_o.unwrap();
                        let mut price1 = ilr.total_amount.clone();
                        let currency1 = price1.amount + ilr_price.amount;
                        price1.amount = currency1;

                        let ilr1 = ItemListRollup {
                            total_lines: ilr.total_lines + 1,
                            total_units: ilr.total_units + qty,
                            total_amount: price1,
                        };
                        rollups.insert(k.clone(), ilr1);
                    }
                }
            }
        }
        return Some(rollups);
    }
    None
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use diesel::{RunQueryDsl, sql_query};
    use diesel_migrations::MigrationHarness;
    use rust_decimal::Decimal;
    use serial_test::serial;

    use crate::common::*;
    use crate::common::tests::context;
    use crate::db;
    use crate::test_helpers::MIGRATIONS;

    use super::*;

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_id() {
        setup(false, false);
        let sort_request = sort(SortKey::Id, false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            false,
            false,
        );
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
        assert_eq!(3, results[2].id.unwrap());
        assert!(results[0].attributes.is_empty());
        assert!(results[1].attributes.is_empty());
        assert!(results[2].attributes.is_empty());
        assert!(results[0].rollups.is_none());
        assert!(results[1].rollups.is_none());
        assert!(results[2].rollups.is_none());
        assert_eq!(1, results[0].list_accounts.len());
        assert_eq!(1, results[1].list_accounts.len());
        assert_eq!(1, results[2].list_accounts.len());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_id_with_attributes_and_rollups() {
        setup(true, true);
        let sort_request = sort(SortKey::Id, false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
        assert_eq!(3, results[2].id.unwrap());
        assert!(!results[0].attributes.is_empty());
        assert!(!results[1].attributes.is_empty());
        assert!(!results[2].attributes.is_empty());
        let list_0_rollups = results[0].rollups.clone().unwrap();
        let list_1_rollups = results[1].rollups.clone().unwrap();
        let list_2_rollups = results[2].rollups.clone().unwrap();
        assert_eq!(2, list_0_rollups.len());

        let l0r_xyz = list_0_rollups.get("xyz").unwrap();
        assert_eq!(3, l0r_xyz.total_lines);
        assert_eq!(6, l0r_xyz.total_units);
        assert_eq!("xyz-source", l0r_xyz.total_amount.source);
        assert_eq!(Decimal::from_str("19.98").unwrap(), l0r_xyz.total_amount.amount);

        let l0r_qwe = list_0_rollups.get("qwe").unwrap();
        assert_eq!(2, l0r_qwe.total_lines);
        assert_eq!(4, l0r_qwe.total_units);
        assert_eq!("qwe-source", l0r_qwe.total_amount.source);
        assert_eq!(Decimal::from_str("9.36").unwrap(), l0r_qwe.total_amount.amount);

        assert_eq!(2, list_1_rollups.len());

        let l1r_xyz = list_1_rollups.get("xyz").unwrap();
        assert_eq!(3, l1r_xyz.total_lines);
        assert_eq!(6, l1r_xyz.total_units);
        assert_eq!("xyz-source", l1r_xyz.total_amount.source);
        assert_eq!(Decimal::from_str("6.66").unwrap(), l1r_xyz.total_amount.amount);

        let l1r_qwe = list_1_rollups.get("qwe").unwrap();
        assert_eq!(2, l1r_qwe.total_lines);
        assert_eq!(4, l1r_qwe.total_units);
        assert_eq!("qwe-source", l1r_qwe.total_amount.source);
        assert_eq!(Decimal::from_str("9.36").unwrap(), l1r_qwe.total_amount.amount);

        assert_eq!(2, list_2_rollups.len());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_name() {
        setup(false, false);
        let sort_request = sort(SortKey::Name, false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!("A3 Naming", results[0].name);
        assert_eq!("B1 My Name", results[1].name);
        assert_eq!("C2 Your Name", results[2].name);
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_id_descending() {
        setup(false, false);
        let sort_request = sort(SortKey::Id, true);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(3, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
        assert_eq!(1, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_name_descending() {
        setup(false, false);
        let sort_request = sort(SortKey::Name, true);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!("C2 Your Name", results[0].name);
        assert_eq!("B1 My Name", results[1].name);
        assert_eq!("A3 Naming", results[2].name);
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_create_date() {
        setup(false, false);
        let sort_request = sort(SortKey::CreatedDate, false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].id.unwrap());
        assert_eq!(1, results[1].id.unwrap());
        assert_eq!(3, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_modified_date_descending() {
        setup(false, false);
        let sort_request = sort(SortKey::ModifiedDate, true);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].id.unwrap());
        assert_eq!(3, results[1].id.unwrap());
        assert_eq!(1, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_nonexistent_attribute_descending() {
        setup(false, false);
        let sort_request = sort(SortKey::Attribute("does not exist".to_string()), false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
        assert_eq!(3, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_boolean_attribute_descending() {
        setup(false, true);
        let sort_request = sort(SortKey::Attribute("my boolean".to_string()), true);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].id.unwrap()); // has true
        assert_eq!(3, results[1].id.unwrap()); // has false, id (descending) tie-breaker
        assert_eq!(2, results[2].id.unwrap()); // has false, id (descending) tie-breaker
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_float_attribute() {
        setup(false, true);
        let sort_request = sort(SortKey::Attribute("my float".to_string()), false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(3, results[0].id.unwrap()); // -3.1
        assert_eq!(2, results[1].id.unwrap()); // -2.1
        assert_eq!(1, results[2].id.unwrap()); // -1.1
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_integer_attribute() {
        setup(false, false);
        let sort_request = sort(SortKey::Attribute("my integer".to_string()), false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
        assert_eq!(3, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_price_attribute() {
        setup(false, true);
        let sort_request = sort(SortKey::Attribute("my price".to_string()), false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].id.unwrap());
        assert_eq!(3, results[1].id.unwrap());
        assert_eq!(1, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_text_attribute() {
        setup(false, true);
        let sort_request = sort(SortKey::Attribute("my text".to_string()), false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].id.unwrap()); // archive C2 Your Name
        assert_eq!(3, results[1].id.unwrap()); // default A3 naming
        assert_eq!(1, results[2].id.unwrap()); // default B1 My Name
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_by_date_attribute() {
        setup(false, true);
        let sort_request = sort(SortKey::Attribute("my date".to_string()), false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 10),
            sort_request,
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].id.unwrap());
        assert_eq!(1, results[1].id.unwrap());
        assert_eq!(3, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_with_paging() {
        setup(false, false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(1, 1),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_with_paging_beyond_end() {
        setup(false, false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(3, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(0, results.len());
    }

    #[test]
    #[serial]
    fn test_retrieve_all_lists_with_no_rows_requested() {
        setup(false, false);
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector(),
            paging(0, 0),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(0, results.len());
    }

    #[test]
    #[serial]
    fn test_retrieve_not_deleted_lists_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_show_deleted = false;
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(2, results.len());
        assert_eq!(2, results[0].id.unwrap());
        assert_eq!(3, results[1].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_deleted_lists_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_show_not_deleted = false;
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(1, results.len());
        assert_eq!(1, results[0].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_editable_lists_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_show_read_only = false;
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        //TODO: read-only lists are not implemented yet
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
        assert_eq!(3, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_lists_in_archive_folder_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_in_folders = vec!["archive".to_string()];
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_private_lists_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_list_access = vec![ListAccess::Private];
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_public_or_shared_lists_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_list_access = vec![ListAccess::Public, ListAccess::Shared];
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(3, results[1].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_transient_lists_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_list_types = vec![ListType::Transient];
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(1, results.len());
        assert_eq!(3, results[0].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_standard_or_program_lists_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_list_types = vec![ListType::Standard, ListType::System];
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_lists_with_keyword_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_name_keywords = Some("name".to_string());
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_lists_with_wildcard_keyword_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_name_keywords = Some("nam*".to_string());
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
        assert_eq!(3, results[2].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_lists_with_multiple_keyword_by_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_name_keywords = Some("Nam* c2".to_string());
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].id.unwrap());
    }

    #[test]
    #[serial]
    fn test_retrieve_lists_by_list_id() {
        setup(false, false);
        let mut selector = selector();
        selector.limit_list_ids = vec![1, 2];
        let results = ListOfListsService().retrieve_lists(
            context(user(), state()),
            selector,
            paging(0, 10),
            sort(SortKey::Id, false),
            true,
            true,
        );
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].id.unwrap());
        assert_eq!(2, results[1].id.unwrap());
    }

    fn paging(start: u64, rows: u64) -> PagingRequest {
        PagingRequest { start, rows }
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
            limit_list_ids: vec![],
        }
    }

    fn sort(key: SortKey, descending: bool) -> SortRequest {
        SortRequest { descending, key }
    }

    fn state() -> UserState {
        UserState {
            active_user_accounts: user().user_accounts,
            user_id: user().id.unwrap(),
        }
    }

    fn user() -> User {
        User {
            id: Some(1),
            name: "One Name".to_string(),
            source: "user-source".to_string(),
            source_id: "ONE-ID".to_string(),
            user_accounts: vec![],
        }
    }

    fn users() -> Vec<User> {
        vec![user()]
    }

    fn setup(need_items: bool, need_attributes: bool) {
        let mut c = db::connection();
        c.run_pending_migrations(MIGRATIONS)
            .expect("Could not run migrations");
        crate::test_helpers::cleanup_db(&mut c);

        let _ = sql_query(r#"
        insert into account_type (id, name, source) values (1000, 'AT1', 'AT1 SOURCE')
            "#).execute(&mut c);
        let _ = sql_query(r#"
        insert into account (id, account_type_id, account_source_id) values (100, 1000, 'AT1-ZERO')
            "#).execute(&mut c);

        let _ = sql_query(r#"
        insert into USER (id, name, source, source_id) values (1, 'One Name', 'user-source', 'ONE-ID')
            "#).execute(&mut c).unwrap();

        let _ = sql_query(r#"
        insert into item_list (id, owner_user_id, created, deleted, folder, access, list_type, name, modified)
        values (1, 1, '2024-07-20 00:00:00.000', true, 'default', 'Public', 'Standard', 'B1 My Name', '2024-07-19 00:00:00.000')
            "#).execute(&mut c).unwrap();
        let _ = sql_query(r#"
        insert into item_list (id, owner_user_id, created, deleted, folder, access, list_type, name, modified)
        values (3, 1, '2024-07-21 00:00:00.000', false, 'default', 'Shared', 'Transient', 'A3 Naming', '2024-07-20 00:00:00.000')
            "#).execute(&mut c).unwrap();
        let _ = sql_query(r#"
        insert into item_list (id, owner_user_id, created, deleted, folder, access, list_type, name, modified)
        values (2, 1, '2024-07-19 00:00:00.000', false, 'archive', 'Private', 'System', 'C2 Your Name', '2024-07-21 00:00:00.000')
            "#).execute(&mut c).unwrap();

        let _ = sql_query(r#"
        insert into item_list_account (item_list_id, account_id) values (1, 100)
            "#).execute(&mut c).unwrap();
        let _ = sql_query(r#"
        insert into item_list_account (item_list_id, account_id) values (2, 100)
            "#).execute(&mut c).unwrap();
        let _ = sql_query(r#"
        insert into item_list_account (item_list_id, account_id) values (3, 100)
            "#).execute(&mut c).unwrap();

        if need_attributes {
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, text_val)
            values (11, 1, 'my price', 'Price', 'PRICE: _3.33 _a-source')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, text_val)
            values (21, 2, 'my price', 'Price', 'PRICE: _1.11 _a-source')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, text_val)
            values (31, 3, 'my price', 'Price', 'PRICE: _2.22 _a-source')
                "#).execute(&mut c).unwrap();

            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, bool_val)
            values (12, 1, 'my boolean', 'Boolean', true)
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, bool_val)
            values (22, 2, 'my boolean', 'Boolean', false)
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, bool_val)
            values (32, 3, 'my boolean', 'Boolean', false)
                "#).execute(&mut c).unwrap();

            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, timestamp_val)
            values (13, 1, 'my date', 'DateTime', '2024-07-20 00:00:00.000')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, timestamp_val)
            values (23, 2, 'my date', 'DateTime', '2024-07-19 00:00:00.000')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, timestamp_val)
            values (33, 3, 'my date', 'DateTime', '2024-07-21 00:00:00.000')
                "#).execute(&mut c).unwrap();

            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, float_val)
            values (14, 1, 'my float', 'Float', -0.1)
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, float_val)
            values (24, 2, 'my float', 'Float', -0.1)
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, float_val)
            values (34, 3, 'my float', 'Float', -0.1)
                "#).execute(&mut c).unwrap();

            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, integer_val)
            values (15, 1, 'my integer', 'Integer', 1)
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, integer_val)
            values (25, 2, 'my integer', 'Integer', 1)
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, integer_val)
            values (35, 3, 'my integer', 'Integer', 1)
                "#).execute(&mut c).unwrap();

            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, text_val)
            values (16, 1, 'my text', 'Text', 'default B1 My Name')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, text_val)
            values (26, 2, 'my text', 'Text', 'archive C2 Your Name')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into item_list_attribute (id, item_list_id, name, type, text_val)
            values (36, 3, 'my text', 'Text', 'default A3 Naming')
                "#).execute(&mut c).unwrap();
        }
        if need_items {
            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (101, 1, 'B1 My Name item one', 'a-source')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (201, 1, 'B1 My Name item two', 'a-source')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (301, 1, 'B1 My Name item three', 'a-source')
                "#).execute(&mut c).unwrap();

            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (102, 2, 'C2 Your Name item one', 'a-source')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (202, 2, 'C2 Your Name item two', 'a-source')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (302, 2, 'C2 Your Name item three', 'a-source')
                "#).execute(&mut c).unwrap();

            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (103, 3, 'A3 Naming item one', 'a-source')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (203, 3, 'A3 Naming item two', 'a-source')
                "#).execute(&mut c).unwrap();
            let _ = sql_query(r#"
            insert into list_item (id, item_list_id, name, source)
            values (303, 3, 'A3 Naming item three', 'a-source')
                "#).execute(&mut c).unwrap();

            if need_attributes {
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (1011, 101, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (1021, 102, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (1031, 103, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (2011, 201, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (2021, 202, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (2031, 203, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (3011, 301, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (3021, 302, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, integer_val)
                values (3031, 303, 'quantity', 'Integer', 2)
                    "#).execute(&mut c).unwrap();

                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (1012, 102, 'xyz', 'Price', 'PRICE: _1.11 _xyz-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (1022, 202, 'xyz', 'Price', 'PRICE: _1.11 _xyz-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (1032, 302, 'xyz', 'Price', 'PRICE: _1.11 _xyz-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (2012, 101, 'xyz', 'Price', 'PRICE: _3.33 _xyz-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (2022, 201, 'xyz', 'Price', 'PRICE: _3.33 _xyz-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (2032, 301, 'xyz', 'Price', 'PRICE: _3.33 _xyz-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (3012, 103, 'xyz', 'Price', 'PRICE: _2.22 _xyz-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (3022, 203, 'xyz', 'Price', 'PRICE: _2.22 _xyz-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (3032, 303, 'xyz', 'Price', 'PRICE: _2.22 _xyz-source')
                    "#).execute(&mut c).unwrap();

                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (2013, 201, 'qwe', 'Price', 'PRICE: _2.34 _qwe-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (3013, 301, 'qwe', 'Price', 'PRICE: _2.34 _qwe-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (2023, 202, 'qwe', 'Price', 'PRICE: _2.34 _qwe-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (3023, 302, 'qwe', 'Price', 'PRICE: _2.34 _qwe-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (2033, 203, 'qwe', 'Price', 'PRICE: _2.34 _qwe-source')
                    "#).execute(&mut c).unwrap();
                let _ = sql_query(r#"
                insert into list_item_attribute (id, list_item_id, name, type, text_val)
                values (3033, 303, 'qwe', 'Price', 'PRICE: _2.34 _qwe-source')
                    "#).execute(&mut c).unwrap();
            }
        }
    }
}
