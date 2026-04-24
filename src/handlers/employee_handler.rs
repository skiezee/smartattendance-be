use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};

use crate::config::app_state::AppState;
use crate::view_models::employee_vm::EmployeeViewModel;
use crate::models::employee::{CreateEmployeeRequest, UpdateEmployeeRequest, EmployeeResponse};

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

pub async fn get_employee_by_nik(
    data: web::Data<AppState>,
    nik: web::Path<String>,
) -> impl Responder {
    let nik_str = nik.into_inner();
    info!("Fetching employee with NIK: {}", nik_str);
    
    let query = "SELECT * FROM employees WHERE nik = $nik LIMIT 1";
    
    match data.db.query(query).bind(("nik", nik_str.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(employees) => {
                    if let Some(employee) = employees.first() {
                        HttpResponse::Ok().json(json!({
                            "status": "success",
                            "data": employee
                        }))
                    } else {
                        HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": "Employee not found"
                        }))
                    }
                }
                Err(e) => {
                    error!("Error parsing employee: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to parse employee data"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database error"
            }))
        }
    }
}

pub async fn create_employee(
    data: web::Data<AppState>,
    req: web::Json<CreateEmployeeRequest>,
) -> impl Responder {
    info!("Creating new employee: {}", req.nik);
    
    // Check if NIK already exists
    let check_query = "SELECT * FROM employees WHERE nik = $nik LIMIT 1";
    match data.db.query(check_query).bind(("nik", req.nik.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(existing) => {
                    if !existing.is_empty() {
                        return HttpResponse::BadRequest().json(json!({
                            "status": "error",
                            "message": format!("Employee with NIK '{}' already exists", req.nik)
                        }));
                    }
                }
                Err(e) => {
                    error!("Error checking existing employee: {:?}", e);
                }
            }
        }
        Err(e) => {
            error!("Database error checking existing employee: {:?}", e);
        }
    }
    
    // Hash password
    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST).unwrap_or_default();
    
    // Create employee
    let query = r#"
        CREATE employees CONTENT {
            nik: $nik,
            full_name: $full_name,
            email: $email,
            password_hash: $password_hash,
            role: $role,
            department: $department,
            status: $status,
            attendance_requirement: $attendance_requirement,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;
    
    match data.db.query(query)
        .bind(("nik", req.nik.clone()))
        .bind(("full_name", req.full_name.clone()))
        .bind(("email", req.email.clone()))
        .bind(("password_hash", password_hash))
        .bind(("role", req.role.clone()))
        .bind(("department", req.department.clone().unwrap_or_else(|| "General".to_string())))
        .bind(("status", req.status.clone().unwrap_or_else(|| "Active".to_string())))
        .bind(("attendance_requirement", req.attendance_requirement.clone()))
        .await
    {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(created) => {
                    if let Some(employee) = created.first() {
                        info!("Successfully created employee: {}", req.nik);
                        HttpResponse::Ok().json(json!({
                            "status": "success",
                            "message": "Employee created successfully",
                            "data": employee
                        }))
                    } else {
                        error!("Employee created but not returned");
                        HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Employee created but not returned"
                        }))
                    }
                }
                Err(e) => {
                    error!("Error parsing created employee: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to create employee"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error creating employee: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database error"
            }))
        }
    }
}

pub async fn update_employee(
    data: web::Data<AppState>,
    nik: web::Path<String>,
    req: web::Json<UpdateEmployeeRequest>,
) -> impl Responder {
    let nik_str = nik.into_inner();
    info!("Updating employee: {}", nik_str);
    
    let mut updates = Vec::new();
    
    if let Some(ref full_name) = req.full_name {
        updates.push(format!("full_name = '{}'", full_name));
    }
    
    if let Some(ref email) = req.email {
        updates.push(format!("email = '{}'", email));
    }
    
    if let Some(ref password) = req.password {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap_or_default();
        updates.push(format!("password_hash = '{}'", password_hash));
    }
    
    if let Some(ref role) = req.role {
        updates.push(format!("role = '{}'", role));
    }
    
    if let Some(ref department) = req.department {
        updates.push(format!("department = '{}'", department));
    }
    
    if let Some(ref status) = req.status {
        updates.push(format!("status = '{}'", status));
    }
    
    if let Some(ref attendance_req) = req.attendance_requirement {
        let json_str = serde_json::to_string(attendance_req).unwrap_or_default();
        updates.push(format!("attendance_requirement = {}", json_str));
    }
    
    if updates.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "No fields to update"
        }));
    }
    
    updates.push("updated_at = time::now()".to_string());
    let update_clause = updates.join(", ");
    
    let query = format!("UPDATE employees SET {} WHERE nik = $nik RETURN AFTER", update_clause);
    
    match data.db.query(&query).bind(("nik", nik_str.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(updated) => {
                    if let Some(employee) = updated.first() {
                        info!("Successfully updated employee: {}", nik_str);
                        HttpResponse::Ok().json(json!({
                            "status": "success",
                            "message": "Employee updated successfully",
                            "data": employee
                        }))
                    } else {
                        HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": "Employee not found"
                        }))
                    }
                }
                Err(e) => {
                    error!("Error parsing updated employee: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to update employee"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error updating employee: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database error"
            }))
        }
    }
}

pub async fn delete_employee(
    data: web::Data<AppState>,
    nik: web::Path<String>,
) -> impl Responder {
    let nik_str = nik.into_inner();
    info!("Deleting employee: {}", nik_str);
    
    let query = "DELETE employees WHERE nik = $nik";
    
    match data.db.query(query).bind(("nik", nik_str.clone())).await {
        Ok(_) => {
            info!("Successfully deleted employee: {}", nik_str);
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Employee deleted successfully"
            }))
        }
        Err(e) => {
            error!("Database error deleting employee: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database error"
            }))
        }
    }
}
