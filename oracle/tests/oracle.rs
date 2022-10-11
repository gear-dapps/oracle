mod utils;

use codec::Encode;
use gstd::MessageId;
use gtest::{Log, System};
use oracle_io::*;
use utils::*;

// use gear_core::{
//     ids::{MessageId, ProgramId},
//     message::{Dispatch, DispatchKind, Message, ReplyDetails, StoredMessage},
// };

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

    let mailbox = sys.get_mailbox(MANAGER);
    let message_log = Log::builder()
        .source(oracle_program.id())
        .dest(MANAGER)
        .payload(0i32);
    mailbox.reply(message_log.clone(), 0i32, 0);
   
    let user_mailbox = sys.get_mailbox(USER);

    let message = Log::builder().source(oracle_program.id()).dest(USER);
    //  .payload(Event::NewValue { value: 1337u128 });
    assert!(user_mailbox.contains(&message));
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
