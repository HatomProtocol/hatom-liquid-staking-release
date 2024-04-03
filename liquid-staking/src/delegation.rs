multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use super::{common, constants::*, errors::*, events, proxies, score, selection, storage};

#[multiversx_sc::module]
pub trait DelegationModule:
    admin::AdminModule
    + common::CommonModule
    + events::EventsModule
    + proxies::ProxyModule
    + score::ScoreModule
    + selection::SelectionModule
    + storage::StorageModule
{
    /// Removes a Delegation smart contract from the Delegation smart contracts list based on its value (i.e. address)
    /// if it belongs to the list.
    ///
    fn remove_delegation_contract_from_list(&self, new_delegation_contract: &ManagedAddress) {
        for node in self.delegation_contracts_list().iter() {
            let node_id = node.get_node_id();
            let delegation_contract = node.into_value();
            if new_delegation_contract == &delegation_contract {
                self.delegation_contracts_list().remove_node_by_id(node_id);
                break;
            }
        }
    }

    /// Adds a given Staking Provider Delegation smart contract into the Delegation smart contracts list based on its
    /// provided delegation score. It is intended to be called after removing the Delegation smart contract from the
    /// Delegation smart contracts list.
    ///
    /// # Notes
    ///
    /// - if there is an existent Delegation smart contract with the same score, the new Delegation smart contract will
    ///   have priority.
    ///
    fn add_and_order_delegation_contract_in_list(
        &self,
        new_delegation_contract: &ManagedAddress,
        delegation_score: &BigUint,
    ) {
        let mut delegation_contracts_mapper = self.delegation_contracts_list();
        require!(
            delegation_contracts_mapper.len() < MAX_DELEGATION_CONTRACTS_LIST_SIZE,
            ERROR_DELEGATION_CONTRACTS_LIST_FULL
        );
        if delegation_contracts_mapper.is_empty() {
            delegation_contracts_mapper.push_front(new_delegation_contract.clone());
        } else {
            let mut added = false;
            for node in delegation_contracts_mapper.iter() {
                let node_id = node.get_node_id();
                let delegation_contract = node.into_value();
                let delegation_contract_data = self.delegation_contract_data(&delegation_contract).get();
                if delegation_score >= &delegation_contract_data.delegation_score {
                    delegation_contracts_mapper.push_before_node_id(node_id, new_delegation_contract.clone());
                    added = true;
                    break;
                }
            }
            if !added {
                delegation_contracts_mapper.push_back(new_delegation_contract.clone());
            }
        }
    }

    /// Computes all delegation scores and sorts the Delegation smart contracts list based on these new values.
    ///
    fn sort_delegation_contracts_list(&self) {
        for node in self.delegation_contracts_list().iter() {
            let delegation_contract = node.into_value();
            let contract_data_mapper = self.delegation_contract_data(&delegation_contract);
            let contract_data = contract_data_mapper.get();

            let delegation_score = self.compute_delegation_score(&contract_data);

            if contract_data.delegation_score != delegation_score {
                self.remove_delegation_contract_from_list(&delegation_contract);
                self.add_and_order_delegation_contract_in_list(&delegation_contract, &delegation_score);
            }

            contract_data_mapper.update(|contract_data| {
                contract_data.delegation_score = delegation_score;
            });
        }
    }
}
