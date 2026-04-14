use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::config::app_state::AppState;
use crate::models::attendance::{
    CheckEnrollmentRequest, ClockInRequest, FaceEnrollRequest, FingerprintEnrollRequest,
};
use crate::view_models::attendance_vm::AttendanceViewModel;

pub async fn check_enrollment(
    req: web::Json<CheckEnrollmentRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match AttendanceViewModel::check_enrollment(req.nik.clone(), &data).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => {
            if e == "Employee not found" {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": e
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": e
                }))
            }
        }
    }
}

pub async fn enroll_face(
    req: web::Json<FaceEnrollRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match AttendanceViewModel::enroll_face(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": e
        })),
    }
}

pub async fn enroll_fingerprint(
    req: web::Json<FingerprintEnrollRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match AttendanceViewModel::enroll_fingerprint(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": e
        })),
    }
}

pub async fn clock_in(
    req: web::Json<ClockInRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match AttendanceViewModel::clock_in(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": e
        })),
    }
}

pub async fn get_all_attendances(data: web::Data<AppState>) -> impl Responder {
    match AttendanceViewModel::get_all_attendances(&data).await {
        Ok(logs) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": logs
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": e
        })),
    }
}
