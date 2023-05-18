use std::{sync::Arc, net::Ipv4Addr, borrow::Cow, time::Duration};
use network_p2p::{config, NetworkService, NetworkWorker};
use network_p2p_types::{IfDisconnected};
use serde::{Deserialize, Serialize};
use bcs_ext::BCSCodec;
use network_p2p::request_responses::ProtocolConfig;

const PROTOCOL_NAME: &str = "/starcoin/notify/1";

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MyReqeust {
    number: i32,
    name: String,
}

fn generate_protocol_config() -> ProtocolConfig {
    ProtocolConfig {
        name: Cow::from(PROTOCOL_NAME),
        max_request_size: 1024 * 1024 * 1024 * 10,
        max_response_size: 1024 * 1024 * 1024 * 10,
        request_timeout: Duration::from_secs(10),
        inbound_queue: None,
    }
}

async fn build_worker(
    config: config::NetworkConfiguration,
) -> (tokio::task::JoinHandle<()>, Arc<NetworkService>) {
    let worker = NetworkWorker::new(config::Params {
        network_config: config,
        protocol_id: config::ProtocolId::from(PROTOCOL_NAME),
        metrics_registry: None,
    })
    .unwrap();

    let service = worker.service().clone();
    // let event_stream = service.event_stream("test");

    let handle = tokio::task::spawn(async move {
        futures::pin_mut!(worker);
        let _ = worker.await;
    });

    (handle, service)
}

pub async fn build_network() -> (tokio::task::JoinHandle<()>, Arc<NetworkService>) {
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let listen_addr = config::build_multiaddr![Ip4(localhost), Tcp(0_u16)];
    println!("listen_addr = {:?}", listen_addr);
    let (handle, service) = build_worker(config::NetworkConfiguration {
        notifications_protocols: vec![From::from(PROTOCOL_NAME)],
        listen_addresses: vec![listen_addr.clone()],
        transport: config::TransportConfig::Normal { enable_mdns: true, allow_private_ip: true },
        request_response_protocols: vec![generate_protocol_config()],
        ..config::NetworkConfiguration::new_default(localhost)
    }).await;

    (handle, service)
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (handle1, service1) = build_network().await;
        let (handle2, service2) = build_network().await;

        println!("peer id 1 = {:?}, peer id 2 = {:?}", service1.peer_id(), service2.peer_id());

        let state2 = service2.network_state().await.unwrap();
        println!("state2 = {:?}", state2.listened_addresses);

        let peer_id = state2.peer_id.clone();
        state2.listened_addresses.iter().for_each(|multiaddr| {
            let addr = format!("{}/p2p/{}", multiaddr.to_string(), peer_id.to_string());
            println!("addr = {}", addr);
            service1.add_reserved_peer(addr).unwrap();
        });

        loop {
            if service1.is_connected(*service2.peer_id()).await {
                println!("service 1 and service 2 is connected!");
                break;
            }         
        }

        let req = MyReqeust {
            number: 1001,
            name: String::from("jack"), 
        };

        let result = service1.request(service2.peer_id().clone(), Cow::from(PROTOCOL_NAME), req.encode().unwrap(), IfDisconnected::TryConnect).await;
        result.unwrap();

        handle1.await;
        handle2.await;
    })
}
