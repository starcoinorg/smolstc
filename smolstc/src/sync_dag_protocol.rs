// use std::hash::Hash;

// use crate::network_dag_rpc::RpcRequest;
// use anyhow::Result;
// use network_p2p_core::{NetRpcError, RpcErrorCode};
// use serde::{Deserialize, Serialize};
// use starcoin_accumulator::accumulator_info::AccumulatorInfo;
// use starcoin_crypto::HashValue;

// pub const MAX_BLOCK_IDS_REQUEST_SIZE: u64 = 10000;

// #[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
// pub struct GetSyncDagAccumulatorInfo {
//     pub accumulator_info: AccumulatorInfo,
// }

// /// the returned block hash values must be in order
// #[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
// pub struct SyncBlockIds {
//     pub block_hash: HashValue,
//     pub parents: Vec<HashValue>,
// }

// impl RpcRequest for GetSyncDagAccumulatorInfo {
//     fn verify(&self) -> Result<()> {
//         if self.depth > MAX_BLOCK_IDS_REQUEST_SIZE {
//             return Err(NetRpcError::new(
//                 RpcErrorCode::BadRequest,
//                 format!("max_size is too big > {}", MAX_BLOCK_IDS_REQUEST_SIZE),
//             )
//             .into());
//         }
//         Ok(())
//     }
// }
