use crate::Horse;
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId, TypeInfo};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    FeeBpsUpdated(u16),
    ManagerUpdated(ActorId),
    OracleUpdated(ActorId),
    LastRunProgressed(u128),
    LastRunCanceled(u128),
    LastRunFinished {
        run_id: u128,
        winner: (String, Horse, u128),
    },
    RunCreated {
        run_id: u128,
        bidding_duration_ms: u64,
        horses: BTreeMap<String, Horse>,
    },
    NewBid {
        horse_name: String,
        amount: u128,
    },
    NewWithdrawCanceled {
        user: ActorId,
        run_id: u128,
        amount: u128,
    },
    NewWithdrawFinished {
        user: ActorId,
        run_id: u128,
        amount: u128,
        profit_amount: u128,
    },
}
