use anyhow::Ok;
use starcoin_service_registry::{ActorService, ServiceFactory};

use crate::network_dag_verified_client::{VerifiedDagRpcClient, NetworkDagServiceRef};


pub struct SyncDagService {
  client: VerifiedDagRpcClient,
}

impl SyncDagService {
    pub fn new(client: VerifiedDagRpcClient) -> Self {
        SyncDagService { 
            client  
        }
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