use std::{borrow::Cow, sync::Arc};

use anyhow::Result;
use futures::FutureExt;
use network_p2p_core::{PeerId, RawRpcClient};
use network_p2p_types::IfDisconnected;
use rand::Error;

use crate::network_dag_rpc::{gen_client::NetworkRpcClient, MyReqeust, MyResponse};

#[derive(Clone)]
pub struct NetworkDagServiceRef {
    network_service: Arc<network_p2p::NetworkService>,
}

impl NetworkDagServiceRef {
    pub fn new(network_service: Arc<network_p2p::NetworkService>) -> Self {
        NetworkDagServiceRef { network_service }
    }
}

impl RawRpcClient for NetworkDagServiceRef {
    fn send_raw_request(
        &self,
        peer_id: network_p2p_core::PeerId,
        rpc_path: std::borrow::Cow<'static, str>,
        message: Vec<u8>,
    ) -> futures_core::future::BoxFuture<anyhow::Result<Vec<u8>>> {
        async move {
            self.network_service
                .request(
                    peer_id.into(),
                    rpc_path,
                    message,
                    IfDisconnected::ImmediateError,
                )
                .await
                .map_err(|e| e.into())
        }
        .boxed()
    }
}

pub struct VerifiedDagRpcClient {
    network_service: Arc<network_p2p::NetworkService>,
    client: NetworkRpcClient,
}

impl VerifiedDagRpcClient {
    pub fn new(network_ref: NetworkDagServiceRef) -> Self {
        VerifiedDagRpcClient {
            network_service: network_ref.network_service.clone(),
            client: NetworkRpcClient::new(network_ref),
        }
    }

    pub async fn send_request(&self, peer_id: PeerId, request: MyReqeust) -> Result<MyResponse> {
        self.client
            .send_request(peer_id, request)
            .await
            .map_err(|e| e.into())
    }
    pub async fn broadcast(
        &self,
        protocol_name: Cow<'static, str>,
        message: Vec<u8>,
    ) -> Result<()> {
        self.network_service
            .broadcast_message(protocol_name, message)
            .await;
        Ok(())
    }

    pub fn add_peer(&self, peer: String) -> anyhow::Result<()> {
        println!("add peer: {}", peer);
        self.network_service
            .add_reserved_peer(peer)
            .map_err(|e| anyhow::Error::msg(e))
    }

    pub async fn is_connected(&self) -> bool {
        let result = self.network_service.known_peers().await;
        result.iter().for_each(|peer_id| {
            println!("check connection: {}", peer_id.to_string());
            loop {
                let connected =
                    async_std::task::block_on(self.network_service.is_connected(peer_id.clone()));
                if connected {
                    println!("check connection: {} is connected.", peer_id.to_string());
                    break;
                }
            }
        });
        true
    }
}
