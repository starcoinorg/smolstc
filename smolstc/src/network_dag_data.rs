use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// it is ChainInfo inn starcoin
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ChainInfo {
    pub conn_number: usize,
}

/// it is ChainInfo inn starcoin
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Status {
    /// Protocol version.
    pub version: u32,
    /// Minimum supported version.
    pub min_supported_version: u32,
    /// Tell other peer which notification protocols we support.
    pub notif_protocols: Vec<Cow<'static, str>>,
    /// Tell other peer which rpc api we support.
    pub rpc_protocols: Vec<Cow<'static, str>>,
    /// the generic data related to the peer
    pub info: ChainInfo,
}
