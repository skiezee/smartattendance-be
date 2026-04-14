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
            .query("SELECT id, nik, full_name, email, role, department, status, join_date FROM employee ORDER BY join_date DESC")
            .await;

        match result {
            Ok(mut res) => {
                let employees: Vec<EmployeeResponse> = res.take(0).unwrap_or_default();
                Ok(employees)
            }
            Err(e) => {
                log::error!("Error fetching employees: {}", e);
                Err("Failed to fetch employees".to_string())
            }
        }
    }
}
