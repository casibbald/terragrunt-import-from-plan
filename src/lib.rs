
pub mod app;
pub mod commands;
pub mod errors;

pub mod importer;
pub mod plan;
pub mod reporting;
pub mod schema;
pub mod scoring;
pub mod utils;

// Re-export specific items to avoid ambiguity
pub use commands::{ImportCommandBuilder, ImportExecutor, ImportCommand, ImportResult, BatchResult};
pub use importer::{PlannedModule, Resource, PlanFile};
pub use plan::{get_id_candidate_fields, score_attributes_for_id};
pub use schema::{write_provider_schema, SchemaManager, AttributeMetadata, ResourceAttributeMap};
pub use scoring::{IdScoringStrategy, ProviderType, GoogleCloudScoringStrategy, AzureScoringStrategy, DefaultScoringStrategy};
pub use utils::{collect_resources, extract_id_candidate_fields, run_terragrunt_init};

