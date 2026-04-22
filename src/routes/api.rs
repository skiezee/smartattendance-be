use actix_web::web;
use crate::handlers::{attendance_handler, auth_handler, dashboard_handler, employee_handler, health_handler, leave_handler, patrol_handler, shift_handler};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_handler::health_check))
            .route("/login", web::post().to(auth_handler::login))
            .route("/register", web::post().to(auth_handler::register))
            .route("/attendance/enrollment-status", web::post().to(attendance_handler::check_enrollment))
            .route("/attendance/enroll-face", web::post().to(attendance_handler::enroll_face))
            .route("/attendance/enroll-fingerprint", web::post().to(attendance_handler::enroll_fingerprint))
            .route("/attendance/clock-in", web::post().to(attendance_handler::clock_in))
            .route("/attendance/logs", web::get().to(attendance_handler::get_all_attendances))
            .route("/employees", web::get().to(employee_handler::get_all_employees))
            .route("/leave", web::post().to(leave_handler::submit_leave))
            .route("/leave", web::get().to(leave_handler::get_leaves))
            .route("/leave/status", web::put().to(leave_handler::update_leave_status))
            .route("/patrol/incident", web::post().to(patrol_handler::submit_incident))
            .route("/patrol/incidents", web::get().to(patrol_handler::get_all_incidents))
            .route("/patrol/incidents/{nik}", web::get().to(patrol_handler::get_incidents_by_nik))
            .route("/shift", web::post().to(shift_handler::create_shift))
            .route("/shift/list", web::post().to(shift_handler::get_shifts))
            .route("/shift/all", web::get().to(shift_handler::get_all_shifts))
            .route("/shift/status", web::put().to(shift_handler::update_shift_status))
            .route("/shift/stats/{nik}", web::get().to(shift_handler::get_shift_stats))
            .route("/shift/{shift_id}", web::delete().to(shift_handler::delete_shift))
            // Dashboard Analytics Routes
            .route("/dashboard/analytics", web::get().to(dashboard_handler::get_dashboard_analytics))
            .route("/dashboard/overview", web::get().to(dashboard_handler::get_overview_only))
            .route("/dashboard/attendance", web::get().to(dashboard_handler::get_attendance_analytics))
            .route("/dashboard/patrol", web::get().to(dashboard_handler::get_patrol_analytics))
            .route("/dashboard/incidents", web::get().to(dashboard_handler::get_incident_analytics))
            .route("/dashboard/performance", web::get().to(dashboard_handler::get_performance_analytics))
            .route("/dashboard/locations", web::get().to(dashboard_handler::get_location_analytics)),
    );
}
