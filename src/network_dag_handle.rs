use bcs_ext::BCSCodec;
use network_p2p_types::business_layer_handle::BusinessLayerHandle;
use crate::network_dag_data::{NodeData, NodeStatus};
use anyhow::anyhow;

pub struct DagDataHandle {
  node_data: NodeData,
}

impl DagDataHandle {
    pub fn new() -> Self {
      DagDataHandle {
        node_data: NodeData {
          name: String::from("node data name"),
          status: NodeStatus { conn_number: 101 },     
        }
      }
    } 
}

/// handle the handshaking and the data related to the chain
impl BusinessLayerHandle for DagDataHandle {
    fn handshake(&self, generic_node_data: &[u8]) -> std::result::Result<(), (&'static str, String)> {
      match NodeData::decode(generic_node_data) {
        std::result::Result::Ok(handshake) => {
          println!("handshake data: {handshake:?}");
          std::result::Result::Ok(())
        },
        Err(err) => {
          std::result::Result::Err(("the node failed to handshake", format!("the node failed to handshake, because {err}")))
        },
      }
    }

    fn get_generic_data(&self) -> std::result::Result<Vec<u8>, anyhow::Error> {
      std::result::Result::Ok(self.node_data.encode().unwrap())
    }

    fn update_generic_data(&mut self, generic_node_data: &[u8]) -> std::result::Result<(), anyhow::Error> {
      match NodeData::decode(generic_node_data) {
        std::result::Result::Ok(node_data) => {
          println!("handshake data: {node_data:?}");
          self.node_data = node_data;
          std::result::Result::Ok(())
        },
        Err(err) => {
          Err(anyhow!("the node failed to handshake, because {err}"))
        },
      }
    }

    fn update_status(&mut self, generic_status: &[u8]) -> std::result::Result<(), anyhow::Error> {
      match NodeStatus::decode(generic_status) {
        std::result::Result::Ok(status) => {
          println!("handshake data: {status:?}");
          self.node_data.status = status;
          std::result::Result::Ok(())
        },
        Err(err) => {
          Err(anyhow!("the node failed to handshake, because {err}"))
        },
      }
    }
}