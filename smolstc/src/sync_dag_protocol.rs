use std::hash::Hash;

use network_p2p_core::{NetRpcError, RpcErrorCode};
use serde::{Serialize, Deserialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use anyhow::Result;
use crate::network_dag_rpc::RpcRequest;


pub const MAX_BLOCK_IDS_REQUEST_SIZE: u64 = 10000;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetBlockIds {
  pub block_hash: Vec<HashValue>,
  pub reverse: bool,
  pub depth: u64,
}

/// the returned block hash values must be in order 
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SyncBlockIds {
    pub block_hash: HashValue,
    pub parents: Vec<HashValue>,
}

impl RpcRequest for GetBlockIds {
  fn verify(&self) -> Result<()> {
      if self.depth > MAX_BLOCK_IDS_REQUEST_SIZE {
          return Err(NetRpcError::new(
              RpcErrorCode::BadRequest,
              format!("max_size is too big > {}", MAX_BLOCK_IDS_REQUEST_SIZE),
          )
          .into());
      }
      Ok(())
  }
}
 