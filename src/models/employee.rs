use serde::{Deserialize, Serialize};

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
}
