use super::{Horse, RunStatus};
use crate::utils::MAX_BPS;
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId, TypeInfo};

#[derive(Debug, Clone, Encode, Decode, TypeInfo, Hash)]
pub struct Run {
    pub start_timestamp: u64,
    pub end_bidding_timestamp: u64,
    pub horses: BTreeMap<String, (Horse, u128)>,
    pub bidders: BTreeMap<ActorId, (String, u128)>,
    pub status: RunStatus,
}

impl Run {
    pub fn progress(&mut self, oracle_round: u128) {
        self.status = RunStatus::InProgress { oracle_round };
    }

    pub fn cancel(&mut self) {
        self.status = RunStatus::Canceled;
    }

    pub fn finish(&mut self, seed: u128, run_id: u128) {
        let mut last_range_index = 0;
        let ranges: Vec<(u128, u128, String)> = self
            .horses
            .iter()
            .map(|(horse_name, (horse, _))| {
                let min = last_range_index;
                let max = last_range_index + horse.get_power();
                let result = (min, max, horse_name.to_owned());
                last_range_index = max;

                result
            })
            .collect();

        let winner_index = seed % (last_range_index.checked_add(1).expect("Math overflow!"));
        let winner_horse: String = {
            let mut result = String::from("");
            for (min, max, horse_name) in ranges {
                if winner_index >= min && winner_index <= max {
                    result = horse_name;
                    break;
                }
            }

            if !self.horses.contains_key(&result) {
                panic!("Winner not found!");
            }

            result
        };

        self.status = RunStatus::Finished {
            horse_name: winner_horse,
            run_id,
        };
    }

    /// Subtracts all funds from `user` and return amount.
    pub fn withdraw_all(&mut self, user: ActorId) -> u128 {
        let (_, amount) = self.bidders.get_mut(&user).expect("Bidder is not found!");
        let result = *amount;
        *amount = 0;

        result
    }

    /// Deposits `amount` to `user` and associated `Horse`.
    pub fn deposit(&mut self, user: ActorId, horse_name: &str, amount: u128) {
        self.bidders
            .entry(user)
            .and_modify(|(existing_horse_name, deposited_amount)| {
                if existing_horse_name != horse_name {
                    panic!("Provided horse didn't match bid horse!");
                }

                *deposited_amount = deposited_amount
                    .checked_add(amount)
                    .expect("Math overflow!");
            })
            .or_insert((horse_name.to_owned(), amount));

        let (_, horse_amount) = self
            .horses
            .get_mut(horse_name)
            .expect("Provided horse is not found!");
        *horse_amount = horse_amount.checked_add(amount).expect("Math overflow!");
    }

    pub fn sum_deposits_except_winner(&self) -> u128 {
        let (horse_name, _, _) = self.get_winner_horse().expect("Run is not finished!");
        let sum: u128 = self
            .horses
            .iter()
            .filter(|(name, _)| &horse_name != *name)
            .map(|(_, (_, amount))| *amount)
            .sum();

        sum
    }

    pub fn get_user_deposit_bps(&self, user: ActorId) -> Option<u128> {
        let (horse_name, user_amount) = self.bidders.get(&user)?;
        let (_, total_deposits) = self.horses.get(horse_name)?;

        Some(
            user_amount
                .checked_mul(MAX_BPS.into())
                .expect("Math overflow!")
                .checked_div(*total_deposits)
                .expect("Math overflow"),
        )
    }

    pub fn get_user_horse(&self, user: ActorId) -> Option<(String, Horse, u128)> {
        let (horse_name, amount) = self.bidders.get(&user)?;
        let (horse, _) = self.horses.get(horse_name)?;

        Some((horse_name.to_owned(), horse.clone(), *amount))
    }

    pub fn get_winner_horse(&self) -> Option<(String, Horse, u128)> {
        match &self.status {
            RunStatus::Finished {
                horse_name,
                run_id: _,
            } => {
                let (horse, amount) = self
                    .horses
                    .get(horse_name)
                    .expect("Winner horse is not found!");

                Some((horse_name.to_owned(), horse.clone(), *amount))
            }
            _ => None,
        }
    }
}
