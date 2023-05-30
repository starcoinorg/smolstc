use std::{sync::Arc, net::Ipv4Addr, borrow::Cow, time::Duration};
use futures::{channel::mpsc::{self, Receiver}, StreamExt};
use network_p2p::{config, NetworkService, NetworkWorker};
use network_p2p_types::{IfDisconnected, IncomingRequest, OutgoingResponse, ReputationChange, parse_str_addr};
use serde::{Deserialize, Serialize};
use bcs_ext::BCSCodec;
use network_p2p::request_responses::ProtocolConfig;
use tokio::task::JoinHandle;
use starcoin_types::startup_info::ChainInfo;

const PROTOCOL_NAME_CHAIN: &str = "/starcoin/chain/1";
const PROTOCOL_NAME_NOTIFY: &str = "/starcoin/notify/1";
const PROTOCOL_NAME_REQUEST_RESPONSE: &str = "/starcoin/reques_response/1";

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MyReqeust {
    number: i32,
    name: String,
}


#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MyResponse {
    number: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MyNotify {
    number: i32,
    name: String,
}

fn generate_protocol_config(protocol_name: &'static str) -> (ProtocolConfig, Receiver<IncomingRequest>) {
    let (sender, receiver) = mpsc::channel(128);
    (ProtocolConfig {
        name: Cow::from(protocol_name),
        max_request_size: 1024 * 1024 * 1024 * 10,
        max_response_size: 1024 * 1024 * 1024 * 10,
        request_timeout: Duration::from_secs(10),
        inbound_queue: Some(sender),
    }, receiver)
}

async fn build_worker(
    config: config::NetworkConfiguration,
) -> (Arc<NetworkService>, JoinHandle<()>, JoinHandle<()>) {
    println!("config: {:?}", config);

    let worker = NetworkWorker::new(config::Params {
        network_config: config,
        protocol_id: config::ProtocolId::from(PROTOCOL_NAME_CHAIN),
        chain_info: starcoin_types::startup_info::ChainInfo::random(),
        metrics_registry: None,
    })
    .unwrap();

    let service = worker.service().clone();
    let mut event_stream = service.event_stream("test");

    let notify_handle = tokio::task::spawn(async move {
        loop {
            let item = event_stream.next().await.unwrap();
            match item {
                network_p2p::Event::NotificationsReceived { remote, messages } => {
                    println!("receive notify, peer id = {remote:?}");
                    messages.into_iter().for_each(|(protocol, buffer)| {
                        println!("received protocol: {protocol:?}");
                        let notify = MyNotify::decode(&buffer).unwrap();
                        println!("notify content: {:?}", notify);
                    });
                }
                _ => ()
            }
        }
    });

    let reqres_handle = tokio::task::spawn(async move {
        futures::pin_mut!(worker);
        let _ = worker.await;
    });

    (service, reqres_handle, notify_handle)
}

pub async fn build_network() -> (Arc<NetworkService>, JoinHandle<()>, JoinHandle<()>, Receiver<IncomingRequest>) {
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let listen_addr = config::build_multiaddr![Ip4(localhost), Tcp(0_u16)];
    println!("listen_addr = {:?}", listen_addr);

    let (protocol_config, receiver) = generate_protocol_config(PROTOCOL_NAME_REQUEST_RESPONSE);
    // let (sender, receiver) = mpsc::channel(128);
    // protocol_config.inbound_queue = Some(sender);
    
    let (service, reqres_handle, notify_handle)= build_worker(config::NetworkConfiguration {
        notifications_protocols: vec![From::from(PROTOCOL_NAME_NOTIFY)],
        listen_addresses: vec![listen_addr.clone()],
        transport: config::TransportConfig::Normal { enable_mdns: true, allow_private_ip: true },
        request_response_protocols: vec![protocol_config],
        ..config::NetworkConfiguration::new_local()
    }).await;

    (service, reqres_handle, notify_handle, receiver)
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (service, worker_handle, notify_handle, mut receiver) = build_network().await;

        println!("peer id = {:?}", service.peer_id());

        std::thread::sleep(Duration::from_secs(3));

        let state = service.network_state().await.unwrap();
        println!("state = {:?}", state);

        let receive_handle = tokio::task::spawn(async move {
            loop {
                println!("waiting from a client");
                match receiver.next().await {
                    Some(request) => {
                        println!("get from a client");
                        let peer_request = MyReqeust::decode(&request.payload).unwrap();
                        println!("receive the request from a peer: {peer_request:?}");
                        let message = MyResponse {
                            number: 1002,
                            name: String::from("pong"),
                        };
                        let res = OutgoingResponse {
                            result: Ok(message.encode().unwrap()),
                            reputation_changes: vec![ReputationChange::new(100, "test reputation change")],
                        };
                        request.pending_response.send(res).unwrap();
                    }
                    None => ()
                }
            }
        });

        if let Some(addr) = std::env::args().nth(1) {
            let (peer_id, _) = parse_str_addr(&addr).unwrap();
            service.add_reserved_peer(addr).unwrap();

            loop {
                if service.is_connected(peer_id).await {
                    println!("service is connected to {peer_id:?}!");
                    break;
                }         
            }
             let req = MyReqeust {
                number: 1001,
                name: String::from("ping"), 
            };

            // ping and pong three times
            let mut count = 0;
            while count < 3 {
                let result = service.request(peer_id, Cow::from(PROTOCOL_NAME_REQUEST_RESPONSE), req.encode().unwrap(), IfDisconnected::TryConnect).await.unwrap();
                let response = MyResponse::decode(&result);
                println!("result = {:?}", response);
                count += 1;
            }

            // broadcast
            let notify = MyNotify {
                number: 1003,
                name: String::from("notify from jack"),
            };
            service.broadcast_message(Cow::from(PROTOCOL_NAME_NOTIFY), notify.encode().unwrap()).await;
        }
        receive_handle.await.unwrap();
        notify_handle.await.unwrap();
        worker_handle.await.unwrap();
    })
}
