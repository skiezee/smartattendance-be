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
    match DashboardViewModel::get_overview_only(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(overview) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "overview": overview
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
    match DashboardViewModel::get_attendance_analytics_only(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(attendance_analytics) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "attendance_analytics": attendance_analytics
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
    match DashboardViewModel::get_patrol_analytics_only(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(patrol_analytics) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "patrol_analytics": patrol_analytics
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
    match DashboardViewModel::get_incident_analytics_only(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(incident_analytics) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "incident_analytics": incident_analytics
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
    match DashboardViewModel::get_performance_analytics_only(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(performance_analytics) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "performance_analytics": performance_analytics
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
    match DashboardViewModel::get_location_analytics_only(
        query.start_date.clone(),
        query.end_date.clone(),
        &data,
    )
    .await
    {
        Ok(location_analytics) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": {
                "location_analytics": location_analytics
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