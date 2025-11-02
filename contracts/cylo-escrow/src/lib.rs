#![no_std]

use soroban_sdk::{Address, Env, String, Symbol, Vec, contract, contracterror, contractimpl, contracttype, symbol_short, xdr::Error};

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowEvent {
    OrderCreated(u32, Address, Address, i128),
    OrderConfirmed(u32),
    OrderRefunded(u32),
    OrderExpired(u32),
}

#[contracttype]
pub struct AdminData {
    pub admin: Address,
    pub initialized: bool,
}


type CyloEscrowResult<T> = Result<T, CyloEscrowErrors>;

pub struct Storage;

// impl Storage {
    // ========== ADMIN STORAGE ==========
//     const ADMIN_KEY= "Admin";

//     pub fn get_admin(env: &Env) -> Option<AdminData> {
//         env.storage().persistent().get(&symbol_short!(DataKey::Farmer))
//     }

//     pub fn set_admin(env: &Env, admin_data: &AdminData) {
//         env.storage().persistent().set(&symbol_short!(Self::ADMIN_KEY), admin_data);
//     }

//     pub fn require_admin(env: &Env, caller: &Address) {
//         if let Some(admin_data) = Self::get_admin(env) {
//             if &admin_data.admin != caller {
//                 env.throw_error(EscrowError::Unauthorized);
//             }
//         } else {
//             env.throw_error(EscrowError::NotInitialized);
//         }
//     }

//     // ========== ORDER STORAGE ==========
//     const ORDER_COUNT_KEY: &'static str = "OrderCount";
//     const ORDER_PREFIX: &'static str = "Order";
//     const BUYER_ORDERS_PREFIX: &'static str = "BuyerOrders";
//     const FARMER_ORDERS_PREFIX: &'static str = "FarmerOrders";
//     const ACTIVE_ORDERS_KEY: &'static str = "ActiveOrders";

//     pub fn get_next_order_id(env: &Env) -> u32 {
//         let current_count: u32 = env.storage().persistent()
//             .get(&symbol_short!(Self::ORDER_COUNT_KEY))
//             .unwrap_or(0);
//         let next_id = current_count + 1;
//         env.storage().persistent()
//             .set(&symbol_short!(Self::ORDER_COUNT_KEY), &next_id);
//         next_id
//     }

//     pub fn get_last_order_id(env: &Env) -> u32 {
//         env.storage().persistent()
//             .get(&symbol_short!(Self::ORDER_COUNT_KEY))
//             .unwrap_or(0)
//     }

//     pub fn save_order(env: &Env, order_id: u32, order: &Order) {
//         let key = (symbol_short!(Self::ORDER_PREFIX), order_id);
//         env.storage().persistent().set(&key, order);

//         // Add to active orders if not completed
//         if !order.confirmed && !order.refunded {
//             Self::add_to_active_orders(env, order_id);
//         } else {
//             Self::remove_from_active_orders(env, order_id);
//         }
//     }

//     pub fn get_order(env: &Env, order_id: u32) -> Option<Order> {
//         let key = (symbol_short!(Self::ORDER_PREFIX), order_id);
//         env.storage().persistent().get(&key)
//     }

//     pub fn require_order(env: &Env, order_id: u32) -> Order {
//         Self::get_order(env, order_id)
//             .unwrap_or_else(|| env.throw_error(EscrowError::OrderNotFound))
//     }

//     // ========== USER ORDER MAPPINGS ==========
//     pub fn add_order_to_buyer(env: &Env, buyer: &Address, order_id: u32) {
//         let key = (symbol_short!(Self::BUYER_ORDERS_PREFIX), buyer);
//         let mut orders: Vec<u32> = env.storage().persistent()
//             .get(&key)
//             .unwrap_or_else(|| Vec::new(env));
//         orders.push_back(order_id);
//         env.storage().persistent().set(&key, &orders);
//     }

//     pub fn get_buyer_orders(env: &Env, buyer: &Address) -> Vec<u32> {
//         let key = (symbol_short!(Self::BUYER_ORDERS_PREFIX), buyer);
//         env.storage().persistent()
//             .get(&key)
//             .unwrap_or_else(|| Vec::new(env))
//     }

//     pub fn add_order_to_farmer(env: &Env, farmer: &Address, order_id: u32) {
//         let key = (symbol_short!(Self::FARMER_ORDERS_PREFIX), farmer);
//         let mut orders: Vec<u32> = env.storage().persistent()
//             .get(&key)
//             .unwrap_or_else(|| Vec::new(env));
//         orders.push_back(order_id);
//         env.storage().persistent().set(&key, &orders);
//     }

//     pub fn get_farmer_orders(env: &Env, farmer: &Address) -> Vec<u32> {
//         let key = (symbol_short!(Self::FARMER_ORDERS_PREFIX), farmer);
//         env.storage().persistent()
//             .get(&key)
//             .unwrap_or_else(|| Vec::new(env))
//     }

//     // ========== ACTIVE ORDERS MANAGEMENT ==========
//     fn add_to_active_orders(env: &Env, order_id: u32) {
//         let key = symbol_short!(Self::ACTIVE_ORDERS_KEY);
//         let mut active_orders: Vec<u32> = env.storage().persistent()
//             .get(&key)
//             .unwrap_or_else(|| Vec::new(env));
        
//         if !active_orders.contains(&order_id) {
//             active_orders.push_back(order_id);
//             env.storage().persistent().set(&key, &active_orders);
//         }
//     }

//     fn remove_from_active_orders(env: &Env, order_id: u32) {
//         let key = symbol_short!(Self::ACTIVE_ORDERS_KEY);
//         if let Some(mut active_orders) = env.storage().persistent().get::<Vec<u32>>(&key) {
//             let mut new_orders = Vec::new(env);
//             for id in active_orders.iter() {
//                 if id != order_id {
//                     new_orders.push_back(id);
//                 }
//             }
//             env.storage().persistent().set(&key, &new_orders);
//         }
//     }

//     pub fn get_active_orders(env: &Env) -> Vec<u32> {
//         let key = symbol_short!(Self::ACTIVE_ORDERS_KEY);
//         env.storage().persistent()
//             .get(&key)
//             .unwrap_or_else(|| Vec::new(env))
//     }
// }
#[contractimpl]
impl CyloEscrow {
    
}