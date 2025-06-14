# AKS Cluster
resource "azurerm_kubernetes_cluster" "example" {
  name                = var.cluster_name
  location            = var.location
  resource_group_name = var.resource_group_name
  dns_prefix          = var.dns_prefix
  kubernetes_version  = var.kubernetes_version

  # Default node pool
  default_node_pool {
    name                = "default"
    node_count          = var.node_count
    vm_size             = var.vm_size
    os_disk_size_gb     = var.os_disk_size_gb
    vnet_subnet_id      = var.subnet_id
    enable_auto_scaling = var.enable_auto_scaling
    min_count           = var.enable_auto_scaling ? var.min_node_count : null
    max_count           = var.enable_auto_scaling ? var.max_node_count : null
    max_pods            = var.max_pods
    zones               = var.availability_zones

    # Node labels and taints
    node_labels = var.node_labels
    node_taints = var.node_taints

    upgrade_settings {
      max_surge = "10%"
    }
  }

  # Identity
  identity {
    type = "SystemAssigned"
  }

  # Network profile
  network_profile {
    network_plugin     = var.network_plugin
    network_policy     = var.network_policy
    dns_service_ip     = var.dns_service_ip
    docker_bridge_cidr = var.docker_bridge_cidr
    service_cidr       = var.service_cidr
    load_balancer_sku  = "standard"
  }

  # API server access profile
  api_server_access_profile {
    authorized_ip_ranges = var.authorized_ip_ranges
  }

  # Role-based access control
  role_based_access_control_enabled = true

  # Azure Active Directory integration
  azure_active_directory_role_based_access_control {
    managed                = true
    admin_group_object_ids = var.admin_group_object_ids
    azure_rbac_enabled     = var.azure_rbac_enabled
  }

  # Auto-scaler profile
  auto_scaler_profile {
    balance_similar_node_groups      = false
    expander                         = "random"
    max_graceful_termination_sec     = "600"
    max_node_provisioning_time       = "15m"
    max_unready_nodes                = 3
    max_unready_percentage           = 45
    new_pod_scale_up_delay           = "10s"
    scale_down_delay_after_add       = "10m"
    scale_down_delay_after_delete    = "10s"
    scale_down_delay_after_failure   = "3m"
    scan_interval                    = "10s"
    scale_down_utilization_threshold = "0.5"
    empty_bulk_delete_max            = "10"
    skip_nodes_with_local_storage    = true
    skip_nodes_with_system_pods      = true
  }

  # Azure Monitor for containers
  oms_agent {
    log_analytics_workspace_id = var.log_analytics_workspace_id
  }

  # Maintenance window
  maintenance_window {
    allowed {
      day   = "Sunday"
      hours = [1, 2]
    }
  }

  tags = merge(var.tags, {
    Name        = var.cluster_name
    Environment = "development"
  })
}

# Additional Node Pool (optional)
resource "azurerm_kubernetes_cluster_node_pool" "additional" {
  count                 = var.enable_additional_node_pool ? 1 : 0
  name                  = var.additional_node_pool_name
  kubernetes_cluster_id = azurerm_kubernetes_cluster.example.id
  vm_size               = var.additional_vm_size
  node_count            = var.additional_node_count
  enable_auto_scaling   = var.additional_enable_auto_scaling
  min_count             = var.additional_enable_auto_scaling ? var.additional_min_node_count : null
  max_count             = var.additional_enable_auto_scaling ? var.additional_max_node_count : null
  max_pods              = var.additional_max_pods
  zones                 = var.availability_zones
  vnet_subnet_id        = var.subnet_id

  # Node labels and taints for additional pool
  node_labels = var.additional_node_labels
  node_taints = var.additional_node_taints

  upgrade_settings {
    max_surge = "10%"
  }

  tags = merge(var.tags, {
    Name        = var.additional_node_pool_name
    Environment = "development"
    Type        = "additional"
  })
}

# Role assignment for AKS to access the subnet
resource "azurerm_role_assignment" "aks_subnet" {
  count                = var.subnet_id != null ? 1 : 0
  scope                = var.subnet_id
  role_definition_name = "Network Contributor"
  principal_id         = azurerm_kubernetes_cluster.example.identity[0].principal_id
}

# Role assignment for AKS to access container registry
resource "azurerm_role_assignment" "aks_acr" {
  count                = var.container_registry_id != null ? 1 : 0
  scope                = var.container_registry_id
  role_definition_name = "AcrPull"
  principal_id         = azurerm_kubernetes_cluster.example.kubelet_identity[0].object_id
} 