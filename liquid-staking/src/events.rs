multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use super::model::{
    DelegationContractData, DelegationScoreModel, Penalty, SamplingModel, UndelegateAttributes, UndelegationMode,
};

#[multiversx_sc::module]
pub trait EventsModule {
    /// Emitted when unbond period is set
    #[event("set_unbond_period_event")]
    fn set_unbond_period_event(&self, #[indexed] unbond_period: u64);

    /// Emitted when the undelegation mode is set
    #[event("set_undelegation_mode_event")]
    fn set_undelegation_mode_event(&self, #[indexed] undelegation_mode: UndelegationMode);

    /// Emitted when the last undelegate epoch is set
    #[event("set_last_undelegate_epoch_event")]
    fn set_last_undelegate_epoch_event(&self, #[indexed] last_undelegate_epoch: u64);

    /// Emitted when the last contract data update epoch is set
    #[event("set_last_contract_data_update_epoch_event")]
    fn set_last_contract_data_update_epoch_event(&self, #[indexed] last_undelegate_epoch: u64);

    /// Emitted when the liquid staking token is registered
    #[event("register_ls_token_event")]
    fn register_ls_token_event(&self, #[indexed] ls_token_id: &TokenIdentifier);

    /// Emitted when the undelegate nft is registered
    #[event("register_undelegate_token_event")]
    fn register_undelegate_token_event(&self, #[indexed] undelegate_id: &TokenIdentifier);

    /// Emitted when a new data manager is set
    #[event("new_data_manager_event")]
    fn new_data_manager_event(&self, #[indexed] old: &Option<ManagedAddress>, #[indexed] new: &ManagedAddress);

    /// Emitted when the delegation score model parameters are set or modified
    #[event("set_delegation_score_model_params_event")]
    fn set_delegation_score_model_params_event(&self, #[indexed] score_model: &DelegationScoreModel<Self::Api>);

    /// Emitted when the delegation sampling model parameters are set or modified
    #[event("set_delegation_sampling_model_params_event")]
    fn set_delegation_sampling_model_params_event(&self, #[indexed] sampling_model: &SamplingModel<Self::Api>);

    /// Emitted when the total fee is set or modified
    #[event("set_total_fee_event")]
    fn set_total_fee_event(&self, #[indexed] total_fee: &BigUint);

    /// Emitted when the delegation sampling model is removed
    #[event("clear_delegation_sampling_model_event")]
    fn clear_delegation_sampling_model_event(&self);

    /// Emitted when the liquid staking state modified
    #[event("set_state_event")]
    fn set_state_event(&self, #[indexed] active: bool);

    /// Emitted when a Delegation smart contract is whitelisted
    #[event("whitelist_delegation_contract_event")]
    fn whitelist_delegation_contract_event(&self, #[indexed] contract_data: &DelegationContractData<Self::Api>);

    /// Emitted when a Delegation smart contract is blacklisted
    #[event("blacklist_delegation_contract_event")]
    fn blacklist_delegation_contract_event(&self, #[indexed] contract_data: &DelegationContractData<Self::Api>);

    /// Emitted when the Delegation smart contract params are modified
    #[event("change_delegation_contract_params_event")]
    fn change_delegation_contract_params_event(&self, #[indexed] contract_data: &DelegationContractData<Self::Api>);

    /// Emitted when a user delegates to Liquid Staking
    #[event("delegate_event")]
    fn delegate_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] egld_amount: &BigUint,
        #[indexed] shares: &BigUint,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when a pending amount is delegated to a Delegation smart contract
    #[event("delegate_pending_amount_event")]
    fn delegate_pending_amount_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] contract: &ManagedAddress,
        #[indexed] egld_amount: &BigUint,
    );

    /// Emitted when a user undelegates from Liquid Staking
    #[event("undelegate_event")]
    fn undelegate_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] undelegate_token_nonce: u64,
        #[indexed] undelegate_attrs: &UndelegateAttributes<Self::Api>,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when a pending amount is undelegated from a Delegation smart contract
    #[event("undelegate_pending_amount_event")]
    fn undelegate_pending_amount_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] contract: &ManagedAddress,
        #[indexed] egld_amount: &BigUint,
    );

    /// Emitted when a user withdraws EGLD from Liquid Staking
    #[event("withdraw_event")]
    fn withdraw_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] undelegate_token_nonce: u64,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when an amount of EGLD in withdrawn from a Delegation smart contract
    #[event("withdraw_from_event")]
    fn withdraw_from_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] contract: &ManagedAddress,
        #[indexed] egld_amount: &BigUint,
    );

    /// Emitted when the admin penalizes via an undelegation from a specific Delegation smart contract
    #[event("penalty_from_undelegation_event")]
    fn penalty_from_undelegation_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] penalty: &Penalty<Self::Api>,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when a Delegation smart contract is penalized reducing its pending to delegate amount
    #[event("penalty_from_pending_to_delegate_event")]
    fn penalty_from_pending_to_delegate_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] penalty: &Penalty<Self::Api>,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when a penalty is marked as withdrawn
    #[event("withdraw_penalty_event")]
    fn withdraw_penalty_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] penalty_id: u64,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when a penalty in its whole or a part of it is delegated
    #[event("delegate_penalty_event")]
    fn delegate_penalty_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] contract: &ManagedAddress,
        #[indexed] penalty_id: u64,
        #[indexed] egld_amount: &BigUint,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when a user withdraws from a penalty in the Free mode
    #[event("withdraw_from_penalty_event")]
    fn withdraw_from_penalty_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] penalty_id: u64,
        #[indexed] egld_amount: &BigUint,
        #[indexed] shares: &BigUint,
    );

    /// Emitted when anyone claims rewards
    #[event("claim_rewards_from_event")]
    fn claim_rewards_from_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] contract: &ManagedAddress,
        #[indexed] reserves_amount: &BigUint,
        #[indexed] rewards_amount: &BigUint,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when the admin delegates rewards
    #[event("delegate_rewards_event")]
    fn delegate_rewards_event(
        &self,
        #[indexed] account: &ManagedAddress,
        #[indexed] contract: &ManagedAddress,
        #[indexed] egld_amount: &BigUint,
        #[indexed] contract_data: &DelegationContractData<Self::Api>,
    );

    /// Emitted when the admin withdraws funds from the protocol reserve
    #[event("withdraw_reserve_event")]
    fn withdraw_reserve_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] withdraw_amount: &BigUint,
        #[indexed] to: &ManagedAddress,
    );

    /// Adds a user to the migration whitelist
    #[event("add_to_migration_whitelist_event")]
    fn add_to_migration_whitelist_event(&self, #[indexed] user: &ManagedAddress, #[indexed] contract: &ManagedAddress);

    /// Removes a user from the migration whitelist
    #[event("remove_from_migration_whitelist_event")]
    fn remove_from_migration_whitelist_event(&self, #[indexed] user: &ManagedAddress);

    /// Emitted when an async call fails
    #[event("async_call_error_event")]
    fn async_call_error_event(&self, #[indexed] error_code: u32, #[indexed] error_msg: ManagedBuffer);

    /// Emitted when an async call fails and contract data is outdated
    #[event("outdated_event")]
    fn outdated_event(&self, #[indexed] contract: &ManagedAddress);
}
