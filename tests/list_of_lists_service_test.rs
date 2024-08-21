use actix_web::{App, test};
use actix_web::http::StatusCode;
use list_management::list_of_lists_service::ListResult;
use list_management::route_config;
use list_management::common::ListAccess;
use list_management::test_setup_db::{insert_account, insert_account_type, insert_user, setup_db, setup_lists};


#[actix_web::test]
async fn test_list_of_lists() {
    let user_id = setup();

    let app = test::init_service(
        App::new()
            .configure(route_config::config)
    ).await;

    let req = test::TestRequest::get()
        .uri("/list_of_lists")
        .insert_header(("user_id", user_id))
        .to_request();
    let service_response = test::call_service(&app, req).await;
    assert_eq!(service_response.status(), StatusCode::OK);
    let lr: Vec<ListResult> = test::read_body_json(service_response).await;
    assert_eq!(2, lr.len());
    assert_eq!("Item List One", lr[0].list.name);
    assert_eq!("Item List Two", lr[1].list.name);
    assert_eq!(ListAccess::Public, lr[0].list.list_access);
}

fn setup() -> i32 {
    setup_db();
    let at1_id =  insert_account_type(
        "at1".to_string(),
        "ats1".to_string(),
    );
    let a1_id =  insert_account(at1_id, "as1".to_string());
    let u1_id = insert_user("User One", "s1", "s1-1");
    setup_lists(vec![a1_id], vec![a1_id], u1_id, u1_id);
    u1_id
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_nothing() {
        let me = "make my IDE recognize this as a test mod!";
        assert_ne!("not testing", me);
    }
}