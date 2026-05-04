use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};

use crate::config::app_state::AppState;
use crate::models::employee::{AttendanceRequirement, EmployeeResponse};

/// Fix existing employees that don't have attendance_requirement
pub async fn fix_attendance_requirements(data: web::Data<AppState>) -> impl Responder {
    info!("Starting fix for employees without attendance_requirement");
    
    // Get all employees
    let query = "SELECT * FROM employee";
    
    match data.db.query(query).await {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(employees) => {
                    let mut fixed_count = 0;
                    let mut failed_niks = Vec::new();
                    
                    for employee in employees {
                        if employee.attendance_requirement.is_none() {
                            info!("Fixing employee: {}", employee.nik);
                            
                            let default_req = AttendanceRequirement::default();
                            let update_query = r#"
                                UPDATE employee SET 
                                    attendance_requirement = $attendance_requirement,
                                    updated_at = time::now()
                                WHERE nik = $nik
                            "#;
                            
                            match data.db.query(update_query)
                                .bind(("nik", employee.nik.clone()))
                                .bind(("attendance_requirement", default_req))
                                .await
                            {
                                Ok(_) => {
                                    fixed_count += 1;
                                    info!("Fixed employee: {}", employee.nik);
                                }
                                Err(e) => {
                                    error!("Failed to fix employee {}: {:?}", employee.nik, e);
                                    failed_niks.push(employee.nik.clone());
                                }
                            }
                        }
                    }
                    
                    if failed_niks.is_empty() {
                        HttpResponse::Ok().json(json!({
                            "status": "success",
                            "message": format!("Fixed {} employees", fixed_count),
                            "fixed_count": fixed_count
                        }))
                    } else {
                        HttpResponse::Ok().json(json!({
                            "status": "partial_success",
                            "message": format!("Fixed {} employees, {} failed", fixed_count, failed_niks.len()),
                            "fixed_count": fixed_count,
                            "failed_niks": failed_niks
                        }))
                    }
                }
                Err(e) => {
                    error!("Error parsing employees: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to parse employees"
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
