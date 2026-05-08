use actix_web::{web, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use serde::{Deserialize, Serialize};
use serde_json::json;
use log::{info, error};
use std::io::Write;
use std::path::Path;

use crate::config::app_state::AppState;
use crate::models::employee::EmployeeResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProfileDetailResponse {
    pub nik: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub join_date: Option<String>,
    pub profile_photo_url: Option<String>,
    pub present_count: i32,
    pub absent_count: i32,
    pub leave_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProfileRequest {
    pub nik: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub profile_photo_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePasswordRequest {
    pub nik: String,
    pub old_password: String,
    pub new_password: String,
}

/// Get profile details with attendance statistics
pub async fn get_profile(
    data: web::Data<AppState>,
    nik: web::Path<String>,
) -> impl Responder {
    let nik_str = nik.into_inner();
    info!("Fetching profile for NIK: {}", nik_str);
    
    // Get employee data
    let employee_query = "SELECT * FROM employee WHERE nik = $nik LIMIT 1";
    
    let employee = match data.db.query(employee_query).bind(("nik", nik_str.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(mut employees) => {
                    if employees.is_empty() {
                        return HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": "Employee not found"
                        }));
                    }
                    employees.remove(0)
                }
                Err(e) => {
                    error!("Error parsing employee: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to parse employee data"
                    }));
                }
            }
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database error"
            }));
        }
    };
    
    // Get attendance statistics
    let stats_query = r#"
        SELECT 
            (SELECT count() FROM attendance WHERE nik = $nik AND status = 'present' GROUP ALL)[0].count AS present_count,
            (SELECT count() FROM attendance WHERE nik = $nik AND status = 'absent' GROUP ALL)[0].count AS absent_count,
            (SELECT count() FROM leave WHERE nik = $nik AND status = 'approved' GROUP ALL)[0].count AS leave_count
    "#;
    
    let (present_count, absent_count, leave_count) = match data.db.query(stats_query).bind(("nik", nik_str.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<serde_json::Value>>(0) {
                Ok(stats) => {
                    if let Some(stat) = stats.first() {
                        let present = stat.get("present_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let absent = stat.get("absent_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let leave = stat.get("leave_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        (present, absent, leave)
                    } else {
                        (0, 0, 0)
                    }
                }
                Err(_) => (0, 0, 0)
            }
        }
        Err(_) => (0, 0, 0)
    };
    
    // Get department name
    let department_name = if let Some(ref dept_id) = employee.department_id {
        let dept_query = format!("SELECT name FROM {}", dept_id);
        match data.db.query(&dept_query).await {
            Ok(mut dept_res) => {
                match dept_res.take::<Vec<serde_json::Value>>(0) {
                    Ok(depts) => {
                        depts.first()
                            .and_then(|d| d.get("name"))
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    }
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    } else {
        employee.department.clone()
    };
    
    // Get position name
    let position_name = if let Some(ref pos_id) = employee.position_id {
        let pos_query = format!("SELECT name FROM {}", pos_id);
        match data.db.query(&pos_query).await {
            Ok(mut pos_res) => {
                match pos_res.take::<Vec<serde_json::Value>>(0) {
                    Ok(positions) => {
                        positions.first()
                            .and_then(|p| p.get("name"))
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    }
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    } else {
        None
    };
    
    let profile = ProfileDetailResponse {
        nik: employee.nik.clone(),
        full_name: employee.full_name.clone(),
        email: Some(employee.email.clone()),
        phone_number: employee.phone_number.clone(),
        department: department_name,
        position: position_name,
        join_date: employee.created_at.clone(),
        profile_photo_url: employee.profile_photo_url.clone(),
        present_count,
        absent_count,
        leave_count,
    };
    
    HttpResponse::Ok().json(json!({
        "status": "success",
        "data": profile
    }))
}

/// Update profile information
pub async fn update_profile(
    data: web::Data<AppState>,
    req: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    info!("Updating profile for NIK: {}", req.nik);
    
    // Check if employee exists
    let check_query = "SELECT * FROM employee WHERE nik = $nik LIMIT 1";
    match data.db.query(check_query).bind(("nik", req.nik.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(employees) => {
                    if employees.is_empty() {
                        return HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": "Employee not found"
                        }));
                    }
                }
                Err(e) => {
                    error!("Error checking employee: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to check employee"
                    }));
                }
            }
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database error"
            }));
        }
    }
    
    // Build update query using parameterized bindings to avoid injection issues
    let mut set_parts: Vec<&str> = Vec::new();
    let mut has_full_name = false;
    let mut has_email = false;
    let mut has_phone = false;
    let mut has_photo = false;

    if req.full_name.is_some() { set_parts.push("full_name = $full_name"); has_full_name = true; }
    if req.email.is_some() { set_parts.push("email = $email"); has_email = true; }
    if req.phone_number.is_some() { set_parts.push("phone_number = $phone_number"); has_phone = true; }
    if req.profile_photo_url.is_some() { set_parts.push("profile_photo_url = $profile_photo_url"); has_photo = true; }

    if set_parts.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "No fields to update"
        }));
    }

    set_parts.push("updated_at = time::now()");
    let update_clause = set_parts.join(", ");
    let query = format!("UPDATE employee SET {} WHERE nik = $nik RETURN AFTER", update_clause);

    let mut db_query = data.db.query(&query).bind(("nik", req.nik.clone()));
    if has_full_name { db_query = db_query.bind(("full_name", req.full_name.clone().unwrap_or_default())); }
    if has_email { db_query = db_query.bind(("email", req.email.clone().unwrap_or_default())); }
    if has_phone { db_query = db_query.bind(("phone_number", req.phone_number.clone().unwrap_or_default())); }
    if has_photo { db_query = db_query.bind(("profile_photo_url", req.profile_photo_url.clone().unwrap_or_default())); }

    match db_query.await {
        Ok(_) => {
            info!("Successfully updated profile for NIK: {}", req.nik);
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Profile updated successfully"
            }))
        }
        Err(e) => {
            error!("Database error updating profile: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to update profile"
            }))
        }
    }
}

/// Change password
pub async fn change_password(
    data: web::Data<AppState>,
    req: web::Json<ChangePasswordRequest>,
) -> impl Responder {
    info!("Changing password for NIK: {}", req.nik);
    
    // Get current employee data
    let employee_query = "SELECT password_hash FROM employee WHERE nik = $nik LIMIT 1";
    
    let current_password_hash = match data.db.query(employee_query).bind(("nik", req.nik.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<serde_json::Value>>(0) {
                Ok(employees) => {
                    if let Some(emp) = employees.first() {
                        emp.get("password_hash")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_default()
                    } else {
                        return HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": "Employee not found"
                        }));
                    }
                }
                Err(e) => {
                    error!("Error parsing employee: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to parse employee data"
                    }));
                }
            }
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database error"
            }));
        }
    };
    
    // Verify old password
    match bcrypt::verify(&req.old_password, &current_password_hash) {
        Ok(valid) => {
            if !valid {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Old password is incorrect"
                }));
            }
        }
        Err(e) => {
            error!("Error verifying password: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to verify password"
            }));
        }
    }
    
    // Hash new password
    let new_password_hash = match bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Error hashing password: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to hash password"
            }));
        }
    };
    
    // Update password
    let update_query = "UPDATE employee SET password_hash = $password_hash, updated_at = time::now() WHERE nik = $nik";
    
    match data.db.query(update_query)
        .bind(("nik", req.nik.clone()))
        .bind(("password_hash", new_password_hash))
        .await
    {
        Ok(_) => {
            info!("Successfully changed password for NIK: {}", req.nik);
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Password changed successfully"
            }))
        }
        Err(e) => {
            error!("Database error updating password: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to update password"
            }))
        }
    }
}

/// Upload profile photo
pub async fn upload_profile_photo(
    data: web::Data<AppState>,
    mut payload: Multipart,
) -> impl Responder {
    info!("Uploading profile photo");
    
    let mut nik: Option<String> = None;
    let mut file_path: Option<String> = None;
    
    // Create uploads directory if it doesn't exist
    let upload_dir = "uploads/profile_photos";
    if let Err(e) = std::fs::create_dir_all(upload_dir) {
        error!("Failed to create upload directory: {:?}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to create upload directory"
        }));
    }
    
    // Process multipart form data
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => {
                error!("Error reading multipart field: {:?}", e);
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Invalid multipart data"
                }));
            }
        };
        
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("");
        
        match field_name {
            "nik" => {
                // Read NIK from form field
                let mut nik_bytes = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(data) => data,
                        Err(e) => {
                            error!("Error reading NIK field: {:?}", e);
                            return HttpResponse::BadRequest().json(json!({
                                "status": "error",
                                "message": "Error reading NIK"
                            }));
                        }
                    };
                    nik_bytes.extend_from_slice(&data);
                }
                nik = Some(String::from_utf8_lossy(&nik_bytes).to_string());
            }
            "photo" => {
                // Get original filename
                let filename = content_disposition
                    .get_filename()
                    .unwrap_or("photo.jpg");
                
                // Generate unique filename
                let extension = Path::new(filename)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("jpg");
                
                let unique_filename = format!(
                    "profile_{}_{}.{}",
                    nik.as_ref().unwrap_or(&"unknown".to_string()),
                    uuid::Uuid::new_v4(),
                    extension
                );
                
                let filepath = format!("{}/{}", upload_dir, unique_filename);
                
                // Save file
                let mut f = match std::fs::File::create(&filepath) {
                    Ok(file) => file,
                    Err(e) => {
                        error!("Failed to create file: {:?}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Failed to create file"
                        }));
                    }
                };
                
                // Write chunks to file
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(data) => data,
                        Err(e) => {
                            error!("Error reading file chunk: {:?}", e);
                            return HttpResponse::BadRequest().json(json!({
                                "status": "error",
                                "message": "Error reading file data"
                            }));
                        }
                    };
                    
                    if let Err(e) = f.write_all(&data) {
                        error!("Failed to write file: {:?}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Failed to write file"
                        }));
                    }
                }
                
                file_path = Some(filepath);
            }
            _ => {
                // Skip unknown fields
                while let Some(_) = field.next().await {}
            }
        }
    }
    
    // Validate required fields
    let nik = match nik {
        Some(n) => n,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "NIK is required"
            }));
        }
    };
    
    let file_path = match file_path {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Photo file is required"
            }));
        }
    };
    
    info!("Photo saved to: {}", file_path);
    
    // Update employee profile with photo URL
    let photo_url = format!("/{}", file_path);
    let query = "UPDATE employee SET profile_photo_url = $photo_url, updated_at = time::now() WHERE nik = $nik RETURN AFTER";
    
    match data.db.query(query)
        .bind(("nik", nik.clone()))
        .bind(("photo_url", photo_url.clone()))
        .await
    {
        Ok(mut response) => {
            match response.take::<Vec<EmployeeResponse>>(0) {
                Ok(updated) => {
                    if updated.is_empty() {
                        error!("Employee not found: {}", nik);
                        // Delete uploaded file if employee not found
                        let _ = std::fs::remove_file(&file_path);
                        return HttpResponse::NotFound().json(json!({
                            "status": "error",
                            "message": "Employee not found"
                        }));
                    }
                    
                    info!("Successfully updated profile photo for employee: {}", nik);
                    HttpResponse::Ok().json(json!({
                        "status": "success",
                        "message": "Profile photo uploaded successfully",
                        "data": {
                            "photo_url": photo_url
                        }
                    }))
                }
                Err(e) => {
                    error!("Error parsing updated employee: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to update profile photo"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error updating profile photo: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database error"
            }))
        }
    }
}
