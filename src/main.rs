use std::{sync::Arc, net::Ipv4Addr, borrow::Cow, time::Duration};
use futures::{channel::mpsc::{self, Receiver}, StreamExt};
use network_p2p::{config, NetworkService, NetworkWorker};
use network_p2p_types::{IfDisconnected, IncomingRequest, OutgoingResponse, ReputationChange};
use serde::{Deserialize, Serialize};
use bcs_ext::BCSCodec;
use network_p2p::request_responses::ProtocolConfig;
use tokio::task::JoinHandle;

const PROTOCOL_NAME: &str = "/starcoin/notify/1";

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

fn generate_protocol_config() -> (ProtocolConfig, Receiver<IncomingRequest>) {
    let (sender, receiver) = mpsc::channel(128);
    (ProtocolConfig {
        name: Cow::from(PROTOCOL_NAME),
        max_request_size: 1024 * 1024 * 1024 * 10,
        max_response_size: 1024 * 1024 * 1024 * 10,
        request_timeout: Duration::from_secs(10),
        inbound_queue: Some(sender),
    }, receiver)
}

async fn build_worker(
    config: config::NetworkConfiguration,
) -> (Arc<NetworkService>, JoinHandle<()>) {
    println!("config: {:?}", config);
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

    (service, handle)
}

pub async fn build_network() -> (Arc<NetworkService>, JoinHandle<()>, Receiver<IncomingRequest>) {
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let listen_addr = config::build_multiaddr![Ip4(localhost), Tcp(0_u16)];
    println!("listen_addr = {:?}", listen_addr);

    let (protocol_config, receiver) = generate_protocol_config();
    // let (sender, receiver) = mpsc::channel(128);
    // protocol_config.inbound_queue = Some(sender);
    
    let (service, handle)= build_worker(config::NetworkConfiguration {
        notifications_protocols: vec![From::from(PROTOCOL_NAME)],
        listen_addresses: vec![listen_addr.clone()],
        transport: config::TransportConfig::Normal { enable_mdns: true, allow_private_ip: true },
        request_response_protocols: vec![protocol_config],
        ..config::NetworkConfiguration::new_default(localhost)
    }).await;

    (service, handle, receiver)
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (service1, handle1, mut receiver1) = build_network().await;
        let (service2, handle2, mut receiver2) = build_network().await;

        println!("peer id 1 = {:?}, peer id 2 = {:?}", service1.peer_id(), service2.peer_id());

        let state1 = service1.network_state().await.unwrap();
        println!("state1 = {:?}", state1.listened_addresses);

        let state2 = service2.network_state().await.unwrap();
        println!("state2 = {:?}", state2.listened_addresses);

        let peer_id = state2.peer_id.clone();
        state2.listened_addresses.iter().for_each(|multiaddr| {
            let addr = format!("{}/p2p/{}", multiaddr.to_string(), peer_id.to_string());
            println!("addr = {}", addr);
            service1.add_reserved_peer(addr).unwrap();
        });

        loop {
            if service1.is_connected(service2.peer_id().clone()).await {
                println!("service 1 and service 2 is connected!");
                break;
            }         
        }

            let req = MyReqeust {
                number: 1001,
                name: String::from("jack"), 
            };

            let handle3 = tokio::task::spawn(async move {
                loop {
                    println!("waiting from a client");
                    match receiver2.next().await {
                        Some(request) => {
                            println!("get from a client");
                            let message = MyResponse {
                                number: 1002,
                                name: String::from("rose"),
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
            let handle4 = tokio::task::spawn(async move {
                loop {
                    println!("waiting from a client");
                    match receiver1.next().await {
                        Some(request) => {
                            println!("get from a client");
                            let message = MyResponse {
                                number: 1002,
                                name: String::from("rose"),
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

            loop {
                
                let result = service1.request(service2.peer_id().clone(), Cow::from(PROTOCOL_NAME), req.encode().unwrap(), IfDisconnected::TryConnect).await;
                println!("result = {:?}", result);
                if service1.is_connected(service2.peer_id().clone()).await {
                    println!("service 1 and service 2 is connected!");
                }         
            }

            handle1.await;
            handle2.await;
            handle3.await;
            handle4.await;
        // let stream = receiver1.map(move |request| {
        //     let message = MyResponse {
        //         number: 1002,
        //         name: String::from("rose"),
        //     };
        //     let response = OutgoingResponse {
        //         result: Ok(message.encode().unwrap()),
        //         reputation_changes: vec![ReputationChange::new(100, "test reputation change")],
        //     };
        //     let result = request.pending_response.send(response);
        //     result.unwrap();
        // });
        // let _ = stream.fuse();

        // handle.await.unwrap();
    })
}
