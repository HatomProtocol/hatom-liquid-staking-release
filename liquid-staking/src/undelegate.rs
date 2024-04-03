multiversx_sc::imports!();
use super::{common, delegation, errors::*, events, model::*, proxies, score, selection, storage};

#[multiversx_sc::module]
pub trait UndelegateModule:
    admin::AdminModule
    + common::CommonModule
    + delegation::DelegationModule
    + events::EventsModule
    + proxies::ProxyModule
    + score::ScoreModule
    + selection::SelectionModule
    + storage::StorageModule
{
    /// Allows users to redeem sEGLD in exchange for EGLD. Instead of immediately sending the EGLD to the user, an
    /// undelegate NFT is minted and sent to the user. This NFT can be redeemed for EGLD once the unbond period has
    /// passed through the `unbond` endpoint. The paid sEGLD is burned.
    ///
    /// If not provided, the Delegation smart contract for the undelegation is selected based on the current
    /// configuration of the delegation algorithm. On the other hand, the Delegation smart contract can be specified
    /// only if the undelegation mode is set to `Free`.
    ///
    /// Notice that this endpoint does not automatically perform the undelegation. Instead, anyone can perform the
    /// undelegation at any given point in time using the `unDelegatePendingAmount` public endpoint.
    ///
    /// # Arguments
    ///
    /// - `opt_delegation_contract`: The address of the Delegation smart contract to undelegate from
    ///
    /// # Notes
    ///
    /// - There is a minimum amount of 1 EGLD for undelegations, which corresponds to a minimum amount of sEGLD
    ///   depending on the current exchange rate.
    ///
    #[payable("*")]
    #[endpoint(unDelegate)]
    fn undelegate(&self, opt_delegation_contract: OptionalValue<ManagedAddress>) -> EsdtTokenPayment {
        let (ls_token_id, shares) = self.call_value().single_fungible_esdt();
        self.require_valid_shares_payment(&ls_token_id, &shares);

        let egld_amount = self.shares_to_egld(&shares);
        self.require_sufficient_egld(&egld_amount);

        let delegation_contract = match opt_delegation_contract {
            OptionalValue::None => self.get_delegation_contract_for_undelegate(&egld_amount),
            OptionalValue::Some(contract) => {
                self.require_open_mode();

                let contract_data_mapper = self.delegation_contract_data(&contract);
                require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

                // make sure it is possible to undelegate since the undelegation algorithm has been bypassed
                let contract_data = contract_data_mapper.get();
                require!(
                    self.is_valid_undelegation_contract_relaxed(&contract_data, &egld_amount),
                    ERROR_INVALID_DELEGATION_CONTRACT
                );
                contract
            },
        };

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        contract_data_mapper.update(|data| {
            // update `total_delegated` here, such that it is taken into consideration when computing the delegation
            // contract at a next call to `get_delegation_contract_for_undelegate` above
            data.total_delegated -= &egld_amount;
            data.pending_to_undelegate += &egld_amount;
        });

        self.redeem_shares(&egld_amount, &shares);

        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_period = self.unbond_period().get();
        let unbond_epoch = current_epoch + unbond_period;

        let attrs = UndelegateAttributes {
            delegation_contract,
            egld_amount,
            shares,
            undelegate_epoch: current_epoch,
            unbond_epoch,
        };

        let (nft_id, nft_nonce, nft_amount) = self.mint_undelegate_nft(&attrs).into_tuple();

        let caller = self.blockchain().get_caller();
        self.send().direct_esdt(&caller, &nft_id, nft_nonce, &nft_amount);

        let contract_data = contract_data_mapper.get();
        self.undelegate_event(&caller, nft_nonce, &attrs, &contract_data);

        EsdtTokenPayment::new(nft_id, nft_nonce, nft_amount)
    }

    /// Initiates the undelegation of the pending amount from the specified Delegation smart contract. This endpoint
    /// performs an asynchronous call to the Delegation smart contract to undelegate the pending amount. It is capable
    /// of handling multiple calls, and the execution order of their callbacks does not need to match the order of the
    /// original calls.
    ///
    /// # Arguments
    ///
    /// - `delegation_contract`: The address of the Delegation smart contract to undelegate from.
    ///
    /// # Notes
    ///
    /// - This endpoint can be called by anyone.
    /// - There is no need to provide an optional argument for the amount of EGLD to undelegate. The pending amount to
    ///   undelegate should always be sufficient and prevent leaving dust at the Delegation smart contract. Both the
    ///   `undelegate` and `penalty_from_undelegation` functions ensure the adequacy of the pending amount.
    ///
    #[endpoint(unDelegatePendingAmount)]
    fn undelegate_pending_amount(&self, delegation_contract: ManagedAddress) {
        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        let contract_data = contract_data_mapper.get();
        let egld_amount = contract_data.pending_to_undelegate;
        require!(egld_amount > BigUint::zero(), ERROR_NO_PENDING_TO_UNDELEGATE);

        // update smart contract data to handle concurrent calls to this endpoint
        self.delegation_contract_data(&delegation_contract).update(|data| {
            data.pending_to_undelegate -= &egld_amount;
        });

        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call();
        let callback = self
            .callbacks()
            .undelegate_pending_amount_cb(&caller, &delegation_contract, &egld_amount);
        self.undelegate_from_delegation_contract(delegation_contract, egld_amount, gas_for_async_call, callback)
    }

    #[callback]
    fn undelegate_pending_amount_cb(
        &self,
        caller: &ManagedAddress,
        delegation_contract: &ManagedAddress,
        egld_amount: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegation_contract_data(delegation_contract).update(|data| {
                    data.total_undelegated += egld_amount;
                });
                self.total_undelegated().update(|amount| *amount += egld_amount);

                if self.undelegation_mode().get() == UndelegationMode::Algorithm {
                    let current_epoch = self.blockchain().get_block_epoch();
                    self.set_last_undelegate_epoch_internal(current_epoch);
                }
                self.undelegate_pending_amount_event(caller, delegation_contract, egld_amount);
            },
            ManagedAsyncCallResult::Err(err) => {
                self.delegation_contract_data(delegation_contract).update(|data| {
                    data.pending_to_undelegate += egld_amount;
                    data.outdated = true;
                });
                self.outdated_event(delegation_contract);
                self.async_call_error_event(err.err_code, err.err_msg);
            },
        }
    }
}
