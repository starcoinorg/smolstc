use std::sync::Arc;
use network_p2p::{config, NetworkService, NetworkWorker, Event};
use futures_core::Stream;

const PROTOCOL_NAME: &str = "/starcoin/notify/1";

fn build_test_full_node(
    config: config::NetworkConfiguration,
) -> (Arc<NetworkService>, impl Stream<Item = Event>) {
    let worker = NetworkWorker::new(config::Params {
        network_config: config,
        protocol_id: config::ProtocolId::from("/test-protocol-name"),
        metrics_registry: None,
    })
    .unwrap();

    let service = worker.service().clone();
    let event_stream = service.event_stream("test");

    tokio::task::spawn(async move {
        futures::pin_mut!(worker);
        let _ = worker.await;
    });

    (service, event_stream)
}

pub fn build_network() {
    let listen_addr = config::build_multiaddr![Memory(rand::random::<u64>())];
    println!("listen_addr = {:?}", listen_addr);
    let (node1, events_stream1) = build_test_full_node(config::NetworkConfiguration {
        //notifications_protocols: vec![(ENGINE_ID, From::from("/foo"))],
        notifications_protocols: vec![From::from(PROTOCOL_NAME)],
        listen_addresses: vec![listen_addr.clone()],
        transport: config::TransportConfig::MemoryOnly,
        ..config::NetworkConfiguration::new_local()
    });
}

fn main() {
    build_network();
}
