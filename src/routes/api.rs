use actix_web::web;
use crate::handlers::{attendance_handler, auth_handler, employee_handler, health_handler, leave_handler};

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
            .route("/leave/status", web::put().to(leave_handler::update_leave_status)),
    );
}
