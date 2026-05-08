use actix_web::web;
use crate::handlers::{admin_handler, attendance_handler, auth_handler, dashboard_handler, employee_handler, group_handler, health_handler, leave_handler, location_handler, maintenance_handler, patrol_handler, profile_handler, shift_handler, shift_management_handler, wifi_handler};
use crate::middleware::auth::JwtAuth;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Public routes (no authentication required)
            .route("/health", web::get().to(health_handler::health_check))
            .route("/login", web::post().to(auth_handler::login))
            .route("/register", web::post().to(auth_handler::register))
            .route("/refresh-token", web::post().to(auth_handler::refresh_token))
            .route("/admin/login", web::post().to(admin_handler::admin_login))
            
            // Protected attendance routes
            .service(
                web::scope("/attendance")
                    .wrap(JwtAuth)
                    .route("/enrollment-status", web::post().to(attendance_handler::check_enrollment))
                    .route("/enroll-face", web::post().to(attendance_handler::enroll_face))
                    .route("/enroll-fingerprint", web::post().to(attendance_handler::enroll_fingerprint))
                    .route("/clock-in", web::post().to(attendance_handler::clock_in))
                    .route("/logs", web::get().to(attendance_handler::get_all_attendances))
            )
            
            // Protected employee routes
            .service(
                web::scope("/employees")
                    .wrap(JwtAuth)
                    .route("", web::get().to(employee_handler::get_all_employees))
                    .route("", web::post().to(employee_handler::create_employee))
                    .route("/bulk", web::post().to(employee_handler::bulk_create_employees))
                    .route("/bulk-attendance", web::put().to(employee_handler::bulk_update_attendance))
                    .route("/{nik}", web::get().to(employee_handler::get_employee_by_nik))
                    .route("/{nik}", web::put().to(employee_handler::update_employee))
                    .route("/{nik}", web::delete().to(employee_handler::delete_employee))
            )
            
            // Protected maintenance routes
            .service(
                web::scope("/maintenance")
                    .wrap(JwtAuth)
                    .route("/fix-attendance-requirements", web::post().to(maintenance_handler::fix_attendance_requirements))
            )
            
            // Protected group management routes
            .service(
                web::scope("/groups")
                    .wrap(JwtAuth)
                    .route("", web::get().to(group_handler::get_all_groups))
                    .route("", web::post().to(group_handler::create_group))
                    .route("/{id}", web::get().to(group_handler::get_group))
                    .route("/{id}", web::put().to(group_handler::update_group))
                    .route("/{id}", web::delete().to(group_handler::delete_group))
            )
            
            // Protected profile management routes
            .service(
                web::scope("/profile")
                    .wrap(JwtAuth)
                    .route("/{nik}", web::get().to(profile_handler::get_profile))
                    .route("", web::put().to(profile_handler::update_profile))
                    .route("/change-password", web::post().to(profile_handler::change_password))
                    .route("/upload-photo", web::post().to(profile_handler::upload_profile_photo))
            )
            
            // Protected leave routes
            .service(
                web::scope("/leave")
                    .wrap(JwtAuth)
                    .route("", web::post().to(leave_handler::submit_leave))
                    .route("", web::get().to(leave_handler::get_leaves))
                    .route("/status", web::put().to(leave_handler::update_leave_status))
            )
            
            // Protected patrol routes
            .service(
                web::scope("/patrol")
                    .wrap(JwtAuth)
                    .route("/incident", web::post().to(patrol_handler::submit_incident))
                    .route("/incidents", web::post().to(patrol_handler::submit_incident))
                    .route("/incidents", web::get().to(patrol_handler::get_all_incidents))
                    .route("/incidents/{nik}", web::get().to(patrol_handler::get_incidents_by_nik))
                    .route("/incidents/{id}", web::put().to(patrol_handler::update_incident))
                    .route("/incidents/{id}", web::delete().to(patrol_handler::delete_incident))
                    // Checkpoint Reports
                    .route("/checkpoint-reports", web::post().to(patrol_handler::create_checkpoint_report))
                    .route("/checkpoint-reports", web::get().to(patrol_handler::get_checkpoint_reports))
                    .route("/checkpoint-reports/{id}", web::get().to(patrol_handler::get_checkpoint_report))
                    .route("/checkpoint-reports/{id}", web::put().to(patrol_handler::update_checkpoint_report))
                    .route("/checkpoint-reports/{id}", web::delete().to(patrol_handler::delete_checkpoint_report))
                    // Areas
                    .route("/areas", web::get().to(patrol_handler::get_areas))
                    .route("/areas", web::post().to(patrol_handler::create_area))
                    .route("/areas/{id}", web::put().to(patrol_handler::update_area))
                    .route("/areas/{id}", web::delete().to(patrol_handler::delete_area))
                    // Checkpoints
                    .route("/checkpoints", web::post().to(patrol_handler::create_checkpoint))
                    .route("/checkpoints", web::get().to(patrol_handler::get_checkpoints))
                    .route("/checkpoints/{id}", web::put().to(patrol_handler::update_checkpoint))
                    .route("/checkpoints/{id}", web::delete().to(patrol_handler::delete_checkpoint))
                    // Assignments
                    .route("/assignments", web::post().to(patrol_handler::create_patrol_assignment))
                    .route("/assignments", web::get().to(patrol_handler::get_patrol_assignments))
                    .route("/assignments/{id}", web::put().to(patrol_handler::update_patrol_assignment))
                    .route("/assignments/{id}", web::delete().to(patrol_handler::delete_patrol_assignment))
                    // Status & Tracking
                    .route("/status/active", web::get().to(patrol_handler::get_active_patrol_status))
                    .route("/tracking/live", web::get().to(patrol_handler::get_live_tracking))
                    .route("/scan", web::post().to(patrol_handler::scan_checkpoint))
            )
            
            // Protected shift routes
            .service(
                web::scope("/shift")
                    .wrap(JwtAuth)
                    .route("", web::post().to(shift_handler::create_shift))
                    .route("/list", web::post().to(shift_handler::get_shifts))
                    .route("/all", web::get().to(shift_handler::get_all_shifts))
                    .route("/status", web::put().to(shift_handler::update_shift_status))
                    .route("/stats/{nik}", web::get().to(shift_handler::get_shift_stats))
                    .route("/{shift_id}", web::delete().to(shift_handler::delete_shift))
            )
            
            // Protected shift management system routes
            .service(
                web::scope("/shift-tasks")
                    .wrap(JwtAuth)
                    .route("", web::get().to(shift_management_handler::get_all_shift_tasks))
                    .route("", web::post().to(shift_management_handler::create_shift_task))
                    .route("/{id}", web::get().to(shift_management_handler::get_shift_task))
                    .route("/{id}", web::put().to(shift_management_handler::update_shift_task))
                    .route("/{id}", web::delete().to(shift_management_handler::delete_shift_task))
                    .route("/{id}/groups", web::get().to(shift_management_handler::get_employee_groups))
                    .route("/{id}/groups", web::put().to(shift_management_handler::save_employee_groups))
                    .route("/{id}/available-employees", web::get().to(shift_management_handler::get_available_employees))
                    .route("/{id}/schedules/generate", web::post().to(shift_management_handler::generate_schedule))
                    .route("/{id}/schedules", web::get().to(shift_management_handler::get_schedules))
                    .route("/{id}/schedules/{date}", web::put().to(shift_management_handler::update_schedule))
                    .route("/{id}/schedules", web::delete().to(shift_management_handler::delete_schedules))
            )
            
            // Protected dashboard routes
            .service(
                web::scope("/dashboard")
                    .wrap(JwtAuth)
                    .route("/analytics", web::get().to(dashboard_handler::get_dashboard_analytics))
                    .route("/overview", web::get().to(dashboard_handler::get_overview_only))
                    .route("/attendance", web::get().to(dashboard_handler::get_attendance_analytics))
                    .route("/patrol", web::get().to(dashboard_handler::get_patrol_analytics))
                    .route("/incidents", web::get().to(dashboard_handler::get_incident_analytics))
                    .route("/performance", web::get().to(dashboard_handler::get_performance_analytics))
                    .route("/locations", web::get().to(dashboard_handler::get_location_analytics))
            )
            
            // Protected WiFi settings routes
            .service(
                web::scope("/wifi-settings")
                    .wrap(JwtAuth)
                    .route("", web::get().to(wifi_handler::get_wifi_settings))
                    .route("/all", web::get().to(wifi_handler::get_all_wifi_settings))
                    .route("/validate", web::post().to(wifi_handler::validate_wifi_ssid))
                    .route("", web::post().to(wifi_handler::create_wifi_setting))
                    .route("/{wifi_id}", web::patch().to(wifi_handler::update_wifi_setting))
                    .route("/{wifi_id}", web::put().to(wifi_handler::update_wifi_setting))
                    .route("/{wifi_id}", web::delete().to(wifi_handler::delete_wifi_setting))
            )
            
            // Protected location boundary routes
            .service(
                web::scope("/location-boundaries")
                    .wrap(JwtAuth)
                    .route("", web::get().to(location_handler::get_location_boundaries))
                    .route("/all", web::get().to(location_handler::get_all_location_boundaries))
                    .route("/validate", web::post().to(location_handler::validate_location))
                    .route("", web::post().to(location_handler::create_location_boundary))
                    .route("/{location_id}", web::patch().to(location_handler::update_location_boundary))
                    .route("/{location_id}", web::delete().to(location_handler::delete_location_boundary))
            ),
    );
}
