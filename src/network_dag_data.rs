use serde::{Deserialize, Serialize};

/// it is ChainInfo inn starcoin
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct NodeStatus {
  pub conn_number: usize,
}

/// it is ChainInfo inn starcoin
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct NodeData {
  pub name: String,
  pub status: NodeStatus,
}