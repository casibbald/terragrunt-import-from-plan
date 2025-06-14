//! # Import Progress and Statistics Reporting Module
//! 
//! This module provides comprehensive reporting functionality for terragrunt import operations.
//! It tracks statistics, displays progress information, and provides summary reports for
//! both normal and dry-run execution modes.
//! 
//! ## Key Components
//! 
//! - **ImportStats**: Tracks detailed statistics about import operations
//! - **ImportOperation**: Represents different types of import operations
//! - **Progress Reporting**: Real-time progress updates during import execution
//! - **Summary Reporting**: Final summaries with detailed statistics
//! 
//! ## Usage Patterns
//! 
//! 1. Create an ImportStats instance to track operation results
//! 2. Use print_import_progress() for real-time progress updates
//! 3. Update statistics as operations complete
//! 4. Print final summary using print_import_summary() or print_dry_run_summary()

/// Import statistics for tracking the results of import operations
/// 
/// This structure maintains comprehensive statistics about import operations including
/// counts of successful imports, skipped resources, failures, and detailed tracking
/// of which resources were actually imported.
/// 
/// # Fields
/// - `imported`: Number of resources successfully imported
/// - `already_in_state`: Number of resources already in terraform state
/// - `skipped`: Number of resources skipped (e.g., couldn't infer ID)
/// - `failed`: Number of resources that failed to import
/// - `imported_resources`: Detailed list of successfully imported resource addresses
#[derive(Debug, Default, Clone)]
pub struct ImportStats {
    /// Count of resources successfully imported
    pub imported: usize,
    /// Count of resources already present in terraform state
    pub already_in_state: usize,
    /// Count of resources skipped during import process
    pub skipped: usize,
    /// Count of resources that failed to import
    pub failed: usize,
    /// Detailed list of successfully imported resource addresses
    pub imported_resources: Vec<String>,
}

impl ImportStats {
    /// Creates a new ImportStats instance with all counters initialized to zero
    /// 
    /// # Returns
    /// A new ImportStats instance ready for tracking import operations
    /// 
    /// # Examples
    /// ```
    /// use terragrunt_import_from_plan::reporting::ImportStats;
    /// 
    /// let mut stats = ImportStats::new();
    /// stats.increment_imported("aws_vpc.main".to_string());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments the imported counter and adds the resource to the imported list
    /// 
    /// This method should be called when a resource is successfully imported into
    /// terraform state. It both increments the counter and maintains a detailed
    /// list of imported resources for reporting.
    /// 
    /// # Arguments
    /// * `resource_address` - Full terraform address of the imported resource
    /// 
    /// # Examples
    /// ```
    /// use terragrunt_import_from_plan::reporting::ImportStats;
    /// 
    /// let mut stats = ImportStats::new();
    /// stats.increment_imported("module.vpc.aws_vpc.main".to_string());
    /// assert_eq!(stats.imported, 1);
    /// ```
    pub fn increment_imported(&mut self, resource_address: String) {
        self.imported += 1;
        self.imported_resources.push(resource_address);
    }

    /// Increments the skipped counter
    /// 
    /// This method should be called when a resource is skipped during the import
    /// process, typically because an ID could not be inferred or other prerequisites
    /// were not met.
    /// 
    /// # Examples
    /// ```
    /// use terragrunt_import_from_plan::reporting::ImportStats;
    /// 
    /// let mut stats = ImportStats::new();
    /// stats.increment_skipped();
    /// assert_eq!(stats.skipped, 1);
    /// ```
    pub fn increment_skipped(&mut self) {
        self.skipped += 1;
    }

    /// Increments the failed counter
    /// 
    /// This method should be called when a resource import attempt fails due to
    /// errors during the terragrunt import command execution.
    /// 
    /// # Examples
    /// ```
    /// use terragrunt_import_from_plan::reporting::ImportStats;
    /// 
    /// let mut stats = ImportStats::new();
    /// stats.increment_failed();
    /// assert_eq!(stats.failed, 1);
    /// ```
    pub fn increment_failed(&mut self) {
        self.failed += 1;
    }

    /// Increments the already-in-state counter
    /// 
    /// This method should be called when a resource is discovered to already exist
    /// in the terraform state and does not need to be imported.
    /// 
    /// # Examples
    /// ```
    /// use terragrunt_import_from_plan::reporting::ImportStats;
    /// 
    /// let mut stats = ImportStats::new();
    /// stats.increment_already_in_state();
    /// assert_eq!(stats.already_in_state, 1);
    /// ```
    pub fn increment_already_in_state(&mut self) {
        self.already_in_state += 1;
    }

    /// Calculates the total number of resources processed across all categories
    /// 
    /// This provides a convenient way to get the total number of resources that
    /// were examined during the import process, regardless of their final status.
    /// 
    /// # Returns
    /// Sum of imported, already_in_state, skipped, and failed counts
    /// 
    /// # Examples
    /// ```
    /// use terragrunt_import_from_plan::reporting::ImportStats;
    /// 
    /// let mut stats = ImportStats::new();
    /// stats.increment_imported("resource1".to_string());
    /// stats.increment_skipped();
    /// assert_eq!(stats.total_processed(), 2);
    /// ```
    pub fn total_processed(&self) -> usize {
        self.imported + self.already_in_state + self.skipped + self.failed
    }
}

/// Prints a comprehensive import summary with detailed statistics
/// 
/// This function displays a complete summary of import operations including counts
/// for each category and a detailed list of successfully imported resources. It's
/// typically called at the end of an import session to provide final results.
/// 
/// # Arguments
/// * `stats` - ImportStats instance containing the operation results
/// 
/// # Output Format
/// - Import counts by category (imported, already in state, skipped, failed)
/// - Detailed list of imported resource addresses (if any)
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::reporting::{ImportStats, print_import_summary};
/// 
/// let mut stats = ImportStats::new();
/// stats.increment_imported("aws_vpc.main".to_string());
/// print_import_summary(&stats);
/// ```
pub fn print_import_summary(stats: &ImportStats) {
    println!(
        "\n✅ Import Summary\nImported:   {}\nAlready in state: {}\nSkipped:     {}\nFailed:      {}",
        stats.imported, stats.already_in_state, stats.skipped, stats.failed
    );

    if !stats.imported_resources.is_empty() {
        println!(" 📦 Imported Resources:");
        for resource in &stats.imported_resources {
            println!("{}", resource);
        }
    }
}

/// Prints a compact import summary for dry-run mode
/// 
/// This function displays a simplified summary appropriate for dry-run operations
/// where no actual imports are performed. It focuses on what would be imported
/// versus what would be skipped.
/// 
/// # Arguments
/// * `stats` - ImportStats instance from dry-run execution
/// 
/// # Output Format
/// - Count of resources that would be imported
/// - Count of resources that would be skipped
/// - List of resources that would be imported (if any)
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::reporting::{ImportStats, print_dry_run_summary};
/// 
/// let mut stats = ImportStats::new();
/// stats.increment_imported("aws_vpc.main".to_string()); // In dry-run, this represents "would import"
/// print_dry_run_summary(&stats);
/// ```
pub fn print_dry_run_summary(stats: &ImportStats) {
    println!(
        "\n🌿 Dry Run Summary\nWould import: {}\nWould skip:   {}",
        stats.imported, stats.skipped
    );

    if !stats.imported_resources.is_empty() {
        println!(" 📋 Resources to import:");
        for resource in &stats.imported_resources {
            println!("{}", resource);
        }
    }
}

/// Prints detailed import progress during execution
/// 
/// This function provides real-time progress updates as import operations are
/// performed. It displays different messages based on the type of operation
/// being performed, using appropriate emojis and formatting for clarity.
/// 
/// # Arguments
/// * `resource_address` - Full terraform address of the resource being processed
/// * `operation` - The specific import operation being performed
/// 
/// # Progress Messages
/// - 🔍 Checking: Resource is being analyzed
/// - 📦 Importing: Resource import is being attempted
/// - ✅ Success: Resource was successfully imported
/// - ⚠️ Skipped: Resource was skipped with reason
/// - ❌ Failed: Resource import failed with error
/// - 🌿 Dry Run: Shows command that would be executed
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::reporting::{print_import_progress, ImportOperation};
/// 
/// print_import_progress("aws_vpc.main", ImportOperation::Checking);
/// print_import_progress("aws_vpc.main", ImportOperation::Importing { 
///     id: "vpc-12345".to_string() 
/// });
/// ```
pub fn print_import_progress(resource_address: &str, operation: ImportOperation) {
    match operation {
        ImportOperation::Checking => {
            println!("🔍 Checking {}...", resource_address);
        }
        ImportOperation::Importing { id } => {
            println!(" 📦 Importing {} with ID: {}", resource_address, id);
        }
        ImportOperation::Success => {
            println!("✅ Imported {}", resource_address);
        }
        ImportOperation::Skipped { reason } => {
            println!("⚠️ Skipped {}: {}", resource_address, reason);
        }
        ImportOperation::Failed { error } => {
            eprintln!("❌ Error importing {}: {}", resource_address, error);
        }
        ImportOperation::DryRun { command } => {
            println!("🌿 [DRY RUN] {}", command);
        }
    }
}

/// Represents different import operations for progress reporting
/// 
/// This enum defines the various states and operations that can occur during
/// the import process, each with associated data for detailed reporting.
/// 
/// # Variants
/// - `Checking`: Resource is being analyzed for import eligibility
/// - `Importing`: Resource import is being attempted with specific ID
/// - `Success`: Resource was successfully imported
/// - `Skipped`: Resource was skipped with a reason
/// - `Failed`: Resource import failed with error details
/// - `DryRun`: Dry-run mode showing the command that would be executed
pub enum ImportOperation {
    /// Resource is being analyzed for import eligibility
    Checking,
    /// Resource import is being attempted with the specified ID
    Importing { 
        /// Cloud resource ID being used for the import
        id: String 
    },
    /// Resource was successfully imported
    Success,
    /// Resource was skipped during import
    Skipped { 
        /// Human-readable reason why the resource was skipped
        reason: String 
    },
    /// Resource import failed
    Failed { 
        /// Error message describing why the import failed
        error: String 
    },
    /// Dry-run mode showing command without execution
    DryRun { 
        /// Full terragrunt import command that would be executed
        command: String 
    },
}

/// Unit tests for the reporting functionality
/// 
/// These tests verify the behavior of statistics tracking, progress reporting,
/// and summary generation functions.
#[cfg(test)]
mod tests {
    use super::*;

    /// **TEST** - Verifies ImportStats creation with default values
    #[test]
    fn test_import_stats_creation() {
        let stats = ImportStats::new();
        assert_eq!(stats.imported, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.already_in_state, 0);
        assert!(stats.imported_resources.is_empty());
    }

    /// **TEST** - Verifies imported counter and resource list tracking
    #[test]
    fn test_import_stats_increment_imported() {
        let mut stats = ImportStats::new();
        stats.increment_imported("test.resource1".to_string());
        stats.increment_imported("test.resource2".to_string());
        
        assert_eq!(stats.imported, 2);
        assert_eq!(stats.imported_resources.len(), 2);
        assert_eq!(stats.imported_resources[0], "test.resource1");
        assert_eq!(stats.imported_resources[1], "test.resource2");
    }

    /// **TEST** - Verifies all counter increment functions
    #[test]
    fn test_import_stats_increment_counters() {
        let mut stats = ImportStats::new();
        stats.increment_skipped();
        stats.increment_skipped();
        stats.increment_failed();
        stats.increment_already_in_state();
        
        assert_eq!(stats.skipped, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.already_in_state, 1);
    }

    /// **TEST** - Verifies total processed calculation
    #[test]
    fn test_import_stats_total_processed() {
        let mut stats = ImportStats::new();
        stats.increment_imported("test.resource".to_string());
        stats.increment_skipped();
        stats.increment_failed();
        stats.increment_already_in_state();
        
        assert_eq!(stats.total_processed(), 4);
    }

    /// **TEST** - Verifies print_import_summary doesn't panic
    #[test]
    fn test_print_import_summary() {
        let mut stats = ImportStats::new();
        stats.increment_imported("test.resource1".to_string());
        stats.increment_imported("test.resource2".to_string());
        stats.increment_skipped();
        
        // This test just verifies the function doesn't panic
        // In a real scenario, you might want to capture stdout for testing
        print_import_summary(&stats);
    }

    /// **TEST** - Verifies print_dry_run_summary doesn't panic
    #[test]
    fn test_print_dry_run_summary() {
        let mut stats = ImportStats::new();
        stats.increment_imported("test.resource".to_string());
        stats.increment_skipped();
        
        // This test just verifies the function doesn't panic
        print_dry_run_summary(&stats);
    }

    /// **TEST** - Verifies print_import_progress handles all operation types
    #[test]
    fn test_print_import_progress() {
        // Test various operations
        print_import_progress("test.resource", ImportOperation::Checking);
        print_import_progress("test.resource", ImportOperation::Importing { id: "test-id".to_string() });
        print_import_progress("test.resource", ImportOperation::Success);
        print_import_progress("test.resource", ImportOperation::Skipped { reason: "no ID found".to_string() });
        print_import_progress("test.resource", ImportOperation::Failed { error: "network error".to_string() });
        print_import_progress("test.resource", ImportOperation::DryRun { command: "terragrunt import test.resource test-id".to_string() });
    }
} 