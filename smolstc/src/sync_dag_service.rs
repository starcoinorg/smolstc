use std::{any, sync::Arc};

use crate::{
    chain_dag_service::{ChainDagService, GetAccumulatorInfo},
    find_ancestor_task::{AncestorCollector, FindAncestorTask},
    network_dag_data::ChainInfo,
    network_dag_rpc::TargetAccumulatorLeaf,
    network_dag_service::{GetBestChainInfo, NetworkDagService},
    network_dag_verified_client::{NetworkDagServiceRef, VerifiedDagRpcClient},
    sync_dag_accumulator_task::{SyncDagAccumulatorCollector, SyncDagAccumulatorTask},
    sync_task_error_handle::ExtSyncTaskErrorHandle,
};
use anyhow::Ok;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator, MerkleAccumulator,
};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceFactory, ServiceHandler, ServiceRef, ServiceRequest,
};
use starcoin_storage::{Storage, Store, SyncFlexiDagStore};
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

    pub fn find_ancestor_task(
        &self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
        best_chain_info: ChainInfo,
    ) -> AccumulatorInfo {
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

        let accumulator_info = async_std::task::block_on(
            ctx.service_ref::<ChainDagService>()
                .unwrap()
                .send(GetAccumulatorInfo),
        )
        .unwrap();

        let accumulator_store = ctx
            .get_shared::<Arc<Storage>>()
            .unwrap()
            .get_accumulator_store(AccumulatorStoreType::SyncDag);
        let accumulator_snapshot = ctx
            .get_shared::<Arc<Storage>>()
            .unwrap()
            .get_accumulator_snapshot_storage();

        let ancestor = async_std::task::spawn(async move {
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
                AncestorCollector::new(
                    Arc::new(MerkleAccumulator::new_with_info(
                        accumulator_info,
                        accumulator_store.clone(),
                    )),
                    accumulator_snapshot.clone(),
                ),
                event_handle.clone(),
                ext_error_handle.clone(),
            )
            .generate();
            let (fut, handle) = sync_task.with_handle();
            match fut.await {
                anyhow::Result::Ok(ancestor) => {
                    println!("receive ancestor {:?}", ancestor);
                    return Ok(ancestor);
                }
                Err(error) => {
                    println!("an error happened: {}", error.to_string());
                    return Err(error.into());
                }
            }
        });
        return async_std::task::block_on(ancestor).unwrap();
    }

    fn sync_accumulator(
        &self,
        ctx: &mut starcoin_service_registry::ServiceContext<'_, SyncDagService>,
        ancestor: AccumulatorInfo,
        best_chain_info: ChainInfo,
    ) -> anyhow::Result<()> {
        let max_retry_times = 10; // in startcoin, it is in config
        let delay_milliseconds_on_error = 100;

        let accumulator_store = ctx
            .get_shared::<Arc<Storage>>()
            .unwrap()
            .get_accumulator_store(AccumulatorStoreType::SyncDag);

        let accumulator_snapshot = ctx
            .get_shared::<Arc<Storage>>()
            .unwrap()
            .get_accumulator_snapshot_storage();

        let fetcher = Arc::clone(
            &self
                .client
                .as_ref()
                .expect("the client must be initialized"),
        );

        let start_index = ancestor.get_num_leaves().saturating_sub(1);

        let event_handle = Arc::new(TaskEventCounterHandle::new());

        let ext_error_handle = Arc::new(ExtSyncTaskErrorHandle::new(Arc::clone(
            &self
                .client
                .as_ref()
                .expect("the client must be initialized"),
        )));

        async_std::task::spawn(async move {
            let sync_task = TaskGenerator::new(
                SyncDagAccumulatorTask::new(
                    start_index,
                    10,
                    best_chain_info.flexi_dag_accumulator_info.num_leaves,
                    fetcher.clone(),
                ),
                2,
                max_retry_times,
                delay_milliseconds_on_error,
                SyncDagAccumulatorCollector::new(
                    MerkleAccumulator::new_with_info(ancestor, accumulator_store.clone()),
                    accumulator_snapshot.clone(),
                    best_chain_info.flexi_dag_accumulator_info,
                    start_index,
                ),
                event_handle.clone(),
                ext_error_handle,
            );
        });
        Ok(())
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
        let best_chain_info = async_std::task::block_on(
            ctx.service_ref::<NetworkDagService>()
                .unwrap()
                .send(GetBestChainInfo),
        )
        .unwrap();
        let ancestor = self.find_ancestor_task(ctx, best_chain_info.clone());
        self.sync_accumulator(ctx, ancestor, best_chain_info.clone());
        Ok(())
    }
}
