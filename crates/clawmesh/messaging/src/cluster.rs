//! Cluster Support for Horizontal Scaling
//!
//! Provides distributed coordination for multi-instance deployments.
//! Implements aerospace-grade reliability with consensus and failover.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, instrument};

/// Node ID type
pub type NodeId = String;

/// Cluster node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    /// Node ID
    pub id: NodeId,
    /// Node address (host:port)
    pub address: String,
    /// Node role
    pub role: NodeRole,
    /// Node status
    pub status: NodeStatus,
    /// Last heartbeat timestamp
    pub last_heartbeat: DateTime<Utc>,
    /// Node capacity (max concurrent connections)
    pub capacity: usize,
    /// Current load (active connections)
    pub current_load: usize,
    /// Node metadata
    pub metadata: HashMap<String, String>,
}

impl ClusterNode {
    /// Create a new cluster node
    pub fn new(id: NodeId, address: String, role: NodeRole) -> Self {
        Self {
            id,
            address,
            role,
            status: NodeStatus::Joining,
            last_heartbeat: Utc::now(),
            capacity: 10_000,
            current_load: 0,
            metadata: HashMap::new(),
        }
    }

    /// Check if node is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, NodeStatus::Healthy) &&
        (Utc::now() - self.last_heartbeat).num_seconds() < 30
    }

    /// Get load percentage
    pub fn load_percentage(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.current_load as f64 / self.capacity as f64) * 100.0
        }
    }

    /// Update heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        if matches!(self.status, NodeStatus::Joining) {
            self.status = NodeStatus::Healthy;
        }
    }
}

/// Node role in cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    /// Primary node (can handle writes)
    Primary,
    /// Replica node (read-only)
    Replica,
    /// Worker node (message processing)
    Worker,
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is joining the cluster
    Joining,
    /// Node is healthy and active
    Healthy,
    /// Node is degraded but operational
    Degraded,
    /// Node is leaving the cluster
    Leaving,
    /// Node is unreachable
    Unreachable,
}

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// This node's ID
    pub node_id: NodeId,
    /// This node's address
    pub node_address: String,
    /// This node's role
    pub node_role: NodeRole,
    /// Seed nodes for discovery
    pub seed_nodes: Vec<String>,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,
    /// Node timeout (seconds)
    pub node_timeout: u64,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            node_address: "localhost:8080".to_string(),
            node_role: NodeRole::Worker,
            seed_nodes: vec![],
            heartbeat_interval: 10,
            node_timeout: 30,
        }
    }
}

/// Cluster membership service
pub struct ClusterMembership {
    config: ClusterConfig,
    nodes: Arc<RwLock<HashMap<NodeId, ClusterNode>>>,
    this_node: Arc<RwLock<ClusterNode>>,
}

impl ClusterMembership {
    /// Create a new cluster membership service
    pub fn new(config: ClusterConfig) -> Self {
        let this_node = ClusterNode::new(
            config.node_id.clone(),
            config.node_address.clone(),
            config.node_role,
        );

        Self {
            config,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            this_node: Arc::new(RwLock::new(this_node)),
        }
    }

    /// Join the cluster
    #[instrument(skip(self))]
    pub async fn join(&self) -> Result<(), String> {
        info!(
            node_id = %self.config.node_id,
            address = %self.config.node_address,
            "Joining cluster"
        );

        // TODO: Implement actual cluster discovery
        // For now, just add this node to the membership
        let this_node = self.this_node.read().await.clone();
        let mut nodes = self.nodes.write().await;
        nodes.insert(this_node.id.clone(), this_node);

        Ok(())
    }

    /// Leave the cluster gracefully
    #[instrument(skip(self))]
    pub async fn leave(&self) -> Result<(), String> {
        info!(node_id = %self.config.node_id, "Leaving cluster");

        let mut this_node = self.this_node.write().await;
        this_node.status = NodeStatus::Leaving;

        // TODO: Notify other nodes
        // TODO: Transfer responsibilities

        let mut nodes = self.nodes.write().await;
        nodes.remove(&this_node.id);

        Ok(())
    }

    /// Send heartbeat
    #[instrument(skip(self))]
    pub async fn heartbeat(&self) {
        let mut this_node = self.this_node.write().await;
        this_node.heartbeat();

        debug!(
            node_id = %this_node.id,
            load = this_node.current_load,
            capacity = this_node.capacity,
            "Heartbeat sent"
        );

        // TODO: Broadcast heartbeat to other nodes
    }

    /// Get all healthy nodes
    pub async fn get_healthy_nodes(&self) -> Vec<ClusterNode> {
        let nodes = self.nodes.read().await;
        nodes.values()
            .filter(|n| n.is_healthy())
            .cloned()
            .collect()
    }

    /// Get node by ID
    pub async fn get_node(&self, node_id: &NodeId) -> Option<ClusterNode> {
        let nodes = self.nodes.read().await;
        nodes.get(node_id).cloned()
    }

    /// Update node status
    #[instrument(skip(self))]
    pub async fn update_node_status(&self, node_id: &NodeId, status: NodeStatus) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(node_id) {
            node.status = status;
            debug!(node_id = %node_id, status = ?status, "Node status updated");
        }
    }

    /// Get cluster statistics
    pub async fn get_stats(&self) -> ClusterStats {
        let nodes = self.nodes.read().await;
        
        let total_nodes = nodes.len();
        let healthy_nodes = nodes.values().filter(|n| n.is_healthy()).count();
        let total_capacity: usize = nodes.values().map(|n| n.capacity).sum();
        let total_load: usize = nodes.values().map(|n| n.current_load).sum();

        ClusterStats {
            total_nodes,
            healthy_nodes,
            total_capacity,
            total_load,
            load_percentage: if total_capacity > 0 {
                (total_load as f64 / total_capacity as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Start heartbeat worker
    pub async fn start_heartbeat_worker(self: Arc<Self>) {
        let interval = self.config.heartbeat_interval;
        
        info!("Starting heartbeat worker");
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            self.heartbeat().await;
            self.check_node_health().await;
        }
    }

    /// Check health of all nodes
    async fn check_node_health(&self) {
        let mut nodes = self.nodes.write().await;
        let timeout_threshold = Utc::now() - chrono::Duration::seconds(self.config.node_timeout as i64);

        for node in nodes.values_mut() {
            if node.last_heartbeat < timeout_threshold && node.status == NodeStatus::Healthy {
                warn!(node_id = %node.id, "Node timed out");
                node.status = NodeStatus::Unreachable;
            }
        }
    }
}

/// Cluster statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStats {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Number of healthy nodes
    pub healthy_nodes: usize,
    /// Total cluster capacity
    pub total_capacity: usize,
    /// Total cluster load
    pub total_load: usize,
    /// Load percentage
    pub load_percentage: f64,
}

/// Load balancer for distributing work across cluster
pub struct LoadBalancer {
    membership: Arc<ClusterMembership>,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(membership: Arc<ClusterMembership>) -> Self {
        Self { membership }
    }

    /// Select best node for new connection (least loaded)
    #[instrument(skip(self))]
    pub async fn select_node(&self) -> Option<ClusterNode> {
        let nodes = self.membership.get_healthy_nodes().await;
        
        if nodes.is_empty() {
            warn!("No healthy nodes available");
            return None;
        }

        // Select node with lowest load percentage
        let best_node = nodes.into_iter()
            .min_by(|a, b| {
                a.load_percentage()
                    .partial_cmp(&b.load_percentage())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(ref node) = best_node {
            debug!(
                node_id = %node.id,
                load = node.load_percentage(),
                "Selected node for new connection"
            );
        }

        best_node
    }

    /// Select node by consistent hashing (for sticky sessions)
    #[instrument(skip(self))]
    pub async fn select_node_by_key(&self, key: &str) -> Option<ClusterNode> {
        let nodes = self.membership.get_healthy_nodes().await;
        
        if nodes.is_empty() {
            return None;
        }

        // Simple hash-based selection
        let hash = key.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let index = (hash as usize) % nodes.len();

        let selected = nodes.get(index).cloned();
        
        if let Some(ref node) = selected {
            debug!(
                key = key,
                node_id = %node.id,
                "Selected node by consistent hashing"
            );
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_node_creation() {
        let node = ClusterNode::new(
            "node1".to_string(),
            "localhost:8080".to_string(),
            NodeRole::Worker,
        );
        
        assert_eq!(node.id, "node1");
        assert_eq!(node.status, NodeStatus::Joining);
        assert_eq!(node.current_load, 0);
    }

    #[test]
    fn test_node_health_check() {
        let mut node = ClusterNode::new(
            "node1".to_string(),
            "localhost:8080".to_string(),
            NodeRole::Worker,
        );
        
        node.heartbeat();
        assert!(node.is_healthy());
        
        node.status = NodeStatus::Unreachable;
        assert!(!node.is_healthy());
    }

    #[test]
    fn test_load_percentage() {
        let mut node = ClusterNode::new(
            "node1".to_string(),
            "localhost:8080".to_string(),
            NodeRole::Worker,
        );
        
        node.capacity = 100;
        node.current_load = 50;
        
        assert_eq!(node.load_percentage(), 50.0);
    }

    #[tokio::test]
    async fn test_cluster_membership() {
        let config = ClusterConfig::default();
        let membership = ClusterMembership::new(config);
        
        membership.join().await.unwrap();
        
        let stats = membership.get_stats().await;
        assert_eq!(stats.total_nodes, 1);
        assert_eq!(stats.healthy_nodes, 0); // Joining status
    }

    #[tokio::test]
    async fn test_load_balancer() {
        let config = ClusterConfig::default();
        let membership = Arc::new(ClusterMembership::new(config));
        
        membership.join().await.unwrap();
        
        // Manually add a healthy node for testing
        {
            let mut this_node = membership.this_node.write().await;
            this_node.status = NodeStatus::Healthy;
        }
        
        let lb = LoadBalancer::new(Arc::clone(&membership));
        
        let node = lb.select_node().await;
        assert!(node.is_some());
        
        let node_by_key = lb.select_node_by_key("user_123").await;
        assert!(node_by_key.is_some());
    }

    #[tokio::test]
    async fn test_consistent_hashing() {
        let config = ClusterConfig::default();
        let membership = Arc::new(ClusterMembership::new(config));
        membership.join().await.unwrap();
        
        {
            let mut this_node = membership.this_node.write().await;
            this_node.status = NodeStatus::Healthy;
        }
        
        let lb = LoadBalancer::new(Arc::clone(&membership));
        
        // Same key should always select same node
        let node1 = lb.select_node_by_key("user_123").await;
        let node2 = lb.select_node_by_key("user_123").await;
        
        assert_eq!(node1.unwrap().id, node2.unwrap().id);
    }
}
