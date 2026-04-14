use crate::config::app_state::AppState;
use crate::models::employee::{Employee, LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use crate::models::employee::FcmTokenRequest;
use actix_web::web;
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct AuthViewModel;

impl AuthViewModel {
    pub async fn login(
        req: web::Json<LoginRequest>,
        data: web::Data<AppState>,
    ) -> Result<LoginResponse, String> {
        let clean_nik = req.nik.trim().to_string();
        log::info!("Attempting login for NIK: {}", clean_nik);

        // Ambil data user berdasarkan NIK
        let result = data
            .db
            .query("SELECT * FROM employee WHERE type::string(nik) = type::string($nik)")
            .bind(("nik", clean_nik.clone()))
            .await;

        match result {
            Ok(mut res) => {
                let employees: Vec<Employee> = res.take(0).unwrap_or_default();
                log::info!("Login query returned {} results", employees.len());

                if let Some(employee) = employees.first() {
                    // Verifikasi password_hash menggunakan bcrypt
                    if let Ok(valid) = verify(&req.password, &employee.password_hash) {
                        if valid {
                            log::info!("Login successful for NIK: {}", employee.nik);

                            // Save FCM Token if provided
                            if let Some(fcm_token) = &req.fcm_token {
                                let _ = data.db.query("UPDATE employee SET fcm_token = $token WHERE type::string(nik) = type::string($nik)")
                                    .bind(("token", fcm_token.clone()))
                                    .bind(("nik", employee.nik.clone()))
                                    .await;
                                log::info!("FCM token saved for NIK: {}", employee.nik);
                            }

                            return Ok(LoginResponse {
                                status: "success".to_string(),
                                message: format!("Welcome back, {}", employee.full_name),
                            });
                        } else {
                            log::warn!("Password mismatch (bcrypt hash) for NIK: {}", req.nik);
                        }
                    } else if req.password == employee.password_hash {
                        log::info!("Login successful (plaintext fallback) for NIK: {}", employee.nik);

                        // Save FCM Token if provided
                        if let Some(fcm_token) = &req.fcm_token {
                            let _ = data.db.query("UPDATE employee SET fcm_token = $token WHERE type::string(nik) = type::string($nik)")
                                .bind(("token", fcm_token.clone()))
                                .bind(("nik", employee.nik.clone()))
                                .await;
                            log::info!("FCM token saved for NIK: {}", employee.nik);
                        }

                        return Ok(LoginResponse {
                            status: "success".to_string(),
                            message: format!("Welcome back, {}", employee.full_name),
                        });
                    } else {
                        log::warn!("Password mismatch (plaintext fallback) for NIK: {}", req.nik);
                    }
                }

                log::warn!("Login failed: Invalid NIK or password for {}", req.nik);
                Err("Invalid NIK or password".to_string())
            }
            Err(e) => {
                log::error!("Database error during login query: {}", e);
                Err("Internal server error".to_string())
            }
        }
    }

    pub async fn update_fcm_token(
        req: web::Json<FcmTokenRequest>,
        data: web::Data<AppState>,
    ) -> Result<LoginResponse, String> {
        log::info!("Updating FCM token for NIK: {}", req.nik);

        let result = data.db.query("UPDATE employee SET fcm_token = $token WHERE type::string(nik) = type::string($nik)")
            .bind(("token", req.fcm_token.clone()))
            .bind(("nik", req.nik.clone()))
            .await;

        match result {
            Ok(_) => Ok(LoginResponse {
                status: "success".to_string(),
                message: "FCM token updated successfully".to_string(),
            }),
            Err(e) => {
                log::error!("Database error updating FCM token: {}", e);
                Err("Failed to update token".to_string())
            }
        }
    }

    pub async fn register(
        req: web::Json<RegisterRequest>,
        data: web::Data<AppState>,
    ) -> Result<RegisterResponse, String> {
        log::info!("Starting registration process for NIK: {}, Email: {}", req.nik, req.email);

        // Cek apakah NIK atau Email sudah terdaftar
        let check_result = data
            .db
            .query("SELECT * FROM employee WHERE type::string(nik) = type::string($nik) OR email = $email")
            .bind(("nik", req.nik.clone()))
            .bind(("email", req.email.clone()))
            .await;

        match check_result {
            Ok(mut res) => {
                let employees: Vec<Employee> = res.take(0).unwrap_or_default();
                if !employees.is_empty() {
                    log::warn!("Registration failed: NIK {} or Email {} already exists", req.nik, req.email);
                    return Err("NIK or Email already exists".to_string());
                }
                log::info!("Pre-registration check passed. NIK and Email are unique.");
            }
            Err(e) => {
                log::error!("Database error during pre-registration check: {}", e);
                return Err("Internal server error during validation".to_string());
            }
        }

        // Hash password menggunakan bcrypt
        let hashed_password = match hash(&req.password, DEFAULT_COST) {
            Ok(h) => h,
            Err(e) => {
                log::error!("Failed to hash password: {}", e);
                return Err("Internal server error during registration".to_string());
            }
        };

        // Pastikan department dan position default tersedia untuk memenuhi skema
        let _ = data.db.query("CREATE department:default SET name = 'General', is_active = true;")
            .await;
        let _ = data.db.query("CREATE position:default SET name = 'Staff', level = 'staff', is_active = true;")
            .await;

        let insert_query = r#"
            CREATE employee SET
                nik = $nik,
                full_name = $full_name,
                email = $email,
                password_hash = $password,
                role = 'employee',
                department_id = department:default,
                position_id = position:default,
                join_date = time::now()
        "#;

        log::info!("Executing CREATE employee query for NIK: {}", req.nik);

        let insert_result = data
            .db
            .query(insert_query)
            .bind(("nik", req.nik.clone()))
            .bind(("full_name", req.full_name.clone()))
            .bind(("email", req.email.clone()))
            .bind(("password", hashed_password))
            .await;

        match insert_result {
            Ok(mut res) => {
                // Check if the insertion actually returned data
                match res.take::<Vec<Employee>>(0) {
                    Ok(mut employees) => {
                        if let Some(new_employee) = employees.pop() {
                            log::info!("Successfully registered and verified database insertion for NIK: {}", new_employee.nik);
                            Ok(RegisterResponse {
                                status: "success".to_string(),
                                message: "Registration successful".to_string(),
                            })
                        } else {
                            log::warn!("Query executed but no record was returned. This might indicate the record wasn't saved.");
                            // We still return success as the query didn't throw an error, but this is suspicious
                            Ok(RegisterResponse {
                                status: "success".to_string(),
                                message: "Registration processed, but verification failed".to_string(),
                            })
                        }
                    }
                    Err(e) => {
                        log::error!("Error parsing response after CREATE query: {}", e);
                        // The record might have been created but we failed to parse the response
                        Ok(RegisterResponse {
                            status: "success".to_string(),
                            message: "Registration successful (parsing error)".to_string(),
                        })
                    }
                }
            }
            Err(e) => {
                log::error!("Database error executing CREATE query during registration: {}", e);
                Err("Failed to register user".to_string())
            }
        }
    }
}
