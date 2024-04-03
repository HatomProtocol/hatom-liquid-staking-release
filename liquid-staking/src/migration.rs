multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use super::{common, errors::*, events, proxies, score, selection, storage};

#[multiversx_sc::module]
pub trait MigrationModule:
    admin::AdminModule
    + common::CommonModule
    + events::EventsModule
    + score::ScoreModule
    + selection::SelectionModule
    + storage::StorageModule
    + proxies::ProxyModule
{
    /// Adds an entry to the migration whitelist. This whitelist allows users to bypass the Delegation Algorithm for
    /// delegates.
    ///
    /// # Arguments
    ///
    /// - `user` - the user that wil be entitled to bypass the delegation algorithm
    /// - `delegation_contract` - the Delegation smart contract
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(addToMigrationWhitelist)]
    fn add_to_migration_whitelist(&self, user: &ManagedAddress, delegation_contract: &ManagedAddress) {
        self.require_admin();

        let migration_whitelist_mapper = self.migration_whitelist(user);
        require!(migration_whitelist_mapper.is_empty(), ERROR_USER_ALREADY_WHITELISTED);

        let contract_data_mapper = self.delegation_contract_data(delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        // the migration bypasses the delegation algorithm, so we make sure the contract is not blacklisted
        let contract_data = contract_data_mapper.get();
        require!(!contract_data.blacklisted, ERROR_BLACKLISTED_DELEGATION_CONTRACT);
        require!(
            self.is_delegation_contract_in_list(delegation_contract),
            ERROR_DELEGATION_CONTRACT_NOT_IN_LIST
        );

        migration_whitelist_mapper.set(delegation_contract);
        self.num_whitelisted_users(delegation_contract)
            .update(|_num| *_num += 1u32);

        self.add_to_migration_whitelist_event(user, delegation_contract);
    }

    /// Removes an entry from the migration whitelist.
    ///
    /// # Arguments
    ///
    /// - `user` - the user that is currently entitled to bypass the delegation algorithm
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(removeFromMigrationWhitelist)]
    fn remove_from_migration_whitelist(&self, user: &ManagedAddress) {
        self.require_admin();
        self.remove_from_migration_whitelist_internal(user);
    }

    /// Removes the caller from the migration whitelist.
    ///
    #[endpoint(removeMeFromMigrationWhitelist)]
    fn remove_me_from_migration_whitelist(&self) {
        let caller = self.blockchain().get_caller();
        self.remove_from_migration_whitelist_internal(&caller)
    }

    fn remove_from_migration_whitelist_internal(&self, user: &ManagedAddress) {
        let migration_whitelist_mapper = self.migration_whitelist(user);
        require!(!migration_whitelist_mapper.is_empty(), ERROR_USER_NOT_WHITELISTED);
        let delegation_contract = migration_whitelist_mapper.get();

        migration_whitelist_mapper.clear();
        self.num_whitelisted_users(&delegation_contract)
            .update(|_num| *_num -= 1u32);

        self.remove_from_migration_whitelist_event(user);
    }

    /// Returns the Delegation smart contract for which the user is entitled to bypass the delegation algorithm.
    ///
    /// # Arguments
    ///
    /// - `user` - the user that is currently entitled to bypass the delegation algorithm
    /// - `egld_amount` - the amount of EGLD to be delegated
    ///
    /// # Notes
    ///
    /// - verifies that the delegation smart contract is a valid smart contract for delegations
    /// - requires that the user has an entry in the migration whitelist
    /// - it verifies that the smart contract has enough capacity for the delegation and its not outdated, it does not
    ///   check its service fee (as in `is_valid_delegation_contract`)
    ///
    fn get_whitelisted_delegation_contract_for_delegate(
        &self,
        user: &ManagedAddress,
        egld_amount: &BigUint,
    ) -> ManagedAddress {
        let migration_whitelist_mapper = self.migration_whitelist(user);
        require!(!migration_whitelist_mapper.is_empty(), ERROR_USER_NOT_WHITELISTED);
        let delegation_contract = migration_whitelist_mapper.get();

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        let contract_data = contract_data_mapper.get();
        require!(!contract_data.blacklisted, ERROR_BLACKLISTED_DELEGATION_CONTRACT);
        require!(
            self.is_delegation_contract_in_list(&delegation_contract),
            ERROR_DELEGATION_CONTRACT_NOT_IN_LIST
        );

        require!(
            self.has_valid_cap(&contract_data, egld_amount) && !contract_data.outdated,
            ERROR_INVALID_DELEGATION_CONTRACT
        );

        delegation_contract
    }
}
