use actix_web::web;
use chrono::Local;

use crate::config::app_state::AppState;
use crate::models::employee::EmployeeResponse;
use crate::models::patrol::{PatrolIncident, PatrolIncidentRequest, PatrolIncidentResponse};

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
}
