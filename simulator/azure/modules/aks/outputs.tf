output "cluster_id" {
  description = "The ID of the AKS cluster"
  value       = azurerm_kubernetes_cluster.example.id
}

output "cluster_name" {
  description = "The name of the AKS cluster"
  value       = azurerm_kubernetes_cluster.example.name
}

output "cluster_fqdn" {
  description = "The FQDN of the AKS cluster"
  value       = azurerm_kubernetes_cluster.example.fqdn
}

output "kube_config" {
  description = "Kubernetes configuration for the AKS cluster"
  value       = azurerm_kubernetes_cluster.example.kube_config
  sensitive   = true
}

output "kube_config_raw" {
  description = "Raw Kubernetes configuration"
  value       = azurerm_kubernetes_cluster.example.kube_config_raw
  sensitive   = true
}

output "cluster_identity_principal_id" {
  description = "The principal ID of the AKS cluster identity"
  value       = azurerm_kubernetes_cluster.example.identity[0].principal_id
}

output "kubelet_identity" {
  description = "The kubelet identity"
  value       = azurerm_kubernetes_cluster.example.kubelet_identity
}

output "node_resource_group" {
  description = "The auto-generated resource group which contains the resources for this managed Kubernetes cluster"
  value       = azurerm_kubernetes_cluster.example.node_resource_group
}

output "portal_fqdn" {
  description = "The FQDN for the Kubernetes Cluster when private link has been enabled"
  value       = azurerm_kubernetes_cluster.example.portal_fqdn
}

output "private_fqdn" {
  description = "The FQDN for the Kubernetes Cluster when private link has been enabled"
  value       = azurerm_kubernetes_cluster.example.private_fqdn
}

output "oms_agent_identity" {
  description = "The OMS agent identity"
  value       = azurerm_kubernetes_cluster.example.oms_agent
}

output "additional_node_pool_id" {
  description = "The ID of the additional node pool (if enabled)"
  value       = var.enable_additional_node_pool ? azurerm_kubernetes_cluster_node_pool.additional[0].id : null
} 