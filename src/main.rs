use futures::FutureExt;
use futures_core::future::BoxFuture;
use network_p2p_core::{server::NetworkRpcServer, InmemoryRpcClient};
use serde::{Deserialize, Serialize};
use network_p2p_derive::net_rpc; 
use network_p2p_types::peer_id::PeerId;
use anyhow::Result;

use crate::{gen_server::NetworkRpc, gen_client::NetworkRpcClient};

// const PROTOCOL_NAME_CHAIN: &str = "/starcoin/chain/1";
// const PROTOCOL_NAME_NOTIFY: &str = "/starcoin/notify/1";
// const PROTOCOL_NAME_REQUEST_RESPONSE: &str = "/starcoin/reques_response/1";

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
pub trait NetworkRpc: Sized + Send + Sync + 'static {
    fn send_request(&self,  peer_id: PeerId, request: MyReqeust) -> BoxFuture<Result<MyResponse>>;
}

#[derive(Default)]
#[allow(clippy::upper_case_acronyms)]
struct NetworkRpcImpl;
impl gen_server::NetworkRpc for NetworkRpcImpl {
    fn send_request(&self, peer_id: PeerId, request: MyReqeust) -> BoxFuture<Result<MyResponse> >  {
        println!("peer id = {peer_id:?}, request = {request:?}");
        futures::future::ready(Ok(MyResponse {
            number: request.number * 2,
            name: request.name + " from response",
        })).boxed()
    }
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let rpc_impl = NetworkRpcImpl::default();
        let rpc_server = NetworkRpcServer::new(rpc_impl.to_delegate());
    
        let rpc_client = NetworkRpcClient::new(InmemoryRpcClient::new(PeerId::random(), rpc_server));
        let request = MyReqeust {
            number: 1001,
            name: "jack".to_string(),
        };
        let result = rpc_client.send_request(PeerId::random(), request.clone()).await;
        println!("result from server: {result:?}, name");
    })
}
