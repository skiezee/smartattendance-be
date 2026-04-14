use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug)]
pub struct FaceEnrollRequest {
    pub nik: String,
    pub face_embedding: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FingerprintEnrollRequest {
    pub nik: String,
    // Fingerprint only saves status to DB since template is saved locally in Android device enclave
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnrollmentResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockInRequest {
    pub nik: String,
    pub lat: f32,
    pub lng: f32,
    pub method: String, // "face" | "fingerprint"
    pub face_confidence: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockInResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckEnrollmentRequest {
    pub nik: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckEnrollmentResponse {
    pub status: String,
    pub face_enrolled: bool,
    pub fingerprint_enrolled: bool,
    pub has_attended_today: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmployeeStatus {
    pub id: Thing,
    pub nik: String,
    pub face_enrolled: bool,
    pub fingerprint_enrolled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttendanceLogResponse {
    pub id: Option<Thing>,
    pub employee_id: Thing,
    pub date: Option<String>,
    pub check_in: Option<String>,
    pub check_out: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
}
