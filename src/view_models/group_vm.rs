use actix_web::web;
use chrono::Local;
use surrealdb::sql::{Thing, Id};

use crate::config::app_state::AppState;
use crate::models::group::{EmployeeGroup, CreateGroupRequest, UpdateGroupRequest, EmployeeGroupResponse};

pub struct GroupViewModel;

impl GroupViewModel {
    pub async fn create_group(
        payload: web::Json<CreateGroupRequest>,
        data: &web::Data<AppState>,
    ) -> Result<EmployeeGroupResponse, String> {
        let employee_ids: Vec<Thing> = payload.employee_ids
            .iter()
            .map(|id| Thing::from(("employee", Id::from(id.as_str()))))
            .collect();

        let group = EmployeeGroup {
            id: None,
            name: payload.name.clone(),
            description: payload.description.clone(),
            employee_ids,
            created_at: Some(Local::now().to_rfc3339()),
            updated_at: Some(Local::now().to_rfc3339()),
        };

        let created: Option<EmployeeGroup> = data
            .db
            .create("groups")
            .content(group)
            .await
            .map_err(|e| format!("Failed to create group: {}", e))?;

        created
            .map(EmployeeGroupResponse::from)
            .ok_or_else(|| "Failed to return created group".to_string())
    }

    pub async fn get_all_groups(
        data: &web::Data<AppState>,
    ) -> Result<Vec<EmployeeGroupResponse>, String> {
        let mut result = data
            .db
            .query("SELECT * FROM groups ORDER BY created_at DESC")
            .await
            .map_err(|e| format!("Database query error: {}", e))?;

        let groups: Vec<EmployeeGroup> = result
            .take(0)
            .map_err(|e| format!("Failed to parse groups: {}", e))?;

        Ok(groups.into_iter().map(EmployeeGroupResponse::from).collect())
    }

    pub async fn get_group_by_id(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<EmployeeGroupResponse, String> {
        let id_part = if id.contains(':') {
            id.split(':').last().unwrap_or(id)
        } else {
            id
        };
        
        let mut result = data.db.query("SELECT * FROM groups WHERE id = type::thing('groups', $id)")
            .bind(("id", id_part.to_string()))
            .await
            .map_err(|e| e.to_string())?;
            
        let group: Option<EmployeeGroup> = result.take(0).map_err(|e| e.to_string())?;
        
        group.map(EmployeeGroupResponse::from).ok_or_else(|| "Group not found".to_string())
    }

    pub async fn update_group(
        id: &str,
        payload: web::Json<UpdateGroupRequest>,
        data: &web::Data<AppState>,
    ) -> Result<EmployeeGroupResponse, String> {
        let id_part = if id.contains(':') {
            id.split(':').last().unwrap_or(id)
        } else {
            id
        };
        
        let mut existing_res = data.db.query("SELECT * FROM groups WHERE id = type::thing('groups', $id)")
            .bind(("id", id_part.to_string()))
            .await
            .map_err(|e| e.to_string())?;
            
        let mut existing: EmployeeGroup = existing_res.take::<Option<EmployeeGroup>>(0)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Group not found".to_string())?;

        if let Some(name) = &payload.name { existing.name = name.clone(); }
        if let Some(desc) = &payload.description { existing.description = Some(desc.clone()); }
        if let Some(emp_ids) = &payload.employee_ids {
            existing.employee_ids = emp_ids.iter().map(|id| Thing::from(("employee", Id::from(id.as_str())))).collect();
        }
        existing.updated_at = Some(Local::now().to_rfc3339());

        let updated: Option<EmployeeGroup> = data.db.update(("groups", id_part)).content(existing).await.map_err(|e| e.to_string())?;
        
        updated.map(EmployeeGroupResponse::from).ok_or_else(|| "Failed to update group".to_string())
    }

    pub async fn delete_group(
        id: &str,
        data: &web::Data<AppState>,
    ) -> Result<String, String> {
        let id_part = if id.contains(':') {
            id.split(':').last().unwrap_or(id)
        } else {
            id
        };
        let _: Option<EmployeeGroup> = data.db.delete(("groups", id_part)).await.map_err(|e| e.to_string())?;
        Ok("Group deleted successfully".to_string())
    }
}
