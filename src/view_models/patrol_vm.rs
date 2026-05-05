use actix_web::web;
use chrono::Local;

use crate::config::app_state::AppState;
use crate::models::employee::EmployeeResponse;
use crate::models::patrol::{
    PatrolIncident, PatrolIncidentRequest, PatrolIncidentResponse,
    Checkpoint, CreateCheckpointRequest, UpdateCheckpointRequest,
    PatrolAssignment, PatrolAssignmentResponse,
    CreatePatrolAssignmentRequest, UpdatePatrolAssignmentRequest,
};
use crate::services::fcm_service::send_fcm_notification;
use surrealdb::sql::{Thing, Id};
use serde_json::Value;

/// Helper: extract a string ID from a SurrealDB Value (handles Thing/Object/String variants)
fn extract_id_from_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(m) => {
            // SurrealDB Thing as JSON: {"tb": "table", "id": {"String": "xxx"}} or {"String": "xxx"}
            if let Some(id_v) = m.get("id") {
                let tb = m.get("tb").and_then(|t| t.as_str()).unwrap_or("");
                let id_str = extract_id_from_value(id_v);
                if tb.is_empty() { id_str } else { format!("{}:{}", tb, id_str) }
            } else if let Some(s) = m.get("String").and_then(|t| t.as_str()) {
                s.to_string()
            } else {
                serde_json::to_string(v).unwrap_or_default()
            }
        }
        _ => serde_json::to_string(v).unwrap_or_default().trim_matches('"').to_string(),
    }
}

pub struct PatrolViewModel;

impl PatrolViewModel {
    pub async fn submit_incident(
        payload: web::Json<PatrolIncidentRequest>,
        data: &web::Data<AppState>,
    ) -> Result<PatrolIncidentResponse, String> {
        // First, verify that the employee exists and get their ID
        let mut result = data
            .db
            .query("SELECT * FROM employee WHERE type::string(nik) = type::string($nik)")
            .bind(("nik", payload.nik.clone()))
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let employees: Vec<EmployeeResponse> = result.take(0).map_err(|e| format!("Failed to parse employee: {}", e))?;

        if employees.is_empty() {
            return Err("Employee not found".to_string());
        }

        let employee = &employees[0];
        let employee_id = employee.id.clone().ok_or("Employee ID not found")?;

        // Save photo to file system if provided
        log::info!("Processing incident photo, has photo_base64: {}", payload.photo_base64.is_some());
        let photo_url = payload.photo_base64.as_deref()
            .and_then(|b64| {
                log::info!("Photo base64 provided, length: {}", b64.len());
                let result = Self::save_photo(b64, "incident");
                if result.is_none() {
                    log::error!("Failed to save incident photo!");
                }
                result
            });
        
        log::info!("Photo URL after save: {:?}", photo_url);

        // Create the incident record - let database set created_at with DEFAULT time::now()
        let query = r#"
            CREATE patrol_incidents CONTENT {
                employee_id: type::thing('employee', $employee_id),
                nik: $nik,
                title: $title,
                description: $description,
                latitude: $latitude,
                longitude: $longitude,
                timestamp: $timestamp,
                photo_url: $photo_url
            }
        "#;

        let employee_id_str = employee_id.to_string();
        let employee_id_part = employee_id_str.split(':').last().unwrap_or(&employee_id_str).to_string();

        // Clone photo_url to avoid lifetime issues
        let photo_url_for_binding = photo_url.clone();
        
        log::info!("Binding incident photo_url: {:?}", photo_url_for_binding);

        let mut result = data
            .db
            .query(query)
            .bind(("employee_id", employee_id_part))
            .bind(("nik", payload.nik.clone()))
            .bind(("title", payload.title.clone()))
            .bind(("description", payload.description.clone()))
            .bind(("latitude", payload.latitude))
            .bind(("longitude", payload.longitude))
            .bind(("timestamp", payload.timestamp.clone()))
            .bind(("photo_url", photo_url_for_binding))
            .await
            .map_err(|e| {
                log::error!("Database error creating incident: {}", e);
                format!("Failed to create incident: {}", e)
            })?;

        let created: Option<PatrolIncident> = result
            .take(0)
            .map_err(|e| {
                log::error!("Failed to parse created incident: {}", e);
                format!("Failed to parse created incident: {}", e)
            })?;

        let incident = created.ok_or_else(|| "Failed to return created incident".to_string())?;

        let incident_id = incident.id
            .map(|id| id.to_string())
            .unwrap_or_default();

        log::info!("Patrol incident created: {} by NIK: {} with photo_url: {:?}", incident_id, payload.nik, incident.photo_url);

        Ok(PatrolIncidentResponse {
            status: "success".to_string(),
            message: "Incident reported successfully".to_string(),
            incident_id: Some(incident_id),
        })
    }

    pub async fn get_all_incidents(
        data: &web::Data<AppState>,
    ) -> Result<Vec<PatrolIncident>, String> {
        let mut result = data
            .db
            .query("SELECT * FROM patrol_incidents ORDER BY created_at DESC")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let incidents: Vec<PatrolIncident> = result
            .take(0)
            .map_err(|e| format!("Failed to parse incidents: {}", e))?;

        Ok(incidents)
    }

    pub async fn get_incidents_by_nik(
        nik: &str,
        data: &web::Data<AppState>,
    ) -> Result<Vec<PatrolIncident>, String> {
        let nik_string = nik.to_string();
        let mut result = data
            .db
            .query("SELECT * FROM patrol_incidents WHERE nik = $nik ORDER BY created_at DESC")
            .bind(("nik", nik_string))
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let incidents: Vec<PatrolIncident> = result
            .take(0)
            .map_err(|e| format!("Failed to parse incidents: {}", e))?;

        Ok(incidents)
    }

    // --- Area Methods ---

    pub async fn create_area(
        payload: web::Json<crate::models::patrol::CreateAreaRequest>,
        data: &web::Data<AppState>,
    ) -> Result<crate::models::patrol::PatrolArea, String> {
        // Don't send created_at/updated_at - let database handle with DEFAULT
        let area = crate::models::patrol::PatrolArea {
            id: None,
            name: payload.name.clone(),
            description: payload.description.clone(),
            created_at: None,  // Let DB set with DEFAULT time::now()
            updated_at: None,  // Let DB set with DEFAULT time::now()
        };

        let created: Option<crate::models::patrol::PatrolArea> = data
            .db
            .create("patrol_areas")
            .content(area)
            .await
            .map_err(|e| format!("Failed to create area: {}", e))?;

        created.ok_or_else(|| "Failed to return created area".to_string())
    }

    pub async fn get_areas(
        data: &web::Data<AppState>,
    ) -> Result<Vec<crate::models::patrol::PatrolArea>, String> {
        let mut result = data
            .db
            .query("SELECT * FROM patrol_areas ORDER BY created_at DESC")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let areas: Vec<crate::models::patrol::PatrolArea> = result
            .take(0)
            .map_err(|e| format!("Failed to parse areas: {}", e))?;

        Ok(areas)
    }

    pub async fn update_area(
        id: &str,
        payload: web::Json<crate::models::patrol::CreateAreaRequest>,
        data: &web::Data<AppState>,
    ) -> Result<crate::models::patrol::PatrolArea, String> {
        let thing = Thing::from(("patrol_areas", Id::from(id)));
        
        let query = "UPDATE type::thing($thing) SET name = $name, description = $description, updated_at = time::now()";
        
        // Clone values to avoid lifetime issues
        let name_val = payload.name.clone();
        let description_val = payload.description.clone().unwrap_or_default();
        
        let mut result = data.db.query(query)
            .bind(("thing", thing))
            .bind(("name", name_val))
            .bind(("description", description_val))
            .await
            .map_err(|e| e.to_string())?;
        
        let updated: Option<crate::models::patrol::PatrolArea> = result.take(0).map_err(|e| e.to_string())?;
        
        updated.ok_or_else(|| "Area not found".to_string())
    }

    pub async fn delete_area(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let _: Option<crate::models::patrol::PatrolArea> = data.db.delete(("patrol_areas", id)).await.map_err(|e| e.to_string())?;
        Ok("Area deleted successfully".to_string())
    }

    // --- Checkpoint Methods ---

    pub async fn create_checkpoint(
        payload: web::Json<CreateCheckpointRequest>,
        data: &web::Data<AppState>,
    ) -> Result<Checkpoint, String> {
        let area_thing = payload.area_id.as_ref().map(|id| Thing::from(("patrol_areas", Id::from(id.as_str()))));

        // Handle photo upload if provided
        let photo_url = payload.photo_base64.as_ref().and_then(|base64| Self::save_photo(base64, "checkpoint"));

        let checkpoint = Checkpoint {
            id: None,
            area_id: area_thing,
            name: payload.name.clone(),
            qr_code_id: payload.qr_code_id.clone(),
            latitude: payload.latitude,
            longitude: payload.longitude,
            description: payload.description.clone(),
            photo_url,
            photo_base64: None, // Don't store base64 in DB
            created_at: None,  // Let DB set with DEFAULT time::now()
            updated_at: None,  // Let DB set with DEFAULT time::now()
        };

        let created: Option<Checkpoint> = data
            .db
            .create("patrol_checkpoints")  // Use correct table name
            .content(checkpoint)
            .await
            .map_err(|e| format!("Failed to create checkpoint: {}", e))?;

        created.ok_or_else(|| "Failed to return created checkpoint".to_string())
    }

    pub async fn get_checkpoints(
        data: &web::Data<AppState>,
    ) -> Result<Vec<Checkpoint>, String> {
        let mut result = data
            .db
            .query("SELECT * FROM patrol_checkpoints ORDER BY created_at DESC")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let checkpoints: Vec<Checkpoint> = result
            .take(0)
            .map_err(|e| format!("Failed to parse checkpoints: {}", e))?;

        Ok(checkpoints)
    }

    pub async fn update_checkpoint(
        id: &str,
        payload: web::Json<UpdateCheckpointRequest>,
        data: &web::Data<AppState>,
    ) -> Result<Checkpoint, String> {
        let thing = Thing::from(("patrol_checkpoints", Id::from(id)));
        
        let mut query = String::from("UPDATE type::thing($thing) SET updated_at = time::now()");
        
        // Clone values to avoid lifetime issues
        let area_id_val = payload.area_id.clone();
        let name_val = payload.name.clone();
        let qr_code_id_val = payload.qr_code_id.clone();
        let latitude_val = payload.latitude;
        let longitude_val = payload.longitude;
        let description_val = payload.description.clone();
        let photo_url_val = payload.photo_base64.as_ref().and_then(|base64| Self::save_photo(base64, "checkpoint"));
        
        if area_id_val.is_some() {
            query.push_str(", area_id = type::thing($area_table, $area_id)");
        }
        if name_val.is_some() {
            query.push_str(", name = $name");
        }
        if qr_code_id_val.is_some() {
            query.push_str(", qr_code_id = $qr_code_id");
        }
        if latitude_val.is_some() {
            query.push_str(", latitude = $latitude");
        }
        if longitude_val.is_some() {
            query.push_str(", longitude = $longitude");
        }
        if description_val.is_some() {
            query.push_str(", description = $description");
        }
        if photo_url_val.is_some() {
            query.push_str(", photo_url = $photo_url");
        }
        
        let mut db_query = data.db.query(&query).bind(("thing", thing));
        
        if let Some(area_id) = area_id_val {
            db_query = db_query
                .bind(("area_table", "patrol_areas".to_string()))
                .bind(("area_id", area_id));
        }
        if let Some(name) = name_val {
            db_query = db_query.bind(("name", name));
        }
        if let Some(qr) = qr_code_id_val {
            db_query = db_query.bind(("qr_code_id", qr));
        }
        if let Some(lat) = latitude_val {
            db_query = db_query.bind(("latitude", lat));
        }
        if let Some(lng) = longitude_val {
            db_query = db_query.bind(("longitude", lng));
        }
        if let Some(desc) = description_val {
            db_query = db_query.bind(("description", desc));
        }
        if let Some(photo_url) = photo_url_val {
            db_query = db_query.bind(("photo_url", photo_url));
        }
        
        let mut result = db_query.await.map_err(|e| e.to_string())?;
        let updated: Option<Checkpoint> = result.take(0).map_err(|e| e.to_string())?;
        
        updated.ok_or_else(|| "Checkpoint not found".to_string())
    }

    pub async fn delete_checkpoint(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let _: Option<Checkpoint> = data.db.delete(("patrol_checkpoints", id)).await.map_err(|e| e.to_string())?;
        Ok("Checkpoint deleted successfully".to_string())
    }

    // --- Patrol Assignment Methods ---

    pub async fn create_patrol_assignment(
        payload: web::Json<CreatePatrolAssignmentRequest>,
        data: &web::Data<AppState>,
    ) -> Result<PatrolAssignment, String> {
        let table = if payload.assignee_type == "group" { "groups" } else { "employee" };
        let assignee_thing = Thing::from((table, Id::from(payload.assignee_id.as_str())));
        
        // Convert checkpoint IDs to Thing references with correct table name
        let checkpoint_things: Vec<Thing> = payload.checkpoints
            .iter()
            .map(|cp_id| Thing::from(("patrol_checkpoints", Id::from(cp_id.as_str()))))
            .collect();

        let assignment = PatrolAssignment {
            id: None,
            assignee_type: payload.assignee_type.clone(),
            assignee_id: assignee_thing.clone(),
            start_time: payload.start_time.clone(),
            end_time: payload.end_time.clone(),
            checkpoints: checkpoint_things,
            status: "scheduled".to_string(),
            created_at: None,  // Let DB set with DEFAULT time::now()
            updated_at: None,  // Let DB set with DEFAULT time::now()
        };

        let created: Option<PatrolAssignment> = data
            .db
            .create("patrol_assignments")
            .content(assignment)
            .await
            .map_err(|e| format!("Failed to create assignment: {}", e))?;

        let created_assignment = created.ok_or_else(|| "Failed to return created assignment".to_string())?;

        // ✅ Send FCM notification to assignee
        let assignee_id_str = payload.assignee_id.clone();
        let start_time = payload.start_time.clone();
        let end_time = payload.end_time.clone();
        let checkpoint_count = payload.checkpoints.len();
        let assignee_type = payload.assignee_type.clone();
        
        // Spawn async task untuk send notification (tidak block response)
        let db_clone = data.db.clone();
        actix_web::rt::spawn(async move {
            log::info!("🔔 Sending patrol assignment notification to {} {}", assignee_type, assignee_id_str);
            
            // Get FCM tokens based on assignee type
            let fcm_tokens: Vec<String> = if assignee_type == "group" {
                // Get all employees in the group
                match db_clone
                    .query("SELECT fcm_token FROM employee WHERE type::string(group_id) = type::string($group_id) AND fcm_token != NONE")
                    .bind(("group_id", assignee_id_str.clone()))
                    .await
                {
                    Ok(mut result) => {
                        let employees: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
                        employees
                            .iter()
                            .filter_map(|e| e.get("fcm_token").and_then(|t| t.as_str()).map(String::from))
                            .collect()
                    }
                    Err(e) => {
                        log::error!("Failed to get group members: {}", e);
                        vec![]
                    }
                }
            } else {
                // Get single employee FCM token
                // assignee_id_str is just the ID part (e.g., "ihg8kdl4b4w3eefzpkin")
                // We need to query by ID directly
                match db_clone
                    .query("SELECT fcm_token FROM employee WHERE type::string(id) = type::string($emp_id) AND fcm_token != NONE")
                    .bind(("emp_id", format!("employee:{}", assignee_id_str)))
                    .await
                {
                    Ok(mut result) => {
                        let employees: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
                        log::debug!("Query result for employee:{}: {:?}", assignee_id_str, employees);
                        employees
                            .iter()
                            .filter_map(|e| e.get("fcm_token").and_then(|t| t.as_str()).map(String::from))
                            .collect()
                    }
                    Err(e) => {
                        log::error!("Failed to get employee FCM token: {}", e);
                        vec![]
                    }
                }
            };

            if fcm_tokens.is_empty() {
                log::warn!("No FCM tokens found for {} {}. Trying alternative query...", assignee_type, assignee_id_str);
                
                // ✅ FALLBACK: Try query by ID without table prefix
                let fallback_tokens: Vec<String> = match db_clone
                    .query("SELECT fcm_token FROM employee WHERE id CONTAINS $id_part AND fcm_token != NONE")
                    .bind(("id_part", assignee_id_str.clone()))
                    .await
                {
                    Ok(mut result) => {
                        let employees: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
                        log::debug!("Fallback query result: {:?}", employees);
                        employees
                            .iter()
                            .filter_map(|e| e.get("fcm_token").and_then(|t| t.as_str()).map(String::from))
                            .collect()
                    }
                    Err(e) => {
                        log::error!("Fallback query failed: {}", e);
                        vec![]
                    }
                };
                
                if fallback_tokens.is_empty() {
                    log::error!("❌ No FCM tokens found even with fallback query. Employee may not have logged in or FCM token not saved.");
                    return;
                }
                
                log::info!("Found {} FCM token(s) from fallback query", fallback_tokens.len());
                
                // Send with fallback tokens
                let project_id = std::env::var("FIREBASE_PROJECT_ID")
                    .unwrap_or_else(|_| "smart-attendance-eef71".to_string());
                
                let title = "🚨 Penugasan Patroli Baru!";
                let body = format!(
                    "Shift {}–{} • {} titik checkpoint. Tap untuk membuka aplikasi.",
                    start_time, end_time, checkpoint_count
                );

                for token in fallback_tokens {
                    match send_fcm_notification(&project_id, &token, title, &body).await {
                        Ok(_) => log::info!("✅ FCM notification sent successfully to token: {}...", &token[..20.min(token.len())]),
                        Err(e) => log::error!("❌ Failed to send FCM notification: {}", e),
                    }
                }
                
                return;
            }

            log::info!("Found {} FCM token(s) for notification", fcm_tokens.len());

            // Send notification to all tokens
            let project_id = std::env::var("FIREBASE_PROJECT_ID")
                .unwrap_or_else(|_| "smart-attendance-eef71".to_string());
            
            let title = "🚨 Penugasan Patroli Baru!";
            let body = format!(
                "Shift {}–{} • {} titik checkpoint. Tap untuk membuka aplikasi.",
                start_time, end_time, checkpoint_count
            );

            for token in fcm_tokens {
                match send_fcm_notification(&project_id, &token, title, &body).await {
                    Ok(_) => log::info!("✅ FCM notification sent successfully to token: {}...", &token[..20.min(token.len())]),
                    Err(e) => log::error!("❌ Failed to send FCM notification: {}", e),
                }
            }
        });

        Ok(created_assignment)
    }

    pub async fn get_patrol_assignments(
        data: &web::Data<AppState>,
    ) -> Result<Vec<PatrolAssignmentResponse>, String> {
        let mut result = data
            .db
            .query("SELECT * FROM patrol_assignments ORDER BY created_at DESC")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let assignments: Vec<PatrolAssignment> = result
            .take(0)
            .map_err(|e| format!("Failed to parse assignments: {}", e))?;

        Ok(assignments.into_iter().map(PatrolAssignmentResponse::from).collect())
    }

    /// Get patrol assignments for a specific employee by NIK.
    /// Joins through the employee table since assignee_id is an employee record reference.
    pub async fn get_patrol_assignments_by_nik(
        nik: &str,
        data: &web::Data<AppState>,
    ) -> Result<Vec<PatrolAssignmentResponse>, String> {
        // 1. Resolve NIK → employee record ID
        let mut emp_result = data
            .db
            .query("SELECT type::string(id) as id FROM employee WHERE type::string(nik) = type::string($nik)")
            .bind(("nik", nik.to_string()))
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let employees: Vec<serde_json::Value> = emp_result
            .take(0)
            .map_err(|e| format!("Failed to parse employee: {}", e))?;

        if employees.is_empty() {
            log::warn!("No employee found for NIK: {}", nik);
            return Ok(vec![]);
        }

        // 2. Get employee record ID string (e.g. "employee:abc123")
        let emp_id = employees[0]["id"].as_str().unwrap_or_default().to_string();

        log::info!("Fetching assignments for employee ID: {} (NIK: {})", emp_id, nik);

        // 3. Query assignments where assignee_id matches the employee record
        let mut result = data
            .db
            .query("SELECT * FROM patrol_assignments WHERE type::string(assignee_id) = type::string($emp_id) ORDER BY created_at DESC")
            .bind(("emp_id", emp_id))
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let assignments: Vec<PatrolAssignment> = result
            .take(0)
            .map_err(|e| format!("Failed to parse assignments: {}", e))?;

        log::info!("Found {} assignments for NIK: {}", assignments.len(), nik);
        Ok(assignments.into_iter().map(PatrolAssignmentResponse::from).collect())
    }

    pub async fn update_patrol_assignment(
        id: &str,
        payload: web::Json<UpdatePatrolAssignmentRequest>,
        data: &web::Data<AppState>,
    ) -> Result<PatrolAssignment, String> {
        let id_part = if id.contains(':') {
            id.split(':').last().unwrap_or(id)
        } else {
            id
        };
        
        // Build update query dynamically to only update provided fields
        let mut query = String::from("UPDATE type::thing($thing) SET updated_at = time::now()");
        
        // Clone values to avoid lifetime issues
        let assignee_type_val = payload.assignee_type.clone();
        let assignee_id_val = payload.assignee_id.clone();
        let start_time_val = payload.start_time.clone();
        let end_time_val = payload.end_time.clone();
        let status_val = payload.status.clone();
        let checkpoints_val = payload.checkpoints.clone();
        
        if assignee_type_val.is_some() && assignee_id_val.is_some() {
            query.push_str(", assignee_type = $assignee_type, assignee_id = type::thing($assignee_table, $assignee_id)");
        }
        if start_time_val.is_some() {
            query.push_str(", start_time = $start_time");
        }
        if end_time_val.is_some() {
            query.push_str(", end_time = $end_time");
        }
        if status_val.is_some() {
            query.push_str(", status = $status");
        }
        if checkpoints_val.is_some() {
            query.push_str(", checkpoints = $checkpoints");
        }
        
        let thing = Thing::from(("patrol_assignments", Id::from(id_part)));
        let mut db_query = data.db.query(&query).bind(("thing", thing));
        
        if let (Some(a_type), Some(a_id)) = (assignee_type_val, assignee_id_val) {
            let table = if a_type == "group" { "groups".to_string() } else { "employees".to_string() };
            db_query = db_query
                .bind(("assignee_type", a_type))
                .bind(("assignee_table", table))
                .bind(("assignee_id", a_id));
        }
        if let Some(start) = start_time_val {
            db_query = db_query.bind(("start_time", start));
        }
        if let Some(end) = end_time_val {
            db_query = db_query.bind(("end_time", end));
        }
        if let Some(status) = status_val {
            db_query = db_query.bind(("status", status));
        }
        if let Some(cps) = checkpoints_val {
            let checkpoint_things: Vec<Thing> = cps.iter()
                .map(|c| Thing::from(("patrol_checkpoints", Id::from(c.as_str()))))
                .collect();
            db_query = db_query.bind(("checkpoints", checkpoint_things));
        }
        
        let mut result = db_query.await.map_err(|e| e.to_string())?;
        let updated: Option<PatrolAssignment> = result.take(0).map_err(|e| e.to_string())?;
        
        updated.ok_or_else(|| "Failed to update assignment".to_string())
    }

    pub async fn delete_patrol_assignment(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let id_part = if id.contains(':') {
            id.split(':').last().unwrap_or(id)
        } else {
            id
        };
        let _: Option<PatrolAssignment> = data.db.delete(("patrol_assignments", id_part)).await.map_err(|e| e.to_string())?;
        Ok("Assignment deleted successfully".to_string())
    }

    // --- Active Patrol Status & Live Tracking ---

    pub async fn get_active_patrol_status(
        data: &web::Data<AppState>,
    ) -> Result<Vec<PatrolAssignmentResponse>, String> {
        // Simpler query: just get in_progress assignments and fetch checkpoints (no nested area fetch)
        let mut result = data
            .db
            .query("SELECT * FROM patrol_assignments WHERE status = 'in_progress' FETCH checkpoints")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        // Define a local struct to capture the fetched data correctly
        #[derive(serde::Deserialize)]
        struct FetchedAssignment {
            id: Thing,
            assignee_type: String,
            assignee_id: Thing,
            start_time: String,
            end_time: String,
            status: String,
            checkpoints: Vec<Checkpoint>,
            created_at: Option<String>,
            updated_at: Option<String>,
        }

        let active_raw: Vec<FetchedAssignment> = result
            .take(0)
            .map_err(|e| format!("Failed to parse active patrols: {}", e))?;

        let mut responses = Vec::new();

        for val in active_raw {
            // Extract id
            let id_raw = val.id.to_string();

            let assignee_type = val.assignee_type.clone();
            let assignee_id_raw = val.assignee_id.to_string();

            let start_time = val.start_time.clone();
            let end_time = val.end_time.clone();
            let status = val.status.clone();

            let id_part = id_raw.split(':').last().unwrap_or(&id_raw).to_string();
            let a_id_part = assignee_id_raw.split(':').last().unwrap_or(&assignee_id_raw).to_string();

            // Get patrol logs for this assignment
            let mut log_res = data.db
                .query("SELECT * FROM patrol_logs WHERE assignment_id = type::thing('patrol_assignments', $id)")
                .bind(("id", id_part.clone()))
                .await
                .map_err(|e| e.to_string())?;
            let logs: Vec<crate::models::patrol::PatrolLog> = log_res.take(0).unwrap_or_default();

            // Parse checkpoint_objs
            let mut details = Vec::new();
            let mut visited_count = 0usize;
            let mut first_area_id: Option<String> = None;
            let first_area_name: Option<String> = None;
            let cp_arr_len;

            if !val.checkpoints.is_empty() {
                cp_arr_len = val.checkpoints.len();
                for cp_val in &val.checkpoints {
                    let cp_id = cp_val.id.as_ref().map(|t| t.to_string()).unwrap_or_default();
                    let cp_name = cp_val.name.clone();
                    let cp_qr = cp_val.qr_code_id.clone();

                    // area_id string
                    if first_area_id.is_none() {
                        if let Some(area_thing) = &cp_val.area_id {
                            first_area_id = Some(area_thing.to_string());
                        }
                    }

                    let log_entry = logs.iter().find(|l| {
                        let l_id = l.checkpoint_id.to_string();
                        l_id == cp_id || l_id.ends_with(&format!(":{}", cp_id.split(':').last().unwrap_or(&cp_id)))
                    });
                    let is_visited = log_entry.is_some();
                    if is_visited { visited_count += 1; }

                    details.push(crate::models::patrol::CheckpointDetail {
                        id: cp_id,
                        name: cp_name,
                        qr_code_id: cp_qr,
                        status: if is_visited { "visited".to_string() } else { "pending".to_string() },
                        scanned_at: log_entry.map(|l| l.scanned_at.clone()),
                    });
                }
            } else {
                cp_arr_len = 0;
            }

            let progress = if cp_arr_len > 0 {
                (visited_count as f64 / cp_arr_len as f64) * 100.0
            } else { 0.0 };

            // Get assignee name
            let table = if assignee_type == "group" { "groups" } else { "employee" };
            let mut name_res = data.db
                .query(format!("SELECT * FROM type::thing('{}', $id)", table))
                .bind(("id", a_id_part))
                .await
                .map_err(|e| e.to_string())?;

            let assignee_name = if assignee_type == "group" {
                let g: Option<crate::models::group::EmployeeGroup> = name_res.take(0).unwrap_or(None);
                g.map(|x| x.name)
            } else {
                let e: Option<crate::models::employee::EmployeeResponse> = name_res.take(0).unwrap_or(None);
                e.map(|x| x.full_name)
            };

            let created_at = val.created_at.clone();
            let updated_at = val.updated_at.clone();

            responses.push(PatrolAssignmentResponse {
                id: id_raw,
                assignee_type,
                assignee_id: assignee_id_raw,
                assignee_name,
                start_time,
                end_time,
                status,
                checkpoints: vec![],
                checkpoint_details: Some(details),
                progress,
                area_id: first_area_id,
                area_name: first_area_name,
                created_at,
                updated_at,
            });
        }

        Ok(responses)
    }



    pub async fn scan_checkpoint(
        payload: web::Json<crate::models::patrol::ScanCheckpointRequest>,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let assignment_id = Thing::from(("patrol_assignments", Id::from(payload.assignment_id.as_str())));
        let checkpoint_id = Thing::from(("patrol_checkpoints", Id::from(payload.checkpoint_id.as_str())));

        let log_entry = crate::models::patrol::PatrolLog {
            id: None,
            assignment_id,
            checkpoint_id,
            scanned_at: Local::now().to_rfc3339(),
            status: payload.status.clone().unwrap_or_else(|| "on_time".to_string()),
            latitude: payload.latitude,
            longitude: payload.longitude,
        };

        let _: Option<crate::models::patrol::PatrolLog> = data
            .db
            .create("patrol_logs")
            .content(log_entry)
            .await
            .map_err(|e| format!("Failed to record scan: {}", e))?;

        Ok("Checkpoint scanned successfully".to_string())
    }

    pub async fn get_live_tracking(
        data: &web::Data<AppState>,
    ) -> Result<Vec<Value>, String> {
        // Fetch active assignments with embedded employee; latest scan log fetched client-side
        let mut result = data
            .db
            .query("SELECT *, type::string(id) as id FROM patrol_assignments WHERE status = 'in_progress' FETCH assignee_id, checkpoints")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let tracking_data: Vec<Value> = result
            .take(0)
            .map_err(|e| format!("Failed to parse live tracking data: {}", e))?;

        Ok(tracking_data)
    }

    // ─── Checkpoint Report Methods ────────────────────────────────────────────

    /// Save a base64 photo to disk and return the URL path.
    fn save_photo(photo_base64: &str, prefix: &str) -> Option<String> {
        use std::fs;
        
        log::info!("Attempting to save photo with prefix: {}", prefix);
        log::debug!("Photo base64 length: {}", photo_base64.len());
        
        let upload_dir = "uploads/patrol";
        if let Err(e) = fs::create_dir_all(upload_dir) {
            log::error!("Failed to create upload directory: {}", e);
            return None;
        }
        
        let filename = format!("{}_{}.jpg", prefix, uuid::Uuid::new_v4());
        let path = format!("{}/{}", upload_dir, filename);
        
        log::debug!("Saving photo to: {}", path);
        
        // Remove data:image/jpeg;base64, prefix if present
        let b64 = if let Some(idx) = photo_base64.find(',') {
            &photo_base64[idx + 1..]
        } else {
            photo_base64
        };
        
        let bytes = match base64_decode(b64) {
            Ok(b) => {
                log::info!("Successfully decoded base64, size: {} bytes", b.len());
                b
            },
            Err(e) => {
                log::error!("Failed to decode base64: {}", e);
                return None;
            }
        };
        
        match fs::write(&path, &bytes) {
            Ok(_) => {
                let url = format!("/uploads/patrol/{}", filename);
                log::info!("Photo saved successfully: {}", url);
                Some(url)
            },
            Err(e) => {
                log::error!("Failed to write photo file: {}", e);
                None
            }
        }
    }

    pub async fn create_checkpoint_report(
        payload: web::Json<crate::models::patrol::CreateCheckpointReportRequest>,
        data: &web::Data<AppState>,
    ) -> Result<crate::models::patrol::CheckpointReport, String> {
        log::info!("Creating checkpoint report for checkpoint: {}", payload.checkpoint_id);
        log::info!("Has photo_base64: {}", payload.photo_base64.is_some());
        
        let photo_url = payload.photo_base64.as_deref()
            .and_then(|b64| {
                log::info!("Photo base64 provided, length: {}", b64.len());
                let result = Self::save_photo(b64, "cp_report");
                if result.is_none() {
                    log::error!("Failed to save checkpoint report photo!");
                }
                result
            });
        
        log::info!("Photo URL after save: {:?}", photo_url);

        // Use query to let database set created_at and updated_at with DEFAULT time::now()
        let query = r#"
            CREATE checkpoint_reports CONTENT {
                assignment_id: $assignment_id,
                checkpoint_id: $checkpoint_id,
                nik: $nik,
                report: $report,
                photo_url: $photo_url
            }
        "#;

        // Clone photo_url to avoid lifetime issues
        let photo_url_for_binding = photo_url.clone();
        
        log::info!("Binding photo_url: {:?}", photo_url_for_binding);

        let mut result = data
            .db
            .query(query)
            .bind(("assignment_id", payload.assignment_id.clone()))
            .bind(("checkpoint_id", payload.checkpoint_id.clone()))
            .bind(("nik", payload.nik.clone()))
            .bind(("report", payload.report.clone()))
            .bind(("photo_url", photo_url_for_binding))
            .await
            .map_err(|e| {
                log::error!("Database error creating checkpoint report: {}", e);
                format!("Failed to create checkpoint report: {}", e)
            })?;

        let created: Option<crate::models::patrol::CheckpointReport> = result
            .take(0)
            .map_err(|e| {
                log::error!("Failed to parse created checkpoint report: {}", e);
                format!("Failed to parse created checkpoint report: {}", e)
            })?;

        let report = created.ok_or_else(|| "Failed to return created checkpoint report".to_string())?;
        
        log::info!("Checkpoint report created successfully with photo_url: {:?}", report.photo_url);
        
        Ok(report)
    }

    pub async fn get_checkpoint_reports(
        nik: Option<&str>,
        data: &web::Data<AppState>,
    ) -> Result<Vec<crate::models::patrol::CheckpointReport>, String> {
        let query = match nik {
            Some(n) => format!(
                "SELECT * FROM checkpoint_reports WHERE nik = '{}' ORDER BY created_at DESC", n
            ),
            None => "SELECT * FROM checkpoint_reports ORDER BY created_at DESC".to_string(),
        };
        let mut result = data.db.query(&query).await
            .map_err(|e| format!("Database query error: {}", e))?;
        let reports: Vec<crate::models::patrol::CheckpointReport> = result
            .take(0)
            .map_err(|e| format!("Failed to parse checkpoint reports: {}", e))?;
        Ok(reports)
    }

    pub async fn get_checkpoint_report_by_id(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<crate::models::patrol::CheckpointReport, String> {
        let id_part = if id.contains(':') { id.split(':').last().unwrap_or(id) } else { id };
        let result: Option<crate::models::patrol::CheckpointReport> = data
            .db
            .select(("checkpoint_reports", id_part))
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        result.ok_or_else(|| "Checkpoint report not found".to_string())
    }

    pub async fn update_checkpoint_report(
        id: &str,
        payload: web::Json<crate::models::patrol::UpdateCheckpointReportRequest>,
        data: &web::Data<AppState>,
    ) -> Result<crate::models::patrol::CheckpointReport, String> {
        let id_part = if id.contains(':') { id.split(':').last().unwrap_or(id) } else { id };
        
        // Build update query dynamically
        let mut query = String::from("UPDATE type::thing($thing) SET updated_at = time::now()");
        
        // Clone values to avoid lifetime issues
        let report_val = payload.report.clone();
        let photo_url_val = payload.photo_base64.as_ref().and_then(|base64| Self::save_photo(base64, "cp_report"));
        
        if report_val.is_some() {
            query.push_str(", report = $report");
        }
        if photo_url_val.is_some() {
            query.push_str(", photo_url = $photo_url");
        }
        
        let thing = Thing::from(("checkpoint_reports", Id::from(id_part)));
        let mut db_query = data.db.query(&query).bind(("thing", thing));
        
        if let Some(report_text) = report_val {
            db_query = db_query.bind(("report", report_text));
        }
        if let Some(photo_url) = photo_url_val {
            db_query = db_query.bind(("photo_url", photo_url));
        }
        
        let mut result = db_query.await.map_err(|e| format!("Failed to update checkpoint report: {}", e))?;
        let updated: Option<crate::models::patrol::CheckpointReport> = result.take(0).map_err(|e| e.to_string())?;
        
        updated.ok_or_else(|| "Failed to update checkpoint report".to_string())
    }

    pub async fn delete_checkpoint_report_by_id(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<(), String> {
        let id_part = if id.contains(':') { id.split(':').last().unwrap_or(id) } else { id };
        let _: Option<crate::models::patrol::CheckpointReport> = data
            .db
            .delete(("checkpoint_reports", id_part))
            .await
            .map_err(|e| format!("Failed to delete checkpoint report: {}", e))?;
        Ok(())
    }

    // ─── Incident Update/Delete Methods ──────────────────────────────────────

    pub async fn update_incident(
        id: &str,
        payload: web::Json<crate::models::patrol::UpdateIncidentRequest>,
        data: &web::Data<AppState>,
    ) -> Result<PatrolIncident, String> {
        let id_part = if id.contains(':') { id.split(':').last().unwrap_or(id) } else { id };
        
        // Build update query dynamically
        let mut query = String::from("UPDATE type::thing($thing)");
        let mut updates = Vec::new();
        
        // Clone values to avoid lifetime issues
        let title_val = payload.title.clone();
        let description_val = payload.description.clone();
        let photo_base64_val = payload.photo_base64.clone();
        
        if title_val.is_some() {
            updates.push("title = $title");
        }
        if description_val.is_some() {
            updates.push("description = $description");
        }
        if photo_base64_val.is_some() {
            updates.push("photo_base64 = $photo_base64");
        }
        
        if !updates.is_empty() {
            query.push_str(" SET ");
            query.push_str(&updates.join(", "));
        }
        
        let thing = Thing::from(("patrol_incidents", Id::from(id_part)));
        let mut db_query = data.db.query(&query).bind(("thing", thing));
        
        if let Some(title) = title_val {
            db_query = db_query.bind(("title", title));
        }
        if let Some(desc) = description_val {
            db_query = db_query.bind(("description", desc));
        }
        if let Some(b64) = photo_base64_val {
            db_query = db_query.bind(("photo_base64", b64));
        }
        
        let mut result = db_query.await.map_err(|e| format!("Failed to update incident: {}", e))?;
        let updated: Option<PatrolIncident> = result.take(0).map_err(|e| e.to_string())?;
        
        updated.ok_or_else(|| "Failed to update incident".to_string())
    }

    pub async fn delete_incident(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<(), String> {
        let id_part = if id.contains(':') { id.split(':').last().unwrap_or(id) } else { id };
        let _: Option<PatrolIncident> = data
            .db
            .delete(("patrol_incidents", id_part))
            .await
            .map_err(|e| format!("Failed to delete incident: {}", e))?;
        Ok(())
    }
}

/// Minimal base64 decoder (avoids adding a new dependency — uses std only)
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    // Remove whitespace/newlines that base64 encoders sometimes add
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [0u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }
    let bytes = clean.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut i = 0;
    while i + 3 < bytes.len() {
        let b0 = lookup[bytes[i] as usize];
        let b1 = lookup[bytes[i + 1] as usize];
        let b2 = if bytes[i + 2] == b'=' { 0 } else { lookup[bytes[i + 2] as usize] };
        let b3 = if bytes[i + 3] == b'=' { 0 } else { lookup[bytes[i + 3] as usize] };
        out.push((b0 << 2) | (b1 >> 4));
        if bytes[i + 2] != b'=' { out.push((b1 << 4) | (b2 >> 2)); }
        if bytes[i + 3] != b'=' { out.push((b2 << 6) | b3); }
        i += 4;
    }
    Ok(out)
}
