#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitConfig {
    pub owner: ActorId,
    pub manager: ActorId,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Action {
    RequestValue,
    ChangeManager(ActorId),
    UpdateValue { id: u128, value: u128 },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    NewUpdateRequest { id: u128, caller: ActorId },
    NewManager(ActorId),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateQuery {
    GetOwner,
    GetManager,
    GetRequestsQueue,
    GetIdNonce,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateResponse {
    Owner(ActorId),
    Manager(ActorId),
    RequestsQueue(Vec<(u128, ActorId)>),
    IdNonce(u128),
}
