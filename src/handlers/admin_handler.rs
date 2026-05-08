use actix_web::{web, HttpResponse, Responder};
use crate::config::app_state::AppState;
use crate::models::admin::{AdminLoginRequest, AdminLoginResponse};
use crate::view_models::admin_vm::AdminViewModel;

pub async fn admin_login(
    req: web::Json<AdminLoginRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match AdminViewModel::login(req, data).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            if e == "Invalid username or password" {
                HttpResponse::Unauthorized().json(AdminLoginResponse {
                    status: "error".to_string(),
                    message: e,
                    token: None,
                    username: None,
                    name: None,
                    role: None,
                })
            } else {
                HttpResponse::InternalServerError().json(AdminLoginResponse {
                    status: "error".to_string(),
                    message: e,
                    token: None,
                    username: None,
                    name: None,
                    role: None,
                })
            }
        }
    }
}
