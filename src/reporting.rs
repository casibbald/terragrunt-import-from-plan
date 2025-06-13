/// Import statistics for tracking the results of import operations
#[derive(Debug, Default, Clone)]
pub struct ImportStats {
    pub imported: usize,
    pub already_in_state: usize,
    pub skipped: usize,
    pub failed: usize,
    pub imported_resources: Vec<String>,
}

impl ImportStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_imported(&mut self, resource_address: String) {
        self.imported += 1;
        self.imported_resources.push(resource_address);
    }

    pub fn increment_skipped(&mut self) {
        self.skipped += 1;
    }

    pub fn increment_failed(&mut self) {
        self.failed += 1;
    }

    pub fn increment_already_in_state(&mut self) {
        self.already_in_state += 1;
    }

    pub fn total_processed(&self) -> usize {
        self.imported + self.already_in_state + self.skipped + self.failed
    }
}

/// Prints a comprehensive import summary with detailed statistics
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
pub enum ImportOperation {
    Checking,
    Importing { id: String },
    Success,
    Skipped { reason: String },
    Failed { error: String },
    DryRun { command: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_stats_creation() {
        let stats = ImportStats::new();
        assert_eq!(stats.imported, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.already_in_state, 0);
        assert!(stats.imported_resources.is_empty());
    }

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

    #[test]
    fn test_import_stats_total_processed() {
        let mut stats = ImportStats::new();
        stats.increment_imported("test.resource".to_string());
        stats.increment_skipped();
        stats.increment_failed();
        stats.increment_already_in_state();
        
        assert_eq!(stats.total_processed(), 4);
    }

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

    #[test]
    fn test_print_dry_run_summary() {
        let mut stats = ImportStats::new();
        stats.increment_imported("test.resource".to_string());
        stats.increment_skipped();
        
        // This test just verifies the function doesn't panic
        print_dry_run_summary(&stats);
    }

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