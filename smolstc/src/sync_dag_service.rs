use std::sync::Arc;

use crate::{
    find_ancestor_task::{AncestorCollector, FindAncestorTask},
    network_dag_service::{GetBestChainInfo, NetworkDagService},
    network_dag_verified_client::{NetworkDagServiceRef, VerifiedDagRpcClient},
    sync_task_error_handle::ExtSyncTaskErrorHandle, chain_dag_service::{ChainDagService, GetAccumulatorInfo},
};
use anyhow::Ok;
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator, node::AccumulatorStoreType};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceFactory, ServiceHandler, ServiceRef, ServiceRequest,
};
use starcoin_storage::{Storage, SyncFlexiDagStore, Store};
use stream_task::{Generator, TaskEventCounterHandle, TaskGenerator};

#[derive(Debug)]
pub struct SyncInitVerifiedClient;

impl ServiceRequest for SyncInitVerifiedClient {
    type Response = anyhow::Result<()>;
}

#[derive(Debug)]
pub struct CheckSync;
impl ServiceRequest for CheckSync {
    type Response = anyhow::Result<()>;
}

#[derive(Debug)]
pub struct SyncConnectToPeers {
    pub peers: Vec<String>,
}

impl ServiceRequest for SyncConnectToPeers {
    type Response = anyhow::Result<()>;
}

struct StartupInfo {
    accumulator_info: AccumulatorInfo,
}

pub struct SyncDagService {
    client: Option<Arc<VerifiedDagRpcClient>>,
    // start info should be in main, here is for test
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
                self.client = Some(Arc::new(VerifiedDagRpcClient::new(client)));
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

impl ServiceHandler<Self, CheckSync> for SyncDagService {
    fn handle(
        &mut self,
        msg: CheckSync,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <CheckSync as ServiceRequest>::Response {
        /// for debug, I use genesis for start.
        /// in practice, it should be the one stored in the startup structure stored in the storage
        let current_block_number = 0;

        let max_retry_times = 10; // in startcoin, it is in config
        let delay_milliseconds_on_error = 100;

        let event_handle = Arc::new(TaskEventCounterHandle::new());

        let ext_error_handle = Arc::new(ExtSyncTaskErrorHandle::new(Arc::clone(
            &self
                .client
                .as_ref()
                .expect("the client must be initialized"),
        )));

        let fetcher = Arc::clone(
            &self
                .client
                .as_ref()
                .expect("the client must be initialized"),
        );

        let best_chain_info = async_std::task::block_on(
            ctx.service_ref::<NetworkDagService>()
                .unwrap()
                .send(GetBestChainInfo),
        )
        .unwrap();

        let accumulator_info = async_std::task::block_on(
            ctx.service_ref::<ChainDagService>()
                .unwrap()
                .send(GetAccumulatorInfo),
        )
        .unwrap();

        let accumulator_store = ctx.get_shared::<Arc<Storage>>().unwrap().get_accumulator_store(AccumulatorStoreType::SyncDag);

        async_std::task::spawn(async move {
            // here should compare the dag's node not accumulator leaf node
            let sync_task = TaskGenerator::new(
                FindAncestorTask::new(
                    current_block_number,
                    best_chain_info.flexi_dag_accumulator_info.num_leaves,
                    fetcher,
                ),
                2,
                max_retry_times,
                delay_milliseconds_on_error,
                AncestorCollector::new(Arc::new(MerkleAccumulator::new_with_info(accumulator_info, accumulator_store))),
                event_handle.clone(),
                ext_error_handle.clone(),
            )
            .generate();
            let (fut, handle) = sync_task.with_handle();
            match fut.await {
                anyhow::Result::Ok(ancestor) => {
                    println!("receive ancestor {:?}", ancestor);
                }
                Err(error) => {
                    println!("an error happened: {}", error.to_string());
                }
            }
        });
        Ok(())
    }
}
