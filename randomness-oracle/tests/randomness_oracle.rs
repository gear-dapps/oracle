mod utils;

use codec::Encode;
use gtest::System;
use randomness_oracle::{io::*, state::*};
use utils::*;

#[test]
fn success_init() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    let result = oracle_program.send(
        OWNER,
        InitConfig {
            manager: MANAGER.into(),
        },
    );
    assert!(result.log().is_empty());

    let meta_result: StateResponse = oracle_program.meta_state(StateQuery::GetOwner).unwrap();
    match meta_result {
        StateResponse::Owner(owner) => assert_eq!(owner, OWNER.into()),
        _ => panic!("Invalid StateResponse!"),
    }

    let meta_result: StateResponse = oracle_program.meta_state(StateQuery::GetManager).unwrap();
    match meta_result {
        StateResponse::Manager(manager) => assert_eq!(manager, MANAGER.into()),
        _ => panic!("Invalid StateResponse!"),
    }

    let meta_result: StateResponse = oracle_program.meta_state(StateQuery::GetValues).unwrap();
    match meta_result {
        StateResponse::Values(values) => assert!(values.is_empty()),
        _ => panic!("Invalid StateResponse!"),
    }

    let meta_result: StateResponse = oracle_program.meta_state(StateQuery::GetLastRound).unwrap();
    match meta_result {
        StateResponse::LastRound(last_round) => assert_eq!(last_round, 0),
        _ => panic!("Invalid StateResponse!"),
    }
}

#[test]
fn success_update_manager() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            manager: MANAGER.into(),
        },
    );

    let result = oracle_program.send(OWNER, Action::UpdateManager(NEW_MANAGER.into()));
    assert!(result.contains(&(OWNER, Event::NewManager(NEW_MANAGER.into()).encode())));

    let result = oracle_program.send(OWNER, Action::UpdateManager(OWNER.into()));
    assert!(result.contains(&(OWNER, Event::NewManager(OWNER.into()).encode())));
}

#[test]
fn success_set_random_value() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            manager: MANAGER.into(),
        },
    );

    let value = Random {
        randomness: (1337, 133700000000001337),
        signature: Vec::new(),
        prev_signature: Vec::new(),
    };

    let result = oracle_program.send(
        MANAGER,
        Action::SetRandomValue {
            round: 1,
            value: value.clone(),
        },
    );
    assert!(result.contains(&(MANAGER, Event::NewRandomValue { round: 1, value }.encode())));
}

#[test]
fn fail_set_random_value_invalid_manager() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            manager: MANAGER.into(),
        },
    );

    let value = Random {
        randomness: (0, 0),
        signature: Vec::new(),
        prev_signature: Vec::new(),
    };

    let result = oracle_program.send(FAKE_MANAGER, Action::SetRandomValue { round: 1, value });
    assert!(result.main_failed());
}

#[test]
fn fail_set_random_value_invalid_round() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            manager: MANAGER.into(),
        },
    );

    let value = Random {
        randomness: (0, 0),
        signature: Vec::new(),
        prev_signature: Vec::new(),
    };

    let result = oracle_program.send(
        MANAGER,
        Action::SetRandomValue {
            round: 1,
            value: value.clone(),
        },
    );
    assert!(!result.main_failed());

    let result = oracle_program.send(MANAGER, Action::SetRandomValue { round: 1, value });
    assert!(result.main_failed());
}

#[test]
fn fail_update_manager_invalid_owner() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            manager: MANAGER.into(),
        },
    );

    let result = oracle_program.send(FAKE_OWNER, Action::UpdateManager(NEW_MANAGER.into()));
    assert!(result.main_failed());
}
