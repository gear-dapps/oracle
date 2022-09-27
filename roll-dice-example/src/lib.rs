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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use gtest::{Program, System};

    #[test]
    fn success_roll() {
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;
        const USER: u64 = 5;
        const ORACLE_ID: u64 = 100;
        const ROLL_DICE_ID: u64 = 200;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::from_file_with_id(
            &sys,
            ORACLE_ID,
            "../target/wasm32-unknown-unknown/release/oracle.wasm",
        );
        let roll_dice_program = Program::current_with_id(&sys, ROLL_DICE_ID);

        let result = oracle_program.send(
            OWNER,
            oracle_io::InitConfig {
                owner: OWNER.into(),
                manager: MANAGER.into(),
            },
        );
        assert!(result.log().is_empty());

        let result = roll_dice_program.send(
            OWNER,
            InitConfig {
                oracle: ORACLE_ID.into(),
            },
        );
        assert!(result.log().is_empty());

        let result = roll_dice_program.send(USER, Action::Roll);
        assert!(!result.main_failed());
        assert!(!result.others_failed());
        assert!(result.contains(&(USER, Event::RollValueRequested(1u128).encode())));
    }

    #[test]
    fn success_roll_finished() {
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;
        const USER: u64 = 5;
        const ORACLE_ID: u64 = 100;
        const ROLL_DICE_ID: u64 = 200;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::from_file_with_id(
            &sys,
            ORACLE_ID,
            "../target/wasm32-unknown-unknown/release/oracle.wasm",
        );
        let roll_dice_program = Program::current_with_id(&sys, ROLL_DICE_ID);

        let result = oracle_program.send(
            OWNER,
            oracle_io::InitConfig {
                owner: OWNER.into(),
                manager: MANAGER.into(),
            },
        );
        assert!(result.log().is_empty());

        let result = roll_dice_program.send(
            OWNER,
            InitConfig {
                oracle: ORACLE_ID.into(),
            },
        );
        assert!(result.log().is_empty());

        let result = roll_dice_program.send(USER, Action::Roll);
        assert!(!result.main_failed());
        assert!(!result.others_failed());

        let meta_result: StateResponse = roll_dice_program
            .meta_state(StateQuery::GetUsersData)
            .unwrap();
        match meta_result {
            StateResponse::UsersData(users_data) => {
                assert_eq!(users_data[0].0, 1u128);
                assert_eq!(users_data[0].1, USER.into());
                assert_eq!(users_data[0].2, RollStatus::Rolling);
            }
        }

        sys.spend_blocks(150);

        let result = oracle_program.send(
            MANAGER,
            oracle_io::Action::UpdateValue { id: 1, value: 1337 },
        );
        assert!(!result.main_failed());
        assert!(!result.others_failed());

        let meta_result: StateResponse = roll_dice_program
            .meta_state(StateQuery::GetUsersData)
            .unwrap();
        match meta_result {
            StateResponse::UsersData(users_data) => {
                assert_eq!(users_data[0].0, 1u128);
                assert_eq!(users_data[0].1, USER.into());
                assert_eq!(users_data[0].2, RollStatus::Finished(false));
            }
        }
    }
}
