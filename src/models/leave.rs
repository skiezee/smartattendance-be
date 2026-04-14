use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaveRequestPayload {
    pub nik: String,
    pub leave_type: String,
    pub start_date: String,
    pub end_date: String,
    pub duration: i32,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaveResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateLeaveStatusRequest {
    pub id: String,
    pub stage: i32, // 1 for Line Manager, 2 for HCGA
    pub status: String, // "APPROVED" or "REJECTED"
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaveRecord {
    pub id: Option<Thing>,
    pub nik: String,
    pub leave_type: String,
    pub start_date: String,
    pub end_date: String,
    pub duration: i32,
    pub reason: String,
    pub status: String,
    pub stage1_status: String,
    pub stage2_status: String,
    pub created_at: String,
}
