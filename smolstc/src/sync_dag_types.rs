use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize, JsonSchema)]
pub struct DagBlockIdAndNumber {
    pub accumulator_info: AccumulatorInfo,
    pub number: BlockNumber,
}
