#![no_std]
#![allow(clippy::missing_safety_doc)]

use gstd::{async_main, msg, prelude::*, ActorId};
use oracle_io::{Action, Event, InitConfig, StateQuery, StateResponse};

gstd::metadata! {
    title: "Oracle",
    init:
        input: InitConfig,
    handle:
        input: Action,
        output: Event,
    state:
        input: StateQuery,
        output: StateResponse,
}

#[derive(Debug, Default)]
pub struct Oracle {
    pub requests_queue: BTreeMap<u128, ActorId>,
    pub owner: ActorId,
    pub manager: ActorId,
    pub id_nonce: u128,
}

impl Oracle {
    pub fn request_value(&mut self) {
        self.id_nonce = self.id_nonce.checked_add(1).expect("Math overflow!");
        let id = self.id_nonce;

        let program = msg::source();

        if self.requests_queue.insert(id, program).is_some() {
            panic!("Invalid queue nonce!");
        }

        // Emit request with id from queue
        msg::reply(
            Event::NewUpdateRequest {
                id,
                caller: program,
            },
            0,
        )
        .unwrap();
    }

    pub fn change_manager(&mut self, new_manager: ActorId) {
        if msg::source() != self.owner {
            panic!("Only owner allowed to call this function!");
        }

        self.manager = new_manager;

        msg::reply(Event::NewManager(new_manager), 0).unwrap();
    }

    pub async fn update_value(&mut self, id: u128, value: u128) {
        if msg::source() != self.manager {
            panic!("Only manager allowed to call this function!");
        }

        let callback_program = *self
            .requests_queue
            .get(&id)
            .expect("Provided ID not found in requests queue!");

        if self.requests_queue.remove(&id).is_none() {
            panic!("Provided ID not found in requests queue!");
        }

        // Callback program with value
        let _callback_result = msg::send_for_reply(callback_program, (id, value).encode(), 0)
            .expect("Unable to send async callback!")
            .await;
    }
}

static mut ORACLE: Option<Oracle> = None;

#[async_main]
async fn main() {
    let action: Action = msg::load().expect("Unable to decode Action.");
    let oracle: &mut Oracle = unsafe { ORACLE.get_or_insert(Oracle::default()) };

    match action {
        Action::RequestValue => oracle.request_value(),
        Action::ChangeManager(new_manager) => oracle.change_manager(new_manager),
        Action::UpdateValue { id, value } => oracle.update_value(id, value).await,
    }
}

#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig.");
    let oracle = Oracle {
        owner: config.owner,
        manager: config.manager,
        ..Default::default()
    };

    ORACLE = Some(oracle);
}

#[no_mangle]
unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state_query: StateQuery = msg::load().expect("Unable to decode StateQuery.");
    let oracle = ORACLE.get_or_insert(Default::default());

    let encoded = match state_query {
        StateQuery::GetOwner => StateResponse::Owner(oracle.owner),
        StateQuery::GetManager => StateResponse::Manager(oracle.manager),
        StateQuery::GetRequestsQueue => StateResponse::RequestsQueue(
            oracle
                .requests_queue
                .iter()
                .map(|(id, callback_program)| (*id, *callback_program))
                .collect::<Vec<(u128, ActorId)>>(),
        ),
        StateQuery::GetIdNonce => StateResponse::IdNonce(oracle.id_nonce),
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtest::{Program, System};

    #[test]
    fn success_init() {
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::current(&sys);

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
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;
        const NEW_MANAGER: u64 = 5;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::current(&sys);

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
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;
        const USER: u64 = 5;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::current(&sys);

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
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;
        const USER: u64 = 5;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::current(&sys);

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
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;
        const USER: u64 = 5;
        const FAKE_MANAGER: u64 = 6;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::current(&sys);

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
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;
        const USER: u64 = 5;
        const FAKE_MANAGER: u64 = 6;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::current(&sys);

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
        const OWNER: u64 = 3;
        const MANAGER: u64 = 4;
        const FAKE_OWNER: u64 = 5;

        let sys = System::new();
        sys.init_logger();

        let oracle_program = Program::current(&sys);

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
}
