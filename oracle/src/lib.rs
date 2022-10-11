#![no_std]
#![allow(clippy::missing_safety_doc)]

use gstd::{async_main, debug, msg, prelude::*, ActorId};
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
    pub owner: ActorId,
    pub manager: ActorId,
}

impl Oracle {
    pub async fn request_value(&mut self) {
        debug!("Before sending message to manager");
        let value = msg::send_for_reply_as(self.manager, 0i32, 0)
            .expect("Can't send message for update value")
            .await
            .expect("Can't obtain updated value");
        debug!("After sending message to manager");
        msg::reply(Event::NewValue { value }, 0).expect("Unable to reply!");
    }

    pub fn change_manager(&mut self, new_manager: ActorId) {
        self.assert_only_owner();

        self.manager = new_manager;

        msg::reply(Event::NewManager(new_manager), 0).expect("Unable to reply!");
    }

    pub fn assert_only_owner(&self) {
        if msg::source() != self.owner {
            panic!("Only owner allowed to call this function!");
        }
    }
}

static mut ORACLE: Option<Oracle> = None;

#[async_main]
async fn main() {
    let action: Action = msg::load().expect("Unable to decode Action.");
    let oracle: &mut Oracle = unsafe { ORACLE.get_or_insert(Oracle::default()) };

    match action {
        Action::RequestValue => oracle.request_value().await,
        Action::ChangeManager(new_manager) => oracle.change_manager(new_manager),
    }
}

#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig.");
    let oracle = Oracle {
        owner: config.owner,
        manager: config.manager,
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
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}
