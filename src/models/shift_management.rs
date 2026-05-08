use serde::{Deserialize, Serialize};
use surrealdb::sql::{Thing, Datetime};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShiftScheduleItem {
    pub shift_number: i32,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShiftTask {
    pub id: Option<Thing>,
    pub name: String,
    pub department: String,
    pub working_location: String,
    pub shift_type: String, // "2 Sesi" or "3 Sesi"
    pub number_of_groups: i32,
    pub schedules: Vec<ShiftScheduleItem>,
    pub is_active: bool,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmployeeGroup {
    pub id: Option<Thing>,
    pub shift_task_id: Thing,
    pub name: String,
    pub employee_ids: Vec<Thing>,  // Changed from employee_niks to employee_ids (record references)
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShiftRotationSchedule {
    pub id: Option<Thing>,
    pub shift_task_id: Thing,
    pub date: String,
    pub day_name: String,
    pub week_number: i32,
    pub shift_1_group_id: Option<Thing>,
    pub shift_2_group_id: Option<Thing>,
    pub shift_3_group_id: Option<Thing>,
    pub off_group_id: Option<Thing>,
    pub is_manual_override: bool,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
}

// Request / Response DTOs

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateShiftTaskRequest {
    pub name: String,
    pub department: String,
    pub working_location: String,
    pub shift_type: String,
    pub number_of_groups: i32,
    pub schedules: Vec<ShiftScheduleItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateShiftTaskRequest {
    pub name: Option<String>,
    pub department: Option<String>,
    pub working_location: Option<String>,
    pub shift_type: Option<String>,
    pub number_of_groups: Option<i32>,
    pub schedules: Option<Vec<ShiftScheduleItem>>,
    pub is_active: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveEmployeeGroupsRequest {
    pub groups: Vec<EmployeeGroupInput>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmployeeGroupInput {
    pub name: String,
    pub employee_niks: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateScheduleRequest {
    pub start_date: String,
    pub weeks: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRotationScheduleRequest {
    pub shift_1_group_id: Option<String>,
    pub shift_2_group_id: Option<String>,
    pub shift_3_group_id: Option<String>,
    pub off_group_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericResponse<T> {
    pub status: String,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupWithEmployees {
    pub id: String,
    pub name: String,
    pub employees: Vec<crate::models::employee::EmployeeResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShiftTaskGroupsResponse {
    pub shift_task_id: String,
    pub shift_task_name: String,
    pub groups: Vec<GroupWithEmployees>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScheduleGenerationSummary {
    pub shift_task_id: String,
    pub start_date: String,
    pub end_date: String,
    pub total_days: i32,
    pub schedules_created: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupMiniResponse {
    pub group_id: String,
    pub group_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RotationScheduleResponse {
    pub id: String,
    pub shift_task_id: String,
    pub date: String,
    pub day_name: String,
    pub week_number: i32,
    pub shift_1: Option<GroupMiniResponse>,
    pub shift_2: Option<GroupMiniResponse>,
    pub shift_3: Option<GroupMiniResponse>,
    pub off: Option<GroupMiniResponse>,
    pub is_manual_override: bool,
    pub created_at: Option<Datetime>,
}
