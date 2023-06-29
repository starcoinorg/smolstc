use anyhow::Result;
use futures::FutureExt;
use futures_core::future::BoxFuture;
use network_p2p_derive::net_rpc;
use network_p2p_types::peer_id::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MyReqeust {
    number: i32,
    name: String,
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

#[net_rpc(client, server)]
pub trait NetworkDagRpc: Sized + Send + Sync + 'static {
    fn send_request(&self, peer_id: PeerId, request: MyReqeust) -> BoxFuture<Result<MyResponse>>;
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
}
