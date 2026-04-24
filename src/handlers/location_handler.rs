use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::{info, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationBoundary {
    pub id: Option<surrealdb::sql::Thing>,
    pub name: String,
    pub description: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius: i32, // radius in meters
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
    pub description: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius: i32,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateLocationRequest {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize)]
pub struct ValidateLocationResponse {
    pub success: bool,
    pub valid: bool,
    pub message: String,
    pub distance: Option<f64>, // distance in meters
    pub nearest_location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub radius: Option<i32>,
    pub is_active: Option<bool>,
}

/// Calculate distance between two coordinates using Haversine formula
fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS: f64 = 6371000.0; // Earth radius in meters

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS * c
}

/// Get all active location boundaries
pub async fn get_location_boundaries(data: web::Data<crate::config::app_state::AppState>) -> impl Responder {
    info!("Fetching all active location boundaries");

    let query = "SELECT * FROM location_boundaries WHERE is_active = true ORDER BY created_at DESC";
    
    match data.db.query(query).await {
        Ok(mut response) => {
            match response.take::<Vec<LocationBoundary>>(0) {
                Ok(locations) => {
                    info!("Successfully fetched {} location boundaries", locations.len());
                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "data": locations
                    }))
                }
                Err(e) => {
                    error!("Error parsing location boundaries: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Failed to parse location boundaries"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error fetching location boundaries: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}

/// Get all location boundaries (including inactive) - Admin only
pub async fn get_all_location_boundaries(data: web::Data<crate::config::app_state::AppState>) -> impl Responder {
    info!("Fetching all location boundaries (including inactive)");

    let query = "SELECT * FROM location_boundaries ORDER BY created_at DESC";
    
    match data.db.query(query).await {
        Ok(mut response) => {
            match response.take::<Vec<LocationBoundary>>(0) {
                Ok(locations) => {
                    info!("Successfully fetched {} location boundaries", locations.len());
                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "data": locations
                    }))
                }
                Err(e) => {
                    error!("Error parsing location boundaries: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Failed to parse location boundaries"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error fetching location boundaries: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}

/// Validate user location against boundaries
pub async fn validate_location(
    data: web::Data<crate::config::app_state::AppState>,
    req: web::Json<ValidateLocationRequest>,
) -> impl Responder {
    info!("Validating location: lat={}, lon={}", req.latitude, req.longitude);

    let query = "SELECT * FROM location_boundaries WHERE is_active = true";
    
    match data.db.query(query).await {
        Ok(mut response) => {
            match response.take::<Vec<LocationBoundary>>(0) {
                Ok(boundaries) => {
                    if boundaries.is_empty() {
                        return HttpResponse::Ok().json(ValidateLocationResponse {
                            success: true,
                            valid: false,
                            message: "Tidak ada lokasi boundary yang aktif. Hubungi admin.".to_string(),
                            distance: None,
                            nearest_location: None,
                        });
                    }

                    let mut nearest_distance = f64::MAX;
                    let mut nearest_location_name = String::new();
                    let mut is_within_boundary = false;

                    for boundary in &boundaries {
                        let distance = calculate_distance(
                            req.latitude,
                            req.longitude,
                            boundary.latitude,
                            boundary.longitude,
                        );

                        if distance < nearest_distance {
                            nearest_distance = distance;
                            nearest_location_name = boundary.name.clone();
                        }

                        if distance <= boundary.radius as f64 {
                            is_within_boundary = true;
                            info!(
                                "Location valid: within {} (distance: {:.2}m, radius: {}m)",
                                boundary.name, distance, boundary.radius
                            );
                            return HttpResponse::Ok().json(ValidateLocationResponse {
                                success: true,
                                valid: true,
                                message: format!("Lokasi valid: Anda berada di area {}", boundary.name),
                                distance: Some(distance),
                                nearest_location: Some(boundary.name.clone()),
                            });
                        }
                    }

                    if !is_within_boundary {
                        let message = format!(
                            "Lokasi tidak valid. Anda berada {:.0}m dari {} (radius: {}m). Harap berada di area kantor untuk melakukan attendance.",
                            nearest_distance,
                            nearest_location_name,
                            boundaries.iter()
                                .find(|b| b.name == nearest_location_name)
                                .map(|b| b.radius)
                                .unwrap_or(0)
                        );
                        
                        info!("Location invalid: {}", message);
                        
                        HttpResponse::Ok().json(ValidateLocationResponse {
                            success: true,
                            valid: false,
                            message,
                            distance: Some(nearest_distance),
                            nearest_location: Some(nearest_location_name),
                        })
                    } else {
                        HttpResponse::Ok().json(ValidateLocationResponse {
                            success: true,
                            valid: false,
                            message: "Lokasi tidak valid".to_string(),
                            distance: Some(nearest_distance),
                            nearest_location: Some(nearest_location_name),
                        })
                    }
                }
                Err(e) => {
                    error!("Error parsing location boundaries: {:?}", e);
                    HttpResponse::InternalServerError().json(ValidateLocationResponse {
                        success: false,
                        valid: false,
                        message: "Error validating location".to_string(),
                        distance: None,
                        nearest_location: None,
                    })
                }
            }
        }
        Err(e) => {
            error!("Database error validating location: {:?}", e);
            HttpResponse::InternalServerError().json(ValidateLocationResponse {
                success: false,
                valid: false,
                message: "Database error".to_string(),
                distance: None,
                nearest_location: None,
            })
        }
    }
}

/// Create new location boundary - Admin only
pub async fn create_location_boundary(
    data: web::Data<crate::config::app_state::AppState>,
    req: web::Json<CreateLocationRequest>,
) -> impl Responder {
    info!("Creating new location boundary: {}", req.name);

    // Validate radius
    if req.radius < 10 || req.radius > 10000 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Radius harus antara 10 dan 10000 meter"
        }));
    }

    // Validate coordinates
    if req.latitude < -90.0 || req.latitude > 90.0 || req.longitude < -180.0 || req.longitude > 180.0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Koordinat tidak valid"
        }));
    }

    let is_active = req.is_active.unwrap_or(true);
    let query = r#"
        CREATE location_boundaries CONTENT {
            name: $name,
            description: $description,
            latitude: $latitude,
            longitude: $longitude,
            radius: $radius,
            is_active: $is_active,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    match data.db.query(query)
        .bind(("name", req.name.clone()))
        .bind(("description", req.description.clone()))
        .bind(("latitude", req.latitude))
        .bind(("longitude", req.longitude))
        .bind(("radius", req.radius))
        .bind(("is_active", is_active))
        .await
    {
        Ok(mut response) => {
            match response.take::<Vec<LocationBoundary>>(0) {
                Ok(created) => {
                    if let Some(location) = created.first() {
                        info!("Successfully created location boundary: {}", req.name);
                        HttpResponse::Ok().json(serde_json::json!({
                            "success": true,
                            "message": "Location boundary berhasil ditambahkan",
                            "data": location
                        }))
                    } else {
                        error!("Location created but not returned");
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "success": false,
                            "message": "Location created but not returned"
                        }))
                    }
                }
                Err(e) => {
                    error!("Error parsing created location: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Failed to create location boundary"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error creating location: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}

/// Update location boundary - Admin only
pub async fn update_location_boundary(
    data: web::Data<crate::config::app_state::AppState>,
    location_id: web::Path<String>,
    req: web::Json<UpdateLocationRequest>,
) -> impl Responder {
    info!("Updating location boundary: {}", location_id.clone());

    let mut updates = Vec::new();
    
    if let Some(ref name) = req.name {
        updates.push(format!("name = '{}'", name));
    }
    
    if let Some(ref description) = req.description {
        updates.push(format!("description = '{}'", description));
    }
    
    if let Some(latitude) = req.latitude {
        if latitude < -90.0 || latitude > 90.0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "Latitude tidak valid"
            }));
        }
        updates.push(format!("latitude = {}", latitude));
    }
    
    if let Some(longitude) = req.longitude {
        if longitude < -180.0 || longitude > 180.0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "Longitude tidak valid"
            }));
        }
        updates.push(format!("longitude = {}", longitude));
    }
    
    if let Some(radius) = req.radius {
        if radius < 10 || radius > 10000 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "Radius harus antara 10 dan 10000 meter"
            }));
        }
        updates.push(format!("radius = {}", radius));
    }
    
    if let Some(is_active) = req.is_active {
        updates.push(format!("is_active = {}", is_active));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "No fields to update"
        }));
    }

    updates.push("updated_at = time::now()".to_string());
    let update_clause = updates.join(", ");

    let id_str = location_id.into_inner();
    let target_id = if id_str.contains(":") {
        id_str.clone()
    } else {
        format!("location_boundaries:{}", id_str)
    };
    let query = format!("UPDATE {} SET {}", target_id, update_clause);

    match data.db.query(&query).await {
        Ok(mut response) => {
            match response.take::<Vec<LocationBoundary>>(0) {
                Ok(updated) => {
                    if let Some(location) = updated.first() {
                        info!("Successfully updated location boundary: {}", target_id);
                        HttpResponse::Ok().json(serde_json::json!({
                            "success": true,
                            "message": "Location boundary berhasil diupdate",
                            "data": location
                        }))
                    } else {
                        HttpResponse::NotFound().json(serde_json::json!({
                            "success": false,
                            "message": "Location boundary not found"
                        }))
                    }
                }
                Err(e) => {
                    error!("Error parsing updated location: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": "Failed to update location boundary"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Database error updating location: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}

/// Delete location boundary - Admin only
pub async fn delete_location_boundary(
    data: web::Data<crate::config::app_state::AppState>,
    location_id: web::Path<String>,
) -> impl Responder {
    info!("Deleting location boundary: {}", location_id.clone());

    let id_str = location_id.into_inner();
    let target_id = if id_str.contains(":") {
        id_str.clone()
    } else {
        format!("location_boundaries:{}", id_str)
    };
    let query = format!("DELETE {}", target_id);

    match data.db.query(&query).await {
        Ok(_) => {
            info!("Successfully deleted location boundary: {}", target_id);
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Location boundary berhasil dihapus"
            }))
        }
        Err(e) => {
            error!("Database error deleting location: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Database error"
            }))
        }
    }
}
