use std::borrow::Cow;

use crate::{
    network_dag_data::{ChainInfo, Status},
    network_dag_service::{PROTOCOL_NAME_NOTIFY, PROTOCOL_NAME_REQRES_1, PROTOCOL_NAME_REQRES_2},
};
use anyhow::{anyhow, Ok};
use bcs_ext::BCSCodec;
use network_p2p::{
    business_layer_handle::{BusinessLayerHandle, HandshakeResult},
    protocol::{generic_proto::NotificationsSink, rep},
};
use network_p2p_core::export::log::error;
use sc_peerset::{ReputationChange, SetId};

pub struct DagDataHandle {
    status: Status,
}

impl DagDataHandle {
    pub fn new(chain_info: ChainInfo) -> Self {
        let status = Status {
            version: 5,
            min_supported_version: 3,
            notif_protocols: [].to_vec(),
            rpc_protocols: [].to_vec(),
            info: chain_info,
        };
        DagDataHandle { status }
    }
}

/// handle the handshaking and the data related to the chain
impl BusinessLayerHandle for DagDataHandle {
    fn get_generic_data(&self) -> std::result::Result<Vec<u8>, anyhow::Error> {
        std::result::Result::Ok(self.status.encode().unwrap())
    }

    fn update_generic_data(
        &mut self,
        generic_node_data: &[u8],
    ) -> std::result::Result<(), anyhow::Error> {
        match Status::decode(generic_node_data) {
            std::result::Result::Ok(status) => {
                println!("handshake data: {status:?}");
                self.status = status;
                std::result::Result::Ok(())
            }
            Err(err) => Err(anyhow!("the node failed to handshake, because {err}")),
        }
    }

    fn update_status(&mut self, generic_status: &[u8]) -> std::result::Result<(), anyhow::Error> {
        match ChainInfo::decode(generic_status) {
            std::result::Result::Ok(chain_info) => {
                println!("handshake data: {chain_info:?}");
                self.status.info = chain_info;
                std::result::Result::Ok(())
            }
            Err(err) => Err(anyhow!("the node failed to handshake, because {err}")),
        }
    }

    fn build_handshake_msg(
        &mut self,
        notif_protocols: Vec<std::borrow::Cow<'static, str>>,
        rpc_protocols: Vec<std::borrow::Cow<'static, str>>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        self.status.notif_protocols = notif_protocols;
        self.status.rpc_protocols = rpc_protocols;
        self.status.encode()
    }

    fn handshake(
        &self,
        peer_id: network_p2p_types::PeerId,
        received_handshake: Vec<u8>,
    ) -> Result<HandshakeResult, ReputationChange> {
        match Status::decode(&received_handshake[..]) {
            std::result::Result::Ok(status) => std::result::Result::Ok(HandshakeResult {
                who: peer_id,
                generic_data: status.info.encode().unwrap(),
                notif_protocols: status.notif_protocols.to_vec(),
                rpc_protocols: status.rpc_protocols.to_vec(),
            }),
            Err(err) => {
                error!(target: "network-p2p", "Couldn't decode handshake packet sent by {}: {:?}: {}", peer_id, hex::encode(received_handshake), err);
                Err(rep::BAD_MESSAGE)
            }
        }
    }
}
