use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::config::app_state::AppState;
use crate::models::shift::{CreateShiftRequest, GetShiftRequest, UpdateShiftStatusRequest};
use crate::view_models::shift_vm::ShiftViewModel;

pub async fn create_shift(
    req: web::Json<CreateShiftRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftViewModel::create_shift(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => {
            log::error!("Error creating shift: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_shifts(
    req: web::Json<GetShiftRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftViewModel::get_shifts_by_nik(
        &req.nik,
        req.start_date.clone(),
        req.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(shifts) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": shifts
        })),
        Err(e) => {
            log::error!("Error getting shifts for NIK {}: {}", req.nik, e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_all_shifts(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let start_date = query.get("start_date").cloned();
    let end_date = query.get("end_date").cloned();

    match ShiftViewModel::get_all_shifts(start_date, end_date, &data).await {
        Ok(shifts) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": shifts
        })),
        Err(e) => {
            log::error!("Error getting all shifts: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn update_shift_status(
    req: web::Json<UpdateShiftStatusRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftViewModel::update_shift_status(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => {
            log::error!("Error updating shift status: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_shift_stats(nik: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    match ShiftViewModel::get_shift_stats(&nik, &data).await {
        Ok(stats) => HttpResponse::Ok().json(json!({
            "status": "success",
            "stats": stats
        })),
        Err(e) => {
            log::error!("Error getting shift stats for NIK {}: {}", nik, e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn delete_shift(
    shift_id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftViewModel::delete_shift(&shift_id, &data).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => {
            log::error!("Error deleting shift {}: {}", shift_id, e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}
