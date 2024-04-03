multiversx_sc::imports!();
use super::{
    constants::*,
    errors::*,
    events,
    model::{State, UndelegateAttributes, UndelegationMode},
    proxies, storage,
};

#[multiversx_sc::module]
pub trait CommonModule:
    admin::AdminModule + events::EventsModule + storage::StorageModule + proxies::ProxyModule
{
    // Checks

    #[view(isLiquidStaking)]
    fn is_liquid_staking(&self) -> bool {
        true
    }

    #[view(isActive)]
    fn is_active(&self) -> bool {
        self.state().get() == State::Active
    }

    /// Verifies if rewards have been already claimed for a given Delegation smart contract
    ///
    fn has_claimed_rewards(&self, delegation_contract: &ManagedAddress) -> bool {
        let current_epoch = self.blockchain().get_block_epoch();
        let last_epoch = self.get_last_rewards_claim_epoch(delegation_contract);
        current_epoch <= last_epoch
    }

    /// Verifies if the provided address is a Delegation smart contract based on its shard
    ///
    fn is_delegation_contract_sc(&self, delegation_contract: &ManagedAddress) -> bool {
        self.blockchain().is_smart_contract(delegation_contract)
            && self.blockchain().get_shard_of_address(delegation_contract) == u32::MAX
    }

    /// Verifies if the given Delegation smart contract is in the list
    ///
    fn is_delegation_contract_in_list(&self, delegation_contract: &ManagedAddress) -> bool {
        for node in self.delegation_contracts_list().iter() {
            let delegation_contract_in_list = node.into_value();
            if delegation_contract == &delegation_contract_in_list {
                return true;
            }
        }
        false
    }

    // Requires

    #[inline]
    fn require_active_state(&self) {
        require!(self.is_active(), ERROR_INACTIVE);
    }

    #[inline]
    fn require_sufficient_egld(&self, egld_amount: &BigUint) {
        require!(
            egld_amount >= &BigUint::from(MIN_DELEGATION_AMOUNT),
            ERROR_INSUFFICIENT_EGLD_AMOUNT
        );
    }

    #[inline]
    fn require_delegation_contract(&self, delegation_contract: &ManagedAddress) {
        require!(
            self.is_delegation_contract_sc(delegation_contract),
            ERROR_INVALID_DELEGATION_CONTRACT
        );
    }

    fn require_valid_shares_payment(&self, ls_token_id: &TokenIdentifier, shares: &BigUint) {
        require!(ls_token_id == &self.ls_token().get_token_id(), ERROR_UNEXPECTED_PAYMENT);
        require!(shares > &BigUint::zero(), ERROR_INSUFFICIENT_SHARES);
    }

    #[inline]
    fn require_valid_penalty_id(&self, penalty_id: u64) {
        require!(!self.penalties(penalty_id).is_empty(), ERROR_UNEXPECTED_PENALTY_ID);
    }

    #[inline]
    fn require_open_mode(&self) {
        require!(
            self.undelegation_mode().get() == UndelegationMode::Open,
            ERROR_ONLY_FREE_UNDELEGATION_MODE
        );
    }

    fn require_open_mode_or_admin(&self) {
        if self.undelegation_mode().get() == UndelegationMode::Open {
            return;
        }
        self.require_admin();
    }

    #[inline]
    fn require_data_manager(&self) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.data_manager().get(), ERROR_ONLY_DATA_MANAGER);
    }

    #[inline]
    fn require_no_dust_left(&self, egld_amount: &BigUint) {
        require!(
            egld_amount == &BigUint::zero() || egld_amount >= &BigUint::from(MIN_DELEGATION_AMOUNT),
            ERROR_WOULD_LEAVE_DUST
        );
    }

    // Sets

    /// Tries to set the unbond period if it has not been previously set
    ///
    fn try_set_unbond_period(&self, unbond_period: u64) {
        if self.unbond_period().is_empty() {
            require!(
                unbond_period == DEVNET_UNBOND_PERIOD || unbond_period == MAINNET_UNBOND_PERIOD,
                ERROR_INVALID_UNBOND_PERIOD
            );
            self.unbond_period().set(unbond_period);
            self.set_unbond_period_event(unbond_period);
        }
    }

    /// Tries to set the undelegation mode if it has not been previously set
    ///
    fn try_set_undelegation_mode(&self, undelegation_mode: UndelegationMode) {
        if self.undelegation_mode().is_empty() {
            self.set_undelegation_mode_internal(undelegation_mode);
        }
    }

    fn set_undelegation_mode_internal(&self, undelegation_mode: UndelegationMode) {
        self.undelegation_mode().set(undelegation_mode);
        self.set_undelegation_mode_event(undelegation_mode);
    }

    /// Tries to set the last undelegate epoch when the undelegation mode is of type Algorithm
    ///
    fn try_set_last_undelegate_epoch(&self, last_undelegate_epoch: u64) {
        if self.last_undelegate_epoch().is_empty() {
            self.set_last_undelegate_epoch_internal(last_undelegate_epoch);
        }
    }

    fn set_last_undelegate_epoch_internal(&self, last_undelegate_epoch: u64) {
        self.last_undelegate_epoch().update(|_epoch| {
            if last_undelegate_epoch > *_epoch {
                *_epoch = last_undelegate_epoch;
                self.set_last_undelegate_epoch_event(last_undelegate_epoch);
            }
        });
    }

    /// Tries to set the last contract data update epoch
    ///
    fn try_set_last_contract_data_update_epoch(&self, last_contract_data_update_epoch: u64) {
        if self.last_contract_data_update_epoch().is_empty() {
            self.set_last_contract_data_update_epoch_internal(last_contract_data_update_epoch);
        }
    }

    fn set_last_contract_data_update_epoch_internal(&self, last_contract_data_update_epoch: u64) {
        self.last_contract_data_update_epoch().update(|_epoch| {
            if last_contract_data_update_epoch > *_epoch {
                *_epoch = last_contract_data_update_epoch;
                self.set_last_contract_data_update_epoch_event(last_contract_data_update_epoch);
            }
        });
    }

    // Gets

    /// Returns the liquid staking token identifier
    ///
    #[view(getLsTokenId)]
    fn get_ls_token_id(&self) -> TokenIdentifier {
        self.ls_token().get_token_id()
    }

    /// Returns the last epoch a successful undelegation occur from any Delegation smart contract or its initial value
    /// set at deployment
    ///
    fn get_last_undelegate_epoch(&self) -> u64 {
        self.last_undelegate_epoch().get()
    }

    /// Returns the last epoch a successful Delegation smart contract data update occur  or its initial value set at
    /// deployment
    ///
    fn get_last_contract_data_update_epoch(&self) -> u64 {
        self.last_contract_data_update_epoch().get()
    }

    /// Returns the last epoch rewards have been claimed for a given Delegation smart contract or 0 if the rewards have
    /// not been claimed yet
    ///
    fn get_last_rewards_claim_epoch(&self, delegation_contract: &ManagedAddress) -> u64 {
        let last_claim_epoch = self.last_rewards_claim_epoch(delegation_contract);
        if last_claim_epoch.is_empty() {
            0u64
        } else {
            last_claim_epoch.get()
        }
    }

    /// Returns the next consecutive penalty identifier starting from 0.
    ///
    fn get_next_penalty_id(&self) -> u64 {
        let next_penalty_id = self.next_penalty_id();

        // get current next
        let penalty_id = if next_penalty_id.is_empty() {
            0u64
        } else {
            next_penalty_id.get()
        };

        // update for next call
        next_penalty_id.update(|_id| *_id += 1u64);

        penalty_id
    }

    /// Returns the data manager address if set
    ///
    fn get_data_manager(&self) -> Option<ManagedAddress> {
        let data_manager_mapper = self.data_manager();
        if data_manager_mapper.is_empty() {
            None
        } else {
            let data_manager = data_manager_mapper.get();
            Some(data_manager)
        }
    }

    /// Returns the gas for the async transaction making sure it is enough
    ///
    fn get_gas_for_async_call(&self) -> u64 {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK,
            ERROR_INSUFFICIENT_GAS
        );
        gas_left - MIN_GAS_FOR_CALLBACK
    }

    /// Computes the current exchange rate in WAD between EGLD and sEGLD
    ///
    #[view(getExchangeRate)]
    fn get_exchange_rate(&self) -> BigUint {
        let wad = BigUint::from(WAD);

        let ls_token_supply = self.ls_token_supply().get();

        // The initial exchange rate between EGLD and sEGLD is fixed to one
        if ls_token_supply == BigUint::zero() {
            return BigUint::from(INITIAL_EXCHANGE_RATE);
        }

        let cash = self.cash_reserve().get();

        cash * wad / ls_token_supply
    }

    /// Translates an amount of EGLD into sEGLD based on the current exchange rate
    ///
    fn egld_to_shares(&self, egld_amount: &BigUint) -> BigUint {
        let wad = BigUint::from(WAD);
        let fx = self.get_exchange_rate();
        egld_amount * &wad / fx
    }

    /// Translates an amount of sEGLD into EGLD based on the current exchange rate
    ///
    fn shares_to_egld(&self, shares: &BigUint) -> BigUint {
        let wad = BigUint::from(WAD);
        let fx = self.get_exchange_rate();
        fx * shares / wad
    }

    /// Mints a given amount of sEGLD
    ///
    fn mint_ls_token(&self, amount: BigUint) -> EsdtTokenPayment<Self::Api> {
        self.ls_token().mint(amount)
    }

    /// Burns a given amount of sEGLD
    ///
    fn burn_ls_token(&self, amount: &BigUint) {
        self.ls_token().burn(amount);
    }

    /// Mints a given amount of sEGLD and updates the pertinent storages
    ///
    fn mint_shares(&self, egld_amount: &BigUint) -> EsdtTokenPayment {
        let shares = self.egld_to_shares(egld_amount);
        require!(shares > BigUint::zero(), ERROR_INSUFFICIENT_SHARES);

        self.cash_reserve().update(|amount| *amount += egld_amount);
        self.ls_token_supply().update(|_shares| *_shares += &shares);

        self.mint_ls_token(shares)
    }

    /// Burns a given amount of sEGLD and updates the pertinent storages
    ///
    fn redeem_shares(&self, egld_amount: &BigUint, shares: &BigUint) {
        self.cash_reserve().update(|amount| *amount -= egld_amount);
        self.ls_token_supply().update(|_shares| *_shares -= shares);
        self.burn_ls_token(shares);
    }

    /// Mints an undelegation NFT with given attributes
    ///
    fn mint_undelegate_nft<T: TopEncode>(&self, attributes: &T) -> EsdtTokenPayment<Self::Api> {
        let token_id = self.undelegate_token().get_token_id();
        let amount = BigUint::from(1u64);
        let token_name = self.undelegate_token_name().get();
        let uri = ManagedBuffer::from(UNDELEGATE_TOKEN_URI);
        let uris = ManagedVec::from_single_item(uri);

        let token_nonce = self.send().esdt_nft_create(
            &token_id,
            &amount,
            &token_name,
            &BigUint::zero(),
            &ManagedBuffer::new(),
            attributes,
            &uris,
        );

        EsdtTokenPayment::new(token_id, token_nonce, amount)
    }

    /// Burns an undelegation NFT with a given nonce
    ///
    fn burn_undelegate_nft(&self, token_nonce: u64) {
        self.undelegate_token().nft_burn(token_nonce, &BigUint::from(1u64));
    }

    /// Computes a linear function on a domain given by `min` and `max`. Also, this linear function has a bounded image
    /// between zero and one (in basis points). Finally, if `down` is true, the line has a negative slope.
    ///
    fn norm_linear_clamp(&self, x: &BigUint, min: &BigUint, max: &BigUint, down: bool) -> BigUint {
        require!(max > min, ERROR_INVALID_DOMAIN);

        let zero = BigUint::zero();
        let bps = BigUint::from(BPS);

        let mut y = if x <= min {
            zero
        } else if x >= max {
            bps.clone()
        } else {
            (x - min) * &bps / (max - min)
        };

        if down {
            y = &bps - &y;
        }

        y
    }

    /// Makes the pertinent checks and updates for `withdraw` and `withdrawPenalty` endpoints
    ///
    fn withdraw_internal(&self, attributes: &UndelegateAttributes<Self::Api>) {
        let UndelegateAttributes {
            egld_amount,
            delegation_contract,
            unbond_epoch,
            ..
        } = attributes;
        let current_epoch = self.blockchain().get_block_epoch();
        require!(current_epoch >= *unbond_epoch, ERROR_UNBOND_PERIOD_NOT_ENDED);

        let contract_data_mapper = self.delegation_contract_data(delegation_contract);
        let contract_data = contract_data_mapper.get();

        // the public `withdrawFrom` endpoint should have been called before this point
        require!(
            &contract_data.total_withdrawable >= egld_amount,
            ERROR_TOO_MUCH_EGLD_AMOUNT
        );

        contract_data_mapper.update(|data| {
            data.total_withdrawable -= egld_amount;
        });

        self.total_withdrawable().update(|amount| *amount -= egld_amount);
    }
}
