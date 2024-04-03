multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug, ManagedVecItem)]
pub struct DelegationContractData<M: ManagedTypeApi> {
    /// The staking provider smart contract address
    pub contract: ManagedAddress<M>,

    /// The total value locked at the staking provider smart contract
    pub total_value_locked: BigUint<M>,

    /// The cap amount for the staking provider smart contract if existent
    pub cap: Option<BigUint<M>>,

    /// The number of nodes
    pub nr_nodes: u64,

    /// The Staking Provider APR
    pub apr: BigUint<M>,

    /// The Staking Provider service fee
    pub service_fee: BigUint<M>,

    /// The current delegation score based on the configuration of the delegation algorithm
    pub delegation_score: BigUint<M>,

    /// Tracks the liquid staking smart contract delegated amount to a staking provider smart contract that has not been
    /// delegated yet but is pending to be delegated
    pub pending_to_delegate: BigUint<M>,

    /// Tracks the liquid staking smart contract delegated amount to a staking provider smart contract that has not been
    /// undelegated yet
    pub total_delegated: BigUint<M>,

    /// Tracks the liquid staking smart contract undelegated amount from a staking provider smart contract that has not
    /// been undelegated yet but is pending to be undelegated
    pub pending_to_undelegate: BigUint<M>,

    /// Tracks the liquid staking smart contract undelegated amount from a staking provider smart contract that has not
    /// been withdrawn  yet
    pub total_undelegated: BigUint<M>,

    /// Tracks the liquid staking smart contract withdrawable amount from a staking provider smart contract. In other
    /// words, it is the amount ready to be withdrawn that has been brought from this Delegation smart contract
    pub total_withdrawable: BigUint<M>,

    /// Indicates whether this data is updated or outdated
    pub outdated: bool,

    /// Indicates whether this Staking Provider has been blacklisted
    pub blacklisted: bool,
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct UndelegateAttributes<M: ManagedTypeApi> {
    pub delegation_contract: ManagedAddress<M>,
    pub egld_amount: BigUint<M>,
    pub shares: BigUint<M>,
    pub undelegate_epoch: u64,
    pub unbond_epoch: u64,
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum PenaltySource {
    FromUndelegate,
    FromPendingToDelegate,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Penalty<M: ManagedTypeApi> {
    pub id: u64,
    pub withdrawn: bool,
    pub attributes: UndelegateAttributes<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode, NestedEncode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum DelegationScoreMethod {
    Tvl,
    Apr,
    Mixed,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode, NestedEncode, PartialEq, Eq, Clone, Debug)]
pub struct DelegationScoreModel<M: ManagedTypeApi> {
    pub method: DelegationScoreMethod,
    pub min_tvl: BigUint<M>,
    pub max_tvl: BigUint<M>,
    pub min_apr: BigUint<M>,
    pub max_apr: BigUint<M>,
    pub omega: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode, NestedEncode, PartialEq, Eq, Clone, Debug)]
pub struct SamplingModel<M: ManagedTypeApi> {
    pub tolerance: BigUint<M>,
    pub max_service_fee: BigUint<M>,
    pub premium: BigUint<M>,
}

#[derive(Clone, PartialEq, Eq, Debug, ManagedVecItem)]
pub struct DelegationCandidate<M: ManagedTypeApi> {
    pub weight: BigUint<M>,
    pub data: DelegationContractData<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum UndelegationMode {
    None,
    Algorithm,
    Open,
}
