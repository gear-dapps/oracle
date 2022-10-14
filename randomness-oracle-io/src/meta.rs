use crate::state;
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId, TypeInfo};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MetaQuery {
    GetOwner,
    GetManager,
    GetValues,
    GetValue(u128),
    GetLastRound,
    GetLastRandomValue,
    GetRandomValueFromRound(u128),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MetaResponse {
    Owner(ActorId),
    Manager(ActorId),
    Values(Vec<(u128, state::Random)>),
    Value(state::Random),
    LastRound(u128),
    LastRandomValue(state::RandomSeed),
    RandomValueFromRound(state::RandomSeed),
}
