#![no_std]
#![allow(clippy::missing_safety_doc)]

pub mod io;
pub mod state;
pub mod utils;

use gstd::{exec, msg, prelude::*, ActorId};
use io::{Action, Event, InitConfig, StateQuery, StateResponse};
use state::{Horse, Run, RunStatus};
use utils::{validate_fee_bps, MAX_BPS};

gstd::metadata! {
    title: "HorseRaces",
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
pub struct HorseRaces {
    pub runs: BTreeMap<u128, Run>,
    pub manager: ActorId,
    pub owner: ActorId,
    pub token: ActorId,
    pub oracle: ActorId,
    pub fee_bps: u16,
    pub run_nonce: u128,
}

impl HorseRaces {
    /// Updates current `fee_bps` for `new_fee_bps`,
    /// which will be used for charing comissions from users.
    ///
    /// - Checks that last `Run` is valid.
    ///
    /// - Only `manager` can call this function.
    pub fn update_fee_bps(&mut self, new_fee_bps: u16) {
        self.assert_manager();
        self.assert_last_run_ended();

        self.fee_bps = validate_fee_bps(new_fee_bps);
        msg::reply(Event::FeeBpsUpdated(new_fee_bps), 0).expect("Unable to reply!");
    }

    /// Updates current `manager` for `new_manager`,
    /// which will be used for calling service functions.
    ///
    /// - Checks that last `Run` is valid.
    ///
    /// - Only `manager` can call this function.
    pub fn update_manager(&mut self, new_manager: ActorId) {
        self.assert_manager();
        self.assert_last_run_ended();

        self.manager = new_manager;
        msg::reply(Event::ManagerUpdated(new_manager), 0).expect("Unable to reply!");
    }

    /// Updates current `oracle` for `new_oracle`,
    /// which will be used for random.
    ///
    /// - Checks that last `Run` is valid.
    ///
    /// - Only `manager` can call this function.
    pub fn update_oracle(&mut self, new_oracle: ActorId) {
        self.assert_manager();
        self.assert_last_run_ended();

        self.oracle = new_oracle;
        msg::reply(Event::OracleUpdated(new_oracle), 0).expect("Unable to reply!");
    }

    /// Change(move) current `Run` `status` to `InProgress`.
    /// At this stage we will expect oracle value.
    ///
    /// - Can be called after bidding time period.
    ///
    /// - Checks that last `Run` is valid.
    ///
    /// - Only `manager` can call this function.
    pub async fn progress_last_run(&mut self) {
        self.assert_manager();
        self.assert_last_run_bidding_finished();

        let current_run = self
            .runs
            .get_mut(&self.run_nonce)
            .expect("Last run is not found!");

        current_run.progress();

        let _oracle_reply: oracle_io::Event =
            msg::send_for_reply_as(self.oracle, oracle_io::Action::RequestValue, 0)
                .expect("Unable to request value from oracle!")
                .await
                .expect("Unable to decode oracle reply!");

        msg::reply(Event::LastRunProgressed(self.run_nonce), 0).expect("Unable to reply!");
    }

    /// Change(move) current `Run` `status` to `Canceled`.
    ///
    /// - Can be called after bidding time period.
    ///
    /// - Checks that last `Run` is valid.
    ///
    /// - Only `manager` can call this function.
    pub fn cancel_last_run(&mut self) {
        self.assert_manager();
        self.assert_last_run_bidding_finished();

        let current_run = self
            .runs
            .get_mut(&self.run_nonce)
            .expect("Last run is not found!");

        current_run.cancel();

        msg::reply(Event::LastRunCanceled(self.run_nonce), 0).expect("Unable to reply!");
    }

    /// Handle oracle result with random value.
    /// Picks horse randomly according to stats.
    ///
    /// - Checks that last `Run` is valid.
    pub fn finish_last_run(&mut self, seed: u128) {
        self.assert_oracle();
        self.assert_last_run_in_progress();

        let run_id = self.run_nonce;

        let current_run = self.runs.get_mut(&run_id).expect("Last run is not found!");

        current_run.finish(seed, run_id);

        msg::reply(
            Event::LastRunFinished {
                run_id,
                winner: current_run
                    .get_winner_horse()
                    .expect("Winner horse is not found!"),
            },
            0,
        )
        .expect("Unable to reply!");
    }

    /// Creates new run.
    ///
    /// - Checks that last `Run` is valid.
    ///
    /// - Only `manager` can call this function.
    pub fn create_run(&mut self, bidding_duration_ms: u64, horses: BTreeMap<String, Horse>) {
        self.assert_manager();
        self.assert_last_run_ended();

        self.run_nonce = self.run_nonce.checked_add(1).expect("Math overflow!");
        let id = self.run_nonce;

        let start_timestamp = exec::block_timestamp();
        let end_bidding_timestamp = start_timestamp
            .checked_add(bidding_duration_ms)
            .expect("Math overflow!");

        if self
            .runs
            .insert(
                id,
                Run {
                    start_timestamp,
                    end_bidding_timestamp,
                    horses: horses
                        .iter()
                        .map(|(horse_name, horse)| (horse_name.to_owned(), (horse.clone(), 0)))
                        .collect(),
                    bidders: BTreeMap::new(),
                    status: RunStatus::Created,
                },
            )
            .is_some()
        {
            panic!("Invalid ID!");
        }

        msg::reply(
            Event::RunCreated {
                run_id: id,
                bidding_duration_ms,
                horses,
            },
            0,
        )
        .expect("Unable to reply!");
    }

    /// Places new bid in last `Run`.
    /// Charges fees(`fee_bps`) from user.
    ///
    /// - Checks that last `Run` is valid.
    pub async fn bid(&mut self, horse_name: &str, amount: u128) {
        self.assert_not_manager();
        self.assert_last_run_bidding();

        // 1. Calculate fee amount with bps
        let fee_amount = amount
            .checked_mul(self.fee_bps.into())
            .expect("Math overflow!")
            .checked_div(MAX_BPS.into())
            .expect("Math overflow!");

        // 2. Calculate actual amount(subtract fee amount)
        let amount = amount.checked_sub(fee_amount).expect("Math overflow!");

        // 3. Track deposit amount
        let current_run = self
            .runs
            .get_mut(&self.run_nonce)
            .expect("Last run is not found!");

        current_run.deposit(msg::source(), horse_name, amount);

        // 4. Collect fee(transfer to `manager`)
        let _reply: ft_io::FTEvent = msg::send_for_reply_as(
            self.token,
            ft_io::FTAction::Transfer {
                from: msg::source(),
                to: self.manager,
                amount: fee_amount,
            },
            0,
        )
        .unwrap()
        .await
        .expect("Failed to transfer fee!");

        // 5. Transfer funds into vault
        let _reply: ft_io::FTEvent = msg::send_for_reply_as(
            self.token,
            ft_io::FTAction::Transfer {
                from: msg::source(),
                to: exec::program_id(),
                amount,
            },
            0,
        )
        .unwrap()
        .await
        .expect("Failed to transfer bid amount!");

        msg::reply(
            Event::NewBid {
                horse_name: horse_name.to_string(),
                amount,
            },
            0,
        )
        .expect("Unable to reply!");
    }

    /// Withdraw full deposited amount from
    /// canceled `Run`, which specified by `run_id`.
    ///
    /// - Checks, that provided `Run` is in canceled stage.
    pub async fn withdraw_canceled(&mut self, run_id: u128) {
        self.assert_canceled(run_id);

        let run = self.runs.get_mut(&run_id).expect("Run is not found!");
        let amount = run.withdraw_all(msg::source());

        let _reply: ft_io::FTEvent = msg::send_for_reply_as(
            self.token,
            ft_io::FTAction::Transfer {
                from: exec::program_id(),
                to: msg::source(),
                amount,
            },
            0,
        )
        .unwrap()
        .await
        .expect("Failed to transfer bid amount, to source!");

        msg::reply(
            Event::NewWithdrawCanceled {
                user: msg::source(),
                run_id,
                amount,
            },
            0,
        )
        .expect("Unable to reply!");
    }

    /// Withdraw full deposited amount from
    /// finished `Run`, which specified by `run_id`.
    ///
    /// - Checks, that provided `Run` is in finished stage.
    pub async fn withdraw_finished(&mut self, run_id: u128) {
        self.assert_finished(run_id);

        let run = self.runs.get_mut(&run_id).expect("Run is not found!");

        let user = msg::source();

        let (winner_horse_name, _, _) = run.get_winner_horse().expect("Winner horse is not found!");
        let (user_horse_name, _, user_deposit_amount) =
            run.get_user_horse(user).expect("Can't get user horse!");

        if winner_horse_name != user_horse_name {
            panic!("Sorry, but you lose!");
        }

        if user_deposit_amount == 0 {
            panic!("Bid amount is empty!");
        }

        // 1. Get user deposit percentage(bps)
        let user_deposit_bps = run
            .get_user_deposit_bps(user)
            .expect("Can't get user deposit percentage!");

        // 2. Get sum of all deposits(across all horses), except winner
        let total_deposits = run.sum_deposits_except_winner();

        // 3. Calculate profit amount
        let profit_amount = total_deposits
            .checked_mul(user_deposit_bps)
            .expect("Math overflow!")
            .checked_div(MAX_BPS.into())
            .expect("Math overflow!");

        let user_deposit_amount = run.withdraw_all(user);

        // 4. Transfer profits with bid amount
        let _reply: ft_io::FTEvent = msg::send_for_reply_as(
            self.token,
            ft_io::FTAction::Transfer {
                from: exec::program_id(),
                to: user,
                amount: user_deposit_amount
                    .checked_add(profit_amount)
                    .expect("Math overflow!"),
            },
            0,
        )
        .unwrap()
        .await
        .expect("Failed to transfer profits with bid amount!");

        msg::reply(
            Event::NewWithdrawFinished {
                user,
                run_id,
                amount: user_deposit_amount,
                profit_amount,
            },
            0,
        )
        .expect("Unable to reply!");
    }

    fn assert_manager(&self) {
        if self.manager != msg::source() {
            panic!("Only manager can call this!");
        }
    }

    fn assert_oracle(&self) {
        if self.oracle != msg::source() {
            panic!("Only oracle can call this!");
        }
    }

    fn assert_last_run_ended(&self) {
        if let Some(last_run) = self.get_last_run() {
            match last_run.status {
                RunStatus::Canceled
                | RunStatus::Finished {
                    horse_name: _,
                    run_id: _,
                } => {}
                _ => panic!("Last run is not ended!"),
            }
        }
    }

    fn assert_last_run_bidding(&self) {
        if let Some(last_run) = self.get_last_run() {
            match last_run.status {
                RunStatus::Created => {
                    let last_timestamp = exec::block_timestamp();

                    if last_run.end_bidding_timestamp <= last_timestamp {
                        panic!("Last run bidding stage is ended!");
                    }
                }
                _ => panic!("Last run stage is invalid!"),
            }
        } else {
            panic!("Last run is not found!");
        }
    }

    fn assert_last_run_bidding_finished(&self) {
        if let Some(last_run) = self.get_last_run() {
            match last_run.status {
                RunStatus::Created => {
                    let last_timestamp = exec::block_timestamp();

                    if last_run.end_bidding_timestamp > last_timestamp {
                        panic!("Last run bidding stage is not ended!");
                    }
                }
                _ => panic!("Last run stage is invalid!"),
            }
        } else {
            panic!("Last run is not found!");
        }
    }

    fn assert_last_run_in_progress(&self) {
        if let Some(last_run) = self.get_last_run() {
            match last_run.status {
                RunStatus::InProgress => {}
                _ => panic!("Last run stage is invalid!"),
            }
        } else {
            panic!("Last run is not found!");
        }
    }

    fn assert_canceled(&self, run_id: u128) {
        if let Some(run) = self.runs.get(&run_id) {
            match run.status {
                RunStatus::Canceled => {}
                _ => panic!("Run stage is invalid!"),
            }
        } else {
            panic!("Provided run id is invalid!");
        }
    }

    fn assert_finished(&self, run_id: u128) {
        if let Some(run) = self.runs.get(&run_id) {
            match run.status {
                RunStatus::Finished {
                    horse_name: _,
                    run_id: _,
                } => {}
                _ => panic!("Run stage is invalid!"),
            }
        } else {
            panic!("Provided run id is invalid!");
        }
    }

    fn assert_not_manager(&self) {
        if self.manager == msg::source() {
            panic!("Manager can't call this!");
        }
    }

    pub fn get_last_run(&self) -> Option<Run> {
        self.runs.get(&self.run_nonce).cloned()
    }

    pub fn get_horses(&self, run_id: u128) -> Vec<(String, Horse, u128)> {
        self.runs
            .get(&run_id)
            .expect("Run is not found!")
            .horses
            .iter()
            .map(|(name, (horse, amount))| (name.clone(), horse.clone(), *amount))
            .collect()
    }

    pub fn get_runs(&self) -> Vec<(u128, Run)> {
        self.runs
            .iter()
            .map(|(id, run)| (*id, run.clone()))
            .collect()
    }
}

static mut HORSE_RACES: Option<HorseRaces> = None;

#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig.");
    let horse_races = HorseRaces {
        manager: config.manager,
        owner: msg::source(),
        token: config.token,
        oracle: config.oracle,
        fee_bps: validate_fee_bps(config.fee_bps),
        ..Default::default()
    };

    HORSE_RACES = Some(horse_races);
}

#[gstd::async_main]
async fn main() {
    let horse_races: &mut HorseRaces = unsafe { HORSE_RACES.get_or_insert(HorseRaces::default()) };

    // Handler(proxy) for oracle messages
    if msg::source() == horse_races.oracle {
        let payload = msg::load_bytes();
        let _id: u128 = u128::from_le_bytes(payload[1..17].try_into().unwrap());
        let seed: u128 = u128::from_le_bytes(payload[17..].try_into().unwrap());

        horse_races.finish_last_run(seed);
        return;
    }

    let action: Action = msg::load().expect("Unable to decode Action.");
    match action {
        Action::UpdateFeeBps(new_fee_bps) => horse_races.update_fee_bps(new_fee_bps),
        Action::UpdateManager(new_manager) => horse_races.update_manager(new_manager),
        Action::UpdateOracle(new_oracle) => horse_races.update_oracle(new_oracle),
        Action::ProgressLastRun => horse_races.progress_last_run().await,
        Action::CancelLastRun => horse_races.cancel_last_run(),
        Action::CreateRun {
            bidding_duration_ms,
            horses,
        } => horse_races.create_run(bidding_duration_ms, horses),
        Action::Bid { horse_name, amount } => horse_races.bid(&horse_name, amount).await,
        Action::WithdrawCanceled(run_id) => horse_races.withdraw_canceled(run_id).await,
        Action::WithdrawFinished(run_id) => horse_races.withdraw_finished(run_id).await,
    }
}

#[no_mangle]
unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state_query: StateQuery = msg::load().expect("Unable to decode StateQuery.");
    let horse_races = HORSE_RACES.get_or_insert(Default::default());

    let encoded = match state_query {
        StateQuery::GetRuns => StateResponse::Runs(horse_races.get_runs()),
        StateQuery::GetHorses(run_id) => StateResponse::Horses(horse_races.get_horses(run_id)),
        StateQuery::GetManager => StateResponse::Manager(horse_races.manager),
        StateQuery::GetOwner => StateResponse::Owner(horse_races.owner),
        StateQuery::GetToken => StateResponse::Token(horse_races.token),
        StateQuery::GetOracle => StateResponse::Oracle(horse_races.oracle),
        StateQuery::GetFeeBps => StateResponse::FeeBps(horse_races.fee_bps),
        StateQuery::GetRunNonce => StateResponse::RunNonce(horse_races.run_nonce),
        StateQuery::GetRun(run_id) => StateResponse::Run(
            horse_races
                .runs
                .get(&run_id)
                .expect("Run is not found!")
                .clone(),
        ),
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
