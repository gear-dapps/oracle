mod utils;

use codec::Encode;
use gtest::System;
use oracle_io::*;
use utils::*;

#[test]
fn success_init() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    let result = oracle_program.send(
        OWNER,
        InitConfig {
            owner: OWNER.into(),
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

    let meta_result: StateResponse = oracle_program
        .meta_state(StateQuery::GetRequestsQueue)
        .unwrap();
    match meta_result {
        StateResponse::RequestsQueue(requests_queue) => assert!(requests_queue.is_empty()),
        _ => panic!("Invalid StateResponse!"),
    }

    let meta_result: StateResponse = oracle_program.meta_state(StateQuery::GetIdNonce).unwrap();
    match meta_result {
        StateResponse::IdNonce(id_nonce) => assert_eq!(id_nonce, 0),
        _ => panic!("Invalid StateResponse!"),
    }
}

#[test]
fn success_change_manager() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            owner: OWNER.into(),
            manager: MANAGER.into(),
        },
    );

    let result = oracle_program.send(OWNER, Action::ChangeManager(NEW_MANAGER.into()));
    assert!(result.contains(&(OWNER, Event::NewManager(NEW_MANAGER.into()).encode())));

    let result = oracle_program.send(OWNER, Action::ChangeManager(OWNER.into()));
    assert!(result.contains(&(OWNER, Event::NewManager(OWNER.into()).encode())));
}

#[test]
fn success_request_value() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            owner: OWNER.into(),
            manager: MANAGER.into(),
        },
    );

    let result = oracle_program.send(USER, Action::RequestValue);
    assert!(!result.main_failed());
    assert!(result.contains(&(
        USER,
        Event::NewUpdateRequest {
            id: 1,
            caller: USER.into()
        }
        .encode()
    )));

    let meta_result: StateResponse = oracle_program
        .meta_state(StateQuery::GetRequestsQueue)
        .unwrap();
    match meta_result {
        StateResponse::RequestsQueue(requests_queue) => {
            assert_eq!(requests_queue, vec![(1, USER.into())])
        }
        _ => panic!("Invalid StateResponse!"),
    }
}

#[test]
fn success_update_value() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            owner: OWNER.into(),
            manager: MANAGER.into(),
        },
    );

    let result = oracle_program.send(USER, Action::RequestValue);
    assert!(!result.main_failed());
    assert!(result.contains(&(
        USER,
        Event::NewUpdateRequest {
            id: 1,
            caller: USER.into()
        }
        .encode()
    )));

    let meta_result: StateResponse = oracle_program
        .meta_state(StateQuery::GetRequestsQueue)
        .unwrap();
    match meta_result {
        StateResponse::RequestsQueue(requests_queue) => {
            assert_eq!(requests_queue, vec![(1, USER.into())])
        }
        _ => panic!("Invalid StateResponse!"),
    }

    sys.spend_blocks(100);

    let result = oracle_program.send(MANAGER, Action::UpdateValue { id: 1, value: 1337 });
    assert!(!result.main_failed());
    assert!(!result.others_failed());

    let meta_result: StateResponse = oracle_program
        .meta_state(StateQuery::GetRequestsQueue)
        .unwrap();
    match meta_result {
        StateResponse::RequestsQueue(requests_queue) => assert!(requests_queue.is_empty()),
        _ => panic!("Invalid StateResponse!"),
    };
}

#[test]
fn fail_update_value_invalid_manager() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            owner: OWNER.into(),
            manager: MANAGER.into(),
        },
    );

    let result = oracle_program.send(USER, Action::RequestValue);
    assert!(!result.main_failed());
    assert!(result.contains(&(
        USER,
        Event::NewUpdateRequest {
            id: 1,
            caller: USER.into()
        }
        .encode()
    )));

    let result = oracle_program.send(FAKE_MANAGER, Action::UpdateValue { id: 1, value: 1337 });
    assert!(result.main_failed());
}

#[test]
fn fail_update_value_invalid_id() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            owner: OWNER.into(),
            manager: MANAGER.into(),
        },
    );

    let result = oracle_program.send(USER, Action::RequestValue);
    assert!(!result.main_failed());
    assert!(result.contains(&(
        USER,
        Event::NewUpdateRequest {
            id: 1,
            caller: USER.into()
        }
        .encode()
    )));

    let result = oracle_program.send(
        FAKE_MANAGER,
        Action::UpdateValue {
            id: 1337,
            value: 1337,
        },
    );
    assert!(result.main_failed());
}

#[test]
fn fail_change_manager_invalid_owner() {
    let sys = System::new();
    let oracle_program = load_program(&sys);

    oracle_program.send(
        OWNER,
        InitConfig {
            owner: OWNER.into(),
            manager: MANAGER.into(),
        },
    );

    let result = oracle_program.send(FAKE_OWNER, Action::ChangeManager(MANAGER.into()));
    assert!(result.main_failed());
}
