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
        // TODO: Check events with backend
        msg::reply(
            Event::NewUpdateRequest {
                id,
                caller: program,
            },
            0,
        )
        .expect("Unable to reply!");
    }

    pub fn change_manager(&mut self, new_manager: ActorId) {
        self.assert_only_owner();

        self.manager = new_manager;

        msg::reply(Event::NewManager(new_manager), 0).expect("Unable to reply!");
    }

    pub async fn update_value(&mut self, id: u128, value: u128) {
        self.assert_only_manager();

        let callback_program = *self
            .requests_queue
            .get(&id)
            .expect("Provided ID not found in requests queue!");

        if self.requests_queue.remove(&id).is_none() {
            panic!("Provided ID not found in requests queue!");
        }

        // Callback program with value
        msg::send(callback_program, (id, value).encode(), 0)
            .expect("Unable to send async callback!");
    }

    pub fn assert_only_owner(&self) {
        if msg::source() != self.owner {
            panic!("Only owner allowed to call this function!");
        }
    }

    pub fn assert_only_manager(&self) {
        if msg::source() != self.manager {
            panic!("Only manager allowed to call this function!");
        }
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
