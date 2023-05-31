mod network_dag_service;
mod network_dag_handle;
mod network_dag_trait;
mod network_dag_worker;
mod network_dag_data;
mod network_dag_rpc_service;
mod network_dag_rpc;

use network_dag_rpc_service::NetworkDagRpcService;
use network_dag_service::{NetworkDagService, NetworkDagServiceFactory};
use starcoin_service_registry::{RegistryService, RegistryAsyncService};

fn main() {
    let system = actix::prelude::System::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {

        let registry = RegistryService::launch(); 
        registry.register::<NetworkDagRpcService>().await.unwrap();
        registry.register_by_factory::<NetworkDagService, NetworkDagServiceFactory>().await.unwrap();

    });
    system.run().unwrap();
}
