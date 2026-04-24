use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttendanceRequirement {
    pub wifi_enabled: bool,
    pub wifi_ssids: Option<Vec<String>>,
    pub location_enabled: bool,
    pub location_boundaries: Option<Vec<String>>,
    pub face_recognition_enabled: bool,
    pub fingerprint_enabled: bool,
}

impl Default for AttendanceRequirement {
    fn default() -> Self {
        Self {
            wifi_enabled: false,
            wifi_ssids: None,
            location_enabled: false,
            location_boundaries: None,
            face_recognition_enabled: false,
            fingerprint_enabled: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Employee {
    pub nik: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub password_hash: String,
    pub department: Option<String>,
    pub status: Option<String>,
    pub fcm_token: Option<String>,
    pub attendance_requirement: Option<AttendanceRequirement>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub nik: String,
    pub password: String,
    pub fcm_token: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FcmTokenRequest {
    pub nik: String,
    pub fcm_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub nik: String,
    pub full_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmployeeResponse {
    pub id: Option<surrealdb::sql::Thing>,
    pub nik: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub department: Option<String>,
    pub status: Option<String>,
    pub attendance_requirement: Option<AttendanceRequirement>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateEmployeeRequest {
    pub nik: String,
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub department: Option<String>,
    pub status: Option<String>,
    pub attendance_requirement: Option<AttendanceRequirement>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateEmployeeRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
    pub status: Option<String>,
    pub attendance_requirement: Option<AttendanceRequirement>,
}
