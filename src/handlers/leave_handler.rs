use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use serde::Deserialize;

use crate::config::app_state::AppState;
use crate::models::leave::{LeaveRequestPayload, UpdateLeaveStatusRequest};
use crate::view_models::leave_vm::LeaveViewModel;

#[derive(Deserialize)]
pub struct LeaveQuery {
    pub nik: Option<String>,
}

pub async fn submit_leave(
    req: web::Json<LeaveRequestPayload>,
    data: web::Data<AppState>,
) -> impl Responder {
    match LeaveViewModel::submit_leave(req, &data).await {
        Ok(msg) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": e
        })),
    }
}

pub async fn get_leaves(
    query: web::Query<LeaveQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    if let Some(nik) = &query.nik {
        match LeaveViewModel::get_leaves(nik.clone(), &data).await {
            Ok(leaves) => HttpResponse::Ok().json(json!({
                "status": "success",
                "data": leaves
            })),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            })),
        }
    } else {
        match data.db.query("SELECT * FROM leaves ORDER BY created_at DESC").await {
            Ok(mut result) => {
                let leaves: Vec<crate::models::leave::LeaveRecord> = result.take(0).unwrap_or_default();
                HttpResponse::Ok().json(json!({
                    "status": "success",
                    "data": leaves
                }))
            }
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e.to_string()
            })),
        }
    }
}

pub async fn update_leave_status(
    req: web::Json<UpdateLeaveStatusRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match LeaveViewModel::update_status(req.id.clone(), req.stage, req.status.clone(), &data).await {
        Ok(msg) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": e
        })),
    }
}
