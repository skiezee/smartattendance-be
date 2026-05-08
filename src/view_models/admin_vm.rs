use crate::config::app_state::AppState;
use crate::models::admin::{Admin, AdminLoginRequest, AdminLoginResponse};
use crate::utils::jwt;
use actix_web::web;
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct AdminViewModel;

impl AdminViewModel {
    pub async fn login(
        req: web::Json<AdminLoginRequest>,
        data: web::Data<AppState>,
    ) -> Result<AdminLoginResponse, String> {
        let clean_username = req.username.trim().to_string();
        log::info!("Admin login attempt for username: {}", clean_username);

        // Query admin table
        let result = data
            .db
            .query("SELECT * FROM admin WHERE username = $username AND is_active = true")
            .bind(("username", clean_username.clone()))
            .await;

        match result {
            Ok(mut res) => {
                let admins: Vec<Admin> = res.take(0).unwrap_or_default();
                log::info!("Admin login query returned {} results", admins.len());

                if let Some(admin) = admins.first() {
                    // Verify password using bcrypt
                    if let Ok(valid) = verify(&req.password, &admin.password_hash) {
                        if valid {
                            log::info!("Admin login successful for username: {}", admin.username);

                            // Generate JWT token
                            let token = jwt::generate_token(
                                admin.username.clone(),
                                admin.full_name.clone(),
                                admin.role.clone(),
                            ).map_err(|e| {
                                log::error!("Failed to generate JWT token: {}", e);
                                "Failed to generate authentication token".to_string()
                            })?;

                            return Ok(AdminLoginResponse {
                                status: "success".to_string(),
                                message: format!("Welcome back, {}", admin.full_name),
                                token: Some(token),
                                username: Some(admin.username.clone()),
                                name: Some(admin.full_name.clone()),
                                role: Some(admin.role.clone()),
                            });
                        } else {
                            log::warn!("Password mismatch for admin: {}", req.username);
                        }
                    } else if req.password == admin.password_hash {
                        // Plaintext fallback (for initial setup)
                        log::info!("Admin login successful (plaintext fallback) for username: {}", admin.username);

                        // Generate JWT token
                        let token = jwt::generate_token(
                            admin.username.clone(),
                            admin.full_name.clone(),
                            admin.role.clone(),
                        ).map_err(|e| {
                            log::error!("Failed to generate JWT token: {}", e);
                            "Failed to generate authentication token".to_string()
                        })?;

                        return Ok(AdminLoginResponse {
                            status: "success".to_string(),
                            message: format!("Welcome back, {}", admin.full_name),
                            token: Some(token),
                            username: Some(admin.username.clone()),
                            name: Some(admin.full_name.clone()),
                            role: Some(admin.role.clone()),
                        });
                    } else {
                        log::warn!("Password mismatch for admin: {}", req.username);
                    }
                }

                log::warn!("Admin login failed: Invalid username or password for {}", req.username);
                Err("Invalid username or password".to_string())
            }
            Err(e) => {
                log::error!("Database error during admin login query: {}", e);
                Err("Internal server error".to_string())
            }
        }
    }

    pub async fn create_default_admin(data: web::Data<AppState>) -> Result<(), String> {
        log::info!("Checking for default admin...");

        // Check if any admin exists
        let check_result = data
            .db
            .query("SELECT * FROM admin LIMIT 1")
            .await;

        match check_result {
            Ok(mut res) => {
                let admins: Vec<Admin> = res.take(0).unwrap_or_default();
                
                if admins.is_empty() {
                    log::info!("No admin found, creating default admin...");
                    
                    // Hash default password
                    let hashed_password = hash("admin123", DEFAULT_COST)
                        .map_err(|e| format!("Failed to hash password: {}", e))?;

                    // Create default admin
                    let create_result = data
                        .db
                        .query(r#"
                            CREATE admin SET
                                username = 'admin',
                                full_name = 'System Administrator',
                                email = 'admin@smartattendance.com',
                                password_hash = $password,
                                role = 'admin',
                                is_active = true,
                                created_at = time::now()
                        "#)
                        .bind(("password", hashed_password))
                        .await;

                    match create_result {
                        Ok(_) => {
                            log::info!("Default admin created successfully!");
                            log::info!("Username: admin, Password: admin123");
                            Ok(())
                        }
                        Err(e) => {
                            log::error!("Failed to create default admin: {}", e);
                            Err(format!("Failed to create default admin: {}", e))
                        }
                    }
                } else {
                    log::info!("Admin already exists, skipping creation");
                    Ok(())
                }
            }
            Err(e) => {
                log::error!("Failed to check for existing admin: {}", e);
                Err(format!("Failed to check for existing admin: {}", e))
            }
        }
    }
}
