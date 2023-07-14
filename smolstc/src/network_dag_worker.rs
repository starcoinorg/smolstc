use std::sync::Arc;

use crate::{
    network_dag_data::ChainInfo, network_dag_handle::DagDataHandle,
    network_dag_service::NetworkDagService, sync_block_dag::SyncBlockDag,
};
use network_p2p::{config, NetworkWorker};
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, MerkleAccumulator};
use starcoin_storage::Storage;

const PROTOCOL_NAME_CHAIN: &str = "/starcoin/notify/1";

pub fn build_worker(
    config: config::NetworkConfiguration,
    ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagService>,
) -> NetworkWorker<DagDataHandle> {
    println!("config: {:?}", config);

    let worker = NetworkWorker::new(config::Params {
        network_config: config,
        protocol_id: config::ProtocolId::from(PROTOCOL_NAME_CHAIN),
        metrics_registry: None,
        business_layer_handle: DagDataHandle::new(ChainInfo {
            flexi_dag_accumulator_info: 
                ctx.get_shared::<Arc<SyncBlockDag>>().unwrap().accumulator_info.clone(),
        }),
    })
    .unwrap();
    worker
}
