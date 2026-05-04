use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmployeeGroup {
    pub id: Option<Thing>,
    pub name: String,
    pub description: Option<String>,
    pub employee_ids: Vec<Thing>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub employee_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub employee_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmployeeGroupResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub employee_ids: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<EmployeeGroup> for EmployeeGroupResponse {
    fn from(g: EmployeeGroup) -> Self {
        Self {
            id: g.id.map(|t| t.to_string()).unwrap_or_default(),
            name: g.name,
            description: g.description,
            employee_ids: g.employee_ids.iter().map(|t| t.to_string()).collect(),
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }
}
