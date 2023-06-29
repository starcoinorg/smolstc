use anyhow::Ok;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceFactory, ServiceHandler, ServiceRequest,
};

use crate::network_dag_verified_client::{NetworkDagServiceRef, VerifiedDagRpcClient};

#[derive(Debug)]
pub struct SyncInitVerifiedClient;

impl ServiceRequest for SyncInitVerifiedClient {
    type Response = anyhow::Result<()>;
}

#[derive(Debug)]
pub struct SyncConnectToPeers {
    pub peers: Vec<String>,
}

impl ServiceRequest for SyncConnectToPeers {
    type Response = anyhow::Result<()>;
}

pub struct SyncDagService {
    client: Option<VerifiedDagRpcClient>,
}

impl SyncDagService {
    pub fn new() -> Self {
        SyncDagService { client: None }
    }
}

impl ServiceFactory<SyncDagService> for SyncDagService {
    fn create(
        ctx: &mut starcoin_service_registry::ServiceContext<SyncDagService>,
    ) -> anyhow::Result<SyncDagService> {
        Ok(SyncDagService::new())
    }
}

impl ActorService for SyncDagService {
    fn service_name() -> &'static str {
        std::any::type_name::<Self>()
    }

    fn started(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn stopped(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

impl ServiceHandler<Self, SyncInitVerifiedClient> for SyncDagService {
    fn handle(
        &mut self,
        msg: SyncInitVerifiedClient,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <SyncInitVerifiedClient as ServiceRequest>::Response {
        if let Some(_) = self.client {
            return Ok(());
        }
        let result_client = ctx.get_shared::<NetworkDagServiceRef>();
        match result_client {
            std::result::Result::Ok(client) => {
                self.client = Some(VerifiedDagRpcClient::new(client));
                return Ok(());
            }
            Err(error) => {
                return Err(anyhow::Error::msg(error.to_string()));
            }
        }
    }
}

impl ServiceHandler<Self, SyncConnectToPeers> for SyncDagService {
    fn handle(
        &mut self,
        msg: SyncConnectToPeers,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <SyncConnectToPeers as ServiceRequest>::Response {
        match &self.client {
            Some(client) => {
                msg.peers.into_iter().for_each(|peer| {
                    client.add_peer(peer);
                });
                async_std::task::block_on(async {
                    let _ = client.is_connected().await;
                });
                return Ok(());
            }
            None => {
                return Err(anyhow::Error::msg(
                    "the verified client is None".to_string(),
                ));
            }
        }
    }
}
