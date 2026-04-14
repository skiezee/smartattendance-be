use actix_web::web;
use chrono::Local;

use crate::config::app_state::AppState;
use crate::models::employee::Employee;
use crate::models::leave::{LeaveRecord, LeaveRequestPayload};
use crate::services::fcm_service::send_fcm_notification;

pub struct LeaveViewModel;

impl LeaveViewModel {
    pub async fn submit_leave(
        payload: web::Json<LeaveRequestPayload>,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let record = LeaveRecord {
            id: None,
            nik: payload.nik.clone(),
            leave_type: payload.leave_type.clone(),
            start_date: payload.start_date.clone(),
            end_date: payload.end_date.clone(),
            duration: payload.duration,
            reason: payload.reason.clone(),
            status: "PENDING".to_string(),
            stage1_status: "WAITING".to_string(),
            stage2_status: "WAITING".to_string(),
            created_at: Local::now().to_rfc3339(),
        };

        let _created: Option<LeaveRecord> = data
            .db
            .create("leaves")
            .content(record)
            .await
            .map_err(|e| e.to_string())?;

        Ok("Leave request submitted successfully".to_string())
    }

    pub async fn get_leaves(
        nik: String,
        data: &web::Data<AppState>,
    ) -> Result<Vec<LeaveRecord>, String> {
        let mut result = data
            .db
            .query("SELECT * FROM leaves WHERE nik = $nik ORDER BY created_at DESC")
            .bind(("nik", nik))
            .await
            .map_err(|e| e.to_string())?;

        let leaves: Vec<LeaveRecord> = result.take(0).map_err(|e| e.to_string())?;

        Ok(leaves)
    }

    pub async fn update_status(
        id: String,
        stage: i32,
        status: String,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let field_to_update = if stage == 1 { "stage1_status" } else { "stage2_status" };

        let query = format!(
            "UPDATE {} SET {} = $status",
            id, field_to_update
        );

        let mut result = data
            .db
            .query(query)
            .bind(("status", status.clone()))
            .await
            .map_err(|e| e.to_string())?;

        // Also update main status if stage 2 is completed or rejected anywhere
        let mut final_status = None;
        if status == "REJECTED" {
            let _ = data.db.query(format!("UPDATE {} SET status = 'REJECTED'", id)).await;
            final_status = Some("REJECTED");
        } else if stage == 2 && status == "APPROVED" {
            let _ = data.db.query(format!("UPDATE {} SET status = 'APPROVED'", id)).await;
            final_status = Some("APPROVED");
        }

        // Send FCM notification if final status changed
        if let Some(f_status) = final_status {
            // First get the NIK from the leave record
            if let Ok(mut leave_res) = data.db.query(format!("SELECT nik, leave_type FROM {}", id)).await {
                if let Some(leave_data) = leave_res.take::<Option<serde_json::Value>>(0).unwrap_or_default() {
                    let nik_opt = leave_data.get("nik").and_then(|n| n.as_str()).map(|s| s.to_string());
                    let leave_type_opt = leave_data.get("leave_type").and_then(|l| l.as_str()).map(|s| s.to_string());

                    if let (Some(nik), Some(leave_type)) = (nik_opt, leave_type_opt) {

                        // Then get the user's FCM token
                        if let Ok(mut emp_res) = data.db.query("SELECT * FROM employee WHERE type::string(nik) = type::string($nik)")
                            .bind(("nik", nik))
                            .await {
                            let employees: Vec<Employee> = emp_res.take(0).unwrap_or_default();
                            if let Some(emp) = employees.first() {
                                if let Some(token) = &emp.fcm_token {
                                    let title = format!("Cuti {} {}", leave_type, if f_status == "APPROVED" { "Disetujui" } else { "Ditolak" });
                                    let body = format!("Halo {}, pengajuan cuti {} Anda telah {}", emp.full_name, leave_type, if f_status == "APPROVED" { "disetujui oleh HRD" } else { "ditolak" });

                                    // Spawn notification task so it doesn't block the HTTP response
                                    let token_clone = token.clone();
                                    actix_web::rt::spawn(async move {
                                        // Replace with your actual Firebase Project ID
                                        let project_id = "smart-attendance-eef71";
                                        let _ = send_fcm_notification(project_id, &token_clone, &title, &body).await;
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok("Leave status updated".to_string())
    }
}
