multiversx_sc::imports!();
use super::{common, delegation, errors::*, events, model::*, proxies, score, selection, storage};

#[multiversx_sc::module]
pub trait PenaltyModule:
    admin::AdminModule
    + common::CommonModule
    + delegation::DelegationModule
    + events::EventsModule
    + proxies::ProxyModule
    + score::ScoreModule
    + selection::SelectionModule
    + storage::StorageModule
{
    /// Initiates a penalty to a Delegation smart contract. Penalties reduce the staked amount of a Delegation smart
    /// contract through two different mechanisms:
    ///
    /// 1. By undelegating an amount of EGLD from the Delegation smart contract.
    /// 2. By reducing the pending amount of EGLD to be delegated to the Delegation smart contract.
    ///
    /// The first mechanism can only be triggered by the admin when detecting misbehavior from the Staking Agency
    /// associated with the Delegation smart contract. This penalty must be unbonded from the Delegation smart contract
    /// at a future time using the `unbondPenalty` public endpoint.
    ///
    /// The second mechanism can be triggered by the admin in cases of Staking Agency misbehavior or when there is a
    /// pending amount to be delegated that cannot be deposited due to current cap and total value locked values. The
    /// community may also initiate this penalty if specific conditions are met. Since this penalty affects the pending
    /// amount to be delegated, the EGLD is already present and does not need to be withdrawn. Therefore, the penalty is
    /// marked as `withdrawn`.
    ///
    /// Finally, all penalties need to be delegated to a new Delegation smart contract. This is achieved by calling the
    /// `delegatePenalty` endpoint. If for some reason this is not done, user will eventually have the chance to
    /// undelegate from penalties.
    ///
    /// # Arguments
    ///
    /// - `delegation_contract`: The address of the Delegation smart contract.
    /// - `source`: The source or type of the penalty.
    /// - `opt_egld_amount`: The amount of EGLD to penalize. If unspecified, it defaults to the total delegated or
    ///   pending amount to be delegated.
    ///
    #[endpoint(penalize)]
    fn penalize(
        &self,
        delegation_contract: ManagedAddress,
        source: PenaltySource,
        opt_egld_amount: OptionalValue<BigUint>,
    ) {
        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        require!(!contract_data_mapper.is_empty(), ERROR_UNEXPECTED_DELEGATION_CONTRACT);

        match source {
            PenaltySource::FromUndelegate => {
                self.require_admin();
                self.penalty_from_undelegation(delegation_contract, opt_egld_amount);
            },
            PenaltySource::FromPendingToDelegate => {
                self.require_open_mode_or_admin();
                self.penalty_from_pending_to_delegate(delegation_contract, opt_egld_amount);
            },
        }
    }

    /// Creates a penalty to a given Delegation smart contract. The EGLD penalty amount, if not given, will default to
    /// the delegated amount, which already takes into consideration the amount of EGLD that is pending to be
    /// undelegated. Similarly to `undelegate`, it does not perform the undelegation automatically, but it can be done
    /// by anyone at any given point in time using the `undelegatePendingAmount` public endpoint.
    ///
    /// # Arguments
    ///
    /// - `delegation_contract` - the Delegation smart contract address
    /// - `opt_egld_amount` - the amount of EGLD associated to the penalty
    ///
    fn penalty_from_undelegation(&self, delegation_contract: ManagedAddress, opt_egld_amount: OptionalValue<BigUint>) {
        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        let contract_data = contract_data_mapper.get();

        let total_delegated = contract_data.total_delegated;
        let egld_amount = match opt_egld_amount {
            OptionalValue::None => total_delegated,
            OptionalValue::Some(amount) => {
                self.require_sufficient_egld(&amount);
                require!(amount <= total_delegated, ERROR_TOO_MUCH_EGLD_AMOUNT);
                let amount_left = &total_delegated - &amount;
                self.require_no_dust_left(&amount_left);
                amount
            },
        };

        contract_data_mapper.update(|data| {
            data.total_delegated -= &egld_amount;
            data.pending_to_undelegate += &egld_amount;
        });

        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_period = self.unbond_period().get();
        let unbond_epoch = current_epoch + unbond_period;

        let penalty_id = self.get_next_penalty_id();
        let attrs = UndelegateAttributes {
            delegation_contract,
            undelegate_epoch: current_epoch,
            egld_amount,
            shares: BigUint::zero(),
            unbond_epoch,
        };

        let penalty = Penalty {
            id: penalty_id,
            withdrawn: false,
            attributes: attrs,
        };

        self.penalties(penalty_id).set(&penalty);

        let caller = self.blockchain().get_caller();
        let contract_data = contract_data_mapper.get();
        self.penalty_from_undelegation_event(&caller, &penalty, &contract_data);
    }

    /// Creates a penalty to a given Delegation smart contract. The EGLD penalty amount, if not given, will default to
    /// the pending amount to be delegated. This penalty is marked as already unbonded, since the EGLD is already here
    /// and it does not need to be unbonded.
    ///
    /// # Arguments
    ///
    /// - `delegation_contract` - the Delegation smart contract address
    /// - `opt_egld_amount` - the amount of EGLD associated to the penalty
    ///
    fn penalty_from_pending_to_delegate(
        &self,
        delegation_contract: ManagedAddress,
        opt_egld_amount: OptionalValue<BigUint>,
    ) {
        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        let contract_data = contract_data_mapper.get();

        let pending_to_delegate = contract_data.pending_to_delegate;
        let egld_amount = match opt_egld_amount {
            OptionalValue::None => pending_to_delegate,
            OptionalValue::Some(amount) => {
                self.require_sufficient_egld(&amount);
                require!(amount <= pending_to_delegate, ERROR_TOO_MUCH_EGLD_AMOUNT);
                let amount_left = &pending_to_delegate - &amount;
                self.require_no_dust_left(&amount_left);
                amount
            },
        };

        contract_data_mapper.update(|data| {
            data.pending_to_delegate -= &egld_amount;
        });

        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch;

        let penalty_id = self.get_next_penalty_id();
        let attrs = UndelegateAttributes {
            delegation_contract,
            undelegate_epoch: current_epoch,
            egld_amount,
            shares: BigUint::zero(),
            unbond_epoch,
        };

        let penalty = Penalty {
            id: penalty_id,
            withdrawn: true,
            attributes: attrs,
        };

        self.penalties(penalty_id).set(&penalty);

        let caller = self.blockchain().get_caller();
        let contract_data = contract_data_mapper.get();
        self.penalty_from_pending_to_delegate_event(&caller, &penalty, &contract_data);
    }

    /// Marks a penalty as withdrawn once the unbond period has passed. In order to be successful, the EGLD must be
    /// already in the liquid staking smart contract. For that reason, the public endpoint `withdrawFrom` should have
    /// been already called before this point.
    ///
    /// # Arguments
    ///
    /// - `penalty_id` - the penalty identifier
    ///
    #[endpoint(withdrawPenalty)]
    fn withdraw_penalty(&self, penalty_id: u64) {
        self.require_valid_penalty_id(penalty_id);

        let penalty_mapper = self.penalties(penalty_id);
        let penalty = penalty_mapper.get();
        require!(!penalty.withdrawn, ERROR_WITHDRAWN_PENALTY);

        self.withdraw_internal(&penalty.attributes);

        penalty_mapper.update(|penalty| {
            penalty.withdrawn = true;
        });

        let caller = self.blockchain().get_caller();
        let delegation_contract = penalty.attributes.delegation_contract;
        let contract_data = self.delegation_contract_data(&delegation_contract).get();
        self.withdraw_penalty_event(&caller, penalty_id, &contract_data);
    }

    /// Allows anyone to delegate a Penalty to a new Delegation smart contract based on the current configuration of the
    /// delegation algorithm, avoiding the penalized Delegation smart contract. Similarly to `delegate`, it does not
    /// perform the delegation automatically, but it can be done by anyone at any given point in time using the
    /// `delegatePendingAmount` public endpoint.
    ///
    /// # Arguments
    ///
    /// - `penalty_id` - the penalty identifier
    /// - `opt_egld_amount` - the amount of EGLD to delegate. If unspecified, it defaults to the penalty amount
    ///
    #[endpoint(delegatePenalty)]
    fn delegate_penalty(&self, penalty_id: u64, opt_egld_amount: OptionalValue<BigUint>) {
        self.require_active_state();
        self.require_valid_penalty_id(penalty_id);

        let penalty = self.penalties(penalty_id).get();
        require!(penalty.withdrawn, ERROR_WITHDRAW_FIRST);

        let egld_amount = match opt_egld_amount {
            OptionalValue::None => penalty.attributes.egld_amount,
            OptionalValue::Some(amount) => {
                self.require_sufficient_egld(&amount);
                require!(amount <= penalty.attributes.egld_amount, ERROR_TOO_MUCH_EGLD_AMOUNT);
                let amount_left = &penalty.attributes.egld_amount - &amount;
                self.require_no_dust_left(&amount_left);
                amount
            },
        };

        self.reduce_penalty(penalty_id, &egld_amount);

        let delegation_contract = self.get_delegation_contract_for_delegate(
            &egld_amount,
            &OptionalValue::Some(penalty.attributes.delegation_contract),
        );

        let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
        contract_data_mapper.update(|data| {
            data.pending_to_delegate += &egld_amount;
        });

        let caller = self.blockchain().get_caller();
        let contract_data = contract_data_mapper.get();
        self.delegate_penalty_event(&caller, &delegation_contract, penalty_id, &egld_amount, &contract_data);
    }

    /// A public endpoint that allows users to withdraw from penalties when the undelegation mode is set to `Free`.
    /// Since penalties must be already marked as `withdrawn`, which means that the EGLD is already available at this
    /// smart contract, the EGLD is sent directly to the user and the penalty is updated or cleared.
    ///
    /// # Arguments
    ///
    /// - `penalty_id` - the penalty identifier
    ///
    #[payable("*")]
    #[endpoint(withdrawFromPenalty)]
    fn withdraw_from_penalty(&self, penalty_id: u64) {
        self.require_open_mode();
        self.require_valid_penalty_id(penalty_id);

        let (ls_token_id, shares) = self.call_value().single_fungible_esdt();
        self.require_valid_shares_payment(&ls_token_id, &shares);

        let penalty = self.penalties(penalty_id).get();
        require!(penalty.withdrawn, ERROR_WITHDRAW_FIRST);

        let egld_amount = self.shares_to_egld(&shares);
        require!(
            egld_amount <= penalty.attributes.egld_amount,
            ERROR_TOO_MUCH_EGLD_AMOUNT
        );
        let amount_left = &penalty.attributes.egld_amount - &egld_amount;
        self.require_no_dust_left(&amount_left);

        self.reduce_penalty(penalty_id, &egld_amount);

        self.redeem_shares(&egld_amount, &shares);

        let caller = self.blockchain().get_caller();
        self.send().direct_egld(&caller, &egld_amount);

        self.withdraw_from_penalty_event(&caller, penalty_id, &egld_amount, &shares);
    }

    fn reduce_penalty(&self, penalty_id: u64, egld_amount: &BigUint) {
        let penalty_mapper = self.penalties(penalty_id);

        penalty_mapper.update(|penalty| {
            penalty.attributes.egld_amount -= egld_amount;
        });

        let penalty = penalty_mapper.get();
        if penalty.attributes.egld_amount == BigUint::zero() {
            penalty_mapper.clear();
        }
    }
}
