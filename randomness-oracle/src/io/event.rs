use crate::state;
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId, TypeInfo};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    NewManager(ActorId),
    NewRandomValue { round: u128, value: state::Random },
}
