multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use super::{common, delegation, errors::*, events, migration, proxies, score, selection, storage};

#[multiversx_sc::module]
pub trait DelegateModule:
    admin::AdminModule
    + common::CommonModule
    + delegation::DelegationModule
    + events::EventsModule
    + migration::MigrationModule
    + proxies::ProxyModule
    + score::ScoreModule
    + selection::SelectionModule
    + storage::StorageModule
{
    /// Allows users to stake EGLD in exchange for sEGLD. The Delegation smart contract is selected based on the current
    /// configuration of the delegation algorithm. However, this endpoint does not automatically perform the delegation.
    /// Instead, anyone can perform the delegation at any given point in time using the `delegatePendingAmount` public
    /// endpoint.
    ///
    /// # Notes
    ///
    /// - There is a minimum amount of 1 EGLD required for delegations.
    /// - If the caller is whitelisted, they may bypass the delegation algorithm.
    /// - The amount of sEGLD minted depends on the current exchange rate between EGLD and sEGLD.
    ///
    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) -> EsdtTokenPayment {
        self.require_active_state();

        let egld_amount = self.call_value().egld_value().clone_value();
        self.require_sufficient_egld(&egld_amount);

        let caller = self.blockchain().get_caller();
        let delegation_contract = if self.migration_whitelist(&caller).is_empty() {
            self.get_delegation_contract_for_delegate(&egld_amount, &OptionalValue::None)
        } else {
            self.get_whitelisted_delegation_contract_for_delegate(&caller, &egld_amount)
        };

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        contract_data_mapper.update(|data| {
            data.pending_to_delegate += &egld_amount;
        });

        let (ls_token_id, _, shares) = self.mint_shares(&egld_amount).into_tuple();
        self.send().direct_esdt(&caller, &ls_token_id, 0, &shares);

        let contract_data = contract_data_mapper.get();
        self.delegate_event(&caller, &egld_amount, &shares, &contract_data);

        EsdtTokenPayment::new(ls_token_id, 0, shares)
    }

    /// Initiates the delegation of the pending amount to the specified Delegation smart contract. This endpoint
    /// performs an asynchronous call to delegate the pending amount. It is capable of handling multiple calls, and the
    /// execution order of their callbacks does not need to match the order of the original calls.
    ///
    /// # Arguments
    ///
    /// - `delegation_contract`: The Delegation smart contract to delegate to.
    /// - `opt_egld_amount`: The optional EGLD amount to delegate. If not specified, it will delegate the entire pending
    ///   amount.
    ///
    /// # Notes
    ///
    /// - This endpoint can be called by anyone.
    /// - The EGLD amount can be smaller than the pending amount if it exceeds the capacity of the Delegation smart
    ///   contract.
    /// - If the delegation fails, the pending amount will be reverted, and the Delegation smart contract will be marked
    ///   as outdated. A new attempt with a smaller EGLD amount can be made later.
    /// - If there is pending amount that cannot be delegated, the admin can penalize the Delegation smart contract and
    ///   delegate that amount to a different Delegation smart contract.
    ///
    #[endpoint(delegatePendingAmount)]
    fn delegate_pending_amount(&self, delegation_contract: ManagedAddress, opt_egld_amount: OptionalValue<BigUint>) {
        self.require_active_state();

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        let contract_data = contract_data_mapper.get();
        let pending_to_delegate = contract_data.pending_to_delegate;
        require!(pending_to_delegate > BigUint::zero(), ERROR_NO_PENDING_TO_DELEGATE);

        let egld_amount = match opt_egld_amount {
            OptionalValue::None => pending_to_delegate,
            OptionalValue::Some(amount) => {
                self.require_sufficient_egld(&amount);
                require!(amount <= pending_to_delegate, ERROR_TOO_MUCH_EGLD_AMOUNT);
                let amount_left = pending_to_delegate - &amount;
                self.require_no_dust_left(&amount_left);
                amount
            },
        };

        // update smart contract data to handle concurrent calls to this endpoint
        contract_data_mapper.update(|data| {
            data.pending_to_delegate -= &egld_amount;
        });

        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call();
        let callback = self
            .callbacks()
            .delegate_pending_amount_cb(&caller, &delegation_contract, &egld_amount);
        self.delegate_to_delegation_contract(delegation_contract, egld_amount, gas_for_async_call, callback);
    }

    #[callback]
    fn delegate_pending_amount_cb(
        &self,
        caller: &ManagedAddress,
        delegation_contract: &ManagedAddress,
        egld_amount: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegation_contract_data(delegation_contract).update(|data| {
                    data.total_delegated += egld_amount;
                });
                self.delegate_pending_amount_event(caller, delegation_contract, egld_amount);
            },
            ManagedAsyncCallResult::Err(err) => {
                self.delegation_contract_data(delegation_contract).update(|data| {
                    data.pending_to_delegate += egld_amount;
                    data.outdated = true;
                });
                self.outdated_event(delegation_contract);
                self.async_call_error_event(err.err_code, err.err_msg);
            },
        }
    }
}
