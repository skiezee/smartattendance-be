use actix_web::web;
use chrono::Local;

use crate::config::app_state::AppState;
use crate::models::employee::EmployeeResponse;
use crate::models::shift::{
    CreateShiftRequest, ShiftResponse, ShiftSchedule, ShiftStats, UpdateShiftStatusRequest,
};

pub struct ShiftViewModel;

impl ShiftViewModel {
    pub async fn create_shift(
        payload: web::Json<CreateShiftRequest>,
        data: &web::Data<AppState>,
    ) -> Result<ShiftResponse, String> {
        // Verify employee exists
        let mut result = data
            .db
            .query("SELECT * FROM employee WHERE type::string(nik) = type::string($nik)")
            .bind(("nik", payload.nik.clone()))
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let employees: Vec<EmployeeResponse> = result
            .take(0)
            .map_err(|e| format!("Failed to parse employee: {}", e))?;

        if employees.is_empty() {
            return Err("Employee not found".to_string());
        }

        let employee = &employees[0];
        let employee_id = employee.id.clone().ok_or("Employee ID not found")?;

        // Create shift schedule
        let shift = ShiftSchedule {
            id: None,
            employee_id: employee_id.clone(),
            nik: payload.nik.clone(),
            employee_name: employee.full_name.clone(),
            shift_type: payload.shift_type.clone(),
            date: payload.date.clone(),
            start_time: payload.start_time.clone(),
            end_time: payload.end_time.clone(),
            location: payload.location.clone(),
            tasks: payload.tasks.clone(),
            status: "SCHEDULED".to_string(),
            notes: payload.notes.clone(),
            created_at: Local::now().to_rfc3339(),
        };

        let created: Option<ShiftSchedule> = data
            .db
            .create("shift_schedules")
            .content(shift)
            .await
            .map_err(|e| format!("Failed to create shift: {}", e))?;

        let shift_id = created
            .and_then(|s| s.id)
            .map(|id| id.to_string())
            .unwrap_or_default();

        log::info!("Shift created: {} for NIK: {}", shift_id, payload.nik);

        Ok(ShiftResponse {
            status: "success".to_string(),
            message: "Shift schedule created successfully".to_string(),
            shift_id: Some(shift_id),
        })
    }

    pub async fn get_shifts_by_nik(
        nik: &str,
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<Vec<ShiftSchedule>, String> {
        let nik_string = nik.to_string();

        let query = if start_date.is_some() && end_date.is_some() {
            "SELECT * FROM shift_schedules WHERE nik = $nik AND date >= $start_date AND date <= $end_date ORDER BY date ASC, start_time ASC"
        } else {
            "SELECT * FROM shift_schedules WHERE nik = $nik ORDER BY date ASC, start_time ASC"
        };

        let mut query_builder = data.db.query(query).bind(("nik", nik_string));

        if let Some(sd) = start_date {
            query_builder = query_builder.bind(("start_date", sd));
        }
        if let Some(ed) = end_date {
            query_builder = query_builder.bind(("end_date", ed));
        }

        let mut result = query_builder
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let shifts: Vec<ShiftSchedule> = result
            .take(0)
            .map_err(|e| format!("Failed to parse shifts: {}", e))?;

        Ok(shifts)
    }

    pub async fn get_all_shifts(
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<Vec<ShiftSchedule>, String> {
        let query = if start_date.is_some() && end_date.is_some() {
            "SELECT * FROM shift_schedules WHERE date >= $start_date AND date <= $end_date ORDER BY date ASC, start_time ASC"
        } else {
            "SELECT * FROM shift_schedules ORDER BY date ASC, start_time ASC"
        };

        let mut query_builder = data.db.query(query);

        if let Some(sd) = start_date {
            query_builder = query_builder.bind(("start_date", sd));
        }
        if let Some(ed) = end_date {
            query_builder = query_builder.bind(("end_date", ed));
        }

        let mut result = query_builder
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let shifts: Vec<ShiftSchedule> = result
            .take(0)
            .map_err(|e| format!("Failed to parse shifts: {}", e))?;

        Ok(shifts)
    }

    pub async fn update_shift_status(
        payload: web::Json<UpdateShiftStatusRequest>,
        data: &web::Data<AppState>,
    ) -> Result<ShiftResponse, String> {
        let query = format!("UPDATE {} SET status = $status", payload.shift_id);

        let _result = data
            .db
            .query(query)
            .bind(("status", payload.status.clone()))
            .await
            .map_err(|e| format!("Failed to update shift status: {}", e))?;

        log::info!(
            "Shift status updated: {} to {}",
            payload.shift_id,
            payload.status
        );

        Ok(ShiftResponse {
            status: "success".to_string(),
            message: "Shift status updated successfully".to_string(),
            shift_id: Some(payload.shift_id.clone()),
        })
    }

    pub async fn get_shift_stats(
        nik: &str,
        data: &web::Data<AppState>,
    ) -> Result<ShiftStats, String> {
        let nik_string = nik.to_string();

        let mut result = data
            .db
            .query("SELECT * FROM shift_schedules WHERE nik = $nik")
            .bind(("nik", nik_string))
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let shifts: Vec<ShiftSchedule> = result
            .take(0)
            .map_err(|e| format!("Failed to parse shifts: {}", e))?;

        let total_shifts = shifts.len();
        let completed_shifts = shifts.iter().filter(|s| s.status == "COMPLETED").count();
        let upcoming_shifts = shifts.iter().filter(|s| s.status == "SCHEDULED").count();
        let cancelled_shifts = shifts.iter().filter(|s| s.status == "CANCELLED").count();

        Ok(ShiftStats {
            total_shifts,
            completed_shifts,
            upcoming_shifts,
            cancelled_shifts,
        })
    }

    pub async fn delete_shift(shift_id: &str, data: &web::Data<AppState>) -> Result<ShiftResponse, String> {
        let query = format!("DELETE {}", shift_id);

        let _result = data
            .db
            .query(query)
            .await
            .map_err(|e| format!("Failed to delete shift: {}", e))?;

        log::info!("Shift deleted: {}", shift_id);

        Ok(ShiftResponse {
            status: "success".to_string(),
            message: "Shift deleted successfully".to_string(),
            shift_id: Some(shift_id.to_string()),
        })
    }
}
