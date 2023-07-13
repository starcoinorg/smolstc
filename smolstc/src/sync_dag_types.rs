use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;

#[derive(
    Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, JsonSchema,
)]
pub struct DagBlockIdAndNumber {
    pub accumulator_leaf: HashValue,
    pub number: BlockNumber,
}
