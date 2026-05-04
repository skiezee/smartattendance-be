use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

use crate::config::app_state::AppState;
use crate::models::patrol::PatrolIncidentRequest;
use crate::view_models::patrol_vm::PatrolViewModel;

#[derive(Deserialize)]
pub struct NikQuery {
    pub nik: Option<String>,
}

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

// --- Area Handlers ---

pub async fn create_area(
    req: web::Json<crate::models::patrol::CreateAreaRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::create_area(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": res
        })),
        Err(e) => {
            log::error!("Error creating area: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_areas(data: web::Data<AppState>) -> impl Responder {
    match PatrolViewModel::get_areas(&data).await {
        Ok(areas) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": areas
        })),
        Err(e) => {
            log::error!("Error getting areas: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn update_area(
    path: web::Path<String>,
    req: web::Json<crate::models::patrol::CreateAreaRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::update_area(&path, req, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": res
        })),
        Err(e) => {
            log::error!("Error updating area: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn delete_area(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::delete_area(&path, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": res
        })),
        Err(e) => {
            log::error!("Error deleting area: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

// --- Checkpoint Handlers ---

pub async fn create_checkpoint(
    req: web::Json<crate::models::patrol::CreateCheckpointRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::create_checkpoint(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": res
        })),
        Err(e) => {
            log::error!("Error creating checkpoint: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_checkpoints(data: web::Data<AppState>) -> impl Responder {
    match PatrolViewModel::get_checkpoints(&data).await {
        Ok(checkpoints) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": checkpoints
        })),
        Err(e) => {
            log::error!("Error getting checkpoints: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn update_checkpoint(
    path: web::Path<String>,
    req: web::Json<crate::models::patrol::UpdateCheckpointRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::update_checkpoint(&path, req, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": res
        })),
        Err(e) => {
            log::error!("Error updating checkpoint: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn delete_checkpoint(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::delete_checkpoint(&path, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": res
        })),
        Err(e) => {
            log::error!("Error deleting checkpoint: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

// --- Patrol Assignment Handlers ---

pub async fn create_patrol_assignment(
    req: web::Json<crate::models::patrol::CreatePatrolAssignmentRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::create_patrol_assignment(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": res
        })),
        Err(e) => {
            log::error!("Error creating patrol assignment: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

/// GET /patrol/assignments          → all assignments (admin)
/// GET /patrol/assignments?nik=xxx  → filtered by employee NIK (mobile)
pub async fn get_patrol_assignments(
    query: web::Query<NikQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    match &query.nik {
        Some(nik) if !nik.is_empty() => {
            // Mobile: return only assignments for this employee
            match PatrolViewModel::get_patrol_assignments_by_nik(nik, &data).await {
                Ok(assignments) => HttpResponse::Ok().json(json!({
                    "status": "success",
                    "data": assignments
                })),
                Err(e) => {
                    log::error!("Error getting assignments for NIK {}: {}", nik, e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": e
                    }))
                }
            }
        }
        _ => {
            // Admin: return all assignments
            match PatrolViewModel::get_patrol_assignments(&data).await {
                Ok(assignments) => HttpResponse::Ok().json(json!({
                    "status": "success",
                    "data": assignments
                })),
                Err(e) => {
                    log::error!("Error getting patrol assignments: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": e
                    }))
                }
            }
        }
    }
}

pub async fn update_patrol_assignment(
    path: web::Path<String>,
    req: web::Json<crate::models::patrol::UpdatePatrolAssignmentRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::update_patrol_assignment(&path, req, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": res
        })),
        Err(e) => {
            log::error!("Error updating patrol assignment: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn delete_patrol_assignment(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match PatrolViewModel::delete_patrol_assignment(&path, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": res
        })),
        Err(e) => {
            log::error!("Error deleting patrol assignment: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

// --- Active Status & Tracking Handlers ---

pub async fn get_active_patrol_status(data: web::Data<AppState>) -> impl Responder {
    match PatrolViewModel::get_active_patrol_status(&data).await {
        Ok(status_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": status_data
        })),
        Err(e) => {
            log::error!("Error getting active patrol status: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_live_tracking(data: web::Data<AppState>) -> impl Responder {
    match PatrolViewModel::get_live_tracking(&data).await {
        Ok(tracking_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": tracking_data
        })),
        Err(e) => {
            log::error!("Error getting live tracking data: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}
