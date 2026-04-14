use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::config::app_state::AppState;
use crate::view_models::employee_vm::EmployeeViewModel;

pub async fn get_all_employees(data: web::Data<AppState>) -> impl Responder {
    match EmployeeViewModel::get_all_employees(&data).await {
        Ok(employees) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": employees
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": e
        })),
    }
}
