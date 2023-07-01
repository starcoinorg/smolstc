use crate::{network_dag_handle::DagDataHandle, network_dag_data::ChainInfo};
use network_p2p::{config, NetworkWorker};

const PROTOCOL_NAME_CHAIN: &str = "/starcoin/notify/1";

pub fn build_worker(config: config::NetworkConfiguration) -> NetworkWorker<DagDataHandle> {
    println!("config: {:?}", config);

    let worker = NetworkWorker::new(config::Params {
        network_config: config,
        protocol_id: config::ProtocolId::from(PROTOCOL_NAME_CHAIN),
        metrics_registry: None,
        business_layer_handle: DagDataHandle::new(ChainInfo {
            conn_number: 101, // test only
        }),
    })
    .unwrap();
    worker
}
