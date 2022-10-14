use crate::Horse;
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId, TypeInfo};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Action {
    UpdateFeeBps(u16),
    UpdateManager(ActorId),
    UpdateOracle(ActorId),
    ProgressLastRun,
    CancelLastRun,
    CreateRun {
        bidding_duration_ms: u64,
        horses: BTreeMap<String, Horse>,
    },
    FinishLastRun,
    Bid {
        horse_name: String,
        amount: u128,
    },
    WithdrawCanceled(u128),
    WithdrawFinished(u128),
}
