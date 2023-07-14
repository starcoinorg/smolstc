use crate::{
    chain_dag_service::{ChainDagService, GetAccumulatorInfo},
    network_dag_data::ChainInfo,
    network_dag_handle::DagDataHandle,
    network_dag_service::NetworkDagService,
};
use network_p2p::{config, NetworkWorker};

const PROTOCOL_NAME_CHAIN: &str = "/starcoin/notify/1";

pub fn build_worker(
    config: config::NetworkConfiguration,
    ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagService>,
) -> NetworkWorker<DagDataHandle> {
    println!("config: {:?}", config);

    let accumulator_info = async_std::task::block_on(
        ctx.service_ref::<ChainDagService>()
            .unwrap()
            .send(GetAccumulatorInfo),
    )
    .unwrap();

    let worker = NetworkWorker::new(config::Params {
        network_config: config,
        protocol_id: config::ProtocolId::from(PROTOCOL_NAME_CHAIN),
        metrics_registry: None,
        business_layer_handle: DagDataHandle::new(ChainInfo {
            flexi_dag_accumulator_info: accumulator_info,
        }),
    })
    .unwrap();
    worker
}
