multiversx_sc::imports!();
use super::{common, errors::*, events, model::*, proxies, storage};

#[multiversx_sc::module]
pub trait WithdrawModule:
    admin::AdminModule + common::CommonModule + events::EventsModule + proxies::ProxyModule + storage::StorageModule
{
    /// Allows users to redeem undelegate NFTs in exchange for EGLD once the unbond period has passed. To successfully
    /// redeem the EGLD, it must already be in the liquid staking smart contract. Therefore, the public endpoint
    /// `withdrawFrom` should have been called prior to using this function. If the redemption is successful, the NFT is
    /// burned, and the corresponding EGLD amount is sent to the caller.
    ///
    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self) -> BigUint {
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == self.undelegate_token().get_token_id(),
            ERROR_UNEXPECTED_PAYMENT
        );

        let token_nonce = payment.token_nonce;
        let undelegate_attributes: UndelegateAttributes<Self::Api> =
            self.undelegate_token().get_token_attributes(token_nonce);

        self.withdraw_internal(&undelegate_attributes);

        self.burn_undelegate_nft(token_nonce);

        let egld_amount = undelegate_attributes.egld_amount;
        let caller = self.blockchain().get_caller();
        self.send().direct_egld(&caller, &egld_amount);

        let delegation_contract = undelegate_attributes.delegation_contract;
        let contract_data = self.delegation_contract_data(&delegation_contract).get();
        self.withdraw_event(&caller, token_nonce, &contract_data);

        egld_amount
    }

    /// Initiates the withdrawal of the withdrawable amount from the specified Delegation smart contract. This endpoint
    /// performs an asynchronous call to withdraw the amount that has been previously undelegated and has passed the
    /// unbond period. It is capable of handling multiple calls, and the execution order of their callbacks does not
    /// need to match the order of the original calls.
    ///
    /// # Arguments
    ///
    /// - `delegation_contract`: The address of the Delegation smart contract to withdraw from.
    ///
    /// # Notes
    ///
    /// - This endpoint can be called by anyone.
    /// - It only needs to be called once per epoch.
    ///
    #[endpoint(withdrawFrom)]
    fn withdraw_from(&self, delegation_contract: ManagedAddress) {
        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call();
        let callback = self.callbacks().withdraw_from_cb(&caller, &delegation_contract);
        self.withdraw_from_delegation_contract(delegation_contract, gas_for_async_call, callback);
    }

    #[callback]
    fn withdraw_from_cb(
        &self,
        caller: &ManagedAddress,
        delegation_contract: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let withdrawn_amount = self.call_value().egld_value().clone_value();
                self.delegation_contract_data(delegation_contract).update(|data| {
                    data.total_withdrawable += &withdrawn_amount;
                    data.total_undelegated -= &withdrawn_amount;
                });
                self.total_withdrawable().update(|amount| *amount += &withdrawn_amount);
                self.total_undelegated().update(|amount| *amount -= &withdrawn_amount);
                self.withdraw_from_event(caller, delegation_contract, &withdrawn_amount);
            },
            ManagedAsyncCallResult::Err(err) => {
                self.async_call_error_event(err.err_code, err.err_msg);
            },
        }
    }
}
