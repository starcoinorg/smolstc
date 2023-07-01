use std::borrow::Cow;

use crate::{network_dag_data::{NodeData, NodeStatus}, network_dag_service::{PROTOCOL_NAME_NOTIFY, PROTOCOL_NAME_REQRES_1, PROTOCOL_NAME_REQRES_2}};
use anyhow::{anyhow, Ok};
use bcs_ext::BCSCodec;
use network_p2p::{
    business_layer_handle::{BusinessLayerHandle, HandshakeResult},
    protocol::{generic_proto::NotificationsSink, rep},
};
use sc_peerset::{ReputationChange, SetId};

pub struct DagDataHandle {
    node_data: NodeData,
}

impl DagDataHandle {
    pub fn new() -> Self {
        DagDataHandle {
            node_data: NodeData {
                name: String::from("node data name"),
                status: NodeStatus { conn_number: 101 },
            },
        }
    }
}

/// handle the handshaking and the data related to the chain
impl BusinessLayerHandle for DagDataHandle {
    fn get_generic_data(&self) -> std::result::Result<Vec<u8>, anyhow::Error> {
        std::result::Result::Ok(self.node_data.encode().unwrap())
    }

    fn update_generic_data(
        &mut self,
        generic_node_data: &[u8],
    ) -> std::result::Result<(), anyhow::Error> {
        match NodeData::decode(generic_node_data) {
            std::result::Result::Ok(node_data) => {
                println!("handshake data: {node_data:?}");
                self.node_data = node_data;
                std::result::Result::Ok(())
            }
            Err(err) => Err(anyhow!("the node failed to handshake, because {err}")),
        }
    }

    fn update_status(&mut self, generic_status: &[u8]) -> std::result::Result<(), anyhow::Error> {
        match NodeStatus::decode(generic_status) {
            std::result::Result::Ok(status) => {
                println!("handshake data: {status:?}");
                self.node_data.status = status;
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
        return Ok(self.node_data.encode().unwrap());
    }

    fn handshake(
        &self,
        peer_id: network_p2p_types::PeerId,
        received_handshake: Vec<u8>,
    ) -> Result<HandshakeResult, ReputationChange> {
        return std::result::Result::Ok(HandshakeResult {
            who: peer_id,
            generic_data: self.node_data.encode().unwrap(),
            notif_protocols: vec![Cow::from(PROTOCOL_NAME_NOTIFY)],
            rpc_protocols: vec![
                Cow::from(PROTOCOL_NAME_REQRES_1),
                Cow::from(PROTOCOL_NAME_REQRES_2),
            ],
        });
   }
}
