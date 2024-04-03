multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use super::storage;

#[multiversx_sc::module]
pub trait ProxyModule: storage::StorageModule {
    fn delegate_to_delegation_contract(
        &self,
        delegation_contract: ManagedAddress,
        egld_amount: BigUint,
        gas_for_async_call: u64,
        callback: CallbackClosure<<Self as ContractBase>::Api>,
    ) {
        self.delegation_contract_proxy()
            .contract(delegation_contract)
            .delegate()
            .with_gas_limit(gas_for_async_call)
            .with_egld_transfer(egld_amount)
            .async_call()
            .with_callback(callback)
            .call_and_exit()
    }

    fn undelegate_from_delegation_contract(
        &self,
        delegation_contract: ManagedAddress,
        egld_amount: BigUint,
        gas_for_async_call: u64,
        callback: CallbackClosure<<Self as ContractBase>::Api>,
    ) {
        self.delegation_contract_proxy()
            .contract(delegation_contract)
            .undelegate(egld_amount)
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(callback)
            .call_and_exit()
    }

    fn withdraw_from_delegation_contract(
        &self,
        delegation_contract: ManagedAddress,
        gas_for_async_call: u64,
        callback: CallbackClosure<<Self as ContractBase>::Api>,
    ) {
        self.delegation_contract_proxy()
            .contract(delegation_contract)
            .withdraw()
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(callback)
            .call_and_exit()
    }

    fn claim_rewards_from_delegation_contract(
        &self,
        delegation_contract: ManagedAddress,
        gas_for_async_call: u64,
        callback: CallbackClosure<<Self as ContractBase>::Api>,
    ) {
        self.delegation_contract_proxy()
            .contract(delegation_contract)
            .claim_rewards()
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(callback)
            .call_and_exit()
    }

    fn get_random(&self, min: &BigUint, max: &BigUint) -> BigUint {
        let random_contract = self.random_oracle().get();
        self.random_proxy()
            .contract(random_contract)
            .get_random(min.clone(), max.clone())
            .execute_on_dest_context()
    }

    #[proxy]
    fn delegation_contract_proxy(&self) -> delegation_mod::Proxy<Self::Api>;

    #[proxy]
    fn random_proxy(&self) -> random_mod::Proxy<Self::Api>;
}

pub mod delegation_mod {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait DelegationProxy {
        #[payable("EGLD")]
        #[endpoint(delegate)]
        fn delegate(&self);

        #[endpoint(unDelegate)]
        fn undelegate(&self, egld_amount: BigUint);

        #[endpoint(withdraw)]
        fn withdraw(&self);

        #[endpoint(claimRewards)]
        fn claim_rewards(&self);
    }
}

pub mod random_mod {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait RandomProxy {
        #[view(getRandom)]
        fn get_random(&self, min: BigUint, max: BigUint) -> BigUint;
    }
}
