use crate::{Horse, Run};
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId, TypeInfo};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateQuery {
    GetRuns,
    GetHorses(u128),
    GetManager,
    GetOwner,
    GetToken,
    GetOracle,
    GetFeeBps,
    GetRunNonce,
    GetRun(u128),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateResponse {
    Runs(Vec<(u128, Run)>),
    Horses(Vec<(String, Horse, u128)>),
    Manager(ActorId),
    Owner(ActorId),
    Token(ActorId),
    Oracle(ActorId),
    FeeBps(u16),
    RunNonce(u128),
    Run(Run),
}
