use std::{str::FromStr, sync::Arc};

use anyhow::Error;
use network_p2p_core::{export::log::debug, NetRpcError, PeerId, RpcErrorCode};
use stream_task::CustomErrorHandle;

use crate::network_dag_verified_client::VerifiedDagRpcClient;

pub struct ExtSyncTaskErrorHandle {
    fetcher: Arc<VerifiedDagRpcClient>,
}

impl ExtSyncTaskErrorHandle {
    pub fn new(fetcher: Arc<VerifiedDagRpcClient>) -> Self {
        Self { fetcher }
    }
}

impl CustomErrorHandle for ExtSyncTaskErrorHandle {
    fn handle(&self, error: Error) {
        let peer = error.to_string();
        debug!("[sync]sync task peer_str: {:?}", peer);
        if let Ok(peer_id) = PeerId::from_str(&peer) {
            if let Ok(prc_error) = error.downcast::<NetRpcError>() {
                match &prc_error.error_code() {
                    RpcErrorCode::Forbidden
                    | RpcErrorCode::MethodNotFound
                    | RpcErrorCode::ServerUnavailable
                    | RpcErrorCode::Unknown
                    | RpcErrorCode::InternalError => {
                        // fixme
                        // let peers = self.fetcher.peer_selector().remove_peer(&peer_id);
                        // debug!("[sync]sync task, peer len {}", peers);
                    }
                    _ => {
                        debug!("[sync]sync task err: {:?}", prc_error);
                    }
                }
            }
        }
    }
}
