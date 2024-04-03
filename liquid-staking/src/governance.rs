multiversx_sc::imports!();
use super::{common, constants::*, delegation, errors::*, events, model::*, proxies, score, selection, storage};

#[multiversx_sc::module]
pub trait GovernanceModule:
    admin::AdminModule
    + common::CommonModule
    + delegation::DelegationModule
    + events::EventsModule
    + proxies::ProxyModule
    + score::ScoreModule
    + selection::SelectionModule
    + storage::StorageModule
{
    /// Issues the liquid staking token, namely the sEGLD token.
    ///
    /// # Arguments
    ///
    /// - `name` - the token name
    /// - `ticker` - the token ticker or symbol
    /// - `decimals` - the token decimals
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[payable("EGLD")]
    #[endpoint(registerLsToken)]
    fn register_ls_token(&self, name: ManagedBuffer, ticker: ManagedBuffer, decimals: usize) {
        self.require_admin();

        require!(self.ls_token().is_empty(), ERROR_LS_TOKEN_ALREADY_ISSUED);

        let issue_cost = self.call_value().egld_value().clone_value();
        let initial_supply = BigUint::zero();

        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(
                issue_cost,
                &name,
                &ticker,
                &initial_supply,
                FungibleTokenProperties {
                    num_decimals: decimals,
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_mint: true,
                    can_burn: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().register_ls_token_cb())
            .call_and_exit()
    }

    #[callback]
    fn register_ls_token_cb(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.ls_token().set_token_id(token_id.clone());
                self.register_ls_token_event(&token_id);
            },
            ManagedAsyncCallResult::Err(err) => {
                let (token_id, returned_tokens) = self.call_value().egld_or_single_fungible_esdt();
                if token_id.is_egld() && returned_tokens > BigUint::zero() {
                    let admin = self.get_admin();
                    self.send().direct_egld(&admin, &returned_tokens);
                }
                self.async_call_error_event(err.err_code, err.err_msg);
            },
        }
    }

    /// Gives Mint and Burn roles for sEGLD to this contract.
    ///
    #[endpoint(setLsTokenRoles)]
    fn set_ls_token_roles(&self) {
        self.require_admin();
        require!(!self.ls_token().is_empty(), ERROR_LS_TOKEN_NOT_ISSUED);
        let sc_address = self.blockchain().get_sc_address();
        let ls_token_id = self.ls_token().get_token_id();
        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];
        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(&sc_address, &ls_token_id, roles[..].iter().cloned())
            .async_call()
            .call_and_exit()
    }

    /// Issues the Undelegate Nft, the token minted at undelegations as a receipt.
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[payable("EGLD")]
    #[endpoint(registerUndelegateToken)]
    fn register_undelegate_token(&self, name: ManagedBuffer, ticker: ManagedBuffer) {
        self.require_admin();

        require!(
            self.undelegate_token().is_empty(),
            ERROR_UNDELEGATE_TOKEN_ALREADY_ISSUED
        );

        let issue_cost = self.call_value().egld_value().clone_value();

        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                issue_cost,
                &name,
                &ticker,
                NonFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_transfer_create_role: true,
                    can_change_owner: false,
                    can_upgrade: false,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().register_undelegate_token_cb(&name))
            .call_and_exit()
    }

    #[callback]
    fn register_undelegate_token_cb(
        &self,
        token_name: &ManagedBuffer,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.undelegate_token().set_token_id(token_id.clone());
                self.undelegate_token_name().set(token_name);
                self.register_undelegate_token_event(&token_id);
            },
            ManagedAsyncCallResult::Err(err) => {
                let (token_id, returned_tokens) = self.call_value().egld_or_single_fungible_esdt();
                if token_id.is_egld() && returned_tokens > BigUint::zero() {
                    let admin = self.get_admin();
                    self.send().direct(&admin, &token_id, 0, &returned_tokens);
                }
                self.async_call_error_event(err.err_code, err.err_msg);
            },
        }
    }

    /// Gives Mint and Burn roles for the Undelegate Nft to this contract.
    ///
    #[endpoint(setUndelegateTokenRoles)]
    fn set_undelegate_token_roles(&self) {
        self.require_admin();
        require!(!self.undelegate_token().is_empty(), ERROR_UNDELEGATE_NFT_NOT_ISSUED);
        let sc_address = self.blockchain().get_sc_address();
        let undelegate_token_id = self.undelegate_token().get_token_id();
        let roles = [EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn];
        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(&sc_address, &undelegate_token_id, roles[..].iter().cloned())
            .async_call()
            .call_and_exit()
    }

    /// Sets the Data Manager entitled to change the data associated to each Delegation smart contract.
    ///
    /// # Arguments
    ///
    /// - `new_data_manager` - the address of the new Data Manager
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(setDataManager)]
    fn set_data_manager(&self, new_data_manager: ManagedAddress) {
        self.require_admin();
        let old_data_manager = self.get_data_manager();
        self.data_manager().set(&new_data_manager);
        self.new_data_manager_event(&old_data_manager, &new_data_manager);
    }

    /// Activates the Liquid Staking Module state. The activation can only occur iff:
    ///
    /// - the total fee has been set
    /// - the Liquid Staking token has been issued
    /// - the undelegate NFT has been issued
    /// - the delegation score model has been defined
    /// - the data manager has been set
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        self.require_admin();

        // check total fee
        require!(!self.total_fee().is_empty(), ERROR_TOTAL_FEE_UNSET);

        // check ls token issuance
        let ls_token_id = self.ls_token().get_token_id();
        require!(ls_token_id.is_valid_esdt_identifier(), ERROR_LS_TOKEN_NOT_ISSUED);

        // check undelegate token issuance
        let undelegate_token_id = self.undelegate_token().get_token_id();
        require!(
            undelegate_token_id.is_valid_esdt_identifier(),
            ERROR_UNDELEGATE_NFT_NOT_ISSUED
        );

        // check delegation score model has been set (sampling model is not mandatory)
        require!(
            !self.delegation_score_model().is_empty(),
            ERROR_DELEGATION_SCORE_MODEL_UNSET
        );

        // check data manager is set
        require!(!self.data_manager().is_empty(), ERROR_DATA_MANAGER_UNSET);

        self.state().set(State::Active);
        self.set_state_event(true);
    }

    /// Deactivates the Liquid Staking Module state.
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.require_admin();
        self.state().set(State::Inactive);
        self.set_state_event(false);
    }

    /// Whitelists a Staking Provider Delegation smart contract. From this point onwards, this smart contract will be
    /// eligible as a Delegation smart contract based on the state of the delegation algorithm. This method can also be
    /// used to whitelist a previously blacklisted Delegation smart contract.
    ///
    /// # Arguments
    ///
    /// - `contract` - the Delegation smart contract address
    /// - `admin` - the address entitled to update this Delegation smart contract data
    /// - `total_value_locked` - the liquidity locked at the Delegation smart contract
    /// - `nr_nodes` - the number of validator nodes
    /// - `apr` - the current APR for the validator
    /// - `service_fee` - the service fee being charged by the validator
    /// - `opt_cap` - the maximum amount that can be locked at the Delegation smart contract (uncapped if `None`)
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    /// - it will compute a delegation score based on the current state of the delegation algorithm
    ///
    #[endpoint(whitelistDelegationContract)]
    fn whitelist_delegation_contract(
        &self,
        delegation_contract: ManagedAddress,
        total_value_locked: BigUint,
        nr_nodes: u64,
        apr: BigUint,
        service_fee: BigUint,
        opt_cap: OptionalValue<BigUint>,
    ) {
        self.require_admin();
        self.require_delegation_contract(&delegation_contract);

        // compute delegation score
        let delegation_score = self.compute_delegation_score_internal(&total_value_locked, &apr);

        // if previously blacklisted, whitelist
        self.blacklisted_delegation_contracts()
            .swap_remove(&delegation_contract);

        let cap = opt_cap.into_option();
        if cap.is_some() {
            require!(cap.as_ref().unwrap() >= &total_value_locked, ERROR_DELEGATION_CAP);
        }

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        let contract_data = if contract_data_mapper.is_empty() {
            DelegationContractData {
                contract: delegation_contract.clone(),
                total_value_locked,
                cap,
                nr_nodes,
                apr,
                delegation_score: delegation_score.clone(),
                service_fee,
                pending_to_delegate: BigUint::zero(),
                total_delegated: BigUint::zero(),
                pending_to_undelegate: BigUint::zero(),
                total_undelegated: BigUint::zero(),
                total_withdrawable: BigUint::zero(),
                outdated: false,
                blacklisted: false,
            }
        } else {
            // we only manage the case where the contract has been blacklisted if the contract data already exist.
            // notice there is no need to remove it from the list, since it has been already removed when blacklisting.
            let contract_data = contract_data_mapper.get();
            require!(contract_data.blacklisted, ERROR_NOT_BLACKLISTED_DELEGATION_CONTRACT);

            DelegationContractData {
                total_value_locked,
                cap,
                nr_nodes,
                apr,
                delegation_score: delegation_score.clone(),
                service_fee,
                outdated: false,
                blacklisted: false,
                ..contract_data
            }
        };
        contract_data_mapper.set(&contract_data);

        self.add_and_order_delegation_contract_in_list(&delegation_contract, &delegation_score);

        self.whitelist_delegation_contract_event(&contract_data);
    }

    /// Blacklists a Delegation smart contract. From this point onwards, this smart contract will not be eligible as a
    /// Delegation smart contract through the delegation algorithm.
    ///
    /// # Arguments
    ///
    /// - `delegation_contract` - the Delegation smart contract address
    ///
    #[endpoint(blacklistDelegationContract)]
    fn blacklist_delegation_contract(&self, delegation_contract: ManagedAddress) {
        self.require_admin();

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        let contract_data = contract_data_mapper.get();
        require!(!contract_data.blacklisted, ERROR_BLACKLISTED_DELEGATION_CONTRACT);

        // make sure the contract is not in the migration whitelist
        require!(
            self.num_whitelisted_users(&delegation_contract).get() == 0u32,
            ERROR_IN_MIGRATION_WHITELIST
        );

        // blacklist Delegation smart contract
        self.blacklisted_delegation_contracts()
            .insert(delegation_contract.clone());

        self.remove_delegation_contract_from_list(&delegation_contract);

        contract_data_mapper.update(|data| {
            data.outdated = true;
            data.blacklisted = true;
        });

        let contract_data = contract_data_mapper.get();
        self.blacklist_delegation_contract_event(&contract_data);
    }

    /// Updates the data for a given Staking Provider Delegation smart contract.
    ///
    /// # Arguments
    ///
    /// - `contract` - the Delegation smart contract address
    /// - `admin` - the address entitled to update this Delegation smart contract data
    /// - `total_value_locked` - the liquidity locked at the Delegation smart contract
    /// - `nr_nodes` - the number of validator nodes
    /// - `apr` - the current APR for the validator
    /// - `service_fee` - the service fee being charged by the validator
    /// - `opt_cap` - the maximum amount that can be locked at the Delegation smart contract (uncapped if `None`)
    ///
    /// # Notes
    ///
    /// - can only be called by the admin set for the Delegation smart contract data
    /// - will revert if the contract has been blacklisted
    /// - it will compute a delegation score based on the current state of the delegation algorithm
    ///
    #[endpoint(changeDelegationContractParams)]
    fn change_delegation_contract_params(
        &self,
        delegation_contract: ManagedAddress,
        total_value_locked: BigUint,
        nr_nodes: u64,
        apr: BigUint,
        service_fee: BigUint,
        opt_cap: OptionalValue<BigUint>,
    ) {
        self.require_data_manager();

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        let old_contract_data = contract_data_mapper.get();
        require!(!old_contract_data.blacklisted, ERROR_BLACKLISTED_DELEGATION_CONTRACT);

        let cap = opt_cap.into_option();
        if cap.is_some() {
            require!(cap.as_ref().unwrap() >= &total_value_locked, ERROR_DELEGATION_CAP);
        }

        let old_delegation_score = old_contract_data.delegation_score;

        contract_data_mapper.update(|data| {
            data.total_value_locked = total_value_locked;
            data.nr_nodes = nr_nodes;
            data.apr = apr;
            data.service_fee = service_fee;
            data.cap = cap;
            data.outdated = false;
        });

        // get updated data and compute new score
        let new_contract_data = contract_data_mapper.get();
        let new_delegation_score = self.compute_delegation_score(&new_contract_data);

        // Check if delegation_score has changed
        if old_delegation_score != new_delegation_score {
            contract_data_mapper.update(|data| {
                data.delegation_score = new_delegation_score.clone();
            });

            self.remove_delegation_contract_from_list(&delegation_contract);
            self.add_and_order_delegation_contract_in_list(&delegation_contract, &new_delegation_score);
        }

        let current_epoch = self.blockchain().get_block_epoch();
        self.set_last_contract_data_update_epoch_internal(current_epoch);

        let contract_data = contract_data_mapper.get();
        self.change_delegation_contract_params_event(&contract_data);
    }

    /// Withdraws a given amount of EGLD from the protocol reserves to an optionally given account.
    ///
    /// # Arguments
    ///
    /// - `egld_amount` - the amount of EGLD to withdraw
    /// - `opt_to` - an optional address to send the EGLD to
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    /// - the EGLD amount is directed to the admin account if none is provided
    ///
    #[endpoint(withdrawReserve)]
    fn withdraw_reserve(&self, egld_amount: BigUint, opt_to: OptionalValue<ManagedAddress>) {
        self.require_admin();

        require!(
            egld_amount <= self.protocol_reserve().get(),
            ERROR_NOT_ENOUGH_PROTOCOL_RESERVES
        );

        self.protocol_reserve().update(|amount| *amount -= &egld_amount);

        // get beneficiary
        let caller = self.blockchain().get_caller();
        let to = match opt_to {
            OptionalValue::None => &caller,
            OptionalValue::Some(ref to) => to,
        };

        self.send().direct_egld(to, &egld_amount);

        self.withdraw_reserve_event(&caller, &egld_amount, to);
    }

    /// Sets the total fee, which represents the final fee end users see discounted from their rewards based on the
    /// service fee charged by each Staking Provider and by this Liquid Staking protocol. For example, if a Staking
    /// Provider has a service fee of 7% and the total fee is set to 17%, the Liquid Staking Protocol will charge a 10%
    /// fee from the total rewards.
    ///
    /// # Arguments
    ///
    /// - `fee` - the total fee in basis points
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(setTotalFee)]
    fn set_total_fee(&self, fee: &BigUint) {
        self.require_admin();
        require!(*fee > BigUint::zero(), ERROR_VALUE_CANNOT_BE_ZERO);
        require!(*fee <= BPS, ERROR_VALUE_EXCEEDS_BPS);
        self.total_fee().set(fee);
        self.set_total_fee_event(fee);
    }

    /// Sets the Delegation Score Model parameters used for the computation of the delegation score for each Staking
    /// Provider Delegation smart contract. Higher scores imply better chances of being selected at delegations as well
    /// as lower chances of being selected for undelegations.
    ///
    /// # Arguments
    ///
    /// - `method` - the score can be based only on Total Value Locked, APR or a weighted mix of both parameters
    /// - `min_tvl` - Delegation smart contracts with lower TVLs than this parameters share the same TVL score
    /// - `max_tvl` - Delegation smart contracts with higher TVLs than this parameters share the same TVL score
    /// - `min_apr` - Delegation smart contracts with lower APRs than this parameters share the same APR score (in bps)
    /// - `max_apr` - Delegation smart contracts with higher APRs than this parameters share the same APR score (in bps)
    /// - `opt_omega` - should be given only for a Mixed delegation score method and defines the weight for both TVL and
    ///   APR scores
    /// - `sort` - if true, the list of Delegation smart contracts will be sorted based on the new delegation score
    ///   model parameters. If false, the sorting is left to `changeDelegationContractParams`.
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(setDelegationScoreModelParams)]
    fn set_delegation_score_model_params(
        &self,
        method: DelegationScoreMethod,
        min_tvl: BigUint,
        max_tvl: BigUint,
        min_apr: BigUint,
        max_apr: BigUint,
        sort: bool,
        opt_omega: OptionalValue<BigUint>,
    ) {
        self.require_admin();

        let omega = match method {
            DelegationScoreMethod::Tvl => {
                require!(max_tvl > min_tvl, ERROR_INVALID_DOMAIN);
                match opt_omega {
                    OptionalValue::Some(_) => sc_panic!(ERROR_UNEXPECTED_VALUE),
                    OptionalValue::None => BigUint::from(BPS),
                }
            },
            DelegationScoreMethod::Apr => {
                require!(max_apr > min_apr, ERROR_INVALID_DOMAIN);
                match opt_omega {
                    OptionalValue::Some(_) => sc_panic!(ERROR_UNEXPECTED_VALUE),
                    OptionalValue::None => BigUint::zero(),
                }
            },
            DelegationScoreMethod::Mixed => {
                require!(max_tvl > min_tvl && max_apr > min_apr, ERROR_INVALID_DOMAIN);
                match opt_omega {
                    OptionalValue::None => sc_panic!(ERROR_EXPECTED_VALUE),
                    OptionalValue::Some(omega) => {
                        require!(omega <= BPS, ERROR_VALUE_EXCEEDS_BPS);
                        omega
                    },
                }
            },
        };

        let model = DelegationScoreModel {
            method,
            min_tvl,
            max_tvl,
            min_apr,
            max_apr,
            omega,
        };
        self.delegation_score_model().set(&model);

        if sort {
            self.sort_delegation_contracts_list();
        }

        self.set_delegation_score_model_params_event(&model);
    }

    /// Sets the Delegation Sampling Model parameters used for the random selection between candidates on a computed
    /// list of Staking Providers Delegation smart contracts.
    ///
    /// # Arguments
    ///
    /// - `tolerance` - the tolerance (as percentage and as bps) used to build the list of candidates
    /// - `max_service_fee` - from this point onwards, staking providers do not receive delegations in bps
    /// - `premium` - the difference between the delegation weight at service_fee = 0 and the delegation weight at
    ///   service_fee = max_service_fee in bps
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(setDelegationSamplingModelParams)]
    fn set_delegation_sampling_model_params(&self, tolerance: BigUint, max_service_fee: BigUint, premium: BigUint) {
        self.require_admin();

        require!(tolerance > BigUint::zero(), ERROR_VALUE_CANNOT_BE_ZERO);
        require!(tolerance <= BPS, ERROR_VALUE_EXCEEDS_BPS);

        require!(max_service_fee > BigUint::zero(), ERROR_VALUE_CANNOT_BE_ZERO);
        require!(max_service_fee <= BPS, ERROR_VALUE_EXCEEDS_BPS);

        let model = SamplingModel {
            tolerance,
            max_service_fee,
            premium,
        };
        self.delegation_sampling_model().set(&model);

        self.set_delegation_sampling_model_params_event(&model);
    }

    /// Clears the Delegation Sampling Model, i.e. removes the sampling from delegation and undelegation candidates.
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    ///
    #[endpoint(clearDelegationSamplingModel)]
    fn clear_delegation_sampling_model(&self) {
        self.require_admin();
        self.delegation_sampling_model().clear();
        self.clear_delegation_sampling_model_event();
    }

    /// A public endpoint that allows to start bypassing the undelegation algorithm in order to undelegate and,
    /// consequently, withdraw EGLD from the protocol.
    ///
    /// # Notes
    ///
    /// - can be called by anyone after `NO_UNDELEGATIONS_EPOCHS` have passed since the last undelegation
    ///
    #[endpoint(deactivateUndelegationAlgorithm)]
    fn deactivate_undelegation_algorithm(&self) {
        let last_undelegate_epoch = self.get_last_undelegate_epoch();
        let last_contract_data_update_epoch = self.get_last_contract_data_update_epoch();
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            current_epoch - last_undelegate_epoch >= NO_UNDELEGATE_EPOCHS
                || current_epoch - last_contract_data_update_epoch >= NO_DATA_UPDATE_EPOCHS,
            ERROR_NOT_ENOUGH_ELAPSED_EPOCHS
        );
        self.set_undelegation_mode_internal(UndelegationMode::Open);
    }

    /// An admin endpoint that reactivates the undelegation algorithm.
    ///
    /// # Notes
    ///
    /// - can only be called by the admin
    /// - can only be reactivated after `NO_UNDELEGATIONS_EPOCHS + COOLDOWN_REACTIVATE_UNDELEGATION_ALGORITHM` have
    ///   elapsed since the last undelegation
    ///
    #[endpoint(reactivateUndelegationAlgorithm)]
    fn reactivate_undelegation_algorithm(&self) {
        self.require_admin();
        let last_epoch = self.get_last_undelegate_epoch();
        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            current_epoch - last_epoch >= NO_UNDELEGATE_EPOCHS + COOLDOWN_REACTIVATE_UNDELEGATION_ALGORITHM,
            ERROR_NOT_ENOUGH_ELAPSED_EPOCHS
        );
        self.set_last_undelegate_epoch_internal(current_epoch);
        self.set_undelegation_mode_internal(UndelegationMode::Algorithm);
    }
}
