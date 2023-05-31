use anyhow::Result;
use futures_core::future::BoxFuture;

pub trait NetworkDag {
  fn broadcast_message(&self, message: Vec<u8>) -> Result<()>;
  fn register_handshaking(&self, fut: BoxFuture<()>);
}