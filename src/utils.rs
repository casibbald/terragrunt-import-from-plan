use crate::importer::{PlannedModule, Resource};

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
