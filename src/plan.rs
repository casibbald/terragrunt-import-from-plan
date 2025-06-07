
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformPlan {
    pub planned_values: PlannedValues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedValues {
    pub root_module: RootModule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootModule {
    pub child_modules: Vec<ChildModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildModule {
    pub resources: Vec<TerraformResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformResource {
    pub address: String,
    pub mode: String,
    pub r#type: String,
    pub name: String,
    pub values: Option<Value>,
}
