use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Order {
    pub id: u32,
    pub buyer: Address,
    pub farmer: Address,
    pub token: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub confirmed: bool,
    pub refunded: bool,
    pub metadata: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderStatus {
    pub is_confirmed: bool,
    pub is_refunded: bool,
    pub is_expired: bool,
    pub time_remaining: u64,
}
