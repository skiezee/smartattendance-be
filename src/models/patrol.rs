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

// --- Checkpoint Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Checkpoint {
    pub id: Option<Thing>,
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
    pub name: String,
    pub qr_code_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCheckpointRequest {
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
    pub employee_id: Thing,
    pub start_time: String,
    pub end_time: String,
    pub checkpoints: Vec<Thing>,
    pub status: String, // "scheduled", "in_progress", "completed"
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Safe serializable response — uses plain String IDs instead of Thing enum
/// to avoid serde_json::Value serialization errors from SurrealDB internals.
#[derive(Serialize, Debug)]
pub struct PatrolAssignmentResponse {
    pub id: String,
    pub employee_id: String,
    pub start_time: String,
    pub end_time: String,
    pub checkpoints: Vec<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<PatrolAssignment> for PatrolAssignmentResponse {
    fn from(a: PatrolAssignment) -> Self {
        Self {
            id: a.id.map(|t| t.id.to_string()).unwrap_or_default(),
            employee_id: a.employee_id.id.to_string(),
            start_time: a.start_time,
            end_time: a.end_time,
            checkpoints: a.checkpoints.iter().map(|c| c.id.to_string()).collect(),
            status: a.status,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePatrolAssignmentRequest {
    pub employee_id: String,
    pub start_time: String,
    pub end_time: String,
    pub checkpoints: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdatePatrolAssignmentRequest {
    pub employee_id: Option<String>,
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
