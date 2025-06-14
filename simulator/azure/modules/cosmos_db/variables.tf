variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
}

variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
}

variable "cosmosdb_account_name" {
  description = "The name of the Cosmos DB account"
  type        = string
}

variable "kind" {
  description = "The kind of Cosmos DB account"
  type        = string
  default     = "GlobalDocumentDB"
  validation {
    condition     = contains(["GlobalDocumentDB", "MongoDB", "Parse"], var.kind)
    error_message = "Kind must be GlobalDocumentDB, MongoDB, or Parse."
  }
}

variable "enable_automatic_failover" {
  description = "Enable automatic failover"
  type        = bool
  default     = false
}

variable "enable_multiple_write_locations" {
  description = "Enable multiple write locations"
  type        = bool
  default     = false
}

variable "consistency_level" {
  description = "The consistency level"
  type        = string
  default     = "Session"
  validation {
    condition     = contains(["BoundedStaleness", "Eventual", "Session", "Strong", "ConsistentPrefix"], var.consistency_level)
    error_message = "Consistency level must be one of: BoundedStaleness, Eventual, Session, Strong, ConsistentPrefix."
  }
}

variable "max_interval_in_seconds" {
  description = "Maximum staleness interval in seconds (for BoundedStaleness)"
  type        = number
  default     = 5
}

variable "max_staleness_prefix" {
  description = "Maximum staleness prefix (for BoundedStaleness)"
  type        = number
  default     = 100
}

variable "geo_locations" {
  description = "List of geo locations for replication"
  type = list(object({
    location          = string
    failover_priority = number
    zone_redundant    = bool
  }))
  default = []
}

variable "capabilities" {
  description = "List of capabilities for the Cosmos DB account"
  type        = list(string)
  default     = []
}

variable "subnet_ids" {
  description = "List of subnet IDs for virtual network rules"
  type        = list(string)
  default     = []
}

variable "allowed_ip_ranges" {
  description = "List of allowed IP ranges"
  type        = list(string)
  default     = []
}

variable "enable_virtual_network_filter" {
  description = "Enable virtual network filter"
  type        = bool
  default     = false
}

variable "public_network_access_enabled" {
  description = "Enable public network access"
  type        = bool
  default     = true
}

variable "backup_type" {
  description = "Type of backup"
  type        = string
  default     = "Periodic"
  validation {
    condition     = contains(["Continuous", "Periodic"], var.backup_type)
    error_message = "Backup type must be Continuous or Periodic."
  }
}

variable "backup_interval_in_minutes" {
  description = "Backup interval in minutes (for Periodic backup)"
  type        = number
  default     = 240
}

variable "backup_retention_in_hours" {
  description = "Backup retention in hours (for Periodic backup)"
  type        = number
  default     = 8
}

variable "backup_storage_redundancy" {
  description = "Backup storage redundancy (for Periodic backup)"
  type        = string
  default     = "Geo"
  validation {
    condition     = contains(["Geo", "Local", "Zone"], var.backup_storage_redundancy)
    error_message = "Backup storage redundancy must be Geo, Local, or Zone."
  }
}

variable "enable_cors" {
  description = "Enable CORS rules"
  type        = bool
  default     = false
}

variable "cors_rules" {
  description = "List of CORS rules"
  type = list(object({
    allowed_headers    = list(string)
    allowed_methods    = list(string)
    allowed_origins    = list(string)
    exposed_headers    = list(string)
    max_age_in_seconds = number
  }))
  default = []
}

# SQL Database variables
variable "create_sql_database" {
  description = "Create SQL database"
  type        = bool
  default     = true
}

variable "sql_database_name" {
  description = "Name of the SQL database"
  type        = string
  default     = "example-database"
}

variable "sql_database_throughput" {
  description = "Throughput for SQL database"
  type        = number
  default     = null
}

variable "sql_database_autoscale_max_throughput" {
  description = "Autoscale max throughput for SQL database"
  type        = number
  default     = 4000
}

# SQL Container variables
variable "create_sql_container" {
  description = "Create SQL container"
  type        = bool
  default     = true
}

variable "sql_container_name" {
  description = "Name of the SQL container"
  type        = string
  default     = "example-container"
}

variable "existing_sql_database_name" {
  description = "Name of existing SQL database (if not creating new)"
  type        = string
  default     = null
}

variable "partition_key_path" {
  description = "Partition key path for SQL container"
  type        = string
  default     = "/id"
}

variable "sql_container_throughput" {
  description = "Throughput for SQL container"
  type        = number
  default     = null
}

variable "sql_container_autoscale_max_throughput" {
  description = "Autoscale max throughput for SQL container"
  type        = number
  default     = 4000
}

variable "included_paths" {
  description = "List of included paths for indexing"
  type        = list(string)
  default     = ["/*"]
}

variable "excluded_paths" {
  description = "List of excluded paths for indexing"
  type        = list(string)
  default     = []
}

variable "unique_keys" {
  description = "List of unique key paths"
  type        = list(list(string))
  default     = []
}

variable "conflict_resolution_mode" {
  description = "Conflict resolution mode"
  type        = string
  default     = "LastWriterWins"
}

variable "conflict_resolution_path" {
  description = "Conflict resolution path"
  type        = string
  default     = "/_ts"
}

# MongoDB variables
variable "create_mongo_database" {
  description = "Create MongoDB database"
  type        = bool
  default     = false
}

variable "mongo_database_name" {
  description = "Name of the MongoDB database"
  type        = string
  default     = "example-mongo-database"
}

variable "mongo_database_throughput" {
  description = "Throughput for MongoDB database"
  type        = number
  default     = null
}

variable "mongo_database_autoscale_max_throughput" {
  description = "Autoscale max throughput for MongoDB database"
  type        = number
  default     = 4000
}

variable "create_mongo_collection" {
  description = "Create MongoDB collection"
  type        = bool
  default     = false
}

variable "mongo_collection_name" {
  description = "Name of the MongoDB collection"
  type        = string
  default     = "example-collection"
}

variable "existing_mongo_database_name" {
  description = "Name of existing MongoDB database (if not creating new)"
  type        = string
  default     = null
}

variable "mongo_collection_throughput" {
  description = "Throughput for MongoDB collection"
  type        = number
  default     = null
}

variable "mongo_collection_autoscale_max_throughput" {
  description = "Autoscale max throughput for MongoDB collection"
  type        = number
  default     = 4000
}

variable "mongo_shard_key" {
  description = "Shard key for MongoDB collection"
  type        = string
  default     = null
}

variable "mongo_indexes" {
  description = "List of indexes for MongoDB collection"
  type = list(object({
    keys   = list(string)
    unique = bool
  }))
  default = []
}

# Private endpoint variables
variable "enable_private_endpoint" {
  description = "Enable private endpoint"
  type        = bool
  default     = false
}

variable "private_endpoint_subnet_id" {
  description = "Subnet ID for private endpoint"
  type        = string
  default     = null
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
} 