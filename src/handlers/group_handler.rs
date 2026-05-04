use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::config::app_state::AppState;
use crate::models::group::{CreateGroupRequest, UpdateGroupRequest};
use crate::view_models::group_vm::GroupViewModel;

pub async fn create_group(
    req: web::Json<CreateGroupRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match GroupViewModel::create_group(req, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": res
        })),
        Err(e) => {
            log::error!("Error creating group: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_all_groups(data: web::Data<AppState>) -> impl Responder {
    match GroupViewModel::get_all_groups(&data).await {
        Ok(groups) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": groups
        })),
        Err(e) => {
            log::error!("Error getting groups: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_group(id: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    match GroupViewModel::get_group_by_id(&id, &data).await {
        Ok(group) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": group
        })),
        Err(e) => {
            log::error!("Error getting group: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn update_group(
    id: web::Path<String>,
    req: web::Json<UpdateGroupRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match GroupViewModel::update_group(&id, req, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": res
        })),
        Err(e) => {
            log::error!("Error updating group: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn delete_group(id: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    match GroupViewModel::delete_group(&id, &data).await {
        Ok(res) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": res
        })),
        Err(e) => {
            log::error!("Error deleting group: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}
