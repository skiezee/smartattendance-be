use crate::config::app_state::AppState;
use crate::models::employee::EmployeeResponse;
use actix_web::web;
use serde_json::Value;

pub struct EmployeeViewModel;

impl EmployeeViewModel {
    pub async fn get_all_employees(
        data: &web::Data<AppState>,
    ) -> Result<Vec<EmployeeResponse>, String> {
        let result = data
            .db
            .query("SELECT * FROM employee ORDER BY created_at DESC")
            .await;

        match result {
            Ok(mut res) => {
                let mut employees: Vec<EmployeeResponse> = res.take(0).unwrap_or_default();
                
                // Enrich employees with department name from department_id
                for employee in &mut employees {
                    if let Some(ref dept_id) = employee.department_id {
                        let dept_query = format!("SELECT name FROM {}", dept_id);
                        if let Ok(mut dept_res) = data.db.query(&dept_query).await {
                            if let Ok(depts) = dept_res.take::<Vec<Value>>(0) {
                                if let Some(dept) = depts.first() {
                                    if let Some(name) = dept.get("name").and_then(|v| v.as_str()) {
                                        employee.department = Some(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                    
                    // Map employment_status to status for backward compatibility
                    if let Some(ref emp_status) = employee.employment_status {
                        employee.status = Some(emp_status.clone());
                    }
                }
                
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
