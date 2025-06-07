use crate::importer::{PlannedModule, Resource};
use serde_json::Value;
use std::collections::{HashMap, HashSet};


pub fn collect_resources<'a>(module: &'a PlannedModule, all: &mut Vec<&'a Resource>) {
    if let Some(res) = &module.resources {
        all.extend(res.iter());
    }
    if let Some(children) = &module.child_modules {
        for child in children {
            collect_resources(child, all);
        }
    }
}


pub fn collect_all_resources<'a>(module: &'a PlannedModule, resources: &mut Vec<&'a Resource>) {
    if let Some(rs) = &module.resources {
        resources.extend(rs.iter());
    }
    if let Some(children) = &module.child_modules {
        for child in children {
            collect_all_resources(child, resources);
        }
    }
}

pub fn extract_id_candidate_fields(schema_json: &Value) -> HashSet<String> {
    let mut candidates = HashSet::new();

    if let Some(resource_schemas) = schema_json
        .get("provider_schemas")
        .and_then(|ps| ps.get("google")) // assumes Google provider
        .and_then(|g| g.get("resource_schemas"))
        .and_then(|rs| rs.as_object())
    {
        for (_resource_type, schema) in resource_schemas {
            if let Some(block) = schema.get("block") {
                if let Some(attributes) = block.get("attributes").and_then(|a| a.as_object()) {
                    for (attr_name, _) in attributes {
                        candidates.insert(attr_name.clone());
                    }
                }
            }
        }
    }

    candidates
}