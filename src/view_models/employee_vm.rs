use crate::config::app_state::AppState;
use crate::models::employee::EmployeeResponse;
use actix_web::web;

pub struct EmployeeViewModel;

impl EmployeeViewModel {
    pub async fn get_all_employees(
        data: &web::Data<AppState>,
    ) -> Result<Vec<EmployeeResponse>, String> {
        let result = data
            .db
            .query("SELECT * FROM employees ORDER BY created_at DESC")
            .await;

        match result {
            Ok(mut res) => {
                let employees: Vec<EmployeeResponse> = res.take(0).unwrap_or_default();
                log::info!("Fetched {} employees from database", employees.len());
                Ok(employees)
            }
            Err(e) => {
                log::error!("Error fetching employees: {}", e);
                Err("Failed to fetch employees".to_string())
            }
        }
    }
}
