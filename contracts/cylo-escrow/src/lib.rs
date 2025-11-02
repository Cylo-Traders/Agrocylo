#![no_std]
use soroban_sdk::{contract, contractimpl, Env, contracttype, contracterror, Address, String};

pub const EXPIRATION_PERIOD: u64 = 96 * 60 * 60;

#[contract]
pub struct CyloEscrow;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Farmer,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum CyloEscrowErrors {
    AlreadyInitialized = 1,
}

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
    pub expired: bool,
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

#[contractimpl]
impl CyloEscrow {
    
}