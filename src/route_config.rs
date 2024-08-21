use actix_web::web;

use crate::routes::health_check::health_check;
use crate::routes::list_of_lists::list_of_lists;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/health_check").route(web::get().to(health_check))
    );
    cfg.service(
        web::resource("/list_of_lists").route(web::get().to(list_of_lists))
    );
}