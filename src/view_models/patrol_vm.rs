use actix_web::web;
use chrono::Local;

use crate::config::app_state::AppState;
use crate::models::employee::EmployeeResponse;
use crate::models::patrol::{
    PatrolIncident, PatrolIncidentRequest, PatrolIncidentResponse,
    Checkpoint, CreateCheckpointRequest, UpdateCheckpointRequest,
    PatrolAssignment, PatrolAssignmentResponse,
    CreatePatrolAssignmentRequest, UpdatePatrolAssignmentRequest,
    PatrolLog, ScanCheckpointRequest
};
use surrealdb::sql::{Thing, Id};
use serde_json::{json, Value};

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

        // Create the incident record
        let incident = PatrolIncident {
            id: None,
            employee_id: employee_id.clone(),
            nik: payload.nik.clone(),
            title: payload.title.clone(),
            description: payload.description.clone(),
            latitude: payload.latitude,
            longitude: payload.longitude,
            timestamp: payload.timestamp.clone(),
            photo_base64: payload.photo_base64.clone(),
            created_at: Local::now().to_rfc3339(),
        };

        // Insert into database
        let created: Option<PatrolIncident> = data
            .db
            .create("patrol_incidents")
            .content(incident)
            .await
            .map_err(|e| format!("Failed to create incident: {}", e))?;

        let incident_id = created
            .and_then(|i| i.id)
            .map(|id| id.to_string())
            .unwrap_or_default();

        log::info!("Patrol incident created: {} by NIK: {}", incident_id, payload.nik);

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
        let area = crate::models::patrol::PatrolArea {
            id: None,
            name: payload.name.clone(),
            description: payload.description.clone(),
            created_at: Some(Local::now().to_rfc3339()),
            updated_at: Some(Local::now().to_rfc3339()),
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
        
        let query = format!("UPDATE {} SET name = '{}', description = '{}', updated_at = '{}'", 
            thing.to_string(), 
            payload.name, 
            payload.description.clone().unwrap_or_default(),
            Local::now().to_rfc3339()
        );
        
        let mut result = data.db.query(query).await.map_err(|e| e.to_string())?;
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

        let checkpoint = Checkpoint {
            id: None,
            area_id: area_thing,
            name: payload.name.clone(),
            qr_code_id: payload.qr_code_id.clone(),
            latitude: payload.latitude,
            longitude: payload.longitude,
            description: payload.description.clone(),
            created_at: Some(Local::now().to_rfc3339()),
            updated_at: Some(Local::now().to_rfc3339()),
        };

        let created: Option<Checkpoint> = data
            .db
            .create("checkpoints")
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
            .query("SELECT * FROM checkpoints ORDER BY created_at DESC")
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
        let thing = Thing::from(("checkpoints", Id::from(id)));
        
        let mut updates = Vec::new();
        if let Some(area_id) = &payload.area_id { updates.push(format!("area_id = type::thing('patrol_areas:{}')", area_id)); }
        if let Some(name) = &payload.name { updates.push(format!("name = '{}'", name)); }
        if let Some(qr) = &payload.qr_code_id { updates.push(format!("qr_code_id = '{}'", qr)); }
        if let Some(lat) = &payload.latitude { updates.push(format!("latitude = {}", lat)); }
        if let Some(lng) = &payload.longitude { updates.push(format!("longitude = {}", lng)); }
        if let Some(desc) = &payload.description { updates.push(format!("description = '{}'", desc)); }
        updates.push(format!("updated_at = '{}'", Local::now().to_rfc3339()));

        let query = format!("UPDATE {} SET {}", thing.to_string(), updates.join(", "));
        
        let mut result = data.db.query(query).await.map_err(|e| e.to_string())?;
        let updated: Option<Checkpoint> = result.take(0).map_err(|e| e.to_string())?;
        
        updated.ok_or_else(|| "Checkpoint not found".to_string())
    }

    pub async fn delete_checkpoint(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let _: Option<Checkpoint> = data.db.delete(("checkpoints", id)).await.map_err(|e| e.to_string())?;
        Ok("Checkpoint deleted successfully".to_string())
    }

    // --- Patrol Assignment Methods ---

    pub async fn create_patrol_assignment(
        payload: web::Json<CreatePatrolAssignmentRequest>,
        data: &web::Data<AppState>,
    ) -> Result<PatrolAssignment, String> {
        let table = if payload.assignee_type == "group" { "groups" } else { "employee" };
        let assignee_thing = Thing::from((table, Id::from(payload.assignee_id.as_str())));
        
        let checkpoint_things: Vec<Thing> = payload.checkpoints
            .iter()
            .map(|cp_id| Thing::from(("checkpoints", Id::from(cp_id.as_str()))))
            .collect();

        let assignment = PatrolAssignment {
            id: None,
            assignee_type: payload.assignee_type.clone(),
            assignee_id: assignee_thing,
            start_time: payload.start_time.clone(),
            end_time: payload.end_time.clone(),
            checkpoints: checkpoint_things,
            status: "scheduled".to_string(),
            created_at: Some(Local::now().to_rfc3339()),
            updated_at: Some(Local::now().to_rfc3339()),
        };

        let created: Option<PatrolAssignment> = data
            .db
            .create("patrol_assignments")
            .content(assignment)
            .await
            .map_err(|e| format!("Failed to create assignment: {}", e))?;

        created.ok_or_else(|| "Failed to return created assignment".to_string())
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

    pub async fn update_patrol_assignment(
        id: &str,
        payload: web::Json<UpdatePatrolAssignmentRequest>,
        data: &web::Data<AppState>,
    ) -> Result<PatrolAssignment, String> {
        let thing = Thing::from(("patrol_assignments", Id::from(id)));
        
        let mut existing_res = data.db.query("SELECT * FROM type::thing($thing)").bind(("thing", thing.clone())).await.map_err(|e| e.to_string())?;
        let mut existing: Option<PatrolAssignment> = existing_res.take(0).map_err(|e| e.to_string())?;
        
        let mut existing = existing.ok_or_else(|| "Assignment not found".to_string())?;

        if let (Some(a_type), Some(a_id)) = (&payload.assignee_type, &payload.assignee_id) {
            let table = if a_type == "group" { "groups" } else { "employee" };
            existing.assignee_type = a_type.clone();
            existing.assignee_id = Thing::from((table, Id::from(a_id.as_str())));
        }
        if let Some(start) = &payload.start_time { existing.start_time = start.clone(); }
        if let Some(end) = &payload.end_time { existing.end_time = end.clone(); }
        if let Some(status) = &payload.status { existing.status = status.clone(); }
        if let Some(cps) = &payload.checkpoints {
            existing.checkpoints = cps.iter().map(|c| Thing::from(("checkpoints", Id::from(c.as_str())))).collect();
        }
        existing.updated_at = Some(Local::now().to_rfc3339());

        let updated: Option<PatrolAssignment> = data.db.update(("patrol_assignments", id)).content(existing).await.map_err(|e| e.to_string())?;
        updated.ok_or_else(|| "Failed to update assignment".to_string())
    }

    pub async fn delete_patrol_assignment(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let _: Option<PatrolAssignment> = data.db.delete(("patrol_assignments", id)).await.map_err(|e| e.to_string())?;
        Ok("Assignment deleted successfully".to_string())
    }

    // --- Active Patrol Status & Live Tracking ---

    pub async fn get_active_patrol_status(
        data: &web::Data<AppState>,
    ) -> Result<Vec<PatrolAssignmentResponse>, String> {
        // Deserialize into PatrolAssignment then convert to safe response struct
        let mut result = data
            .db
            .query("SELECT * FROM patrol_assignments WHERE status = 'in_progress'")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let active: Vec<PatrolAssignment> = result
            .take(0)
            .map_err(|e| format!("Failed to parse active patrols: {}", e))?;

        Ok(active.into_iter().map(PatrolAssignmentResponse::from).collect())
    }

    pub async fn get_live_tracking(
        data: &web::Data<AppState>,
    ) -> Result<Vec<Value>, String> {
        // Fetch active assignments with embedded employee; latest scan log fetched client-side
        let mut result = data
            .db
            .query("SELECT * FROM patrol_assignments WHERE status = 'in_progress' FETCH employee_id, checkpoints")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let tracking_data: Vec<Value> = result
            .take(0)
            .map_err(|e| format!("Failed to parse live tracking data: {}", e))?;

        Ok(tracking_data)
    }
}
