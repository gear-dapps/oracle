#![no_std]
#![allow(clippy::missing_safety_doc)]

pub mod io;
pub mod state;

use gstd::{async_main, msg, prelude::*, ActorId};

gstd::metadata! {
    title: "RandomnessOracle",
    init:
        input: io::InitConfig,
    handle:
        input: io::Action,
        output: io::Event,
    state:
        input: io::StateQuery,
        output: io::StateResponse,
}

static mut RANDOMNESS_ORACLE: Option<RandomnessOracle> = None;

#[derive(Debug, Default)]
pub struct RandomnessOracle {
    pub owner: ActorId,
    pub values: BTreeMap<u128, state::Random>,
    pub last_round: u128,
    pub manager: ActorId,
}

impl RandomnessOracle {
    pub fn set_random_value(&mut self, round: u128, value: &state::Random) {
        self.assert_manager();

        if round <= self.last_round {
            panic!("Invalid round!");
        }

        self.last_round = round;

        if self.values.insert(round, value.clone()).is_some() {
            panic!("Unable to update existing value!");
        }

        msg::reply(
            io::Event::NewRandomValue {
                round,
                value: value.clone(),
            },
            0,
        )
        .expect("Unable to reply!");
    }

    pub fn update_manager(&mut self, new_manager: &ActorId) {
        self.assert_owner();

        self.manager = *new_manager;
        msg::reply(io::Event::NewManager(*new_manager), 0).expect("Unable to reply!");
    }

    pub fn get_value(&self, round: u128) -> state::Random {
        self.values
            .get(&round)
            .expect("Unable to find round!")
            .clone()
    }

    pub fn get_values(&self) -> Vec<(u128, state::Random)> {
        self.values
            .iter()
            .map(|(round, value)| (*round, value.clone()))
            .collect()
    }

    pub fn get_random_value(&self, round: u128) -> state::RandomSeed {
        self.get_value(round).randomness
    }

    fn assert_manager(&self) {
        if msg::source() != self.manager {
            panic!("Only manager allowed to call this!");
        }
    }

    fn assert_owner(&self) {
        if msg::source() != self.owner {
            panic!("Only owner allowed to call this!");
        }
    }
}

#[async_main]
async fn main() {
    let action: io::Action = msg::load().expect("Unable to decode Action.");
    let randomness_oracle: &mut RandomnessOracle =
        unsafe { RANDOMNESS_ORACLE.get_or_insert(RandomnessOracle::default()) };

    match action {
        io::Action::SetRandomValue { round, value } => {
            randomness_oracle.set_random_value(round, &value)
        }
        io::Action::UpdateManager(new_manager) => randomness_oracle.update_manager(&new_manager),
    }
}

#[no_mangle]
unsafe extern "C" fn init() {
    let config: io::InitConfig = msg::load().expect("Unable to decode InitConfig.");
    let randomness_oracle = RandomnessOracle {
        owner: msg::source(),
        manager: config.manager,
        ..Default::default()
    };

    RANDOMNESS_ORACLE = Some(randomness_oracle);
}

#[no_mangle]
unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state_query: io::StateQuery = msg::load().expect("Unable to decode StateQuery.");
    let randomness_oracle = RANDOMNESS_ORACLE.get_or_insert(Default::default());

    let encoded = match state_query {
        io::StateQuery::GetOwner => io::StateResponse::Owner(randomness_oracle.owner),
        io::StateQuery::GetManager => io::StateResponse::Manager(randomness_oracle.manager),
        io::StateQuery::GetValue(round) => {
            io::StateResponse::Value(randomness_oracle.get_value(round))
        }
        io::StateQuery::GetValues => io::StateResponse::Values(randomness_oracle.get_values()),
        io::StateQuery::GetLastRound => io::StateResponse::LastRound(randomness_oracle.last_round),
        io::StateQuery::GetLastRandomValue => io::StateResponse::LastRandomValue(
            randomness_oracle.get_random_value(randomness_oracle.last_round),
        ),
        io::StateQuery::GetRandomValueFromRound(round) => {
            io::StateResponse::RandomValueFromRound(randomness_oracle.get_random_value(round))
        }
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}
