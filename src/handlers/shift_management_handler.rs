use actix_web::{web, HttpResponse, Responder};
use crate::config::app_state::AppState;
use crate::models::shift_management::*;
use crate::view_models::shift_management_vm::ShiftManagementViewModel;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ShiftTaskQuery {
    pub department: Option<String>,
    pub shift_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct ScheduleQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub week_number: Option<i32>,
}

// 1. Shift Task Handlers

pub async fn get_all_shift_tasks(
    query: web::Query<ShiftTaskQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::get_all_shift_tasks(
        query.department.clone(),
        query.shift_type.clone(),
        query.is_active,
        &data
    ).await {
        Ok(tasks) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data: tasks }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}

pub async fn get_shift_task(
    id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::get_shift_task_by_id(&id, &data).await {
        Ok(Some(task)) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data: task }),
        Ok(None) => HttpResponse::NotFound().json(MessageResponse { status: "error".to_string(), message: "Shift task not found".to_string() }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}

pub async fn create_shift_task(
    payload: web::Json<CreateShiftTaskRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::create_shift_task(payload, &data).await {
        Ok(task) => HttpResponse::Created().json(GenericResponse { status: "success".to_string(), data: task }),
        Err(e) => {
            if e.starts_with("Invalid") || e.contains("too long") || e.contains("must be") {
                HttpResponse::BadRequest().json(MessageResponse { status: "error".to_string(), message: e })
            } else {
                HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e })
            }
        }
    }
}

pub async fn update_shift_task(
    id: web::Path<String>,
    payload: web::Json<UpdateShiftTaskRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::update_shift_task(&id, payload, &data).await {
        Ok(task) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data: task }),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(MessageResponse { status: "error".to_string(), message: e })
            } else {
                HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e })
            }
        }
    }
}

pub async fn delete_shift_task(
    id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::delete_shift_task(&id, &data).await {
        Ok(_) => HttpResponse::Ok().json(MessageResponse { status: "success".to_string(), message: "Deleted successfully".to_string() }),
        Err(e) => {
            if e.starts_with("CONFLICT") {
                HttpResponse::Conflict().json(MessageResponse { status: "error".to_string(), message: e })
            } else {
                HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e })
            }
        }
    }
}

// 2. Employee Group Handlers

pub async fn get_employee_groups(
    id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::get_employee_groups(&id, &data).await {
        Ok(data) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}

pub async fn save_employee_groups(
    id: web::Path<String>,
    payload: web::Json<SaveEmployeeGroupsRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::save_employee_groups(&id, payload, &data).await {
        Ok(data) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}

pub async fn get_available_employees(
    id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::get_available_employees(&id, &data).await {
        Ok(data) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}

// 3. Shift Schedule Handlers

pub async fn generate_schedule(
    id: web::Path<String>,
    payload: web::Json<GenerateScheduleRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::generate_schedule(&id, payload, &data).await {
        Ok(data) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}

pub async fn get_schedules(
    id: web::Path<String>,
    query: web::Query<ScheduleQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::get_schedules(
        &id,
        query.start_date.clone(),
        query.end_date.clone(),
        query.week_number,
        &data
    ).await {
        Ok(data) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}

pub async fn update_schedule(
    path: web::Path<(String, String)>,
    payload: web::Json<UpdateRotationScheduleRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (id, date) = path.into_inner();
    match ShiftManagementViewModel::update_schedule(&id, &date, payload, &data).await {
        Ok(data) => HttpResponse::Ok().json(GenericResponse { status: "success".to_string(), data }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}

pub async fn delete_schedules(
    id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match ShiftManagementViewModel::delete_schedules(&id, &data).await {
        Ok(_) => HttpResponse::Ok().json(MessageResponse { status: "success".to_string(), message: "Schedules deleted successfully".to_string() }),
        Err(e) => HttpResponse::InternalServerError().json(MessageResponse { status: "error".to_string(), message: e }),
    }
}
