use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use schemars::JsonSchema;


#[derive(
  Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, JsonSchema,
)]
pub struct DagBlockIdAndNumber {
  pub id: HashValue,
  pub number: BlockNumber,
  pub parents: Vec<HashValue>
}