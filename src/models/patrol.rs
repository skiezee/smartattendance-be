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
