use actix_web::web;
use chrono::Local;
use std::collections::HashMap;

use crate::config::app_state::AppState;
use crate::models::dashboard::*;

pub struct DashboardViewModel;

impl DashboardViewModel {
    // Helper function to process date parameters
    fn process_date_params(start_date: Option<String>, end_date: Option<String>) -> (String, String) {
        let start_date = start_date.unwrap_or_else(|| {
            let date = Local::now() - chrono::Duration::days(1825);
            date.format("%Y-%m-%d").to_string()
        });

        let end_date = end_date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

        let from = format!("{}T00:00:00Z", start_date);
        let to = format!("{}T23:59:59Z", end_date);

        (from, to)
    }

    pub async fn get_dashboard_analytics(
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<DashboardData, String> {
        let (from, to) = Self::process_date_params(start_date, end_date);

        let overview = Self::get_overview(&from, &to, data).await?;
        let attendance_analytics = Self::get_attendance_analytics(&from, &to, data).await?;
        let patrol_analytics = Self::get_patrol_analytics(&from, &to, data).await?;
        let incident_analytics = Self::get_incident_analytics(&from, &to, data).await?;
        let performance_analytics = Self::get_performance_analytics(&from, &to, data).await?;
        let location_analytics = Self::get_location_analytics(&from, &to, data).await?;

        Ok(DashboardData {
            overview,
            attendance_analytics,
            patrol_analytics,
            incident_analytics,
            performance_analytics,
            location_analytics,
        })
    }

    // Individual analytics functions with dynamic date parameters
    pub async fn get_overview_only(
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<DashboardOverview, String> {
        let (from, to) = Self::process_date_params(start_date, end_date);
        Self::get_overview(&from, &to, data).await
    }

    pub async fn get_attendance_analytics_only(
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<AttendanceAnalytics, String> {
        let (from, to) = Self::process_date_params(start_date, end_date);
        Self::get_attendance_analytics(&from, &to, data).await
    }

    pub async fn get_patrol_analytics_only(
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<PatrolAnalytics, String> {
        let (from, to) = Self::process_date_params(start_date, end_date);
        Self::get_patrol_analytics(&from, &to, data).await
    }

    pub async fn get_incident_analytics_only(
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<IncidentAnalytics, String> {
        let (from, to) = Self::process_date_params(start_date, end_date);
        Self::get_incident_analytics(&from, &to, data).await
    }

    pub async fn get_performance_analytics_only(
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<PerformanceAnalytics, String> {
        let (from, to) = Self::process_date_params(start_date, end_date);
        Self::get_performance_analytics(&from, &to, data).await
    }

    pub async fn get_location_analytics_only(
        start_date: Option<String>,
        end_date: Option<String>,
        data: &web::Data<AppState>,
    ) -> Result<LocationAnalytics, String> {
        let (from, to) = Self::process_date_params(start_date, end_date);
        Self::get_location_analytics(&from, &to, data).await
    }

    // ─── Helper: id to string ────────────────────────────────────────────────
    fn id_to_string(val: &serde_json::Value) -> String {
        // SurrealDB returns id as {"tb":"employee","id":{"String":"abc"}} or plain string
        if let Some(s) = val.as_str() {
            return s.to_string();
        }
        val.to_string()
    }

    // ─── Overview ────────────────────────────────────────────────────────────
    async fn get_overview(
        from: &str,
        to: &str,
        data: &web::Data<AppState>,
    ) -> Result<DashboardOverview, String> {
        let query = format!(
            r#"
            LET $from = <datetime>"{from}";
            LET $to   = <datetime>"{to}";
            LET $today_start = time::floor(time::now(), 1d);
            LET $today_end   = $today_start + 1d;

            LET $total_on_duty = (
                SELECT count() AS total FROM attendance_log
                WHERE work_date >= $today_start AND work_date < $today_end AND status = 'present'
                GROUP ALL
            )[0].total ?? 0;

            LET $total_employees = (
                SELECT count() AS total FROM employee
                WHERE employment_status = 'active'
                GROUP ALL
            )[0].total ?? 0;

            LET $present_count = (
                SELECT count() AS total FROM attendance_log
                WHERE work_date >= $from AND work_date < $to AND status = 'present'
                GROUP ALL
            )[0].total ?? 0;

            LET $total_attendance = (
                SELECT count() AS total FROM attendance_log
                WHERE work_date >= $from AND work_date < $to
                GROUP ALL
            )[0].total ?? 0;

            LET $attendance_rate = IF $total_attendance > 0
                THEN math::round(($present_count * 100.0) / ($total_attendance * 1.0))
                ELSE 0.0
            END;

            LET $patrol_resolved = (
                SELECT count() AS total FROM patrol_incidents
                WHERE <datetime>created_at >= $from AND <datetime>created_at < $to AND status = 'resolved'
                GROUP ALL
            )[0].total ?? 0;

            LET $patrol_total = (
                SELECT count() AS total FROM patrol_incidents
                WHERE <datetime>created_at >= $from AND <datetime>created_at < $to AND status != NONE
                GROUP ALL
            )[0].total ?? 0;

            LET $patrol_completion_rate = IF $patrol_total > 0
                THEN math::round(($patrol_resolved * 100.0) / ($patrol_total * 1.0))
                ELSE 0.0
            END;

            LET $total_incidents = $patrol_total;

            RETURN {{
                total_on_duty:          $total_on_duty,
                total_active_employees: $total_employees,
                attendance_rate:        $attendance_rate,
                patrol_completion_rate: $patrol_completion_rate,
                total_incidents:        $total_incidents
            }};
            "#,
            from = from,
            to = to
        );

        let mut result = data
            .db
            .query(&query)
            .await
            .map_err(|e| format!("DB error overview: {}", e))?;

        let raw: Option<serde_json::Value> = result
            .take(13) // RETURN is the last statement
            .map_err(|e| format!("Parse error overview: {}", e))?;

        let v = raw.unwrap_or_default();

        Ok(DashboardOverview {
            total_personnel_on_duty: v["total_on_duty"].as_i64().unwrap_or(0),
            total_active_employees: v["total_active_employees"].as_i64().unwrap_or(0),
            attendance_rate: v["attendance_rate"].as_f64().unwrap_or(0.0),
            patrol_completion_rate: v["patrol_completion_rate"].as_f64().unwrap_or(0.0),
            total_incidents: v["total_incidents"].as_i64().unwrap_or(0),
        })
    }

    // ─── Attendance Analytics ─────────────────────────────────────────────────
    async fn get_attendance_analytics(
        from: &str,
        to: &str,
        data: &web::Data<AppState>,
    ) -> Result<AttendanceAnalytics, String> {
        let query = format!(
            r#"
            LET $from = <datetime>"{from}";
            LET $to   = <datetime>"{to}";

            LET $on_time = (
                SELECT count() AS total FROM attendance_log
                WHERE work_date >= $from AND work_date < $to AND status = 'present' AND late_minutes = 0
                GROUP ALL
            )[0].total ?? 0;

            LET $late = (
                SELECT count() AS total FROM attendance_log
                WHERE work_date >= $from AND work_date < $to AND status = 'present' AND late_minutes > 0
                GROUP ALL
            )[0].total ?? 0;

            LET $absent_count = (
                SELECT count() AS total FROM attendance_log
                WHERE work_date >= $from AND work_date < $to AND status = 'absent'
                GROUP ALL
            )[0].total ?? 0;

            LET $total_logs = (
                SELECT count() AS total FROM attendance_log
                WHERE work_date >= $from AND work_date < $to
                GROUP ALL
            )[0].total ?? 0;

            LET $daily_trend_raw = (
                SELECT 
                    type::string(work_date) AS work_date,
                    type::string(employee_id) AS employee_id,
                    status, 
                    late_minutes
                FROM attendance_log
                WHERE work_date >= $from AND work_date < $to
                ORDER BY work_date ASC
            );

            LET $late_raw = (
                SELECT 
                    type::string(employee_id) AS employee_id,

                    employee_id.nik AS nik,
                    employee_id.full_name AS full_name,
                    employee_id.department_id.name AS department,

                    late_minutes

                FROM attendance_log
                WHERE work_date >= $from AND work_date < $to
                    AND late_minutes > 0
                    AND employee_id != NONE

                FETCH employee_id, employee_id.department_id
            );

            RETURN {{
                on_time:         $on_time,
                late:            $late,
                absent_count:    $absent_count,
                total_logs:      $total_logs,
                daily_trend_raw: $daily_trend_raw,
                late_raw:        $late_raw
            }};
            "#,
            from = from,
            to = to
        );

        let mut result = data
            .db
            .query(&query)
            .await
            .map_err(|e| format!("DB error attendance: {}", e))?;

        let raw: Option<serde_json::Value> = result
            .take(8)  // ✅ confirmed dari debug
            .map_err(|e| format!("Parse error attendance: {}", e))?;

        let v = raw.unwrap_or_default();

        let on_time = v["on_time"].as_i64().unwrap_or(0);
        let late = v["late"].as_i64().unwrap_or(0);
        let absent_count = v["absent_count"].as_i64().unwrap_or(0);
        let total_logs = v["total_logs"].as_i64().unwrap_or(0);
        let total_present = on_time + late;

        let on_time_percentage = if total_present > 0 {
            (on_time as f64 / total_present as f64) * 100.0
        } else {
            0.0
        };
        let late_percentage = if total_present > 0 {
            (late as f64 / total_present as f64) * 100.0
        } else {
            0.0
        };
        let absentee_rate = if total_logs > 0 {
            (absent_count as f64 / total_logs as f64) * 100.0
        } else {
            0.0
        };

        let daily_attendance_trend =
            Self::build_daily_attendance_trend(v["daily_trend_raw"].as_array());

        let top_late_users = Self::build_top_late_users(v["late_raw"].as_array());

        Ok(AttendanceAnalytics {
            on_time_vs_late: OnTimeLateComparison {
                on_time,
                late,
                total: total_present,
                on_time_percentage,
                late_percentage,
            },
            absentee: AbsenteeStats {
                absent_count,
                total_logs,
                absentee_rate,
            },
            daily_attendance_trend,
            top_late_users,
        })
    }

    fn build_daily_attendance_trend(
        raw: Option<&Vec<serde_json::Value>>,
    ) -> Vec<DailyAttendanceTrend> {
        let records = match raw {
            Some(r) => r,
            None => return vec![],
        };

        // Group by date (YYYY-MM-DD)
        let mut map: HashMap<String, (i64, i64, i64)> = HashMap::new(); // (present, absent, late)

        for rec in records {
            let work_date = rec["work_date"]
                .as_str()
                .unwrap_or("")
                .get(..10)
                .unwrap_or("")
                .to_string();

            if work_date.is_empty() {
                continue;
            }

            let status = rec["status"].as_str().unwrap_or("");
            let late_minutes = rec["late_minutes"].as_i64().unwrap_or(0);

            let entry = map.entry(work_date).or_insert((0, 0, 0));
            if status == "present" {
                entry.0 += 1;
                if late_minutes > 0 {
                    entry.2 += 1;
                }
            } else if status == "absent" {
                entry.1 += 1;
            }
        }

        let mut trend: Vec<DailyAttendanceTrend> = map
            .into_iter()
            .map(|(date, (present, absent, late))| DailyAttendanceTrend {
                date,
                present,
                absent,
                late,
                total: present + absent,
            })
            .collect();

        trend.sort_by(|a, b| a.date.cmp(&b.date));
        trend
    }

    fn build_top_late_users(raw: Option<&Vec<serde_json::Value>>) -> Vec<LateUser> {
        let records = match raw {
            Some(r) => r,
            None => return vec![],
        };

        // Group by employee_id string, keeping employee details from first occurrence
        let mut map: HashMap<String, (i64, i64, String, String, String)> = HashMap::new(); // (late_count, total_late_minutes, nik, full_name, department)

        for rec in records {
            let emp_id = Self::id_to_string(&rec["employee_id"]);
            let late_minutes = rec["late_minutes"].as_i64().unwrap_or(0);
            let nik = rec["nik"].as_str().unwrap_or("").to_string();
            let full_name = rec["full_name"].as_str().unwrap_or("").to_string();
            let department = rec["department"].as_str().unwrap_or("").to_string();
            
            let entry = map.entry(emp_id).or_insert((0, 0, nik, full_name, department));
            entry.0 += 1;
            entry.1 += late_minutes;
        }

        let mut users: Vec<LateUser> = map
            .into_iter()
            .map(|(employee_id, (late_count, total_late_minutes, nik, full_name, department))| {
                let avg_late_minutes = if late_count > 0 {
                    total_late_minutes as f64 / late_count as f64
                } else {
                    0.0
                };
                LateUser {
                    employee_id,
                    nik,
                    full_name,
                    department,
                    late_count,
                    total_late_minutes,
                    avg_late_minutes,
                }
            })
            .collect();

        users.sort_by(|a, b| b.late_count.cmp(&a.late_count));
        users.truncate(10);
        users
    }

    // ─── Patrol Analytics ─────────────────────────────────────────────────────
    async fn get_patrol_analytics(
        from: &str,
        to: &str,
        data: &web::Data<AppState>,
    ) -> Result<PatrolAnalytics, String> {
        let query = format!(
            r#"
            LET $from = <datetime>"{from}";
            LET $to   = <datetime>"{to}";

            LET $patrol_completed = (
                SELECT count() AS total FROM patrol_session
                WHERE started_at >= $from AND started_at < $to AND status = 'completed'
                GROUP ALL
            )[0].total ?? 0;

            LET $patrol_incomplete = (
                SELECT count() AS total FROM patrol_session
                WHERE started_at >= $from AND started_at < $to AND status = 'incomplete'
                GROUP ALL
            )[0].total ?? 0;

            LET $patrol_in_progress = (
                SELECT count() AS total FROM patrol_session
                WHERE started_at >= $from AND started_at < $to AND status = 'in_progress'
                GROUP ALL
            )[0].total ?? 0;

            LET $total_missed = (
                SELECT math::sum(checkpoints_missed) AS total FROM patrol_session
                WHERE started_at >= $from AND started_at < $to
                GROUP ALL
            )[0].total ?? 0;

            LET $total_checkpoints = (
                SELECT math::sum(checkpoints_total) AS total FROM patrol_session
                WHERE started_at >= $from AND started_at < $to
                GROUP ALL
            )[0].total ?? 0;

            LET $avg_duration = (
                SELECT math::mean(duration_minutes) AS avg FROM patrol_session
                WHERE started_at >= $from AND started_at < $to
                    AND status = 'completed' AND duration_minutes != NONE
                GROUP ALL
            )[0].avg ?? 0;

            LET $patrol_trend_raw = (
                SELECT
                    type::string(started_at) AS started_at,
                    type::string(employee_id) AS employee_id,
                    employee_id.nik AS employee_nik,
                    employee_id.full_name AS employee_name,
                    employee_id.department_id.name AS employee_department,
                    type::string(location_id) AS location_id,
                    location_id.name AS location_name,
                    location_id.address AS location_address,
                    status, duration_minutes,
                    checkpoints_total, checkpoints_visited, checkpoints_missed
                FROM patrol_session
                WHERE started_at >= $from AND started_at < $to
                ORDER BY started_at ASC
                FETCH employee_id, employee_id.department_id, location_id
            );

            RETURN {{
                patrol_completed:   $patrol_completed,
                patrol_incomplete:  $patrol_incomplete,
                patrol_in_progress: $patrol_in_progress,
                total_missed:       $total_missed,
                total_checkpoints:  $total_checkpoints,
                avg_duration:       $avg_duration,
                patrol_trend_raw:   $patrol_trend_raw
            }};
            "#,
            from = from,
            to = to
        );

        let mut result = data
            .db
            .query(&query)
            .await
            .map_err(|e| format!("DB error patrol: {}", e))?;

        let raw: Option<serde_json::Value> = result
            .take(9)
            .map_err(|e| format!("Parse error patrol: {}", e))?;

        let v = raw.unwrap_or_default();

        let completed = v["patrol_completed"].as_i64().unwrap_or(0);
        let incomplete = v["patrol_incomplete"].as_i64().unwrap_or(0);
        let in_progress = v["patrol_in_progress"].as_i64().unwrap_or(0);
        let total = completed + incomplete + in_progress;

        let completion_rate = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let daily_patrol_trend =
            Self::build_daily_patrol_trend(v["patrol_trend_raw"].as_array());
        
        let patrol_records = Self::build_patrol_records(v["patrol_trend_raw"].as_array());

        Ok(PatrolAnalytics {
            completion: PatrolCompletion {
                completed,
                incomplete,
                in_progress,
                total,
                completion_rate,
            },
            checkpoints: CheckpointStats {
                total_missed: v["total_missed"].as_i64().unwrap_or(0),
                total_checkpoints: v["total_checkpoints"].as_i64().unwrap_or(0),
            },
            average_patrol_duration_minutes: v["avg_duration"].as_f64().unwrap_or(0.0),
            daily_patrol_trend,
            patrol_records,
        })
    }

    fn build_daily_patrol_trend(
        raw: Option<&Vec<serde_json::Value>>,
    ) -> Vec<DailyPatrolTrend> {
        let records = match raw {
            Some(r) => r,
            None => return vec![],
        };

        let mut map: HashMap<String, (i64, i64, i64)> = HashMap::new(); // (total, completed, incomplete)

        for rec in records {
            let date = rec["started_at"]
                .as_str()
                .unwrap_or("")
                .get(..10)
                .unwrap_or("")
                .to_string();

            if date.is_empty() {
                continue;
            }

            let status = rec["status"].as_str().unwrap_or("");
            let entry = map.entry(date).or_insert((0, 0, 0));
            entry.0 += 1;
            if status == "completed" {
                entry.1 += 1;
            } else if status == "incomplete" {
                entry.2 += 1;
            }
        }

        let mut trend: Vec<DailyPatrolTrend> = map
            .into_iter()
            .map(|(date, (total, completed, incomplete))| DailyPatrolTrend {
                date,
                total_patrols: total,
                completed,
                incomplete,
            })
            .collect();

        trend.sort_by(|a, b| a.date.cmp(&b.date));
        trend
    }

    fn build_patrol_records(raw: Option<&Vec<serde_json::Value>>) -> Vec<PatrolRecord> {
        let records = match raw {
            Some(r) => r,
            None => return vec![],
        };

        records
            .iter()
            .map(|rec| PatrolRecord {
                started_at: rec["started_at"].as_str().unwrap_or("").to_string(),
                employee_id: Self::id_to_string(&rec["employee_id"]),
                employee_nik: rec["employee_nik"].as_str().unwrap_or("").to_string(),
                employee_name: rec["employee_name"].as_str().unwrap_or("").to_string(),
                employee_department: rec["employee_department"].as_str().unwrap_or("").to_string(),
                location_id: Self::id_to_string(&rec["location_id"]),
                location_name: rec["location_name"].as_str().unwrap_or("").to_string(),
                location_address: rec["location_address"].as_str().unwrap_or("").to_string(),
                status: rec["status"].as_str().unwrap_or("").to_string(),
                duration_minutes: rec["duration_minutes"].as_f64(),
                checkpoints_total: rec["checkpoints_total"].as_i64().unwrap_or(0),
                checkpoints_visited: rec["checkpoints_visited"].as_i64().unwrap_or(0),
                checkpoints_missed: rec["checkpoints_missed"].as_i64().unwrap_or(0),
            })
            .collect()
    }

    // ─── Incident Analytics ───────────────────────────────────────────────────
    async fn get_incident_analytics(
        from: &str,
        to: &str,
        data: &web::Data<AppState>,
    ) -> Result<IncidentAnalytics, String> {
        let query = format!(
            r#"
            LET $from = <datetime>"{from}";
            LET $to   = <datetime>"{to}";

            LET $by_severity = (
                SELECT severity, count() AS total
                FROM patrol_incidents
                WHERE <datetime>created_at >= $from AND <datetime>created_at < $to AND severity != NONE
                GROUP BY severity
                ORDER BY total DESC
            );

            LET $by_status = (
                SELECT status, count() AS total
                FROM patrol_incidents
                WHERE <datetime>created_at >= $from AND <datetime>created_at < $to AND status != NONE
                GROUP BY status
                ORDER BY total DESC
            );

            LET $incident_trend_raw = (
                SELECT created_at, severity, status, title
                FROM patrol_incidents
                WHERE <datetime>created_at >= $from AND <datetime>created_at < $to
                    AND severity != NONE AND status != NONE
                ORDER BY created_at ASC
            );

            LET $high_severity = (
                SELECT 
                    type::string(id) AS id,              
                    type::string(employee_id) AS employee_id,  
                    title, description, severity, status, nik,
                    latitude, longitude, created_at
                FROM patrol_incidents
                WHERE <datetime>created_at >= $from AND <datetime>created_at < $to AND severity = 'high'
                ORDER BY created_at DESC
            );

            LET $top_locations_raw = (
                SELECT latitude, longitude, title, severity, status, created_at
                FROM patrol_incidents
                WHERE <datetime>created_at >= $from AND <datetime>created_at < $to
                    AND latitude != NONE AND longitude != NONE AND severity != NONE
                ORDER BY created_at DESC
            );

            RETURN {{
                by_severity:        $by_severity,
                by_status:          $by_status,
                incident_trend_raw: $incident_trend_raw,
                high_severity:      $high_severity,
                top_locations_raw:  $top_locations_raw
            }};
            "#,
            from = from,
            to = to
        );

        let mut result = data
            .db
            .query(&query)
            .await
            .map_err(|e| format!("DB error incident: {}", e))?;

        let raw: Option<serde_json::Value> = result
            .take(7)
            .map_err(|e| format!("Parse error incident: {}", e))?;

        let v = raw.unwrap_or_default();

        // by_severity with percentage
        let by_severity_arr = v["by_severity"].as_array().cloned().unwrap_or_default();
        let total_incidents: i64 = by_severity_arr
            .iter()
            .map(|x| x["total"].as_i64().unwrap_or(0))
            .sum();

        let by_severity = by_severity_arr
            .iter()
            .map(|x| {
                let count = x["total"].as_i64().unwrap_or(0);
                IncidentBySeverity {
                    severity: x["severity"].as_str().unwrap_or("unknown").to_string(),
                    total: count,
                    percentage: if total_incidents > 0 {
                        (count as f64 / total_incidents as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect();

        let by_status = v["by_status"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|x| IncidentByStatus {
                status: x["status"].as_str().unwrap_or("unknown").to_string(),
                total: x["total"].as_i64().unwrap_or(0),
            })
            .collect();

        // incident trend: group by date
        let incident_trend =
            Self::build_incident_trend(v["incident_trend_raw"].as_array());

        // high severity
        let high_severity_incidents = v["high_severity"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|x| HighSeverityIncident {
                title: x["title"].as_str().unwrap_or("").to_string(),
                description: x["description"].as_str().unwrap_or("").to_string(),
                severity: x["severity"].as_str().unwrap_or("").to_string(),
                status: x["status"].as_str().unwrap_or("").to_string(),
                nik: x["nik"].as_str().unwrap_or("").to_string(),
                latitude: x["latitude"].as_f64().unwrap_or(0.0),
                longitude: x["longitude"].as_f64().unwrap_or(0.0),
                created_at: x["created_at"].as_str().unwrap_or("").to_string(),
            })
            .collect();

        // top locations: group by lat/lng
        let top_incident_locations =
            Self::build_top_incident_locations(v["top_locations_raw"].as_array());

        Ok(IncidentAnalytics {
            by_severity,
            by_status,
            incident_trend,
            high_severity_incidents,
            top_incident_locations,
        })
    }

    fn build_incident_trend(raw: Option<&Vec<serde_json::Value>>) -> Vec<IncidentTrend> {
        let records = match raw {
            Some(r) => r,
            None => return vec![],
        };

        let mut map: HashMap<String, i64> = HashMap::new();

        for rec in records {
            let date = rec["created_at"]
                .as_str()
                .unwrap_or("")
                .get(..10)
                .unwrap_or("")
                .to_string();
            if !date.is_empty() {
                *map.entry(date).or_insert(0) += 1;
            }
        }

        let mut trend: Vec<IncidentTrend> = map
            .into_iter()
            .map(|(date, total)| IncidentTrend { date, total })
            .collect();

        trend.sort_by(|a, b| a.date.cmp(&b.date));
        trend
    }

    fn build_top_incident_locations(
        raw: Option<&Vec<serde_json::Value>>,
    ) -> Vec<IncidentLocation> {
        let records = match raw {
            Some(r) => r,
            None => return vec![],
        };

        // Group by lat+lng key
        let mut map: HashMap<String, (f64, f64, i64, i64)> =
            HashMap::new(); // (lat, lng, count, high_count)

        for rec in records {
            let lat = rec["latitude"].as_f64().unwrap_or(0.0);
            let lng = rec["longitude"].as_f64().unwrap_or(0.0);
            let key = format!("{:.4}_{:.4}", lat, lng);
            let severity = rec["severity"].as_str().unwrap_or("");

            let entry = map.entry(key).or_insert((lat, lng, 0, 0));
            entry.2 += 1;
            if severity == "high" {
                entry.3 += 1;
            }
        }

        let mut locations: Vec<IncidentLocation> = map
            .into_iter()
            .map(|(_key, (lat, lng, count, _high))| IncidentLocation {
                location_name: format!("({:.4}, {:.4})", lat, lng),
                latitude: lat,
                longitude: lng,
                incident_count: count,
            })
            .collect();

        locations.sort_by(|a, b| b.incident_count.cmp(&a.incident_count));
        locations.truncate(10);
        locations
    }

    // ─── Performance Analytics ────────────────────────────────────────────────
    async fn get_performance_analytics(
        from: &str,
        to: &str,
        data: &web::Data<AppState>,
    ) -> Result<PerformanceAnalytics, String> {
        let query = format!(
            r#"
            LET $from = <datetime>"{from}";
            LET $to   = <datetime>"{to}";

            LET $employees = (
                SELECT type::string(id) AS id, full_name, nik, department_id.name AS department
                FROM employee WHERE employment_status = 'active'
            );
            LET $present_per_emp = (
                SELECT type::string(employee_id) AS employee_id, count() AS present_days
                FROM attendance_log
                WHERE work_date >= $from AND work_date < $to AND status = 'present'
                GROUP BY employee_id
            );

            LET $absent_per_emp = (
                SELECT <string>employee_id AS employee_id, count() AS absent_days
                FROM attendance_log
                WHERE work_date >= $from AND work_date < $to AND status = 'absent'
                GROUP BY employee_id
            );

            LET $ontime_per_emp = (
                SELECT <string>employee_id AS employee_id, count() AS on_time_days
                FROM attendance_log
                WHERE work_date >= $from AND work_date < $to AND status = 'present' AND late_minutes = 0
                GROUP BY employee_id
            );

            LET $late_per_emp = (
                SELECT <string>employee_id AS employee_id, count() AS late_days, math::sum(late_minutes) AS total_late_minutes
                FROM attendance_log
                WHERE work_date >= $from AND work_date < $to AND late_minutes > 0
                GROUP BY employee_id
            );

            LET $patrol_per_emp = (
                SELECT <string>employee_id AS employee_id, count() AS completed_patrols
                FROM patrol_session
                WHERE started_at >= $from AND started_at < $to AND status = 'completed'
                GROUP BY employee_id
            );

            RETURN {{
                employees:       $employees,
                present_per_emp: $present_per_emp,
                absent_per_emp:  $absent_per_emp,
                ontime_per_emp:  $ontime_per_emp,
                late_per_emp:    $late_per_emp,
                patrol_per_emp:  $patrol_per_emp
            }};
            "#,
            from = from,
            to = to
        );

        let mut result = data
            .db
            .query(&query)
            .await
            .map_err(|e| format!("DB error performance: {}", e))?;

        let raw: Option<serde_json::Value> = result
            .take(8)
            .map_err(|e| format!("Parse error performance: {}", e))?;

        let v = raw.unwrap_or_default();

        let empty_vec = vec![];
        let employees = v["employees"].as_array().unwrap_or(&empty_vec);
        let present_per_emp = v["present_per_emp"].as_array().unwrap_or(&empty_vec);
        let absent_per_emp = v["absent_per_emp"].as_array().unwrap_or(&empty_vec);
        let ontime_per_emp = v["ontime_per_emp"].as_array().unwrap_or(&empty_vec);
        let late_per_emp = v["late_per_emp"].as_array().unwrap_or(&empty_vec);
        let patrol_per_emp = v["patrol_per_emp"].as_array().unwrap_or(&empty_vec);

        // Helper: find value in array by employee_id
        let find_by_emp = |arr: &Vec<serde_json::Value>, emp_id: &str, field: &str| -> i64 {
            arr.iter()
                .find(|x| Self::id_to_string(&x["employee_id"]) == emp_id)
                .and_then(|x| x[field].as_i64())
                .unwrap_or(0)
        };

        let mut performance_users: Vec<PerformanceUser> = employees
            .iter()
            .map(|emp| {
                let emp_id = Self::id_to_string(&emp["id"]);
                let present = find_by_emp(present_per_emp, &emp_id, "present_days");
                let absent = find_by_emp(absent_per_emp, &emp_id, "absent_days");
                let on_time = find_by_emp(ontime_per_emp, &emp_id, "on_time_days");
                let late_days = find_by_emp(late_per_emp, &emp_id, "late_days");
                let total_late_minutes =
                    find_by_emp(late_per_emp, &emp_id, "total_late_minutes");
                let patrol_completed =
                    find_by_emp(patrol_per_emp, &emp_id, "completed_patrols");

                let total_days = present + absent;
                let compliance_score = if total_days > 0 {
                    (present as f64 / total_days as f64) * 100.0
                } else {
                    0.0
                };
                let activity_score = if present > 0 {
                    (on_time as f64 / present as f64) * 100.0
                } else {
                    0.0
                };
                let final_score = (compliance_score * 0.6) + (activity_score * 0.4);

                PerformanceUser {
                    employee_id: emp_id,
                    nik: emp["nik"].as_str().unwrap_or("").to_string(),
                    full_name: emp["full_name"].as_str().unwrap_or("").to_string(),
                    department: emp["department"].as_str().unwrap_or("").to_string(),
                    present_days: present,
                    absent_days: absent,
                    on_time_days: on_time,
                    late_days,
                    total_late_minutes,
                    patrol_completed,
                    compliance_score,
                    activity_score,
                    final_score,
                }
            })
            .collect();

        performance_users
            .sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());

        let top_performers = performance_users.iter().take(5).cloned().collect();

        performance_users
            .sort_by(|a, b| a.final_score.partial_cmp(&b.final_score).unwrap());

        let low_performers = performance_users.iter().take(5).cloned().collect();

        let total = performance_users.len() as f64;
        let average_compliance_score = if total > 0.0 {
            performance_users.iter().map(|u| u.compliance_score).sum::<f64>() / total
        } else {
            0.0
        };
        let average_activity_score = if total > 0.0 {
            performance_users.iter().map(|u| u.activity_score).sum::<f64>() / total
        } else {
            0.0
        };
        let average_final_score = if total > 0.0 {
            performance_users.iter().map(|u| u.final_score).sum::<f64>() / total
        } else {
            0.0
        };

        Ok(PerformanceAnalytics {
            top_performers,
            low_performers,
            average_compliance_score,
            average_activity_score,
            average_final_score,
        })
    }

    // ─── Location Analytics ───────────────────────────────────────────────────
    async fn get_location_analytics(
        from: &str,
        to: &str,
        data: &web::Data<AppState>,
    ) -> Result<LocationAnalytics, String> {
        let query = format!(
            r#"
            LET $from = <datetime>"{from}";
            LET $to   = <datetime>"{to}";

            LET $all_locations = (
                SELECT type::string(id) AS id, name, address, latitude, longitude, radius_meters
                FROM work_location WHERE is_active = true
            );

            LET $patrol_per_site = (
                SELECT type::string(location_id) AS location_id, location_id.name AS location_name, count() AS total_patrols
                FROM patrol_session
                WHERE started_at >= $from AND started_at < $to 
                    AND location_id != NONE
                GROUP BY location_id
            );

            LET $patrol_completed_per_site = (
                SELECT type::string(location_id) AS location_id, count() AS completed_patrols
                FROM patrol_session
                WHERE started_at >= $from AND started_at < $to
                    AND location_id != NONE AND status = 'completed'
                GROUP BY location_id
            );

            LET $incidents_raw = (
                SELECT latitude, longitude, severity
                FROM patrol_incidents
                WHERE <datetime>created_at >= $from AND <datetime>created_at < $to
                    AND latitude != NONE AND longitude != NONE AND severity != NONE
            );

            RETURN {{
                all_locations:             $all_locations,
                patrol_per_site:           $patrol_per_site,
                patrol_completed_per_site: $patrol_completed_per_site,
                incidents_raw:             $incidents_raw
            }};
            "#,
            from = from,
            to = to
        );

        let mut result = data
            .db
            .query(&query)
            .await
            .map_err(|e| format!("DB error location: {}", e))?;

        let raw: Option<serde_json::Value> = result
            .take(6)
            .map_err(|e| format!("Parse error location: {}", e))?;

        let v = raw.unwrap_or_default();

        let empty_vec = vec![];
        let all_locations = v["all_locations"].as_array().unwrap_or(&empty_vec);
        let patrol_per_site = v["patrol_per_site"].as_array().unwrap_or(&empty_vec);
        let patrol_completed = v["patrol_completed_per_site"].as_array().unwrap_or(&empty_vec);
        let incidents_raw = v["incidents_raw"].as_array().unwrap_or(&empty_vec);

        // Match incidents to nearest location by coordinates
        let mut incident_counts: HashMap<String, (i64, i64)> = HashMap::new(); // (total, high)
        for inc in incidents_raw {
            let inc_lat = inc["latitude"].as_f64().unwrap_or(0.0);
            let inc_lng = inc["longitude"].as_f64().unwrap_or(0.0);
            let severity = inc["severity"].as_str().unwrap_or("");

            // Find nearest location
            let matched_id = all_locations
                .iter()
                .min_by(|a, b| {
                    let dist_a = (a["latitude"].as_f64().unwrap_or(0.0) - inc_lat).abs()
                        + (a["longitude"].as_f64().unwrap_or(0.0) - inc_lng).abs();
                    let dist_b = (b["latitude"].as_f64().unwrap_or(0.0) - inc_lat).abs()
                        + (b["longitude"].as_f64().unwrap_or(0.0) - inc_lng).abs();
                    dist_a.partial_cmp(&dist_b).unwrap()
                })
                .map(|loc| Self::id_to_string(&loc["id"]))
                .unwrap_or_default();

            if !matched_id.is_empty() {
                let entry = incident_counts.entry(matched_id).or_insert((0, 0));
                entry.0 += 1;
                if severity == "high" {
                    entry.1 += 1;
                }
            }
        }

        // Build site_comparison
        let site_comparison = all_locations
            .iter()
            .map(|loc| {
                let loc_id = Self::id_to_string(&loc["id"]);
                let total_patrols = patrol_per_site
                    .iter()
                    .find(|x| Self::id_to_string(&x["location_id"]) == loc_id)
                    .and_then(|x| x["total_patrols"].as_i64())
                    .unwrap_or(0);
                let completed = patrol_completed
                    .iter()
                    .find(|x| Self::id_to_string(&x["location_id"]) == loc_id)
                    .and_then(|x| x["completed_patrols"].as_i64())
                    .unwrap_or(0);
                let completion_rate = if total_patrols > 0 {
                    (completed as f64 / total_patrols as f64) * 100.0
                } else {
                    0.0
                };
                let incident_count = incident_counts
                    .get(&loc_id)
                    .map(|x| x.0)
                    .unwrap_or(0);

                SiteComparison {
                    location_id: loc_id,
                    site_name: loc["name"].as_str().unwrap_or("").to_string(),
                    address: loc["address"].as_str().unwrap_or("").to_string(),
                    latitude: loc["latitude"].as_f64().unwrap_or(0.0),
                    longitude: loc["longitude"].as_f64().unwrap_or(0.0),
                    total_patrols,
                    completed_patrols: completed,
                    patrol_completion_rate: completion_rate,
                    incident_count,
                }
            })
            .collect();

        // incidents_per_site
        let incidents_per_site = all_locations
            .iter()
            .map(|loc| {
                let loc_id = Self::id_to_string(&loc["id"]);
                let (total, high) = incident_counts.get(&loc_id).copied().unwrap_or((0, 0));
                SiteIncidents {
                    location_name: loc["name"].as_str().unwrap_or("").to_string(),
                    latitude: loc["latitude"].as_f64().unwrap_or(0.0),
                    longitude: loc["longitude"].as_f64().unwrap_or(0.0),
                    incident_count: total,
                    high_severity_count: high,
                }
            })
            .collect();

        // patrols_per_site
        let patrols_per_site = patrol_per_site
            .iter()
            .map(|ps| {
                let loc_id = Self::id_to_string(&ps["location_id"]);
                let total = ps["total_patrols"].as_i64().unwrap_or(0);
                let completed_count = patrol_completed
                    .iter()
                    .find(|x| Self::id_to_string(&x["location_id"]) == loc_id)
                    .and_then(|x| x["completed_patrols"].as_i64())
                    .unwrap_or(0);
                let rate = if total > 0 {
                    (completed_count as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                SitePatrols {
                    location_id: loc_id,
                    site_name: ps["location_name"].as_str().unwrap_or("").to_string(),
                    total_patrols: total,
                    completed_patrols: completed_count,
                    completion_rate: rate,
                }
            })
            .collect();

        Ok(LocationAnalytics {
            site_comparison,
            incidents_per_site,
            patrols_per_site,
        })
    }
}