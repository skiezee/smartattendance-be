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
    
    let query = "SELECT *, attendance_requirement FROM employee WHERE nik = $nik LIMIT 1";
    
    match data.db.query(query).bind(("nik", nik_str.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(employees) => {
                    if let Some(employee) = employees.first() {
                        info!("Found employee: {} with attendance_requirement: {:?}", nik_str, employee.attendance_requirement);
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
            department = $department,
            status = $status,
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
        .bind(("department", req.department.clone()))
        .bind(("status", req.status.clone()))
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
    info!("Update request: {:?}", req);
    
    // First, find the employee record (need full record to get ID)
    let find_query = "SELECT * FROM employee WHERE nik = $nik LIMIT 1";
    
    match data.db.query(find_query).bind(("nik", nik_str.clone())).await {
        Ok(mut find_response) => {
            match find_response.take::<Vec<EmployeeResponse>>(0) {
                Ok(employees) => {
                    if let Some(employee) = employees.first() {
                        if let Some(ref record_id) = employee.id {
                            info!("Found employee record: {:?}", record_id);
                            
                            // Handle password separately if provided
                            let password_hash = if let Some(ref password) = req.password {
                                if !password.is_empty() {
                                    Some(bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap_or_default())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            
                            // Build update query using record ID
                            let update_query = if password_hash.is_some() {
                                format!(
                                    r#"UPDATE {} SET
                                        full_name = $full_name,
                                        email = $email,
                                        password_hash = $password_hash,
                                        role = $role,
                                        department = $department,
                                        status = $status,
                                        phone = $phone,
                                        address = $address,
                                        date_of_birth = $date_of_birth,
                                        hire_date = $hire_date,
                                        position = $position,
                                        emergency_contact = $emergency_contact,
                                        emergency_phone = $emergency_phone,
                                        attendance_requirement = $attendance_requirement,
                                        updated_at = time::now()
                                    RETURN AFTER"#,
                                    record_id
                                )
                            } else {
                                format!(
                                    r#"UPDATE {} SET
                                        full_name = $full_name,
                                        email = $email,
                                        role = $role,
                                        department = $department,
                                        status = $status,
                                        phone = $phone,
                                        address = $address,
                                        date_of_birth = $date_of_birth,
                                        hire_date = $hire_date,
                                        position = $position,
                                        emergency_contact = $emergency_contact,
                                        emergency_phone = $emergency_phone,
                                        attendance_requirement = $attendance_requirement,
                                        updated_at = time::now()
                                    RETURN AFTER"#,
                                    record_id
                                )
                            };
                            
                            let mut db_query = data.db.query(&update_query)
                                .bind(("full_name", req.full_name.clone().unwrap_or_default()))
                                .bind(("email", req.email.clone().unwrap_or_default()))
                                .bind(("role", req.role.clone().unwrap_or_default()))
                                .bind(("department", req.department.clone()))
                                .bind(("status", req.status.clone()))
                                .bind(("phone", req.phone.clone()))
                                .bind(("address", req.address.clone()))
                                .bind(("date_of_birth", req.date_of_birth.clone()))
                                .bind(("hire_date", req.hire_date.clone()))
                                .bind(("position", req.position.clone()))
                                .bind(("emergency_contact", req.emergency_contact.clone()))
                                .bind(("emergency_phone", req.emergency_phone.clone()))
                                .bind(("attendance_requirement", req.attendance_requirement.clone()));
                            
                            if let Some(hash) = password_hash {
                                db_query = db_query.bind(("password_hash", hash));
                            }
                            
                            match db_query.await {
                                Ok(mut response) => {
                                    match response.take::<Vec<EmployeeResponse>>(0) {
                                        Ok(updated) => {
                                            if let Some(updated_emp) = updated.first() {
                                                info!("✅ Successfully updated employee: {} - attendance_requirement: {:?}", nik_str, updated_emp.attendance_requirement);
                                                HttpResponse::Ok().json(json!({
                                                    "status": "success",
                                                    "message": "Employee updated successfully",
                                                    "data": updated_emp
                                                }))
                                            } else {
                                                error!("❌ Update returned no data");
                                                HttpResponse::InternalServerError().json(json!({
                                                    "status": "error",
                                                    "message": "Update returned no data"
                                                }))
                                            }
                                        }
                                        Err(e) => {
                                            error!("❌ Error parsing updated employee: {:?}", e);
                                            HttpResponse::InternalServerError().json(json!({
                                                "status": "error",
                                                "message": format!("Failed to update employee: {}", e)
                                            }))
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("❌ Database error updating employee: {:?}", e);
                                    HttpResponse::InternalServerError().json(json!({
                                        "status": "error",
                                        "message": format!("Database error: {}", e)
                                    }))
                                }
                            }
                        } else {
                            error!("❌ Employee {} has no record ID", nik_str);
                            HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": "Employee has no record ID"
                            }))
                        }
                    } else {
                        HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": "Employee not found"
                        }))
                    }
                }
                Err(e) => {
                    error!("❌ Error parsing find result: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to find employee"
                    }))
                }
            }
        }
        Err(e) => {
            error!("❌ Database error finding employee: {:?}", e);
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
    info!("Request data: {:?}", req);
    
    let mut success_count = 0;
    let mut failed_niks = Vec::new();
    let mut updated_employees = Vec::new();
    
    for nik in &req.employee_niks {
        info!("Processing employee: {}", nik);
        
        // Get the full employee record to extract the ID
        let find_query = "SELECT * FROM employee WHERE nik = $nik LIMIT 1";
        
        match data.db.query(find_query).bind(("nik", nik.clone())).await {
            Ok(mut find_response) => {
                match find_response.take::<Vec<EmployeeResponse>>(0) {
                    Ok(employees) => {
                        if let Some(employee) = employees.first() {
                            if let Some(ref record_id) = employee.id {
                                info!("Found employee record: {:?}", record_id);
                                
                                // Convert to SurrealDB native object syntax
                                let wifi_ssids_str = if let Some(ref ssids) = req.attendance_requirement.wifi_ssids {
                                    format!("[{}]", ssids.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "))
                                } else {
                                    "NONE".to_string()
                                };
                                
                                let location_boundaries_str = if let Some(ref boundaries) = req.attendance_requirement.location_boundaries {
                                    format!("[{}]", boundaries.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "))
                                } else {
                                    "NONE".to_string()
                                };
                                
                                // Build SurrealDB object syntax
                                let surql_object = format!(
                                    "{{ wifi_enabled: {}, wifi_ssids: {}, location_enabled: {}, location_boundaries: {}, face_recognition_enabled: {}, fingerprint_enabled: {} }}",
                                    req.attendance_requirement.wifi_enabled,
                                    wifi_ssids_str,
                                    req.attendance_requirement.location_enabled,
                                    location_boundaries_str,
                                    req.attendance_requirement.face_recognition_enabled,
                                    req.attendance_requirement.fingerprint_enabled
                                );
                                
                                info!("📝 SurrealDB object: {}", surql_object);
                                
                                // Use SurrealDB native syntax
                                let update_query = format!(
                                    "UPDATE {} SET attendance_requirement = {}, updated_at = time::now() RETURN AFTER",
                                    record_id,
                                    surql_object
                                );
                                
                                info!("🚀 Executing update query: {}", update_query);
                                info!("🎯 Record ID: {}", record_id);
                                
                                match data.db.query(&update_query).await {
                                            Ok(mut update_response) => {
                                                info!("Update query executed successfully");
                                                match update_response.take::<Vec<EmployeeResponse>>(0) {
                                                    Ok(updated) => {
                                                        if let Some(updated_emp) = updated.first() {
                                                            success_count += 1;
                                                            info!("✅ Updated attendance requirement for employee: {} - New value: {:?}", nik, updated_emp.attendance_requirement);
                                                            updated_employees.push(updated_emp.clone());
                                                            
                                                            // VERIFY: Re-fetch from database to confirm persistence
                                                            info!("🔍 Verifying update in database...");
                                                            let verify_query = "SELECT * FROM employee WHERE nik = $nik LIMIT 1";
                                                            match data.db.query(verify_query).bind(("nik", nik.clone())).await {
                                                                Ok(mut verify_response) => {
                                                                    match verify_response.take::<Vec<EmployeeResponse>>(0) {
                                                                        Ok(verified) => {
                                                                            if let Some(verified_emp) = verified.first() {
                                                                                info!("🔍 Database verification - attendance_requirement: {:?}", verified_emp.attendance_requirement);
                                                                            }
                                                                        }
                                                                        Err(e) => {
                                                                            error!("Failed to parse verification result: {:?}", e);
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    error!("Failed to verify in database: {:?}", e);
                                                                }
                                                            }
                                                        } else {
                                                            error!("❌ Update returned no data for: {}", nik);
                                                            failed_niks.push(json!({
                                                                "nik": nik,
                                                                "reason": "Update returned no data"
                                                            }));
                                                        }
                                                    }
                                                    Err(e) => {
                                                        error!("❌ Failed to parse update result for {}: {:?}", nik, e);
                                                        failed_niks.push(json!({
                                                            "nik": nik,
                                                            "reason": format!("Parse error: {}", e)
                                                        }));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("❌ Failed to execute update for {}: {:?}", nik, e);
                                                failed_niks.push(json!({
                                                    "nik": nik,
                                                    "reason": format!("Update error: {}", e)
                                                }));
                                            }
                                        }
                            } else {
                                error!("❌ Employee {} has no record ID", nik);
                                failed_niks.push(json!({
                                    "nik": nik,
                                    "reason": "No record ID"
                                }));
                            }
                        } else {
                            error!("❌ Employee not found: {}", nik);
                            failed_niks.push(json!({
                                "nik": nik,
                                "reason": "Employee not found"
                            }));
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to parse find result for {}: {:?}", nik, e);
                        failed_niks.push(json!({
                            "nik": nik,
                            "reason": format!("Find parse error: {}", e)
                        }));
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to find employee {}: {:?}", nik, e);
                failed_niks.push(json!({
                    "nik": nik,
                    "reason": format!("Find error: {}", e)
                }));
            }
        }
    }
    
    if failed_niks.is_empty() {
        HttpResponse::Ok().json(json!({
            "status": "success",
            "message": format!("Successfully updated {} employees", success_count),
            "updated_count": success_count,
            "data": updated_employees
        }))
    } else {
        HttpResponse::Ok().json(json!({
            "status": "partial_success",
            "message": format!("Updated {} employees, {} failed", success_count, failed_niks.len()),
            "updated_count": success_count,
            "failed_count": failed_niks.len(),
            "data": updated_employees,
            "failed": failed_niks
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
                department = $department,
                status = $status,
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
            .bind(("department", employee_req.department.clone()))
            .bind(("status", employee_req.status.clone()))
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
