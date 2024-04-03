/// Basis point are useful to represent percentages, such that 43.21% is represented as 4321 basis points.
pub const BPS: u64 = 10_000;

/// A WAD equals 1e18
pub const WAD: u64 = 1_000_000_000_000_000_000;

/// The devnet unbond period
pub const DEVNET_UNBOND_PERIOD: u64 = 1;

/// The mainnet unbond period
pub const MAINNET_UNBOND_PERIOD: u64 = 10;

/// The initial exchange rate between EGLD and sEGLD
pub const INITIAL_EXCHANGE_RATE: u64 = 1_000_000_000_000_000_000;

/// The minimum amount of EGLD that can be delegated or undelegated from a Delegation smart contract
pub const MIN_DELEGATION_AMOUNT: u64 = 1_000_000_000_000_000_000;

/// The maximum number of Delegation smart contracts that can be registered based on gas limits
pub const MAX_DELEGATION_CONTRACTS_LIST_SIZE: usize = 100;

/// The gas provided for any async call to a delegation contract, including delegate, undelegate, withdraw or claim
/// rewards
pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;

/// The gas reserved for any of the callbacks located at the Liquid Staking smart contract
pub const MIN_GAS_FOR_CALLBACK: u64 = 12_000_000;

/// The undelegate token Uri
pub const UNDELEGATE_TOKEN_URI: &[u8] =
    b"https://arweave.net/mfjIHO6ckE8m1ck_b46BdV4ZFVGEEHJSno2MnFKuzgk/undelegate-nft.png";

/// The number of epochs before the undelegation algorithm can be deactivated and bypassed by anyone if no successful
/// undelegations occur
pub const NO_UNDELEGATE_EPOCHS: u64 = 10;

/// The number of epochs before the undelegation algorithm can be deactivated and bypassed by anyone if no successful
/// Delegation contract data updates occur
pub const NO_DATA_UPDATE_EPOCHS: u64 = 10;

/// The number of epochs the admin must wait until the undelegation algorithm can be reactivated once it has been
/// deactivated.
pub const COOLDOWN_REACTIVATE_UNDELEGATION_ALGORITHM: u64 = 1;
