use anyhow::Result;
use futures::FutureExt;
use futures_core::future::BoxFuture;
use network_p2p_derive::net_rpc;
use network_p2p_types::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

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

#[net_rpc(client, server)]
pub trait NetworkDagRpc: Sized + Send + Sync + 'static {
    fn send_request(&self, peer_id: PeerId, request: MyReqeust) -> BoxFuture<Result<MyResponse>>;
    fn get_accumulator_leaves(
        &self,
        peer_id: PeerId,
        req: GetAccumulatorLeaves,
    ) -> BoxFuture<Result<Vec<HashValue>>>;
}

#[derive(Default)]
#[allow(clippy::upper_case_acronyms)]
pub struct NetworkDagRpcImpl;
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
        peer_id: PeerId,
        req: GetAccumulatorLeaves,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        todo!()
    }
}
