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
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<String>,
    pub hire_date: Option<String>,
    pub position: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
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
pub struct RefreshTokenRequest {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nik: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmployeeResponse {
    pub id: Option<surrealdb::sql::Thing>,
    pub nik: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hire_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emergency_contact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emergency_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_id: Option<surrealdb::sql::Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_id: Option<surrealdb::sql::Thing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employment_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_photo_url: Option<String>,
    pub attendance_requirement: Option<AttendanceRequirement>,
    #[serde(default)]
    pub face_enrolled: bool,
    #[serde(default)]
    pub fingerprint_enrolled: bool,
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
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<String>,
    pub hire_date: Option<String>,
    pub position: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
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
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<String>,
    pub hire_date: Option<String>,
    pub position: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
    pub attendance_requirement: Option<AttendanceRequirement>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulkAttendanceRequest {
    pub employee_niks: Vec<String>,
    pub attendance_requirement: AttendanceRequirement,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulkCreateEmployeeRequest {
    pub employees: Vec<CreateEmployeeRequest>,
}
