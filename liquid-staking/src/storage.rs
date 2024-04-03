multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use super::model::*;

#[multiversx_sc::module]
pub trait StorageModule {
    /// The current state of the Liquid Staking module
    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    /// The Liquid Staking token, i.e. the sEGLD ESDT token
    #[storage_mapper("lsTokenMapper")]
    fn ls_token(&self) -> FungibleTokenMapper<Self::Api>;

    /// The current outstanding supply of sEGLD or the current amount of total shares
    #[view(getLsSupply)]
    #[storage_mapper("lsTokenSupply")]
    fn ls_token_supply(&self) -> SingleValueMapper<BigUint>;

    /// The NFT given in exchange for sEGLD at unDelegations
    #[view(getUndelegateTokenId)]
    #[storage_mapper("undelegateTokenId")]
    fn undelegate_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    /// The Undelegate NFT name
    #[view(getUndelegateTokenName)]
    #[storage_mapper("undelegateTokenName")]
    fn undelegate_token_name(&self) -> SingleValueMapper<ManagedBuffer>;

    /// The current amount of EGLD being staked via Liquid Staking amongst all the whitelisted Staking Providers
    #[view(getCashReserve)]
    #[storage_mapper("cashReserve")]
    fn cash_reserve(&self) -> SingleValueMapper<BigUint>;

    /// The current amount of rewards in EGLD
    #[view(getRewardsReserve)]
    #[storage_mapper("rewardsReserve")]
    fn rewards_reserve(&self) -> SingleValueMapper<BigUint>;

    /// The current amount of EGLD that belongs to the protocol
    #[view(getProtocolReserves)]
    #[storage_mapper("protocolReserves")]
    fn protocol_reserve(&self) -> SingleValueMapper<BigUint>;

    /// The current total amount of EGLD being undelegated from all staking providers
    #[view(getTotalUndelegated)]
    #[storage_mapper("totalUndelegated")]
    fn total_undelegated(&self) -> SingleValueMapper<BigUint>;

    /// The current total amount of EGLD that can be unbonded or withdraw from all staking providers
    #[view(getTotalWithdrawable)]
    #[storage_mapper("totalWithdrawable")]
    fn total_withdrawable(&self) -> SingleValueMapper<BigUint>;

    /// Penalties by their identifiers
    #[view(getPenaltyById)]
    #[storage_mapper("penalties")]
    fn penalties(&self, id: u64) -> SingleValueMapper<Penalty<Self::Api>>;

    /// The next penalty identifier
    #[view(getNextPenaltyId)]
    #[storage_mapper("nextPenaltyId")]
    fn next_penalty_id(&self) -> SingleValueMapper<u64>;

    /// A linked list of Delegation smart contracts ordered by their delegation score
    #[view(getDelegationContractsList)]
    #[storage_mapper("delegationContractsList")]
    fn delegation_contracts_list(&self) -> LinkedListMapper<ManagedAddress>;

    /// Allows users to delegate their EGLD to a given staking provider Delegation smart contract bypassing the Delegation
    /// Algorithm
    #[view(getMigrationWhitelist)]
    #[storage_mapper("migrationWhitelist")]
    fn migration_whitelist(&self, user: &ManagedAddress) -> SingleValueMapper<ManagedAddress>;

    #[view(getNumWhitelistedUsers)]
    #[storage_mapper("numWhitelistedUsers")]
    fn num_whitelisted_users(&self, delegation_contract: &ManagedAddress) -> SingleValueMapper<u32>;

    /// A list of blacklisted Delegation smart contracts
    #[view(getBlacklistedDelegationContracts)]
    #[storage_mapper("blacklistedDelegationContracts")]
    fn blacklisted_delegation_contracts(&self) -> UnorderedSetMapper<ManagedAddress>;

    /// The metadata for each Delegation smart contract
    #[view(getDelegationContractData)]
    #[storage_mapper("delegationContractData")]
    fn delegation_contract_data(
        &self,
        delegation_contract: &ManagedAddress,
    ) -> SingleValueMapper<DelegationContractData<Self::Api>>;

    /// The undelegation mode
    #[view(getUndelegationMode)]
    #[storage_mapper("undelegationMode")]
    fn undelegation_mode(&self) -> SingleValueMapper<UndelegationMode>;

    /// The last epoch a successful undelegation occur when the undelegation mode is of `Algorithm` type
    #[view(getLastUndelegateEpoch)]
    #[storage_mapper("lastUndelegateEpoch")]
    fn last_undelegate_epoch(&self) -> SingleValueMapper<u64>;

    /// The last epoch a successful contract data update occur
    #[view(getLastContractDataUpdateEpoch)]
    #[storage_mapper("lastContractDataUpdateEpoch")]
    fn last_contract_data_update_epoch(&self) -> SingleValueMapper<u64>;

    /// The last epoch rewards have been claimed for a given Staking Provider Delegation smart contract
    #[view(getLastClaimRewardsEpoch)]
    #[storage_mapper("lastClaimRewardsEpoch")]
    fn last_rewards_claim_epoch(&self, delegation_contract: &ManagedAddress) -> SingleValueMapper<u64>;

    /// The period between undelegations and unbonds
    #[view(getUnbondPeriod)]
    #[storage_mapper("unbondPeriod")]
    fn unbond_period(&self) -> SingleValueMapper<u64>;

    /// The final fee charged to users, including staking providers service fee and liquid staking fee
    #[view(getTotalFee)]
    #[storage_mapper("totalFee")]
    fn total_fee(&self) -> SingleValueMapper<BigUint>;

    /// The Delegation Score model parameters
    #[view(getDelegationScoreModel)]
    #[storage_mapper("delegationScoreModel")]
    fn delegation_score_model(&self) -> SingleValueMapper<DelegationScoreModel<Self::Api>>;

    /// The Delegation Sampling model parameters
    #[view(getDelegationSamplingModel)]
    #[storage_mapper("delegationSamplingModel")]
    fn delegation_sampling_model(&self) -> SingleValueMapper<SamplingModel<Self::Api>>;

    /// Stores the Delegation smart contract data manager address
    #[view(getDataManager)]
    #[storage_mapper("dataManager")]
    fn data_manager(&self) -> SingleValueMapper<ManagedAddress>;

    /// Stores the random oracle address, used only for testing purposes
    #[view(getRandomOracle)]
    #[storage_mapper("randomOracle")]
    fn random_oracle(&self) -> SingleValueMapper<ManagedAddress>;
}
