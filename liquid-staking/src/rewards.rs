multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use super::{common, constants::*, delegation, errors::*, events, proxies, score, selection, storage};

#[multiversx_sc::module]
pub trait RewardsModule:
    admin::AdminModule
    + common::CommonModule
    + events::EventsModule
    + delegation::DelegationModule
    + proxies::ProxyModule
    + score::ScoreModule
    + selection::SelectionModule
    + storage::StorageModule
{
    /// Allows anyone to claim rewards from a given Delegation smart contract.
    ///
    /// # Arguments
    ///
    /// - `delegation_contract` - the Delegation smart contract address
    ///
    #[endpoint(claimRewardsFrom)]
    fn claim_rewards_from(&self, delegation_contract: ManagedAddress) {
        self.require_active_state();

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        require!(
            !self.has_claimed_rewards(&delegation_contract),
            ERROR_REWARDS_ALREADY_CLAIMED
        );

        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call();
        let callback = self.callbacks().claim_rewards_from_cb(&caller, &delegation_contract);
        self.claim_rewards_from_delegation_contract(delegation_contract, gas_for_async_call, callback);
    }

    #[callback]
    fn claim_rewards_from_cb(
        &self,
        caller: &ManagedAddress,
        delegation_contract: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let claimed_rewards = self.call_value().egld_value().clone_value();

                let contract_data_mapper = self.delegation_contract_data(delegation_contract);
                let contract_data = contract_data_mapper.get();
                let service_fee = &contract_data.service_fee;
                let total_fee = self.total_fee().get();

                // ideally, this should never happen
                let (reserves, rewards) = if service_fee >= &total_fee {
                    let reserves = BigUint::zero();
                    let rewards = claimed_rewards;
                    (reserves, rewards)
                } else {
                    let bps = BigUint::from(BPS);
                    let protocol_fee = (&total_fee - service_fee) * &bps / (&bps - service_fee);
                    let reserves = protocol_fee * &claimed_rewards / &bps;
                    let rewards = claimed_rewards - &reserves;
                    (reserves, rewards)
                };

                self.rewards_reserve().update(|_rewards| *_rewards += &rewards);
                self.protocol_reserve().update(|_reserves| *_reserves += &reserves);

                let current_epoch = self.blockchain().get_block_epoch();
                self.last_rewards_claim_epoch(delegation_contract).update(|_epoch| {
                    if current_epoch > *_epoch {
                        *_epoch = current_epoch;
                    }
                });

                self.claim_rewards_from_event(caller, delegation_contract, &reserves, &rewards, &contract_data);
            },
            ManagedAsyncCallResult::Err(err) => {
                self.async_call_error_event(err.err_code, err.err_msg);
            },
        }
    }

    /// Allows anyone to delegate EGLD rewards balance to a staking provider based on the current configuration of the
    /// delegation algorithm. If the delegation is successful, the callback updates the storage. Otherwise, it sets the
    /// Delegation smart contract data as outdated. Notice that the smart contract data will be outdated until it is updated.
    ///
    /// # Arguments
    ///
    /// - `opt_egld_amount` - an optional amount of EGLD from the rewards balance to delegate
    ///
    #[endpoint(delegateRewards)]
    fn delegate_rewards(&self, opt_egld_amount: OptionalValue<BigUint>) {
        self.require_active_state();

        let caller = self.blockchain().get_caller();

        let rewards_reserve = self.rewards_reserve().get();
        self.require_sufficient_egld(&rewards_reserve);

        let egld_amount = match opt_egld_amount {
            OptionalValue::None => rewards_reserve,
            OptionalValue::Some(amount) => {
                self.require_sufficient_egld(&amount);
                require!(amount <= rewards_reserve, ERROR_TOO_MUCH_EGLD_AMOUNT);
                let amount_left = rewards_reserve - &amount;
                self.require_no_dust_left(&amount_left);
                amount
            },
        };

        self.rewards_reserve().update(|amount| *amount -= &egld_amount);

        let delegation_contract = self.get_delegation_contract_for_delegate(&egld_amount, &OptionalValue::None);

        let gas_for_async_call = self.get_gas_for_async_call();
        let callback = self
            .callbacks()
            .delegate_rewards_cb(&caller, &delegation_contract, &egld_amount);
        self.delegate_to_delegation_contract(delegation_contract, egld_amount, gas_for_async_call, callback);
    }

    #[callback]
    fn delegate_rewards_cb(
        &self,
        caller: &ManagedAddress,
        delegation_contract: &ManagedAddress,
        egld_amount: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let contract_data_mapper = self.delegation_contract_data(delegation_contract);
                contract_data_mapper.update(|data| {
                    data.total_delegated += egld_amount;
                });
                self.cash_reserve().update(|amount| *amount += egld_amount);
                let contract_data = contract_data_mapper.get();
                self.delegate_rewards_event(caller, delegation_contract, egld_amount, &contract_data)
            },
            ManagedAsyncCallResult::Err(err) => {
                self.rewards_reserve().update(|amount| *amount += egld_amount);
                self.delegation_contract_data(delegation_contract).update(|data| {
                    data.outdated = true;
                });
                self.outdated_event(delegation_contract);
                self.async_call_error_event(err.err_code, err.err_msg);
            },
        }
    }
}
