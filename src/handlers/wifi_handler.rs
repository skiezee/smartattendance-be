use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::{info, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct WiFiSetting {
    pub id: Option<surrealdb::sql::Thing>,
    pub ssid: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWiFiRequest {
    pub ssid: String,
    pub description: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateWiFiRequest {
    pub ssid: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateWiFiResponse {
    pub success: bool,
    pub valid: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWiFiRequest {
    pub is_active: Option<bool>,
    pub description: Option<String>,
}

/// Get all active WiFi settings
pub async fn get_wifi_settings(data: web::Data<crate::config::app_state::AppState>) -> impl Responder {
    info!("Fetching all active WiFi settings");

    let query = "SELECT * FROM wifi_settings WHERE is_active = true ORDER BY created_at DESC";
    
    match data.db.query(query).await {
        Ok(mut response) => {
            match response.take::<Vec<WiFiSetting>>(0) {
                Ok(wifi_list) => {
                    info!("Successfully fetched {} WiFi settings", wifi_list.len());
                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "data": wifi_list
                    }))
                }
                Err(e) => {
                    error!("Error parsing WiFi settings: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Failed to parse WiFi settings"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error fetching WiFi settings: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}

/// Get all WiFi settings (including inactive) - Admin only
pub async fn get_all_wifi_settings(data: web::Data<crate::config::app_state::AppState>) -> impl Responder {
    info!("Fetching all WiFi settings (including inactive)");

    let query = "SELECT * FROM wifi_settings ORDER BY created_at DESC";
    
    match data.db.query(query).await {
        Ok(mut response) => {
            match response.take::<Vec<WiFiSetting>>(0) {
                Ok(wifi_list) => {
                    info!("Successfully fetched {} WiFi settings", wifi_list.len());
                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "data": wifi_list
                    }))
                }
                Err(e) => {
                    error!("Error parsing WiFi settings: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Failed to parse WiFi settings"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error fetching WiFi settings: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}

/// Validate WiFi SSID
pub async fn validate_wifi_ssid(
    data: web::Data<crate::config::app_state::AppState>,
    req: web::Json<ValidateWiFiRequest>,
) -> impl Responder {
    info!("Validating WiFi SSID: {}", req.ssid);

    let query = "SELECT * FROM wifi_settings WHERE ssid = $ssid AND is_active = true LIMIT 1";
    
    match data.db.query(query).bind(("ssid", req.ssid.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<WiFiSetting>>(0) {
                Ok(wifi_list) => {
                    let valid = !wifi_list.is_empty();
                    let message = if valid {
                        format!("WiFi '{}' valid", req.ssid)
                    } else {
                        format!("WiFi '{}' tidak terdaftar. Harap terhubung ke WiFi kantor yang terdaftar.", req.ssid)
                    };

                    info!("WiFi validation result for '{}': {}", req.ssid, valid);

                    HttpResponse::Ok().json(ValidateWiFiResponse {
                        success: true,
                        valid,
                        message,
                    })
                }
                Err(e) => {
                    error!("Error parsing WiFi validation result: {:?}", e);
                    HttpResponse::InternalServerError().json(ValidateWiFiResponse {
                        success: false,
                        valid: false,
                        message: "Error validating WiFi".to_string(),
                    })
                }
            }
        }
        Err(e) => {
            error!("Database error validating WiFi: {:?}", e);
            HttpResponse::InternalServerError().json(ValidateWiFiResponse {
                success: false,
                valid: false,
                message: "Database error".to_string(),
            })
        }
    }
}

/// Create new WiFi setting - Admin only
pub async fn create_wifi_setting(
    data: web::Data<crate::config::app_state::AppState>,
    req: web::Json<CreateWiFiRequest>,
) -> impl Responder {
    info!("Creating new WiFi setting: {}", req.ssid);

    // Check if SSID already exists
    let check_query = "SELECT * FROM wifi_settings WHERE ssid = $ssid LIMIT 1";
    match data.db.query(check_query).bind(("ssid", req.ssid.clone())).await {
        Ok(mut response) => {
            match response.take::<Vec<WiFiSetting>>(0) {
                Ok(existing) => {
                    if !existing.is_empty() {
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "success": false,
                            "message": format!("WiFi dengan SSID '{}' sudah terdaftar", req.ssid)
                        }));
                    }
                }
                Err(e) => {
                    error!("Error checking existing WiFi: {:?}", e);
                }
            }
        }
        Err(e) => {
            error!("Database error checking existing WiFi: {:?}", e);
        }
    }

    // Create new WiFi setting
    let is_active = req.is_active.unwrap_or(true);
    let query = r#"
        CREATE wifi_settings CONTENT {
            ssid: $ssid,
            description: $description,
            is_active: $is_active,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    match data.db.query(query)
        .bind(("ssid", req.ssid.clone()))
        .bind(("description", req.description.clone()))
        .bind(("is_active", is_active))
        .await
    {
        Ok(mut response) => {
            match response.take::<Vec<WiFiSetting>>(0) {
                Ok(created) => {
                    if let Some(wifi) = created.first() {
                        info!("Successfully created WiFi setting: {}", req.ssid);
                        HttpResponse::Ok().json(serde_json::json!({
                            "success": true,
                            "message": "WiFi berhasil ditambahkan",
                            "data": wifi
                        }))
                    } else {
                        error!("WiFi created but not returned");
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "success": false,
                            "message": "WiFi created but not returned"
                        }))
                    }
                }
                Err(e) => {
                    error!("Error parsing created WiFi: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Failed to create WiFi setting"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error creating WiFi: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}

/// Update WiFi setting - Admin only
pub async fn update_wifi_setting(
    data: web::Data<crate::config::app_state::AppState>,
    wifi_id: web::Path<String>,
    req: web::Json<UpdateWiFiRequest>,
) -> impl Responder {
    info!("Updating WiFi setting: {}", wifi_id.clone());

    let mut updates = Vec::new();
    
    if let Some(is_active) = req.is_active {
        updates.push(format!("is_active = {}", is_active));
    }
    
    if let Some(ref description) = req.description {
        updates.push(format!("description = '{}'", description));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "No fields to update"
        }));
    }

    updates.push("updated_at = time::now()".to_string());
    let update_clause = updates.join(", ");

    let id_str = wifi_id.into_inner();
    let target_id = if id_str.contains(":") {
        id_str.clone()
    } else {
        format!("wifi_settings:{}", id_str)
    };
    let query = format!("UPDATE {} SET {}", target_id, update_clause);

    match data.db.query(&query).await {
        Ok(mut response) => {
            match response.take::<Vec<WiFiSetting>>(0) {
                Ok(updated) => {
                    if let Some(wifi) = updated.first() {
                        info!("Successfully updated WiFi setting: {}", target_id);
                        HttpResponse::Ok().json(serde_json::json!({
                            "success": true,
                            "message": "WiFi berhasil diupdate",
                            "data": wifi
                        }))
                    } else {
                        HttpResponse::NotFound().json(serde_json::json!({
                            "success": false,
                            "message": "WiFi setting not found"
                        }))
                    }
                }
                Err(e) => {
                    error!("Error parsing updated WiFi: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Failed to update WiFi setting"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error updating WiFi: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}

/// Delete WiFi setting - Admin only
pub async fn delete_wifi_setting(
    data: web::Data<crate::config::app_state::AppState>,
    wifi_id: web::Path<String>,
) -> impl Responder {
    info!("Deleting WiFi setting: {}", wifi_id.clone());

    let id_str = wifi_id.into_inner();
    let target_id = if id_str.contains(":") {
        id_str.clone()
    } else {
        format!("wifi_settings:{}", id_str)
    };
    let query = format!("DELETE {}", target_id);

    match data.db.query(&query).await {
        Ok(_) => {
            info!("Successfully deleted WiFi setting: {}", target_id);
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "WiFi berhasil dihapus"
            }))
        }
        Err(e) => {
            error!("Database error deleting WiFi: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}
