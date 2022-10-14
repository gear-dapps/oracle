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
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    NewValue { value: u128 },
    NewManager(ActorId),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateQuery {
    GetOwner,
    GetManager,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateResponse {
    Owner(ActorId),
    Manager(ActorId),
}
