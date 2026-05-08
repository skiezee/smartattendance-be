use actix_web::{web, HttpResponse, Responder};
use crate::config::app_state::AppState;
use crate::models::employee::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, RefreshTokenRequest};
use crate::view_models::auth_vm::AuthViewModel;
use crate::utils::jwt;

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
                    token: None,
                    nik: None,
                    name: None,
                    role: None,
                })
            } else {
                HttpResponse::InternalServerError().json(LoginResponse {
                    status: "error".to_string(),
                    message: e,
                    token: None,
                    nik: None,
                    name: None,
                    role: None,
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

pub async fn refresh_token(
    req: web::Json<RefreshTokenRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Verify the old token first
    match jwt::verify_token(&req.token) {
        Ok(claims) => {
            // Token is valid, now check if user still exists in database
            let result = data
                .db
                .query("SELECT * FROM employee WHERE type::string(nik) = type::string($nik)")
                .bind(("nik", claims.sub.clone()))
                .await;

            match result {
                Ok(mut res) => {
                    let employees: Vec<crate::models::employee::Employee> = res.take(0).unwrap_or_default();
                    
                    if let Some(employee) = employees.first() {
                        // Generate new token
                        match jwt::generate_token(
                            employee.nik.clone(),
                            employee.full_name.clone(),
                            employee.role.clone(),
                        ) {
                            Ok(new_token) => {
                                log::info!("Token refreshed for NIK: {}", employee.nik);
                                HttpResponse::Ok().json(LoginResponse {
                                    status: "success".to_string(),
                                    message: "Token refreshed successfully".to_string(),
                                    token: Some(new_token),
                                    nik: Some(employee.nik.clone()),
                                    name: Some(employee.full_name.clone()),
                                    role: Some(employee.role.clone()),
                                })
                            }
                            Err(e) => {
                                log::error!("Failed to generate new token: {}", e);
                                HttpResponse::InternalServerError().json(LoginResponse {
                                    status: "error".to_string(),
                                    message: "Failed to generate new token".to_string(),
                                    token: None,
                                    nik: None,
                                    name: None,
                                    role: None,
                                })
                            }
                        }
                    } else {
                        log::warn!("User not found for token refresh: {}", claims.sub);
                        HttpResponse::Unauthorized().json(LoginResponse {
                            status: "error".to_string(),
                            message: "User not found".to_string(),
                            token: None,
                            nik: None,
                            name: None,
                            role: None,
                        })
                    }
                }
                Err(e) => {
                    log::error!("Database error during token refresh: {}", e);
                    HttpResponse::InternalServerError().json(LoginResponse {
                        status: "error".to_string(),
                        message: "Internal server error".to_string(),
                        token: None,
                        nik: None,
                        name: None,
                        role: None,
                    })
                }
            }
        }
        Err(e) => {
            log::warn!("Invalid token for refresh: {:?}", e);
            HttpResponse::Unauthorized().json(LoginResponse {
                status: "error".to_string(),
                message: "Invalid or expired token".to_string(),
                token: None,
                nik: None,
                name: None,
                role: None,
            })
        }
    }
}
