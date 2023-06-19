use anyhow::Ok;
use starcoin_service_registry::{ActorService, ServiceFactory, EventHandler, ServiceRequest, ServiceHandler};

use crate::network_dag_verified_client::{VerifiedDagRpcClient, NetworkDagServiceRef};

#[derive(Debug)]
pub struct SyncAddPeers {
    pub peers: Vec<String>,
}

impl ServiceRequest for SyncAddPeers {
    type Response = ();
}

pub struct SyncDagService {
  client: VerifiedDagRpcClient,
}

impl SyncDagService {
    pub fn new(client: VerifiedDagRpcClient) -> Self {
        SyncDagService { 
            client  
        }
    }

    pub async fn add_peer(&self, peer: String) -> anyhow::Result<()> {
        self.client.add_peer(peer).await
    }
}

impl ServiceFactory<SyncDagService> for SyncDagService {
    fn create(ctx: &mut starcoin_service_registry::ServiceContext<SyncDagService>) -> anyhow::Result<SyncDagService> {
        Ok(SyncDagService::new(VerifiedDagRpcClient::new(ctx.get_shared::<NetworkDagServiceRef>()?)))
    }
}

impl ActorService for SyncDagService {
    fn service_name() -> &'static str {
        std::any::type_name::<Self>()
    }

    fn started(&mut self, ctx: &mut starcoin_service_registry::ServiceContext<Self>) -> anyhow::Result<()> {
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut starcoin_service_registry::ServiceContext<Self>) -> anyhow::Result<()> {
        Ok(())
    }
}

impl ServiceHandler<Self, SyncAddPeers> for SyncDagService {
    fn handle(&mut self, msg: SyncAddPeers, ctx: &mut starcoin_service_registry::ServiceContext<Self>) -> <SyncAddPeers as ServiceRequest>::Response {
        msg.peers.into_iter().for_each(|peer| {
            self.add_peer(peer);
        });
    }
}
