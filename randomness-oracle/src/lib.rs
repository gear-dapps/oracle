#![no_std]
#![allow(clippy::missing_safety_doc)]

use gstd::{async_main, msg, prelude::*, ActorId};
use randomness_oracle_io::*;

gstd::metadata! {
    title: "RandomnessOracle",
    init:
        input: InitConfig,
    handle:
        input: Action,
        output: Event,
    state:
        input: MetaQuery,
        output: MetaResponse,
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
            Event::NewRandomValue {
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
        msg::reply(Event::NewManager(*new_manager), 0).expect("Unable to reply!");
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
    let action: Action = msg::load().expect("Unable to decode Action.");
    let randomness_oracle: &mut RandomnessOracle =
        unsafe { RANDOMNESS_ORACLE.get_or_insert(RandomnessOracle::default()) };

    match action {
        Action::SetRandomValue { round, value } => {
            randomness_oracle.set_random_value(round, &value)
        }
        Action::GetLastRoundWithRandomValue => {
            let round = randomness_oracle.last_round;
            let random_value = randomness_oracle.get_random_value(round);

            msg::reply(
                Event::LastRoundWithRandomValue {
                    round,
                    random_value,
                },
                0,
            )
            .expect("Unable to reply!");
        }
        Action::UpdateManager(new_manager) => randomness_oracle.update_manager(&new_manager),
    }
}

#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig.");
    let randomness_oracle = RandomnessOracle {
        owner: msg::source(),
        manager: config.manager,
        ..Default::default()
    };

    RANDOMNESS_ORACLE = Some(randomness_oracle);
}

#[no_mangle]
unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state_query: MetaQuery = msg::load().expect("Unable to decode MetaQuery.");
    let randomness_oracle = RANDOMNESS_ORACLE.get_or_insert(Default::default());

    let encoded = match state_query {
        MetaQuery::GetOwner => MetaResponse::Owner(randomness_oracle.owner),
        MetaQuery::GetManager => MetaResponse::Manager(randomness_oracle.manager),
        MetaQuery::GetValue(round) => MetaResponse::Value(randomness_oracle.get_value(round)),
        MetaQuery::GetValues => MetaResponse::Values(randomness_oracle.get_values()),
        MetaQuery::GetLastRound => MetaResponse::LastRound(randomness_oracle.last_round),
        MetaQuery::GetLastRandomValue => MetaResponse::LastRandomValue(
            randomness_oracle.get_random_value(randomness_oracle.last_round),
        ),
        MetaQuery::GetRandomValueFromRound(round) => {
            MetaResponse::RandomValueFromRound(randomness_oracle.get_random_value(round))
        }
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}
