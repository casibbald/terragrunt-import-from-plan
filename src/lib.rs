pub mod importer;
pub mod plan;
pub mod schema;
pub mod utils;

// Re-export specific items to avoid ambiguity
pub use importer::{PlannedModule, Resource, PlanFile};
pub use plan::{get_id_candidate_fields, score_attributes_for_id};
pub use schema::write_provider_schema;
pub use utils::{collect_resources, collect_all_resources, extract_id_candidate_fields, run_terragrunt_init};
