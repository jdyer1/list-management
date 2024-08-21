use actix_web::{App, http::StatusCode, test};
use list_management::route_config;

#[actix_web::test]
async fn test_health_check() {
    let app = test::init_service(
        App::new()
            .configure(route_config::config)
    ).await;
    let req = test::TestRequest::get().uri("/health_check").to_request();
    let service_response = test::call_service(&app, req).await;
    assert_eq!(service_response.status(), StatusCode::OK);
    let body_bytes = test::read_body(service_response).await;
    assert_eq!("OK".to_string(), std::str::from_utf8(&body_bytes).unwrap());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_nothing() {
        let me = "make my IDE recognize this as a test mod!";
        assert_ne!("not testing", me);
    }
}



