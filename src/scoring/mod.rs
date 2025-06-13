pub mod strategies;
pub mod traits;

pub use strategies::{GoogleCloudScoringStrategy, AzureScoringStrategy, DefaultScoringStrategy};
pub use traits::{IdScoringStrategy, ProviderType}; 