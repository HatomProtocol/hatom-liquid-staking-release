multiversx_sc::imports!();
use super::{common, constants::*, errors::*, events, model::*, proxies, score, storage};

#[multiversx_sc::module]
pub trait SelectionModule:
    admin::AdminModule
    + common::CommonModule
    + events::EventsModule
    + proxies::ProxyModule
    + score::ScoreModule
    + storage::StorageModule
{
    /// Returns a Delegation smart contract for delegations based on the current configuration of the delegation score
    /// model and the delegation sampling model.
    ///
    /// # Arguments
    ///
    /// - `egld_amount` - the EGLD amount being delegated
    /// - `opt_address` - avoid this Delegation smart contract to be selected
    ///
    fn get_delegation_contract_for_delegate(
        &self,
        egld_amount: &BigUint,
        opt_address: &OptionalValue<ManagedAddress>,
    ) -> ManagedAddress<Self::Api> {
        // get the best candidate as a node from the linked list
        let best_node = self.get_max_delegation_contract_node(egld_amount, opt_address);

        // return the best candidate iff there is no sampling
        if self.delegation_sampling_model().is_empty() {
            return best_node.into_value();
        }

        // compute a list of candidates based on their scores
        let (candidates, sum_weights) = self.get_delegation_candidates(&best_node, egld_amount, opt_address);

        // return the best candidate iff no other candidates have been found
        if candidates.len() == 1usize {
            return best_node.into_value();
        }

        self.weighted_sample(candidates, sum_weights)
    }

    /// Returns an undelegation smart contract for delegations based on the current configuration of the delegation
    /// score model and the delegation sampling model.
    ///
    /// # Arguments
    ///
    /// - `egld_amount` - the EGLD amount being undelegated
    ///
    fn get_delegation_contract_for_undelegate(&self, egld_amount: &BigUint) -> ManagedAddress<Self::Api> {
        // get the best candidate as a node from the linked list
        let min_node = self.get_min_delegation_contract_node(egld_amount);

        // return the best candidate iff there is no sampling
        if self.delegation_sampling_model().is_empty() {
            return min_node.into_value();
        }

        let (candidates, sum_weights) = self.get_undelegation_candidates(&min_node, egld_amount);

        if candidates.len() == 1usize {
            return min_node.into_value();
        }

        self.weighted_sample(candidates, sum_weights)
    }

    /// Returns the Delegation smart contract with the highest score that is not outdated, can receive the delegated
    /// amount and, optionally, does not match the provided smart contract address.
    ///
    fn get_max_delegation_contract_node(
        &self,
        egld_amount: &BigUint,
        opt_skip_address: &OptionalValue<ManagedAddress>,
    ) -> LinkedListNode<ManagedAddress<Self::Api>> {
        require!(
            !self.delegation_contracts_list().is_empty(),
            ERROR_NO_DELEGATION_CONTRACTS
        );

        for node in self.delegation_contracts_list().iter() {
            let delegation_contract = node.get_value_cloned();
            let contract_data = self.delegation_contract_data(&delegation_contract).get();

            if !self.is_valid_delegation_contract(&contract_data, egld_amount, opt_skip_address) {
                continue;
            }

            return node;
        }

        sc_panic!(ERROR_DELEGATION_CONTRACT_NOT_AVAILABLE);
    }

    /// Returns the undelegation smart contract with the lowest score that is not outdated and has received a sufficient
    /// delegated amount.
    ///
    fn get_min_delegation_contract_node(&self, egld_amount: &BigUint) -> LinkedListNode<ManagedAddress<Self::Api>> {
        require!(
            !self.delegation_contracts_list().is_empty(),
            ERROR_NO_DELEGATION_CONTRACTS
        );

        let delegation_contracts_mapper = self.delegation_contracts_list();

        let mut opt_node = delegation_contracts_mapper.back();
        while opt_node.is_some() {
            let node = opt_node.unwrap();
            let delegation_contract = node.get_value_cloned();
            let contract_data = self.delegation_contract_data(&delegation_contract).get();
            if self.is_valid_undelegation_contract(&contract_data, egld_amount) {
                return node;
            }

            opt_node = delegation_contracts_mapper.get_node_by_id(node.get_prev_node_id());
        }

        sc_panic!(ERROR_DELEGATION_CONTRACT_NOT_AVAILABLE);
    }

    /// Returns delegation candidates based on their closeness to the best node delegation score. It also computes their
    /// weights.
    ///
    fn get_delegation_candidates(
        &self,
        best_node: &LinkedListNode<ManagedAddress<Self::Api>>,
        egld_amount: &BigUint,
        opt_skip_address: &OptionalValue<ManagedAddress>,
    ) -> (ManagedVec<DelegationCandidate<Self::Api>>, BigUint<Self::Api>) {
        let best_node_id = best_node.get_node_id();
        let best_delegation_contract = best_node.get_value_cloned();
        let best_delegation_contract_data = self.delegation_contract_data(&best_delegation_contract).get();
        let best_delegation_score = best_delegation_contract_data.delegation_score;

        let mut candidates: ManagedVec<DelegationCandidate<Self::Api>> = ManagedVec::new();
        let cutoff_score = self.get_max_cutoff_score(&best_delegation_score);

        let mut sum_weights = BigUint::zero();
        for node in self.delegation_contracts_list().iter_from_node_id(best_node_id) {
            let delegation_contract = node.get_value_cloned();
            let contract_data = self.delegation_contract_data(&delegation_contract).get();

            if contract_data.delegation_score < cutoff_score {
                break;
            }

            if !self.is_valid_delegation_contract(&contract_data, egld_amount, opt_skip_address) {
                continue;
            }

            let weight = self.compute_delegate_weight(&contract_data);
            let candidate = DelegationCandidate {
                data: contract_data,
                weight: weight.clone(),
            };

            candidates.push(candidate);
            sum_weights += weight;
        }

        (candidates, sum_weights)
    }

    /// Returns undelegation candidates based on their closeness to the best node delegation score. It also computes
    /// their weights.
    ///
    fn get_undelegation_candidates(
        &self,
        best_node: &LinkedListNode<ManagedAddress<Self::Api>>,
        egld_amount: &BigUint,
    ) -> (ManagedVec<DelegationCandidate<Self::Api>>, BigUint<Self::Api>) {
        let best_node_id = best_node.get_node_id();
        let best_delegation_contract = best_node.get_value_cloned();
        let best_delegation_contract_data = self.delegation_contract_data(&best_delegation_contract).get();
        let best_delegation_score = best_delegation_contract_data.delegation_score;

        let mut sum_weights = BigUint::zero();
        let mut candidates: ManagedVec<DelegationCandidate<Self::Api>> = ManagedVec::new();
        let cutoff_score = self.get_min_cutoff_score(&best_delegation_score);
        let delegation_contracts_mapper = self.delegation_contracts_list();
        let mut opt_node = delegation_contracts_mapper.get_node_by_id(best_node_id);
        while opt_node.is_some() {
            let node = opt_node.unwrap();
            let address = node.get_value_cloned();
            let contract_data = self.delegation_contract_data(&address).get();

            // get next node
            opt_node = delegation_contracts_mapper.get_node_by_id(node.get_prev_node_id());

            if contract_data.delegation_score > cutoff_score {
                break;
            }

            if !self.is_valid_undelegation_contract(&contract_data, egld_amount) {
                continue;
            }

            let weight = self.compute_undelegate_weight(&contract_data);
            let candidate = DelegationCandidate {
                data: contract_data,
                weight: weight.clone(),
            };

            candidates.push(candidate);
            sum_weights += weight;
        }

        (candidates, sum_weights)
    }

    fn is_valid_delegation_contract(
        &self,
        contract_data: &DelegationContractData<Self::Api>,
        egld_amount: &BigUint,
        opt_skip_address: &OptionalValue<ManagedAddress>,
    ) -> bool {
        if contract_data.outdated {
            return false;
        }

        if opt_skip_address.is_some() && opt_skip_address.clone().into_option().unwrap() == contract_data.contract {
            return false;
        }

        if !self.has_valid_cap(contract_data, egld_amount) {
            return false;
        }

        if !self.has_valid_service_fee(contract_data) {
            return false;
        }

        true
    }

    fn has_valid_cap(&self, contract_data: &DelegationContractData<Self::Api>, egld_amount: &BigUint) -> bool {
        if contract_data.cap.is_some() {
            let cap = contract_data.cap.as_ref().unwrap();
            let amount_left = cap - &contract_data.total_value_locked;
            if egld_amount > &amount_left {
                return false;
            }
        }
        true
    }

    fn has_valid_service_fee(&self, contract_data: &DelegationContractData<Self::Api>) -> bool {
        // don't check the service fee if there is no sampling.
        if self.delegation_sampling_model().is_empty() {
            return true;
        }

        let model = self.delegation_sampling_model().get();
        let max_service_fee = model.max_service_fee;
        if contract_data.service_fee > max_service_fee {
            return false;
        }
        true
    }

    fn is_valid_undelegation_contract(
        &self,
        contract_data: &DelegationContractData<Self::Api>,
        egld_amount: &BigUint,
    ) -> bool {
        if contract_data.outdated {
            return false;
        }
        self.is_valid_undelegation_contract_relaxed(contract_data, egld_amount)
    }

    fn is_valid_undelegation_contract_relaxed(
        &self,
        contract_data: &DelegationContractData<Self::Api>,
        egld_amount: &BigUint,
    ) -> bool {
        if egld_amount > &contract_data.total_delegated {
            return false;
        }
        // avoid leaving dust at the Delegation smart contract
        let min_egld_to_delegate = BigUint::from(MIN_DELEGATION_AMOUNT);
        let amount_left = &contract_data.total_delegated - egld_amount;
        if amount_left != BigUint::zero() && amount_left < min_egld_to_delegate {
            return false;
        }
        true
    }

    /// Gets a cutoff score using a tolerance that is applied to the region `[0, delegation_score]`
    ///
    fn get_max_cutoff_score(&self, delegation_score: &BigUint) -> BigUint {
        let model = self.delegation_sampling_model().get();
        let sampling_tolerance = model.tolerance;
        let bps = BigUint::from(BPS);
        let delta_score = delegation_score * &sampling_tolerance / &bps;
        delegation_score - &delta_score
    }

    /// Gets a cutoff score using a tolerance that is applied to the region `[delegation_score, 1]`.
    ///
    fn get_min_cutoff_score(&self, delegation_score: &BigUint) -> BigUint {
        let model = self.delegation_sampling_model().get();
        let sampling_tolerance = model.tolerance;
        let bps = BigUint::from(BPS);
        let delta_score = (&bps - delegation_score) * sampling_tolerance / &bps;
        delegation_score + &delta_score
    }

    /// Returns a delegation weight based on the Delegation smart contract service fee and the following function
    /// (dotted line):
    ///
    //             weight
    //                 │
    //                 │
    //                 │
    //   bps + premium +
    //                 │    .
    //                 │         .
    //                 │              .
    //                 │                   .
    //                 │                        .
    //                 │                             .
    //                 │                                  .
    //             bps _──────────────────────────────────────+
    //                 │                                      .
    //                 │                                      .
    //                 │                                      .
    //                 │                                      .
    //                0└──────────────────────────────────────+...................> service_fee
    //                 0                               max_service_fee
    ///
    fn compute_delegate_weight(&self, candidate: &DelegationContractData<Self::Api>) -> BigUint {
        let bps = BigUint::from(BPS);
        let sampling_model = self.delegation_sampling_model().get();
        let SamplingModel {
            max_service_fee,
            premium,
            ..
        } = sampling_model;

        // cannot be sampled
        if candidate.service_fee > max_service_fee {
            return BigUint::zero();
        }

        let f = self.norm_linear_clamp(&candidate.service_fee, &BigUint::zero(), &max_service_fee, true);
        f * premium / &bps + bps
    }

    /// Returns an undelegation weight based on the Delegation smart contract service fee and the following function
    /// (dotted line):
    ///
    //             weight
    //                 │                                                   .
    //                 │                                               .
    //                 │                                           .
    //   bps + premium +──────────────────────────────────────+
    //                 │                                  .
    //                 │                             .
    //                 │                        .
    //                 │                   .
    //                 │              .
    //                 │         .
    //                 │    .
    //             bps +
    //                 │
    //                 │
    //                 │
    //                 │
    //                0└──────────────────────────────────────+─────────> service_fee
    //                 0                             max_service_fee
    ///
    fn compute_undelegate_weight(&self, candidate: &DelegationContractData<Self::Api>) -> BigUint {
        let bps = BigUint::from(BPS);
        let sampling_model = self.delegation_sampling_model().get();
        let SamplingModel {
            max_service_fee,
            premium,
            ..
        } = sampling_model;

        premium * &candidate.service_fee / max_service_fee + bps
    }

    /// Select a single random integer in `0..weights.len()-1` with probabilities proportional to the weights.
    ///
    fn weighted_sample(
        &self,
        candidates: ManagedVec<DelegationCandidate<Self::Api>>,
        sum_weights: BigUint,
    ) -> ManagedAddress {
        let t = if self.random_oracle().is_empty() {
            let bps = BigUint::from(BPS);
            let mut rng = RandomnessSource::default();
            let rand = BigUint::from(rng.next_u32_in_range(0, BPS as u32));
            rand * sum_weights / bps
        } else {
            self.get_random(&BigUint::zero(), &sum_weights)
        };

        let n = candidates.len();
        let mut i = 0usize;
        let mut cw = candidates.get(0usize).weight;

        while cw < t && i < n - 1usize {
            i += 1usize;
            cw += candidates.get(i).weight;
        }

        let winner = candidates.get(i);
        winner.data.contract
    }
}
