use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum CyloEscrowErrors {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidOrderId = 3,
    Unauthorized = 4,
    InvalidInput = 5,
    TransferFailed = 6,
    AlreadyProcessed = 7,
    NotExpired = 8,
}
