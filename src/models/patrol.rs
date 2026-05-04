use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug)]
pub struct PatrolIncidentRequest {
    pub nik: String,
    pub title: String,
    pub description: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: String,
    pub photo_base64: Option<String>, // Base64 encoded photo
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatrolIncidentResponse {
    pub status: String,
    pub message: String,
    pub incident_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatrolIncident {
    pub id: Option<Thing>,
    pub employee_id: Thing,
    pub nik: String,
    pub title: String,
    pub description: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: String,
    pub photo_base64: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetIncidentsResponse {
    pub status: String,
    pub data: Vec<PatrolIncident>,
}

// --- Area Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatrolArea {
    pub id: Option<Thing>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAreaRequest {
    pub name: String,
    pub description: Option<String>,
}

// --- Checkpoint Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Checkpoint {
    pub id: Option<Thing>,
    pub area_id: Option<Thing>, // Reference to patrol_area
    pub name: String,
    pub qr_code_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCheckpointRequest {
    pub area_id: Option<String>,
    pub name: String,
    pub qr_code_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCheckpointRequest {
    pub area_id: Option<String>,
    pub name: Option<String>,
    pub qr_code_id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
}

// --- Patrol Assignment Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatrolAssignment {
    pub id: Option<Thing>,
    #[serde(default = "default_assignee_type")]
    pub assignee_type: String, // "individual", "group"
    #[serde(alias = "employee_id")]
    pub assignee_id: Thing,    // Links to employee:id or group:id
    pub start_time: String,
    pub end_time: String,
    pub checkpoints: Vec<Thing>,
    #[serde(default = "default_status")]
    pub status: String, // "scheduled", "in_progress", "completed"
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_assignee_type() -> String {
    "individual".to_string()
}

fn default_status() -> String {
    "scheduled".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckpointDetail {
    pub id: String,
    pub name: String,
    pub qr_code_id: String,
    pub status: String, // "pending", "visited"
    pub scanned_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatrolAssignmentResponse {
    pub id: String,
    pub assignee_type: String,
    pub assignee_id: String,
    pub assignee_name: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub checkpoints: Vec<String>,
    pub checkpoint_details: Option<Vec<CheckpointDetail>>,
    pub status: String,
    pub progress: f64,
    pub area_id: Option<String>,
    pub area_name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<PatrolAssignment> for PatrolAssignmentResponse {
    fn from(a: PatrolAssignment) -> Self {
        Self {
            id: a.id.map(|t| t.to_string()).unwrap_or_default(),
            assignee_type: a.assignee_type,
            assignee_id: a.assignee_id.to_string(),
            assignee_name: None,
            start_time: a.start_time,
            end_time: a.end_time,
            checkpoints: a.checkpoints.iter().map(|c| c.to_string()).collect(),
            checkpoint_details: None,
            status: a.status,
            progress: 0.0,
            area_id: None,
            area_name: None,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePatrolAssignmentRequest {
    pub assignee_type: String,
    pub assignee_id: String,
    pub start_time: String,
    pub end_time: String,
    pub checkpoints: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdatePatrolAssignmentRequest {
    pub assignee_type: Option<String>,
    pub assignee_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub checkpoints: Option<Vec<String>>,
    pub status: Option<String>,
}

// --- Patrol Log Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatrolLog {
    pub id: Option<Thing>,
    pub assignment_id: Thing,
    pub checkpoint_id: Thing,
    pub scanned_at: String,
    pub status: String, // "on_time", "late"
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScanCheckpointRequest {
    pub assignment_id: String,
    pub checkpoint_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub status: Option<String>,
}
