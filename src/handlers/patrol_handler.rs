use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::config::app_state::AppState;
use crate::models::patrol::PatrolIncidentRequest;
use crate::view_models::patrol_vm::PatrolViewModel;

pub async fn submit_incident(
    req: web::Json<PatrolIncidentRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::submit_incident(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => {
            log::error!("Error submitting incident: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_all_incidents(data: web::Data<AppState>) -> impl Responder {
    match PatrolViewModel::get_all_incidents(&data).await {
        Ok(incidents) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": incidents
        })),
        Err(e) => {
            log::error!("Error getting incidents: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_incidents_by_nik(
    nik: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::get_incidents_by_nik(&nik, &data).await {
        Ok(incidents) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": incidents
        })),
        Err(e) => {
            log::error!("Error getting incidents for NIK {}: {}", nik, e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}
