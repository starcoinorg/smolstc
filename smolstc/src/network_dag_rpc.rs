use anyhow::Result;
use futures::FutureExt;
use futures_core::future::BoxFuture;
use network_p2p_derive::net_rpc;
use network_p2p_types::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRef;

use crate::{
    chain_dag_service::{self, ChainDagService},
    sync_block_dag::RelationshipPair,
};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MyReqeust {
    pub number: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MyResponse {
    number: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MyNotify {
    number: i32,
    name: String,
}

pub trait RpcRequest {
    fn verify(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetAccumulatorLeaves {
    pub accumulator_leaf_index: u64,
    pub batch_size: u64,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TargetAccumulatorLeaf {
    pub accumulator_root: HashValue, // accumulator info root
    pub leaf_index: u64,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetTargetAccumulatorLeafDetail {
    pub leaf_index: u64,
    pub batch_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TargetAccumulatorLeafDetail {
    pub accumulator_root: HashValue,
    pub relationship_pair: Vec<RelationshipPair>,
}

#[net_rpc(client, server)]
pub trait NetworkDagRpc: Sized + Send + Sync + 'static {
    fn send_request(&self, peer_id: PeerId, request: MyReqeust) -> BoxFuture<Result<MyResponse>>;
    fn get_accumulator_leaves(
        &self,
        peer_id: PeerId,
        req: GetAccumulatorLeaves,
    ) -> BoxFuture<Result<Vec<TargetAccumulatorLeaf>>>;
    fn get_accumulator_leaf_detail(
        &self,
        peer_id: PeerId,
        req: GetTargetAccumulatorLeafDetail,
    ) -> BoxFuture<Result<Option<Vec<TargetAccumulatorLeafDetail>>>>;
}

pub struct NetworkDagRpcImpl {
    chain_service: ServiceRef<ChainDagService>,
}

impl NetworkDagRpcImpl {
    pub fn new(chain_service: ServiceRef<ChainDagService>) -> Self {
        NetworkDagRpcImpl { chain_service }
    }
}

impl gen_server::NetworkDagRpc for NetworkDagRpcImpl {
    fn send_request(&self, peer_id: PeerId, request: MyReqeust) -> BoxFuture<Result<MyResponse>> {
        println!("peer id = {peer_id:?}, request = {request:?}");
        futures::future::ready(Ok(MyResponse {
            number: request.number * 2,
            name: request.name + " from response",
        }))
        .boxed()
    }

    fn get_accumulator_leaves(
        &self,
        _peer_id: PeerId,
        req: GetAccumulatorLeaves,
    ) -> BoxFuture<Result<Vec<TargetAccumulatorLeaf>>> {
        self.chain_service
            .send(chain_dag_service::GetAccumulatorLeaves {
                start_index: req.accumulator_leaf_index,
                batch_size: req.batch_size,
            })
            .boxed()
    }

    fn get_accumulator_leaf_detail(
        &self,
        peer_id: PeerId,
        req: GetTargetAccumulatorLeafDetail,
    ) -> BoxFuture<Result<Option<Vec<TargetAccumulatorLeafDetail>>>> {
        self.chain_service
            .send(chain_dag_service::GetDagAccumulatorLeafDetails {
                start_index: req.leaf_index,
                batch_size: req.batch_size,
            })
            .boxed()
    }
}
