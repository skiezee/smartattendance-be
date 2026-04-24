use serde::{Deserialize, Serialize};

// ─── Raw SurrealDB response structs ───────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttendanceRaw {
    pub employee_id: Option<serde_json::Value>,
    pub status: Option<String>,
    pub late_minutes: Option<i64>,
    pub work_date: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatrolSessionRaw {
    pub employee_id: Option<serde_json::Value>,
    pub location_id: Option<serde_json::Value>,
    pub status: Option<String>,
    pub started_at: Option<serde_json::Value>,
    pub ended_at: Option<serde_json::Value>,
    pub duration_minutes: Option<f64>,
    pub checkpoints_total: Option<i64>,
    pub checkpoints_visited: Option<i64>,
    pub checkpoints_missed: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncidentRaw {
    pub id: Option<serde_json::Value>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub nik: Option<String>,
    pub employee_id: Option<serde_json::Value>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmployeeRaw {
    pub id: Option<serde_json::Value>,
    pub full_name: Option<String>,
    pub nik: Option<String>,
    pub department: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkLocationRaw {
    pub id: Option<serde_json::Value>,
    pub name: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub radius_meters: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatrolPerSiteRaw {
    pub location_id: Option<serde_json::Value>,
    pub location_name: Option<String>,
    pub total_patrols: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatrolCompletedPerSiteRaw {
    pub location_id: Option<serde_json::Value>,
    pub completed_patrols: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CountResult {
    pub total: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SumResult {
    pub total: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeanResult {
    pub avg: Option<f64>,
}

// ─── Overview ─────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct DashboardOverview {
    pub total_personnel_on_duty: i64,
    pub total_active_employees: i64,
    pub attendance_rate: f64,
    pub patrol_completion_rate: f64,
    pub total_incidents: i64,
}

// ─── Attendance Analytics ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct AttendanceAnalytics {
    pub on_time_vs_late: OnTimeLateComparison,
    pub absentee: AbsenteeStats,
    pub daily_attendance_trend: Vec<DailyAttendanceTrend>,
    pub top_late_users: Vec<LateUser>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OnTimeLateComparison {
    pub on_time: i64,
    pub late: i64,
    pub total: i64,
    pub on_time_percentage: f64,
    pub late_percentage: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AbsenteeStats {
    pub absent_count: i64,
    pub total_logs: i64,
    pub absentee_rate: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DailyAttendanceTrend {
    pub date: String,
    pub present: i64,
    pub absent: i64,
    pub late: i64,
    pub total: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LateUser {
    pub employee_id: String,
    pub nik: String,
    pub full_name: String,
    pub department: String,
    pub late_count: i64,
    pub total_late_minutes: i64,
    pub avg_late_minutes: f64,
}

// ─── Patrol Analytics ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct PatrolAnalytics {
    pub completion: PatrolCompletion,
    pub checkpoints: CheckpointStats,
    pub average_patrol_duration_minutes: f64,
    pub daily_patrol_trend: Vec<DailyPatrolTrend>,
    pub patrol_records: Vec<PatrolRecord>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatrolCompletion {
    pub completed: i64,
    pub incomplete: i64,
    pub in_progress: i64,
    pub total: i64,
    pub completion_rate: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckpointStats {
    pub total_missed: i64,
    pub total_checkpoints: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DailyPatrolTrend {
    pub date: String,
    pub total_patrols: i64,
    pub completed: i64,
    pub incomplete: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatrolRecord {
    pub started_at: String,
    pub employee_id: String,
    pub employee_nik: String,
    pub employee_name: String,
    pub employee_department: String,
    pub location_id: String,
    pub location_name: String,
    pub location_address: String,
    pub status: String,
    pub duration_minutes: Option<f64>,
    pub checkpoints_total: i64,
    pub checkpoints_visited: i64,
    pub checkpoints_missed: i64,
}

// ─── Incident Analytics ───────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct IncidentAnalytics {
    pub by_severity: Vec<IncidentBySeverity>,
    pub by_status: Vec<IncidentByStatus>,
    pub incident_trend: Vec<IncidentTrend>,
    pub high_severity_incidents: Vec<HighSeverityIncident>,
    pub top_incident_locations: Vec<IncidentLocation>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncidentBySeverity {
    pub severity: String,
    pub total: i64,
    pub percentage: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncidentByStatus {
    pub status: String,
    pub total: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncidentTrend {
    pub date: String,
    pub total: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HighSeverityIncident {
    pub title: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub nik: String,
    pub latitude: f64,
    pub longitude: f64,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncidentLocation {
    pub location_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub incident_count: i64,
}

// ─── Performance Analytics ────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct PerformanceAnalytics {
    pub top_performers: Vec<PerformanceUser>,
    pub low_performers: Vec<PerformanceUser>,
    pub average_compliance_score: f64,
    pub average_activity_score: f64,
    pub average_final_score: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerformanceUser {
    pub employee_id: String,
    pub nik: String,
    pub full_name: String,
    pub department: String,
    pub present_days: i64,
    pub absent_days: i64,
    pub on_time_days: i64,
    pub late_days: i64,
    pub total_late_minutes: i64,
    pub patrol_completed: i64,
    pub compliance_score: f64,
    pub activity_score: f64,
    pub final_score: f64,
}

// ─── Location Analytics ───────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct LocationAnalytics {
    pub site_comparison: Vec<SiteComparison>,
    pub incidents_per_site: Vec<SiteIncidents>,
    pub patrols_per_site: Vec<SitePatrols>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SiteComparison {
    pub location_id: String,
    pub site_name: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub total_patrols: i64,
    pub completed_patrols: i64,
    pub patrol_completion_rate: f64,
    pub incident_count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SiteIncidents {
    pub location_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub incident_count: i64,
    pub high_severity_count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SitePatrols {
    pub location_id: String,
    pub site_name: String,
    pub total_patrols: i64,
    pub completed_patrols: i64,
    pub completion_rate: f64,
}

// ─── Full Dashboard Response ──────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct DashboardData {
    pub overview: DashboardOverview,
    pub attendance_analytics: AttendanceAnalytics,
    pub patrol_analytics: PatrolAnalytics,
    pub incident_analytics: IncidentAnalytics,
    pub performance_analytics: PerformanceAnalytics,
    pub location_analytics: LocationAnalytics,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DashboardResponse {
    pub status: String,
    pub data: DashboardData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DateRangeRequest {
    pub start_date: Option<String>, // YYYY-MM-DD
    pub end_date: Option<String>,   // YYYY-MM-DD
}