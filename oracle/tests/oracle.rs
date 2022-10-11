mod utils;

use codec::Encode;
use gtest::{Log, System};
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

    sys.mint_to(MANAGER, 1000000000000);
    sys.mint_to(oracle_program.id(), 1000000000000);

    sys.spend_blocks(10);

    let result = oracle_program.send(USER, Action::RequestValue);
    assert!(!result.log().is_empty());
    assert!(!result.main_failed());
    assert!(!result.others_failed());

    sys.spend_blocks(10);

    let mailbox = sys.get_mailbox(MANAGER);
    let msg_replier = mailbox.take_message(
        Log::builder()
            .source(oracle_program.id())
            .dest(MANAGER)
            .payload(0i32),
    );
    let result = msg_replier.reply(1337u128, 0);
    assert!(!result.main_failed());
    assert!(!result.others_failed());
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

    let result = oracle_program.send(FAKE_OWNER, Action::ChangeManager(FAKE_MANAGER.into()));
    assert!(result.main_failed());
}
