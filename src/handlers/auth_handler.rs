use actix_web::{web, HttpResponse, Responder};
use crate::config::app_state::AppState;
use crate::models::employee::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use crate::view_models::auth_vm::AuthViewModel;

pub async fn login(
    req: web::Json<LoginRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match AuthViewModel::login(req, data).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            if e == "Invalid NIK or password" {
                HttpResponse::Unauthorized().json(LoginResponse {
                    status: "error".to_string(),
                    message: e,
                })
            } else {
                HttpResponse::InternalServerError().json(LoginResponse {
                    status: "error".to_string(),
                    message: e,
                })
            }
        }
    }
}

pub async fn register(
    req: web::Json<RegisterRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match AuthViewModel::register(req, data).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            if e == "NIK or Email already exists" {
                HttpResponse::Conflict().json(RegisterResponse {
                    status: "error".to_string(),
                    message: e,
                })
            } else {
                HttpResponse::InternalServerError().json(RegisterResponse {
                    status: "error".to_string(),
                    message: e,
                })
            }
        }
    }
}
