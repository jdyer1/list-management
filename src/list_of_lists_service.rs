use std::cmp::Ordering;
use std::collections::HashMap;

use chrono::FixedOffset;
use currency_rs::Currency;

use crate::common::{ATTRIBUTE_QUANTITY, ItemList, ItemListRollup, ListAccess, ListAttribute, ListItem, ListStorage, ListType, LMContext, PagingRequest, Price, SortKey, SortRequest};
use crate::common::ListAttribute::DateTime;

pub struct ListSelector {
    limit_show_read_only: bool,
    limit_list_types: Vec<ListType>,
    limit_list_access: Vec<ListAccess>,
    limit_show_deleted: bool,
    limit_show_not_deleted: bool,
    limit_in_folders: Vec<String>,
    limit_name_keywords: Option<String>,
}

pub struct ListResult {
    list: ItemList,
    rollups: HashMap<String, ItemListRollup>,
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
    let a = sort_list_of_lists(a, sort);
    let mut i: usize = 0;
    let mut a1: Vec<ListResult> = Vec::new();
    for mut item_list in a {
        let mut include: bool = i >= start;
        include = include && (selector.limit_show_not_deleted || item_list.deleted);
        include = include && (selector.limit_show_deleted || !item_list.deleted);
        include = include && (selector.limit_show_read_only || !item_list.read_only);
        include = include && (selector.limit_list_access.is_empty() || selector.limit_list_access.contains(&item_list.list_access));
        include = include && (selector.limit_list_types.is_empty() || selector.limit_list_types.contains(&item_list.list_type));
        include = include && (selector.limit_in_folders.is_empty() || selector.limit_in_folders.contains(&item_list.folder));
        if include && selector.limit_name_keywords.is_some() {
            let name_tokens: Vec<String> = item_list.name.split_whitespace().map(|a| a.to_ascii_lowercase()).collect();
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
            if !return_attributes {
                item_list.attributes = HashMap::with_capacity(0);
            }
            let rollups: HashMap<String, ItemListRollup> = compute_rollup_values(return_rollups, &item_list.items);
            a1.push(ListResult {
                list: item_list,
                rollups,
            });
        }
        i = i + 1;
        if i == end {
            break;
        }
    }
    return a1;
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
                                ordering = Some(v1.cmp(v2));
                            }
                        }
                        DateTime(v1) => {
                            if let DateTime(v2) = two_attribute_value {
                                ordering = Some(v1.cmp(v2));
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
                                ordering = Some(v1.cmp(v2));
                            }
                        }
                        ListAttribute::Price(v1) => {
                            if let ListAttribute::Price(v2) = two_attribute_value {
                                let value1 = v1.amount.value();
                                let value2 = v2.amount.value();
                                if value1.min(value2) == value1 {
                                    return Ordering::Less;
                                }
                                return Ordering::Greater;
                            }
                        }
                        ListAttribute::Text(v1) => {
                            if let ListAttribute::Text(v2) = two_attribute_value {
                                ordering = Some(v1.cmp(v2));
                            }
                        }
                    }
                    if ordering.is_some() {
                        return ordering.unwrap();
                    }
                }
                one.id.cmp(&two.id)
            }
            SortKey::CreatedDate => { one.created.cmp(&two.created) }
            SortKey::Id => one.id.cmp(&two.id),
            SortKey::ModifiedDate => { one.modified.cmp(&two.modified) }
            SortKey::Name => one.name.cmp(&two.name),
        }
    });
    return a;
}

fn compute_rollup_values(return_rollups: bool, items: &Vec<ListItem>) -> HashMap<String, ItemListRollup> {
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
                    let mut ilr_price = price.clone();
                    ilr_price.amount = ilr_price.amount * qty as f64;

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
                        let currency1 = price1.amount.add(ilr_price.amount.value());
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
        return rollups;
    }
    return HashMap::with_capacity(0);
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{DateTime, FixedOffset};
    use currency_rs::Currency;

    use crate::common::*;

    use super::*;

    #[test]
    fn test_retrieve_all_lists_by_id() {
        let sort_request = sort(SortKey::Id, false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, false, false);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(3, results[2].list.id);
        assert!(results[0].list.attributes.is_empty());
        assert!(results[1].list.attributes.is_empty());
        assert!(results[2].list.attributes.is_empty());
        assert!(results[0].rollups.is_empty());
        assert!(results[1].rollups.is_empty());
        assert!(results[2].rollups.is_empty());
    }

    #[test]
    fn test_retrieve_all_lists_by_id_with_attributes_and_rollups() {
        let sort_request = sort(SortKey::Id, false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(3, results[2].list.id);
        assert!(!results[0].list.attributes.is_empty());
        assert!(!results[1].list.attributes.is_empty());
        assert!(!results[2].list.attributes.is_empty());
        let list_0_rollups = results[0].rollups.clone();
        let list_1_rollups = results[1].rollups.clone();
        let list_2_rollups = results[2].rollups.clone();
        assert_eq!(2, list_0_rollups.len());

        let l0r_xyz = list_0_rollups.get("xyz").unwrap();
        assert_eq!(3, l0r_xyz.total_lines);
        assert_eq!(6, l0r_xyz.total_units);
        assert_eq!("xyz-source", l0r_xyz.total_amount.source);
        assert_eq!(19.98, l0r_xyz.total_amount.amount.value());

        let l0r_qwe = list_0_rollups.get("qwe").unwrap();
        assert_eq!(2, l0r_qwe.total_lines);
        assert_eq!(4, l0r_qwe.total_units);
        assert_eq!("qwe-source", l0r_qwe.total_amount.source);
        assert_eq!(9.36, l0r_qwe.total_amount.amount.value());

        assert_eq!(2, list_1_rollups.len());

        let l1r_xyz = list_1_rollups.get("xyz").unwrap();
        assert_eq!(3, l1r_xyz.total_lines);
        assert_eq!(6, l1r_xyz.total_units);
        assert_eq!("xyz-source", l1r_xyz.total_amount.source);
        assert_eq!(6.66, l1r_xyz.total_amount.amount.value());

        let l1r_qwe = list_1_rollups.get("qwe").unwrap();
        assert_eq!(2, l1r_qwe.total_lines);
        assert_eq!(4, l1r_qwe.total_units);
        assert_eq!("qwe-source", l1r_qwe.total_amount.source);
        assert_eq!(9.36, l1r_qwe.total_amount.amount.value());

        assert_eq!(2, list_2_rollups.len());
    }

    #[test]
    fn test_retrieve_all_lists_by_name() {
        let sort_request = sort(SortKey::Name, false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!("A3 Naming", results[0].list.name);
        assert_eq!("B1 My Name", results[1].list.name);
        assert_eq!("C2 Your Name", results[2].list.name);
    }

    #[test]
    fn test_retrieve_all_lists_by_id_descending() {
        let sort_request = sort(SortKey::Id, true);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(3, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(1, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_name_descending() {
        let sort_request = sort(SortKey::Name, true);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!("C2 Your Name", results[0].list.name);
        assert_eq!("B1 My Name", results[1].list.name);
        assert_eq!("A3 Naming", results[2].list.name);
    }

    #[test]
    fn test_retrieve_all_lists_by_create_date() {
        let sort_request = sort(SortKey::CreatedDate, false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].list.id);
        assert_eq!(1, results[1].list.id);
        assert_eq!(3, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_modified_date_descending() {
        let sort_request = sort(SortKey::ModifiedDate, true);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].list.id);
        assert_eq!(3, results[1].list.id);
        assert_eq!(1, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_nonexistent_attribute_descending() {
        let sort_request = sort(SortKey::Attribute("does not exist".to_string()), false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(3, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_boolean_attribute_descending() {
        let sort_request = sort(SortKey::Attribute("my boolean".to_string()), true);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id); // has true
        assert_eq!(3, results[1].list.id); // has false, id (descending) tie-breaker
        assert_eq!(2, results[2].list.id); // has false, id (descending) tie-breaker
    }

    #[test]
    fn test_retrieve_all_lists_by_float_attribute() {
        let sort_request = sort(SortKey::Attribute("my float".to_string()), false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(3, results[0].list.id);  // -3.1
        assert_eq!(2, results[1].list.id);  // -2.1
        assert_eq!(1, results[2].list.id);  // -1.1
    }

    #[test]
    fn test_retrieve_all_lists_by_integer_attribute() {
        let sort_request = sort(SortKey::Attribute("my integer".to_string()), false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(3, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_price_attribute() {
        let sort_request = sort(SortKey::Attribute("my price".to_string()), false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].list.id);
        assert_eq!(3, results[1].list.id);
        assert_eq!(1, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_by_text_attribute() {
        let sort_request = sort(SortKey::Attribute("my text".to_string()), false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].list.id); // archive C2 Your Name
        assert_eq!(3, results[1].list.id); // default A3 naming
        assert_eq!(1, results[2].list.id); // default B1 My Name
    }

    #[test]
    fn test_retrieve_all_lists_by_date_attribute() {
        let sort_request = sort(SortKey::Attribute("my date".to_string()), false);
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 10), sort_request, true, true);
        assert_eq!(3, results.len());
        assert_eq!(2, results[0].list.id);
        assert_eq!(1, results[1].list.id);
        assert_eq!(3, results[2].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_with_paging() {
        let results = retrieve_lists(context(item_lists()), selector(), paging(1, 1), sort(SortKey::Id, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].list.id);
    }

    #[test]
    fn test_retrieve_all_lists_with_paging_beyond_end() {
        let results = retrieve_lists(context(item_lists()), selector(), paging(3, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(0, results.len());
    }

    #[test]
    fn test_retrieve_all_lists_with_no_rows_requested() {
        let results = retrieve_lists(context(item_lists()), selector(), paging(0, 0), sort(SortKey::Id, false), true, true);
        assert_eq!(0, results.len());
    }

    #[test]
    fn test_retrieve_not_deleted_lists_by_id() {
        let mut selector = selector();
        selector.limit_show_deleted = false;
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(2, results[0].list.id);
        assert_eq!(3, results[1].list.id);
    }

    #[test]
    fn test_retrieve_deleted_lists_by_id() {
        let mut selector = selector();
        selector.limit_show_not_deleted = false;
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(1, results[0].list.id);
    }

    #[test]
    fn test_retrieve_editable_lists_by_id() {
        let mut selector = selector();
        selector.limit_show_read_only = false;
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
    }

    #[test]
    fn test_retrieve_lists_in_archive_folder_by_id() {
        let mut selector = selector();
        selector.limit_in_folders = vec!["archive".to_string()];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].list.id);
    }

    #[test]
    fn test_retrieve_private_lists_by_id() {
        let mut selector = selector();
        selector.limit_list_access = vec![ListAccess::Private];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(2, results[0].list.id);
    }

    #[test]
    fn test_retrieve_public_or_shared_lists_by_id() {
        let mut selector = selector();
        selector.limit_list_access = vec![ListAccess::Public, ListAccess::Shared];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(3, results[1].list.id);
    }

    #[test]
    fn test_retrieve_transient_lists_by_id() {
        let mut selector = selector();
        selector.limit_list_types = vec![ListType::Transient];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(1, results.len());
        assert_eq!(3, results[0].list.id);
    }

    #[test]
    fn test_retrieve_standard_or_program_lists_by_id() {
        let mut selector = selector();
        selector.limit_list_types = vec![ListType::Standard, ListType::System];
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
    }

    #[test]
    fn test_retrieve_lists_with_keyword_by_id() {
        let mut selector = selector();
        selector.limit_name_keywords = Some("name".to_string());
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(2, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
    }

    #[test]
    fn test_retrieve_lists_with_wildcard_keyword_by_id() {
        let mut selector = selector();
        selector.limit_name_keywords = Some("nam*".to_string());
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
        assert_eq!(3, results.len());
        assert_eq!(1, results[0].list.id);
        assert_eq!(2, results[1].list.id);
        assert_eq!(3, results[2].list.id);
    }

    #[test]
    fn test_retrieve_lists_with_multiple_keyword_by_id() {
        let mut selector = selector();
        selector.limit_name_keywords = Some("Nam* c2".to_string());
        let results = retrieve_lists(context(item_lists()), selector, paging(0, 10), sort(SortKey::Id, false), true, true);
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
            item_list(1, "B1 My Name".to_string(), "default".to_string(), true, false, ListAccess::Public, ListType::Standard, d2, d1, "3.33".to_string()),
            item_list(3, "A3 Naming".to_string(), "default".to_string(), false, true, ListAccess::Shared, ListType::Transient, d3, d2, "2.22".to_string()),
            item_list(2, "C2 Your Name".to_string(), "archive".to_string(), false, false, ListAccess::Private, ListType::System, d1, d3, "1.11".to_string()),
        ]
    }

    fn item_list(id: u64, name: String, folder: String, deleted: bool, read_only: bool,
                 list_access: ListAccess, list_type: ListType, created: DateTime<FixedOffset>,
                 modified: DateTime<FixedOffset>, price: String) -> ItemList {
        let mut attributes: HashMap<String, ListAttribute> = HashMap::new();
        attributes.insert("my boolean".to_string(), ListAttribute::Boolean(deleted));
        attributes.insert("my date".to_string(), ListAttribute::DateTime(created));
        attributes.insert("my float".to_string(), ListAttribute::Float(id as f64 * -0.1f64)); // ex: 1 becomes -1.1
        attributes.insert("my integer".to_string(), ListAttribute::Integer(id as i64));
        attributes.insert("my price".to_string(), ListAttribute::Price(Price { //ex: 1 becomes 1.21
            amount: Currency::new_string(&price, None).unwrap(),
            source: "a-source".to_string(),
        }));
        attributes.insert("my text".to_string(), ListAttribute::Text(folder.clone() + " " + &name));

        let mut list_item_attributes1: HashMap<String, ListAttribute> = HashMap::new();
        list_item_attributes1.insert(ATTRIBUTE_QUANTITY.to_string(), ListAttribute::Integer(2));
        list_item_attributes1.insert("xyz".to_string(), ListAttribute::Price(Price {
            amount: Currency::new_string(&price, None).unwrap(),
            source: "xyz-source".to_string(),
        }));
        let mut list_item_attributes2: HashMap<String, ListAttribute> = list_item_attributes1.clone();
        list_item_attributes2.insert("qwe".to_string(), ListAttribute::Price(Price {
            amount: Currency::new_string("2.34", None).unwrap(),
            source: "qwe-source".to_string(),
        }));

        let i1 = ListItem {
            id: 100 + id,
            attributes: list_item_attributes1.clone(),
            created,
            modified,
            name: name.clone() + " item one",
            source: "a-source".to_string(),
        };
        let i2 = ListItem {
            id: 200 + id,
            attributes: list_item_attributes2.clone(),
            created,
            modified,
            name: name.clone() + " item two",
            source: "a-source".to_string(),
        };
        let i3 = ListItem {
            id: 300 + id,
            attributes: list_item_attributes2.clone(),
            created,
            modified,
            name: name.clone() + " item three",
            source: "a-source".to_string(),
        };
        let mut items: Vec<ListItem> = vec![i1, i2, i3];


        ItemList {
            id,
            attributes,
            read_only,
            created,
            deleted,
            folder,
            items,
            list_access,
            list_type,
            modified,
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
