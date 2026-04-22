use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::config::app_state::AppState;
use crate::models::dashboard::DateRangeRequest;
use crate::view_models::dashboard_vm::DashboardViewModel;

pub async fn get_dashboard_analytics(
    query: web::Query<DateRangeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match DashboardViewModel::get_dashboard_analytics(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(dashboard_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": dashboard_data
        })),
        Err(e) => {
            log::error!("Error getting dashboard analytics: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_overview_only(
    query: web::Query<DateRangeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match DashboardViewModel::get_dashboard_analytics(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(dashboard_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "overview": dashboard_data.overview
            }
        })),
        Err(e) => {
            log::error!("Error getting overview: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_attendance_analytics(
    query: web::Query<DateRangeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match DashboardViewModel::get_dashboard_analytics(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(dashboard_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "attendance_analytics": dashboard_data.attendance_analytics
            }
        })),
        Err(e) => {
            log::error!("Error getting attendance analytics: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_patrol_analytics(
    query: web::Query<DateRangeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match DashboardViewModel::get_dashboard_analytics(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(dashboard_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "patrol_analytics": dashboard_data.patrol_analytics
            }
        })),
        Err(e) => {
            log::error!("Error getting patrol analytics: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_incident_analytics(
    query: web::Query<DateRangeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match DashboardViewModel::get_dashboard_analytics(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(dashboard_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "incident_analytics": dashboard_data.incident_analytics
            }
        })),
        Err(e) => {
            log::error!("Error getting incident analytics: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_performance_analytics(
    query: web::Query<DateRangeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match DashboardViewModel::get_dashboard_analytics(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(dashboard_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "performance_analytics": dashboard_data.performance_analytics
            }
        })),
        Err(e) => {
            log::error!("Error getting performance analytics: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}

pub async fn get_location_analytics(
    query: web::Query<DateRangeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match DashboardViewModel::get_dashboard_analytics(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(dashboard_data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "location_analytics": dashboard_data.location_analytics
            }
        })),
        Err(e) => {
            log::error!("Error getting location analytics: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": e
            }))
        }
    }
}