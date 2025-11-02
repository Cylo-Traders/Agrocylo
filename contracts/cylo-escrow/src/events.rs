use soroban_sdk::{contractevent, Address};

#[contractevent]
pub struct ContractInitialized {
    #[topic]
    pub timestamp: u64,
    #[topic]
    pub admin: Address,
}

#[contractevent]
pub struct OrderCreated {
    #[topic]
    pub id: u32,
    #[topic]
    pub buyer: Address,
    #[topic]
    pub farmer: Address,
    pub token: Address,
    pub amount: i128,
}

#[contractevent]
pub struct OrderConfirmed {
    #[topic]
    pub id: u32,
}

#[contractevent]
pub struct OrderRefunded {
    #[topic]
    pub id: u32,
}
