use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};

use crate::config::app_state::AppState;
use crate::view_models::employee_vm::EmployeeViewModel;
use crate::models::employee::{CreateEmployeeRequest, UpdateEmployeeRequest, EmployeeResponse, BulkAttendanceRequest, BulkCreateEmployeeRequest};

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
    
    let query = "SELECT * FROM employee WHERE nik = $nik LIMIT 1";
    
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
    let check_query = "SELECT * FROM employee WHERE nik = $nik LIMIT 1";
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
    
    // Ensure default department and position exist
    let _ = data.db.query("CREATE department:default SET name = 'General', is_active = true;").await;
    let _ = data.db.query("CREATE position:default SET name = 'Staff', level = 'staff', is_active = true;").await;
    
    // Create employee with proper schema
    let query = r#"
        CREATE employee SET
            nik = $nik,
            full_name = $full_name,
            email = $email,
            password_hash = $password_hash,
            role = $role,
            phone = $phone,
            address = $address,
            date_of_birth = $date_of_birth,
            hire_date = $hire_date,
            position = $position,
            emergency_contact = $emergency_contact,
            emergency_phone = $emergency_phone,
            department_id = department:default,
            position_id = position:default,
            employment_status = 'active',
            join_date = time::now(),
            attendance_requirement = $attendance_requirement,
            created_at = time::now(),
            updated_at = time::now()
    "#;
    
    match data.db.query(query)
        .bind(("nik", req.nik.clone()))
        .bind(("full_name", req.full_name.clone()))
        .bind(("email", req.email.clone()))
        .bind(("password_hash", password_hash))
        .bind(("role", req.role.clone()))
        .bind(("phone", req.phone.clone()))
        .bind(("address", req.address.clone()))
        .bind(("date_of_birth", req.date_of_birth.clone()))
        .bind(("hire_date", req.hire_date.clone()))
        .bind(("position", req.position.clone()))
        .bind(("emergency_contact", req.emergency_contact.clone()))
        .bind(("emergency_phone", req.emergency_phone.clone()))
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
                        "message": format!("Failed to create employee: {}", e)
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
    
    if let Some(ref phone) = req.phone {
        updates.push(format!("phone = '{}'", phone));
    }
    
    if let Some(ref address) = req.address {
        let escaped_address = address.replace("'", "''");
        updates.push(format!("address = '{}'", escaped_address));
    }
    
    if let Some(ref date_of_birth) = req.date_of_birth {
        updates.push(format!("date_of_birth = '{}'", date_of_birth));
    }
    
    if let Some(ref hire_date) = req.hire_date {
        updates.push(format!("hire_date = '{}'", hire_date));
    }
    
    if let Some(ref position) = req.position {
        updates.push(format!("position = '{}'", position));
    }
    
    if let Some(ref emergency_contact) = req.emergency_contact {
        updates.push(format!("emergency_contact = '{}'", emergency_contact));
    }
    
    if let Some(ref emergency_phone) = req.emergency_phone {
        updates.push(format!("emergency_phone = '{}'", emergency_phone));
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
    
    let query = format!("UPDATE employee SET {} WHERE nik = $nik RETURN AFTER", update_clause);
    
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
    
    let query = "DELETE employee WHERE nik = $nik";
    
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

pub async fn bulk_update_attendance(
    data: web::Data<AppState>,
    req: web::Json<BulkAttendanceRequest>,
) -> impl Responder {
    info!("Bulk updating attendance requirements for {} employees", req.employee_niks.len());
    
    let mut success_count = 0;
    let mut failed_niks = Vec::new();
    
    for nik in &req.employee_niks {
        let json_str = serde_json::to_string(&req.attendance_requirement).unwrap_or_default();
        let query = format!(
            "UPDATE employee SET attendance_requirement = {}, updated_at = time::now() WHERE nik = $nik",
            json_str
        );
        
        match data.db.query(&query).bind(("nik", nik.clone())).await {
            Ok(_) => {
                success_count += 1;
                info!("Updated attendance requirement for employee: {}", nik);
            }
            Err(e) => {
                error!("Failed to update attendance requirement for {}: {:?}", nik, e);
                failed_niks.push(nik.clone());
            }
        }
    }
    
    if failed_niks.is_empty() {
        HttpResponse::Ok().json(json!({
            "status": "success",
            "message": format!("Successfully updated {} employees", success_count),
            "updated_count": success_count
        }))
    } else {
        HttpResponse::Ok().json(json!({
            "status": "partial_success",
            "message": format!("Updated {} employees, {} failed", success_count, failed_niks.len()),
            "updated_count": success_count,
            "failed_niks": failed_niks
        }))
    }
}

pub async fn bulk_create_employees(
    data: web::Data<AppState>,
    req: web::Json<BulkCreateEmployeeRequest>,
) -> impl Responder {
    info!("Bulk creating {} employees", req.employees.len());
    
    let mut success_count = 0;
    let mut failed_employees = Vec::new();
    let mut created_employees = Vec::new();
    
    // Ensure default department and position exist
    let _ = data.db.query("CREATE department:default SET name = 'General', is_active = true;").await;
    let _ = data.db.query("CREATE position:default SET name = 'Staff', level = 'staff', is_active = true;").await;
    
    for employee_req in &req.employees {
        // Check if NIK already exists
        let check_query = "SELECT * FROM employee WHERE nik = $nik LIMIT 1";
        match data.db.query(check_query).bind(("nik", employee_req.nik.clone())).await {
            Ok(mut response) => {
                match response.take::<Vec<EmployeeResponse>>(0) {
                    Ok(existing) => {
                        if !existing.is_empty() {
                            error!("Employee with NIK '{}' already exists", employee_req.nik);
                            failed_employees.push(json!({
                                "nik": employee_req.nik,
                                "reason": "NIK already exists"
                            }));
                            continue;
                        }
                    }
                    Err(e) => {
                        error!("Error checking existing employee: {:?}", e);
                        failed_employees.push(json!({
                            "nik": employee_req.nik,
                            "reason": "Database error"
                        }));
                        continue;
                    }
                }
            }
            Err(e) => {
                error!("Database error checking existing employee: {:?}", e);
                failed_employees.push(json!({
                    "nik": employee_req.nik,
                    "reason": "Database error"
                }));
                continue;
            }
        }
        
        // Hash password
        let password_hash = bcrypt::hash(&employee_req.password, bcrypt::DEFAULT_COST).unwrap_or_default();
        
        // Create employee
        let query = r#"
            CREATE employee SET
                nik = $nik,
                full_name = $full_name,
                email = $email,
                password_hash = $password_hash,
                role = $role,
                phone = $phone,
                address = $address,
                date_of_birth = $date_of_birth,
                hire_date = $hire_date,
                position = $position,
                emergency_contact = $emergency_contact,
                emergency_phone = $emergency_phone,
                department_id = department:default,
                position_id = position:default,
                employment_status = 'active',
                join_date = time::now(),
                attendance_requirement = $attendance_requirement,
                created_at = time::now(),
                updated_at = time::now()
        "#;
        
        match data.db.query(query)
            .bind(("nik", employee_req.nik.clone()))
            .bind(("full_name", employee_req.full_name.clone()))
            .bind(("email", employee_req.email.clone()))
            .bind(("password_hash", password_hash))
            .bind(("role", employee_req.role.clone()))
            .bind(("phone", employee_req.phone.clone()))
            .bind(("address", employee_req.address.clone()))
            .bind(("date_of_birth", employee_req.date_of_birth.clone()))
            .bind(("hire_date", employee_req.hire_date.clone()))
            .bind(("position", employee_req.position.clone()))
            .bind(("emergency_contact", employee_req.emergency_contact.clone()))
            .bind(("emergency_phone", employee_req.emergency_phone.clone()))
            .bind(("attendance_requirement", employee_req.attendance_requirement.clone()))
            .await
        {
            Ok(mut response) => {
                match response.take::<Vec<EmployeeResponse>>(0) {
                    Ok(created) => {
                        if let Some(employee) = created.first() {
                            success_count += 1;
                            created_employees.push(employee.clone());
                            info!("Successfully created employee: {}", employee_req.nik);
                        } else {
                            error!("Employee created but not returned: {}", employee_req.nik);
                            failed_employees.push(json!({
                                "nik": employee_req.nik,
                                "reason": "Created but not returned"
                            }));
                        }
                    }
                    Err(e) => {
                        error!("Error parsing created employee {}: {:?}", employee_req.nik, e);
                        failed_employees.push(json!({
                            "nik": employee_req.nik,
                            "reason": format!("Parse error: {}", e)
                        }));
                    }
                }
            }
            Err(e) => {
                error!("Database error creating employee {}: {:?}", employee_req.nik, e);
                failed_employees.push(json!({
                    "nik": employee_req.nik,
                    "reason": "Database error"
                }));
            }
        }
    }
    
    if failed_employees.is_empty() {
        HttpResponse::Ok().json(json!({
            "status": "success",
            "message": format!("Successfully created {} employees", success_count),
            "created_count": success_count,
            "data": created_employees
        }))
    } else {
        HttpResponse::Ok().json(json!({
            "status": "partial_success",
            "message": format!("Created {} employees, {} failed", success_count, failed_employees.len()),
            "created_count": success_count,
            "failed_count": failed_employees.len(),
            "data": created_employees,
            "failed": failed_employees
        }))
    }
}
