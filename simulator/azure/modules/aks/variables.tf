variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
}

variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
}

variable "cluster_name" {
  description = "The name of the AKS cluster"
  type        = string
}

variable "dns_prefix" {
  description = "DNS prefix for the AKS cluster"
  type        = string
}

variable "kubernetes_version" {
  description = "Version of Kubernetes to use"
  type        = string
  default     = null
}

variable "node_count" {
  description = "Number of nodes in the default node pool"
  type        = number
  default     = 3
}

variable "vm_size" {
  description = "Size of the virtual machines"
  type        = string
  default     = "Standard_D2s_v3"
}

variable "os_disk_size_gb" {
  description = "Size of the OS disk in GB"
  type        = number
  default     = 30
}

variable "subnet_id" {
  description = "ID of the subnet for the AKS cluster"
  type        = string
  default     = null
}

variable "enable_auto_scaling" {
  description = "Enable auto-scaling for the default node pool"
  type        = bool
  default     = true
}

variable "min_node_count" {
  description = "Minimum number of nodes"
  type        = number
  default     = 1
}

variable "max_node_count" {
  description = "Maximum number of nodes"
  type        = number
  default     = 5
}

variable "max_pods" {
  description = "Maximum number of pods per node"
  type        = number
  default     = 110
}

variable "availability_zones" {
  description = "List of availability zones"
  type        = list(string)
  default     = ["1", "2", "3"]
}

variable "node_labels" {
  description = "Labels to apply to nodes"
  type        = map(string)
  default     = {}
}

variable "node_taints" {
  description = "Taints to apply to nodes"
  type        = list(string)
  default     = []
}

variable "network_plugin" {
  description = "Network plugin to use"
  type        = string
  default     = "kubenet"
}

variable "network_policy" {
  description = "Network policy to use"
  type        = string
  default     = null
}

variable "dns_service_ip" {
  description = "IP address for the DNS service"
  type        = string
  default     = "10.0.0.10"
}

variable "docker_bridge_cidr" {
  description = "CIDR block for Docker bridge"
  type        = string
  default     = "172.17.0.1/16"
}

variable "service_cidr" {
  description = "CIDR block for services"
  type        = string
  default     = "10.0.0.0/16"
}

variable "authorized_ip_ranges" {
  description = "List of authorized IP ranges for API server access"
  type        = list(string)
  default     = []
}

variable "admin_group_object_ids" {
  description = "List of Azure AD group object IDs with admin access"
  type        = list(string)
  default     = []
}

variable "azure_rbac_enabled" {
  description = "Enable Azure RBAC"
  type        = bool
  default     = false
}

variable "log_analytics_workspace_id" {
  description = "ID of the Log Analytics workspace"
  type        = string
  default     = null
}

variable "enable_additional_node_pool" {
  description = "Enable additional node pool"
  type        = bool
  default     = false
}

variable "additional_node_pool_name" {
  description = "Name of the additional node pool"
  type        = string
  default     = "additional"
}

variable "additional_vm_size" {
  description = "VM size for additional node pool"
  type        = string
  default     = "Standard_D2s_v3"
}

variable "additional_node_count" {
  description = "Node count for additional node pool"
  type        = number
  default     = 2
}

variable "additional_enable_auto_scaling" {
  description = "Enable auto-scaling for additional node pool"
  type        = bool
  default     = true
}

variable "additional_min_node_count" {
  description = "Minimum nodes for additional node pool"
  type        = number
  default     = 1
}

variable "additional_max_node_count" {
  description = "Maximum nodes for additional node pool"
  type        = number
  default     = 5
}

variable "additional_max_pods" {
  description = "Maximum pods per node for additional node pool"
  type        = number
  default     = 110
}

variable "additional_node_labels" {
  description = "Labels for additional node pool"
  type        = map(string)
  default     = {}
}

variable "additional_node_taints" {
  description = "Taints for additional node pool"
  type        = list(string)
  default     = []
}

variable "container_registry_id" {
  description = "ID of the container registry"
  type        = string
  default     = null
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
} 