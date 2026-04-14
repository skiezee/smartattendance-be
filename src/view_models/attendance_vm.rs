use crate::config::app_state::AppState;
use crate::models::attendance::{
    CheckEnrollmentResponse, ClockInRequest, ClockInResponse, EmployeeStatus, EnrollmentResponse,
    FaceEnrollRequest, FingerprintEnrollRequest, AttendanceLogResponse,
};
use actix_web::web;

pub struct AttendanceViewModel;

impl AttendanceViewModel {
    pub async fn check_enrollment(
        nik: String,
        data: &web::Data<AppState>,
    ) -> Result<CheckEnrollmentResponse, String> {
        let mut result = data
            .db
            .query("SELECT id, nik, face_enrolled, fingerprint_enrolled FROM employee WHERE nik = $nik")
            .bind(("nik", nik.clone()))
            .await
            .map_err(|e| {
                log::error!("DB error checking enrollment: {}", e);
                "Internal server error".to_string()
            })?;

        let employees: Vec<EmployeeStatus> = result.take(0).unwrap_or_default();
        if let Some(employee) = employees.first() {
            let check_today_query = r#"
                SELECT clock_in_at, type::string(clock_in_at) as clock_in_str, type::string(time::now()) as current_time_str
                FROM attendance_log
                WHERE employee_id = $employee_id
                ORDER BY clock_in_at DESC LIMIT 1
            "#;

            let mut today_res = data
                .db
                .query(check_today_query)
                .bind(("employee_id", employee.id.clone()))
                .await
                .map_err(|e| {
                    log::error!("Error checking today's attendance: {}", e);
                    e.to_string()
                })?;

            let logs: Vec<serde_json::Value> = today_res.take(0).unwrap_or_default();
            let mut has_attended_today = false;

            if let Some(log) = logs.first() {
                let clock_in = log.get("clock_in_str").and_then(|v| v.as_str());
                let current_time = log.get("current_time_str").and_then(|v| v.as_str());

                if let (Some(c), Some(t)) = (clock_in, current_time) {
                    if c.len() >= 10 && t.len() >= 10 && &c[0..10] == &t[0..10] {
                        has_attended_today = true;
                    }
                }
            }

            Ok(CheckEnrollmentResponse {
                status: "success".to_string(),
                face_enrolled: employee.face_enrolled,
                fingerprint_enrolled: employee.fingerprint_enrolled,
                has_attended_today,
            })
        } else {
            Err("Employee not found".to_string())
        }
    }

    pub async fn enroll_face(
        req: web::Json<FaceEnrollRequest>,
        data: &web::Data<AppState>,
    ) -> Result<EnrollmentResponse, String> {
        let mut result = data
            .db
            .query("SELECT id, nik, face_enrolled, fingerprint_enrolled FROM employee WHERE nik = $nik")
            .bind(("nik", req.nik.clone()))
            .await
            .map_err(|e| e.to_string())?;

        let employees: Vec<EmployeeStatus> = result.take(0).unwrap_or_default();
        let employee_id = match employees.first() {
            Some(emp) => emp.id.clone(),
            None => return Err("Employee not found".to_string()),
        };

        let update_query = r#"
            UPDATE $id SET
                face_embedding = $embedding,
                face_enrolled = true
        "#;

        let res = data
            .db
            .query(update_query)
            .bind(("id", employee_id))
            .bind(("embedding", req.face_embedding.clone()))
            .await;

        match res {
            Ok(_) => Ok(EnrollmentResponse {
                status: "success".to_string(),
                message: "Face enrolled successfully".to_string(),
            }),
            Err(e) => {
                log::error!("Failed to enroll face: {}", e);
                Err("Database error".to_string())
            }
        }
    }

    pub async fn enroll_fingerprint(
        req: web::Json<FingerprintEnrollRequest>,
        data: &web::Data<AppState>,
    ) -> Result<EnrollmentResponse, String> {
        let mut result = data
            .db
            .query("SELECT id, nik, face_enrolled, fingerprint_enrolled FROM employee WHERE nik = $nik")
            .bind(("nik", req.nik.clone()))
            .await
            .map_err(|e| e.to_string())?;

        let employees: Vec<EmployeeStatus> = result.take(0).unwrap_or_default();
        let employee_id = match employees.first() {
            Some(emp) => emp.id.clone(),
            None => return Err("Employee not found".to_string()),
        };

        let res = data
            .db
            .query("UPDATE $id SET fingerprint_enrolled = true")
            .bind(("id", employee_id))
            .await;

        match res {
            Ok(_) => Ok(EnrollmentResponse {
                status: "success".to_string(),
                message: "Fingerprint enrolled successfully".to_string(),
            }),
            Err(e) => {
                log::error!("Failed to enroll fingerprint: {}", e);
                Err("Database error".to_string())
            }
        }
    }

    pub async fn clock_in(
        req: web::Json<ClockInRequest>,
        data: &web::Data<AppState>,
    ) -> Result<ClockInResponse, String> {
        let mut result = data
            .db
            .query("SELECT id, nik, face_enrolled, fingerprint_enrolled FROM employee WHERE nik = $nik")
            .bind(("nik", req.nik.clone()))
            .await
            .map_err(|e| e.to_string())?;

        let employees: Vec<EmployeeStatus> = result.take(0).unwrap_or_default();
        let employee_id = match employees.first() {
            Some(emp) => emp.id.clone(),
            None => return Err("Employee not found".to_string()),
        };

        // Check if user already clocked in today
        let check_today_query = r#"
            SELECT clock_in_at, type::string(clock_in_at) as clock_in_str, type::string(time::now()) as current_time_str
            FROM attendance_log
            WHERE employee_id = $employee_id
            ORDER BY clock_in_at DESC LIMIT 1
        "#;

        let mut today_res = data
            .db
            .query(check_today_query)
            .bind(("employee_id", employee_id.clone()))
            .await
            .map_err(|e| {
                log::error!("Error checking today's attendance: {}", e);
                e.to_string()
            })?;

        let logs: Vec<serde_json::Value> = today_res.take(0).unwrap_or_default();
        if let Some(log) = logs.first() {
            let clock_in = log.get("clock_in_str").and_then(|v| v.as_str());
            let current_time = log.get("current_time_str").and_then(|v| v.as_str());

            if let (Some(c), Some(t)) = (clock_in, current_time) {
                if c.len() >= 10 && t.len() >= 10 && &c[0..10] == &t[0..10] {
                    return Err("Anda sudah melakukan presensi hari ini.".to_string());
                }
            }
        }

        // Setup default shift to fulfill schema requirements
        let _ = data.db.query("CREATE shift:default SET name = 'Default Shift', start_time = '08:00', end_time = '17:00';").await;

        let insert_query = r#"
            CREATE attendance_log SET
                employee_id = $employee_id,
                shift_id = shift:default,
                work_date = time::now(),
                clock_in_at = time::now(),
                clock_in_lat = $lat,
                clock_in_lng = $lng,
                clock_in_method = $method,
                face_confidence_in = $face_confidence,
                fingerprint_verified_in = $finger_verified,
                status = 'present'
        "#;

        let finger_verified = req.method == "fingerprint" || req.method == "face+fingerprint";
        let face_confidence = req.face_confidence.unwrap_or(0.0);

        let res = data
            .db
            .query(insert_query)
            .bind(("employee_id", employee_id))
            .bind(("lat", req.lat))
            .bind(("lng", req.lng))
            .bind(("method", req.method.clone()))
            .bind(("face_confidence", face_confidence))
            .bind(("finger_verified", finger_verified))
            .await;

        match res {
            Ok(_) => Ok(ClockInResponse {
                status: "success".to_string(),
                message: "Clock In successful".to_string(),
            }),
            Err(e) => {
                log::error!("Failed to clock in: {}", e);
                Err("Database error while clocking in".to_string())
            }
        }
    }

    pub async fn get_all_attendances(
        data: &web::Data<AppState>,
    ) -> Result<Vec<AttendanceLogResponse>, String> {
        let query = r#"
            SELECT
                id,
                employee_id,
                work_date,
                IF work_date IS NOT NONE THEN type::string(work_date) ELSE NONE END as date,
                IF clock_in_at IS NOT NONE THEN type::string(clock_in_at) ELSE NONE END as check_in,
                IF clock_out_at IS NOT NONE THEN type::string(clock_out_at) ELSE NONE END as check_out,
                status,
                'Office' as location
            FROM attendance_log
            ORDER BY work_date DESC
            LIMIT 100
        "#;

        let result = data.db.query(query).await;

        match result {
            Ok(mut res) => {
                let logs_result: Result<Vec<AttendanceLogResponse>, _> = res.take(0);
                if let Err(err) = &logs_result { log::error!("Deser error: {}", err); }
                let logs = logs_result.unwrap_or_default();
                Ok(logs)
            }
            Err(e) => {
                log::error!("Error fetching attendance logs: {}", e);
                Err("Failed to fetch attendance logs".to_string())
            }
        }
    }
}
