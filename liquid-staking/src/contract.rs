#![no_std]
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub use admin;
use constants::NO_DATA_UPDATE_EPOCHS;

pub mod common;
pub mod constants;
pub mod delegate;
pub mod delegation;
pub mod errors;
pub mod events;
pub mod governance;
pub mod migration;
pub mod model;
pub mod penalty;
pub mod proxies;
pub mod rewards;
pub mod score;
pub mod selection;
pub mod storage;
pub mod undelegate;
pub mod withdraw;

use crate::{constants::NO_UNDELEGATE_EPOCHS, model::*};

#[multiversx_sc::contract]
pub trait LiquidStaking:
    admin::AdminModule
    + common::CommonModule
    + delegate::DelegateModule
    + delegation::DelegationModule
    + events::EventsModule
    + governance::GovernanceModule
    + migration::MigrationModule
    + rewards::RewardsModule
    + score::ScoreModule
    + selection::SelectionModule
    + storage::StorageModule
    + undelegate::UndelegateModule
    + withdraw::WithdrawModule
    + penalty::PenaltyModule
    + proxies::ProxyModule
{
    /// Initializes the contract.
    ///
    /// # Arguments
    ///
    /// - `unbond_period` - the unbond period in epochs. Devnet has an unbond period of 1 epoch while Mainnet has an
    ///   unbond period of 10 epochs
    /// - `opt_admin` - the optional admin address
    ///
    #[init]
    fn init(&self, unbond_period: u64, opt_admin: OptionalValue<ManagedAddress>) {
        // try set unbond period
        self.try_set_unbond_period(unbond_period);

        // try set undelegation mode
        self.try_set_undelegation_mode(UndelegationMode::Algorithm);

        // try set last undelegate epoch (with a buffer)
        let undelegate_buffer = 2 * NO_UNDELEGATE_EPOCHS;
        let current_epoch = self.blockchain().get_block_epoch();
        self.try_set_last_undelegate_epoch(current_epoch + undelegate_buffer);

        // try set last contract data update epoch (with a buffer)
        let data_buffer = 2 * NO_DATA_UPDATE_EPOCHS;
        self.try_set_last_contract_data_update_epoch(current_epoch + data_buffer);

        // try set admin
        self.try_set_admin(opt_admin);

        // protocol status
        self.state().set(State::Inactive);
    }

    #[upgrade]
    fn upgrade(&self) {
        self.state().set(State::Inactive);
    }
}
