
use actix_web::{HttpRequest, HttpResponse};

use crate::common::{ListStorage, LMContext, PagingRequest, SortKey, SortRequest, User, UserState, UserStorage};
use crate::list_of_lists_service::{ListSelector, retrieve_lists};
use crate::list_storage::DatabaseListStorage;
use crate::user_storage::DatabaseUserStorage;

pub async fn list_of_lists(req: HttpRequest) -> HttpResponse {
    let user_id: u64 = req.headers().get("user_id").unwrap()
        .to_str().unwrap().parse().unwrap();

    let context = Context {
        user_state: UserState { active_user_accounts: vec![], user_id },
    };

    let selector = ListSelector {
        limit_show_read_only: false,
        limit_list_types: vec![],
        limit_list_access: vec![],
        limit_show_deleted: true,
        limit_show_not_deleted: true,
        limit_in_folders: vec![],
        limit_name_keywords: None,
        limit_list_ids: vec![],
    };

    let paging = PagingRequest {
        start: 0,
        rows: 10,
    };

    let sort = SortRequest {
        descending: false,
        key: SortKey::Id,
    };

    let a = retrieve_lists(context, selector, paging, sort, true, true);

    HttpResponse::Ok().body(serde_json::to_string(&a).unwrap())
}

struct Context {
    user_state: UserState,
}

impl LMContext for Context {
    fn list_storage(self) -> impl ListStorage {
        DatabaseListStorage()
    }

    fn user_storage(self) -> impl UserStorage {
        DatabaseUserStorage()
    }

    fn current_user(self) -> (User, Self) {
        let (state, self1) = self.current_user_state();
        let u = DatabaseUserStorage().retrieve_user_by_id(&state.user_id).unwrap().1;
        (u, self1)
    }

    fn current_user_state(self) -> (UserState, Self) {
        (self.user_state.clone(), self)
    }
}