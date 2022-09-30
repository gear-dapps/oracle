#![no_std]
#![allow(clippy::missing_safety_doc)]

use codec::{Decode, Encode};
use gstd::{msg, prelude::*, ActorId, TypeInfo};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Action {
    Roll,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    RollValueRequested(u128),
    RollFinished((u128, u128)),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateQuery {
    GetUsersData,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateResponse {
    UsersData(Vec<(u128, ActorId, RollStatus)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum RollStatus {
    Rolling,
    Finished(bool),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitConfig {
    pub oracle: ActorId,
}

#[derive(Debug, Default)]
pub struct RollDice {
    pub users_data: BTreeMap<u128, (ActorId, RollStatus)>,
    pub oracle: ActorId,
}

impl RollDice {
    /// Request random value from `oracle`.
    pub async fn roll(&mut self) {
        let oracle_reply: oracle_io::Event =
            msg::send_for_reply_as(self.oracle, oracle_io::Action::RequestValue, 0)
                .expect("Unable to request value from oracle!")
                .await
                .expect("Unable to decode oracle reply!");

        if let oracle_io::Event::NewUpdateRequest { id, caller: _ } = oracle_reply {
            self.users_data
                .insert(id, (msg::source(), RollStatus::Rolling));
            msg::reply(Event::RollValueRequested(id), 0).unwrap();
        } else {
            panic!("Invalid oracle reply!");
        }
    }

    /// Handle reply from `oracle` with random value and id.
    pub fn roll_finished(&mut self, id: u128, value: u128) {
        let (_, roll_status) = self.users_data.get_mut(&id).expect("Invalid ID!");
        *roll_status = RollStatus::Finished(value % 2 == 0);

        msg::reply(Event::RollFinished((id, value)).encode(), 0).expect("Unable to reply!");
    }
}

static mut ROLL_DICE: Option<RollDice> = None;

#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig.");
    let roll_dice = RollDice {
        oracle: config.oracle,
        ..Default::default()
    };

    ROLL_DICE = Some(roll_dice);
}

#[gstd::async_main]
async fn main() {
    let roll_dice: &mut RollDice = unsafe { ROLL_DICE.get_or_insert(RollDice::default()) };

    // Handler(proxy) for oracle messages
    if msg::source() == roll_dice.oracle {
        let payload = msg::load_bytes();
        let id: u128 = u128::from_le_bytes(payload[1..17].try_into().unwrap());
        let value: u128 = u128::from_le_bytes(payload[17..].try_into().unwrap());

        roll_dice.roll_finished(id, value);
        return;
    }

    let action: Action = msg::load().expect("Unable to decode Action.");
    match action {
        Action::Roll => roll_dice.roll().await,
    }
}

#[no_mangle]
unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state_query: StateQuery = msg::load().expect("Unable to decode StateQuery.");
    let roll_dice = ROLL_DICE.get_or_insert(Default::default());

    let encoded = match state_query {
        StateQuery::GetUsersData => StateResponse::UsersData(
            roll_dice
                .users_data
                .iter()
                .map(|(id, (user, status))| (*id, *user, *status))
                .collect(),
        ),
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}
