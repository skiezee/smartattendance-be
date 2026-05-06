use actix_web::web;
use chrono::{Datelike, Duration, NaiveDate};
use serde::Deserialize;
use crate::config::app_state::AppState;
use crate::models::employee::EmployeeResponse;
use crate::models::shift_management::*;
use surrealdb::sql::{Thing, Id};

pub struct ShiftManagementViewModel;

impl ShiftManagementViewModel {
    // 1. Shift Task CRUD
    
    pub async fn get_all_shift_tasks(
        department: Option<String>,
        shift_type: Option<String>,
        is_active: Option<bool>,
        data: &web::Data<AppState>,
    ) -> Result<Vec<ShiftTask>, String> {
        let mut query = "SELECT * FROM shift_task WHERE 1=1".to_string();
        
        if department.is_some() {
            query.push_str(" AND department = $department");
        }
        if shift_type.is_some() {
            query.push_str(" AND shift_type = $shift_type");
        }
        if is_active.is_some() {
            query.push_str(" AND is_active = $is_active");
        }
        
        query.push_str(" ORDER BY created_at DESC");
        
        let mut q = data.db.query(query);
        
        if let Some(d) = department { q = q.bind(("department", d)); }
        if let Some(st) = shift_type { q = q.bind(("shift_type", st)); }
        if let Some(ia) = is_active { q = q.bind(("is_active", ia)); }
        
        let mut result = q.await.map_err(|e| format!("Database error: {}", e))?;
        let tasks: Vec<ShiftTask> = result.take(0).map_err(|e| format!("Parse error: {}", e))?;
        
        Ok(tasks)
    }

    pub async fn get_shift_task_by_id(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<Option<ShiftTask>, String> {
        let mut result = data.db.query("SELECT * FROM type::thing($id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| format!("Database error: {}", e))?;
            
        let task: Option<ShiftTask> = result.take(0).map_err(|e| format!("Parse error: {}", e))?;
        Ok(task)
    }

    pub async fn create_shift_task(
        payload: web::Json<CreateShiftTaskRequest>,
        data: &web::Data<AppState>,
    ) -> Result<ShiftTask, String> {
        // Validation
        if payload.name.len() > 255 { return Err("Name too long".to_string()); }
        let valid_depts = ["Security", "Network Operation Center", "Service Operation Center", "Customer Service"];
        if !valid_depts.contains(&payload.department.as_str()) {
            return Err(format!("Invalid department. Must be one of: {}", valid_depts.join(", ")));
        }
        if payload.shift_type != "2 Sesi" && payload.shift_type != "3 Sesi" {
            return Err("shift_type must be '2 Sesi' or '3 Sesi'".to_string());
        }
        let expected_schedules = if payload.shift_type == "2 Sesi" { 2 } else { 3 };
        if payload.schedules.len() != expected_schedules {
            return Err(format!("Schedules must have {} items for {}", expected_schedules, payload.shift_type));
        }
        if payload.number_of_groups < 1 || payload.number_of_groups > 10 {
            return Err("number_of_groups must be between 1 and 10".to_string());
        }

        let now = surrealdb::sql::Datetime::from(chrono::Utc::now());
        let task = ShiftTask {
            id: None,
            name: payload.name.clone(),
            department: payload.department.clone(),
            working_location: payload.working_location.clone(),
            shift_type: payload.shift_type.clone(),
            number_of_groups: payload.number_of_groups,
            schedules: payload.schedules.clone(),
            is_active: true,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };

        let created: Option<ShiftTask> = data.db.create("shift_task")
            .content(task)
            .await
            .map_err(|e| format!("Failed to create: {}", e))?;
            
        created.ok_or_else(|| "Failed to create shift task".to_string())
    }

    pub async fn update_shift_task(
        id: &str,
        payload: web::Json<UpdateShiftTaskRequest>,
        data: &web::Data<AppState>,
    ) -> Result<ShiftTask, String> {
        let now = surrealdb::sql::Datetime::from(chrono::Utc::now());
        
        let mut update_query = "UPDATE type::thing($id) SET updated_at = $now".to_string();
        if payload.name.is_some() { update_query.push_str(", name = $name"); }
        if payload.department.is_some() { update_query.push_str(", department = $department"); }
        if payload.working_location.is_some() { update_query.push_str(", working_location = $working_location"); }
        if payload.shift_type.is_some() { update_query.push_str(", shift_type = $shift_type"); }
        if payload.number_of_groups.is_some() { update_query.push_str(", number_of_groups = $number_of_groups"); }
        if payload.schedules.is_some() { update_query.push_str(", schedules = $schedules"); }
        if payload.is_active.is_some() { update_query.push_str(", is_active = $is_active"); }

        let mut q = data.db.query(update_query).bind(("id", id.to_string())).bind(("now", now.clone()));
        if let Some(n) = &payload.name { q = q.bind(("name", n.clone())); }
        if let Some(d) = &payload.department { q = q.bind(("department", d.clone())); }
        if let Some(wl) = &payload.working_location { q = q.bind(("working_location", wl.clone())); }
        if let Some(st) = &payload.shift_type { q = q.bind(("shift_type", st.clone())); }
        if let Some(nog) = payload.number_of_groups { q = q.bind(("number_of_groups", nog)); }
        if let Some(s) = &payload.schedules { q = q.bind(("schedules", s.clone())); }
        if let Some(ia) = payload.is_active { q = q.bind(("is_active", ia)); }

        let mut result = q.await.map_err(|e| format!("Update error: {}", e))?;
        let updated: Vec<ShiftTask> = result.take(0).map_err(|e| format!("Parse error: {}", e))?;
        
        updated.into_iter().next().ok_or_else(|| "Shift task not found".to_string())
    }

    pub async fn delete_shift_task(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<(), String> {
        // Check for active employee groups
        let mut check = data.db.query("SELECT * FROM employee_group WHERE shift_task_id = type::thing($id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| format!("Check error: {}", e))?;
        
        let groups: Vec<EmployeeGroup> = check.take(0).map_err(|e| format!("Parse error: {}", e))?;
        if !groups.is_empty() {
            return Err("CONFLICT: Cannot delete shift task with active employee groups".to_string());
        }

        data.db.query("DELETE type::thing($id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| format!("Delete error: {}", e))?;
            
        Ok(())
    }

    // 2. Employee Group Management
    
    pub async fn get_available_employees(
        shift_task_id: &str,
        data: &web::Data<AppState>,
    ) -> Result<Vec<EmployeeResponse>, String> {
        let task = Self::get_shift_task_by_id(shift_task_id, data).await?
            .ok_or_else(|| "Shift task not found".to_string())?;

        // Fetch all active employees
        let mut result = data.db.query("SELECT * FROM employee WHERE status = 'active'")
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        let employees: Vec<EmployeeResponse> = result.take(0).map_err(|e| format!("Parse error: {}", e))?;

        // Filter by flexible department matching
        let filtered: Vec<EmployeeResponse> = employees.into_iter()
            .filter(|emp| {
                if let Some(emp_dept) = &emp.department {
                    matches_department(&task.department, emp_dept)
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }

    pub async fn get_employee_groups(
        shift_task_id: &str,
        data: &web::Data<AppState>,
    ) -> Result<ShiftTaskGroupsResponse, String> {
        let task = Self::get_shift_task_by_id(shift_task_id, data).await?
            .ok_or_else(|| "Shift task not found".to_string())?;

        // Get employee groups
        let mut result = data.db.query("SELECT * FROM employee_group WHERE shift_task_id = type::thing($id)")
            .bind(("id", shift_task_id.to_string()))
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        let groups: Vec<EmployeeGroup> = result.take(0)
            .map_err(|e| format!("Parse error: {}", e))?;

        // For each group, fetch employees manually
        let mut group_with_employees = Vec::new();
        
        for group in groups {
            let mut employees = Vec::new();
            
            // Fetch each employee by ID
            for emp_id in &group.employee_ids {
                let mut emp_result = data.db.query("SELECT * FROM type::thing($id)")
                    .bind(("id", emp_id.to_string()))
                    .await
                    .map_err(|e| format!("Failed to fetch employee: {}", e))?;
                
                let emp_list: Vec<crate::models::employee::EmployeeResponse> = emp_result.take(0)
                    .map_err(|e| format!("Failed to parse employee: {}", e))?;
                
                if let Some(emp) = emp_list.into_iter().next() {
                    employees.push(emp);
                }
            }
            
            group_with_employees.push(GroupWithEmployees {
                id: group.id.as_ref().map(|t| t.to_string()).unwrap_or_default(),
                name: group.name,
                employees,
            });
        }

        Ok(ShiftTaskGroupsResponse {
            shift_task_id: shift_task_id.to_string(),
            shift_task_name: task.name,
            groups: group_with_employees,
        })
    }

    pub async fn save_employee_groups(
        shift_task_id: &str,
        payload: web::Json<SaveEmployeeGroupsRequest>,
        data: &web::Data<AppState>,
    ) -> Result<ShiftTaskGroupsResponse, String> {
        let task = Self::get_shift_task_by_id(shift_task_id, data).await?
            .ok_or_else(|| "Shift task not found".to_string())?;

        if payload.groups.len() as i32 != task.number_of_groups {
            return Err(format!("Number of groups must be {}", task.number_of_groups));
        }

        // Delete existing groups
        data.db.query("DELETE employee_group WHERE shift_task_id = type::thing($id)")
            .bind(("id", shift_task_id.to_string()))
            .await
            .map_err(|e| format!("Clear existing groups error: {}", e))?;

        let id_part = shift_task_id.split(':').last().unwrap_or(shift_task_id);
        let shift_task_thing = Thing::from(("shift_task", Id::from(id_part)));

        // Create groups with employee_ids (record references)
        for g in &payload.groups {
            println!("Creating group '{}' with {} NIKs: {:?}", g.name, g.employee_niks.len(), g.employee_niks);
            
            // Convert NIKs to employee IDs (record references)
            let mut employee_ids = Vec::new();
            for nik in &g.employee_niks {
                let mut result = data.db.query("SELECT VALUE id FROM employee WHERE nik = $nik LIMIT 1")
                    .bind(("nik", nik.clone()))
                    .await
                    .map_err(|e| format!("Failed to find employee {}: {}", nik, e))?;
                
                // Using SELECT VALUE id returns the Thing directly, not wrapped in an object
                let emp_ids: Vec<Thing> = result.take(0)
                    .map_err(|e| format!("Failed to parse employee {}: {}", nik, e))?;
                
                if let Some(emp_id) = emp_ids.first() {
                    employee_ids.push(emp_id.clone());
                    println!("  Found employee {}: {}", nik, emp_id);
                } else {
                    println!("  WARNING: Employee with NIK {} not found", nik);
                }
            }
            
            println!("  Total employee IDs found: {}", employee_ids.len());
            
            // Create employee_group with record references
            let group = EmployeeGroup {
                id: None,
                shift_task_id: shift_task_thing.clone(),
                name: g.name.clone(),
                employee_ids,  // Array of Thing (record references)
                created_at: None,  // Will be set by DEFAULT time::now()
                updated_at: None,
            };
            
            let created: Option<EmployeeGroup> = data.db.create("employee_group")
                .content(group)
                .await
                .map_err(|e| format!("Failed to create group {}: {}", g.name, e))?;
            
            println!("  Created group: {:?}", created.as_ref().map(|eg| &eg.id));
        }

        Self::get_employee_groups(shift_task_id, data).await
    }

    // 3. Shift Schedule Management

    pub async fn generate_schedule(
        shift_task_id: &str,
        payload: web::Json<GenerateScheduleRequest>,
        data: &web::Data<AppState>,
    ) -> Result<ScheduleGenerationSummary, String> {
        let task = Self::get_shift_task_by_id(shift_task_id, data).await?
            .ok_or_else(|| "Shift task not found".to_string())?;

        let mut group_result = data.db.query("SELECT * FROM employee_group WHERE shift_task_id = type::thing($id) ORDER BY name ASC")
            .bind(("id", shift_task_id.to_string()))
            .await
            .map_err(|e| format!("Groups fetch error: {}", e))?;
        
        let groups: Vec<EmployeeGroup> = group_result.take(0).map_err(|e| format!("Parse error: {}", e))?;
        if groups.is_empty() {
            return Err("No employee groups found for this task".to_string());
        }

        let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")
            .map_err(|_| "Invalid start_date format. Use YYYY-MM-DD".to_string())?;
        
        let weeks = payload.weeks.unwrap_or(4);
        let total_days = weeks * 7;
        let now = surrealdb::sql::Datetime::from(chrono::Utc::now());
        let id_part = shift_task_id.split(':').last().unwrap_or(shift_task_id);
        let shift_task_thing = Thing::from(("shift_task", Id::from(id_part)));

        for i in 0..total_days {
            let current_date = start_date + Duration::days(i as i64);
            let week_num = (i / 7) % 4;
            let day_num = current_date.weekday().num_days_from_monday() as u8;
            
            let rotation = calculate_rotation_logic(week_num as u8, day_num, groups.len() as u8, &task.shift_type);
            
            let find_group_id = |name: &str| {
                groups.iter().find(|g| g.name.to_uppercase().contains(name))
                    .and_then(|g| g.id.clone())
            };

            let schedule = ShiftRotationSchedule {
                id: None,
                shift_task_id: shift_task_thing.clone(),
                date: current_date.format("%Y-%m-%d").to_string(),
                day_name: get_day_name_indonesia(day_num),
                week_number: (week_num + 1) as i32,
                shift_1_group_id: find_group_id(rotation.shift_1),
                shift_2_group_id: find_group_id(rotation.shift_2),
                shift_3_group_id: rotation.shift_3.and_then(|s| find_group_id(s)),
                off_group_id: find_group_id(rotation.off),
                is_manual_override: false,
                created_at: Some(now.clone()),
                updated_at: Some(now.clone()),
            };

            let _: Option<ShiftRotationSchedule> = data.db.create("shift_rotation_schedule").content(schedule).await
                .map_err(|e| format!("Failed to create schedule for {}: {}", current_date, e))?;
        }

        Ok(ScheduleGenerationSummary {
            shift_task_id: shift_task_id.to_string(),
            start_date: payload.start_date.clone(),
            end_date: (start_date + Duration::days((total_days - 1) as i64)).format("%Y-%m-%d").to_string(),
            total_days: total_days,
            schedules_created: total_days,
        })
    }

    pub async fn get_schedules(
        shift_task_id: &str,
        start_date: Option<String>,
        end_date: Option<String>,
        week_number: Option<i32>,
        data: &web::Data<AppState>,
    ) -> Result<Vec<RotationScheduleResponse>, String> {
        let mut query = "SELECT * FROM shift_rotation_schedule WHERE shift_task_id = type::thing($id)".to_string();
        if start_date.is_some() { query.push_str(" AND date >= $start_date"); }
        if end_date.is_some() { query.push_str(" AND date <= $end_date"); }
        if week_number.is_some() { query.push_str(" AND week_number = $week_number"); }
        query.push_str(" ORDER BY date ASC");

        let mut q = data.db.query(query).bind(("id", shift_task_id.to_string()));
        if let Some(sd) = start_date { q = q.bind(("start_date", sd.clone())); }
        if let Some(ed) = end_date { q = q.bind(("end_date", ed.clone())); }
        if let Some(wn) = week_number { q = q.bind(("week_number", wn)); }

        let mut result = q.await.map_err(|e| format!("Query error: {}", e))?;
        let schedules: Vec<ShiftRotationSchedule> = result.take(0).map_err(|e| format!("Parse error: {}", e))?;

        let mut group_result = data.db.query("SELECT * FROM employee_group WHERE shift_task_id = type::thing($id)")
            .bind(("id", shift_task_id.to_string()))
            .await
            .map_err(|e| format!("Groups fetch error: {}", e))?;
        let groups: Vec<EmployeeGroup> = group_result.take(0).map_err(|e| format!("Parse error: {}", e))?;

        let map_group = |id: &Option<Thing>| {
            id.as_ref().and_then(|thing| {
                groups.iter().find(|g| g.id.as_ref() == Some(thing))
                    .map(|g| GroupMiniResponse {
                        group_id: thing.to_string(),
                        group_name: g.name.clone(),
                    })
            })
        };

        let response = schedules.into_iter().map(|s| {
            RotationScheduleResponse {
                id: s.id.as_ref().map(|t| t.to_string()).unwrap_or_default(),
                shift_task_id: s.shift_task_id.to_string(),
                date: s.date,
                day_name: s.day_name,
                week_number: s.week_number,
                shift_1: map_group(&s.shift_1_group_id),
                shift_2: map_group(&s.shift_2_group_id),
                shift_3: map_group(&s.shift_3_group_id),
                off: map_group(&s.off_group_id),
                is_manual_override: s.is_manual_override,
                created_at: s.created_at,
            }
        }).collect();

        Ok(response)
    }

    pub async fn update_schedule(
        shift_task_id: &str,
        date: &str,
        payload: web::Json<UpdateRotationScheduleRequest>,
        data: &web::Data<AppState>,
    ) -> Result<RotationScheduleResponse, String> {
        let now = surrealdb::sql::Datetime::from(chrono::Utc::now());
        
        let mut update_query = "UPDATE shift_rotation_schedule SET is_manual_override = true, updated_at = $now".to_string();
        if payload.shift_1_group_id.is_some() { update_query.push_str(", shift_1_group_id = type::thing($s1)"); }
        if payload.shift_2_group_id.is_some() { update_query.push_str(", shift_2_group_id = type::thing($s2)"); }
        if payload.shift_3_group_id.is_some() { update_query.push_str(", shift_3_group_id = type::thing($s3)"); }
        if payload.off_group_id.is_some() { update_query.push_str(", off_group_id = type::thing($off)"); }
        update_query.push_str(" WHERE shift_task_id = type::thing($task_id) AND date = $date");

        let mut q = data.db.query(update_query)
            .bind(("now", now.clone()))
            .bind(("task_id", shift_task_id.to_string()))
            .bind(("date", date.to_string()));

        if let Some(s1) = &payload.shift_1_group_id { q = q.bind(("s1", s1.clone())); }
        if let Some(s2) = &payload.shift_2_group_id { q = q.bind(("s2", s2.clone())); }
        if let Some(s3) = &payload.shift_3_group_id { q = q.bind(("s3", s3.clone())); }
        if let Some(off) = &payload.off_group_id { q = q.bind(("off", off.clone())); }

        q.await.map_err(|e| format!("Update error: {}", e))?;

        let schedules = Self::get_schedules(shift_task_id, Some(date.to_string()), Some(date.to_string()), None, data).await?;
        schedules.into_iter().next().ok_or_else(|| "Schedule not found".to_string())
    }

    pub async fn delete_schedules(
        shift_task_id: &str,
        data: &web::Data<AppState>,
    ) -> Result<(), String> {
        data.db.query("DELETE shift_rotation_schedule WHERE shift_task_id = type::thing($id)")
            .bind(("id", shift_task_id.to_string()))
            .await
            .map_err(|e| format!("Delete error: {}", e))?;
        Ok(())
    }
}

// Helpers

fn matches_department(shift_dept: &str, emp_dept: &str) -> bool {
    let shift_dept_lower = shift_dept.to_lowercase();
    let emp_dept_lower = emp_dept.to_lowercase();
    let shift_words: Vec<&str> = shift_dept_lower.split_whitespace().collect();
    let emp_words: Vec<&str> = emp_dept_lower.split_whitespace().collect();
    
    shift_words.iter().any(|shift_word| {
        emp_words.iter().any(|emp_word| {
            emp_word.contains(shift_word) || shift_word.contains(emp_word)
        })
    })
}

struct Rotation<'a> {
    pub shift_1: &'a str,
    pub shift_2: &'a str,
    pub shift_3: Option<&'a str>,
    pub off: &'a str,
}

fn calculate_rotation_logic(week: u8, day: u8, num_groups: u8, shift_type: &str) -> Rotation<'static> {
    let group_names = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
    
    let offset = if shift_type == "2 Sesi" {
        if week % 2 == 0 {
            if day <= 1 { 0 } else if day <= 3 { 2 } else if day <= 5 { 0 } else { 2 }
        } else {
            if day <= 1 { 2 } else if day <= 3 { 0 } else if day <= 5 { 2 } else { 0 }
        }
    } else {
        if week % 2 == 0 {
            if day <= 1 { 0 } else if day <= 3 { 3 } else if day <= 5 { 2 } else { 1 }
        } else {
            if day <= 1 { 1 } else if day <= 3 { 0 } else if day <= 5 { 3 } else { 2 }
        }
    };
    
    Rotation {
        shift_1: group_names[offset as usize % num_groups as usize],
        shift_2: group_names[(offset + 1) as usize % num_groups as usize],
        shift_3: if shift_type == "3 Sesi" { 
            Some(group_names[(offset + 2) as usize % num_groups as usize]) 
        } else { 
            None 
        },
        off: group_names[(offset + 3) as usize % num_groups as usize],
    }
}

fn get_day_name_indonesia(day: u8) -> String {
    match day {
        0 => "SENIN",
        1 => "SELASA",
        2 => "RABU",
        3 => "KAMIS",
        4 => "JUMAT",
        5 => "SABTU",
        6 => "MINGGU",
        _ => "",
    }.to_string()
}
