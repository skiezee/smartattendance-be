use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShiftSchedule {
    pub id: Option<Thing>,
    pub employee_id: Thing,
    pub nik: String,
    pub employee_name: String,
    pub shift_type: String, // "PAGI", "SIANG", "MALAM"
    pub date: String,       // YYYY-MM-DD format
    pub start_time: String, // HH:mm format
    pub end_time: String,   // HH:mm format
    pub location: String,
    pub tasks: Vec<String>,
    pub status: String, // "SCHEDULED", "COMPLETED", "CANCELLED"
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateShiftRequest {
    pub nik: String,
    pub shift_type: String,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub location: String,
    pub tasks: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetShiftRequest {
    pub nik: String,
    pub start_date: Option<String>, // YYYY-MM-DD
    pub end_date: Option<String>,   // YYYY-MM-DD
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateShiftStatusRequest {
    pub shift_id: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShiftResponse {
    pub status: String,
    pub message: String,
    pub shift_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShiftListResponse {
    pub status: String,
    pub data: Vec<ShiftSchedule>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShiftStats {
    pub total_shifts: usize,
    pub completed_shifts: usize,
    pub upcoming_shifts: usize,
    pub cancelled_shifts: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShiftStatsResponse {
    pub status: String,
    pub stats: ShiftStats,
}
