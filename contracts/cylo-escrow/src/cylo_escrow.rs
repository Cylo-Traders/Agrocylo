


use soroban_sdk::{
    contract, contractevent, contracterror, contractimpl, contracttype, Address,
    Env, String, Vec,
};

use soroban_sdk::token::TokenClient;

/// Expiration window in seconds (96 hours)
pub const EXPIRATION_PERIOD: u64 = 96 * 60 * 60;

#[contract]
pub struct CyloEscrow;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,        
    LastOrderId,  
    Order(u32), 
    BuyerOrders,  
    FarmerOrders, 
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminData {
    pub admin: Address,
    pub initialized: bool,
}

impl CyloEscrow {
    
    fn set_admin(env: &Env, admin: &Address) {
        env.storage()
            .instance()
            .set(&DataKey::Admin, &AdminData { admin: admin.clone(), initialized: true });
    }

    fn admin(env: &Env) -> Option<AdminData> {
        env.storage().instance().get(&DataKey::Admin)
    }

    fn next_order_id(env: &Env) -> u32 {
        let last: u32 = env.storage().instance().get(&DataKey::LastOrderId).unwrap_or(0u32);
        let next = last.saturating_add(1);
        env.storage().instance().set(&DataKey::LastOrderId, &next);
        next
    }

    fn last_order_id(env: &Env) -> u32 {
        env.storage().instance().get(&DataKey::LastOrderId).unwrap_or(0u32)
    }

    fn save_order(env: &Env, order: &Order) {
        env.storage().instance().set(&DataKey::Order(order.id), order);
    }

    fn load_order(env: &Env, id: u32) -> Option<Order> {
        env.storage().instance().get(&DataKey::Order(id))
    }

    fn add_buyer_order(env: &Env, buyer: &Address, id: u32) {
        // store under composite key: (DataKey::BuyerOrders, buyer)
        let key = (DataKey::BuyerOrders, buyer.clone());
        let mut list: Vec<u32> = env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env));
        list.push_back(id);
        env.storage().instance().set(&key, &list);
    }

    fn get_buyer_orders_internal(env: &Env, buyer: &Address) -> Vec<u32> {
        // retrieve list stored under composite key: (DataKey::BuyerOrders, buyer)
        let key = (DataKey::BuyerOrders, buyer.clone());
        env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env))
    }

    fn add_farmer_order(env: &Env, farmer: &Address, id: u32) {
        // store under composite key: (DataKey::FarmerOrders, farmer)
        let key = (DataKey::FarmerOrders, farmer.clone());
        let mut list: Vec<u32> = env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env));
        list.push_back(id);
        env.storage().instance().set(&key, &list);
    }

    fn get_farmer_orders_internal(env: &Env, farmer: &Address) -> Vec<u32> {
        // retrieve list stored under composite key: (DataKey::FarmerOrders, farmer)
        let key = (DataKey::FarmerOrders, farmer.clone());
        env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env))
    }

    fn refund_if_expired(env: &Env, id: u32) -> Result<bool, CyloEscrowErrors> {
        let mut order = match Self::load_order(env, id) {
            Some(o) => o,
            None => return Err(CyloEscrowErrors::InvalidOrderId),
        };

        if order.refunded || order.confirmed {
            return Ok(false);
        }

        let now = env.ledger().timestamp();
        let expiry = order.timestamp.saturating_add(EXPIRATION_PERIOD);
        if now < expiry {
            return Ok(false);
        }

        // mark state first
        order.refunded = true;
        save_order_internal(env, &order);

        // transfer tokens back to buyer (contract -> buyer)
        let client = TokenClient::new(env, &order.token);
        let contract_addr = env.current_contract_address();
        client
            .try_transfer(&contract_addr, &order.buyer, &order.amount)
            .map_err(|_| CyloEscrowErrors::TransferFailed)?;

        OrderRefunded { id }.publish(env);

        Ok(true)
    }
}

// Helper to call save_order from nested functions (avoid borrow checker around impl block)
fn save_order_internal(env: &Env, order: &Order) {
    env.storage().instance().set(&DataKey::Order(order.id), order);
}

#[contractimpl]
impl CyloEscrow {
    /// Initialize contract with an admin (must authorize).
    pub fn __init(env: &Env, admin: Address) -> Result<(), CyloEscrowErrors> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(CyloEscrowErrors::AlreadyInitialized);
        }
        // set admin and init counter
        Self::set_admin(env, &admin);
        env.storage().instance().set(&DataKey::LastOrderId, &0u32);
        ContractInitialized { timestamp: env.ledger().timestamp(), admin }.publish(env);
        Ok(())
    }

    /// create_order: buyer calls to create an order and lock tokens.
    /// Buyer address is provided and must `require_auth()`. This pattern makes
    /// caller explicit and avoids implicit assumptions about invoker.
    /// Returns the new order id.
    pub fn create_order(
        env: &Env,
        buyer: Address,
        farmer: Address,
        token: Address,
        amount: i128,
        metadata: String,
    ) -> Result<u32, CyloEscrowErrors> {
        // ensure buyer authorized this action
        buyer.require_auth();

        // input validation
        if amount <= 0 {
            return Err(CyloEscrowErrors::InvalidInput);
        }
        if buyer == farmer {
            return Err(CyloEscrowErrors::InvalidInput);
        }

        // Pull tokens from buyer into this contract.
        // Use TokenClient::try_transfer(from, to, amount). This requires the `from` signer.
        let token_client = TokenClient::new(env, &token);
        let contract_addr = env.current_contract_address();
        token_client
            .try_transfer(&buyer, &contract_addr, &amount)
            .map_err(|_| CyloEscrowErrors::TransferFailed)?;

        // Create and persist order
        let id = Self::next_order_id(env);
        let now = env.ledger().timestamp();
        let order = Order {
            id,
            buyer: buyer.clone(),
            farmer: farmer.clone(),
            token: token.clone(),
            amount,
            timestamp: now,
            confirmed: false,
            refunded: false,
            metadata,
        };

        save_order_internal(env, &order);
        Self::add_buyer_order(env, &buyer, id);
        Self::add_farmer_order(env, &farmer, id);

        OrderCreated { id, buyer, farmer, token, amount }.publish(env);

        Ok(id)
    }

    /// confirm_receipt: buyer confirms the order and funds are transferred to farmer.
    /// Buyer must authorize the call and must match the order buyer.
    pub fn confirm_receipt(env: &Env, buyer: Address, order_id: u32) -> Result<(), CyloEscrowErrors> {
        buyer.require_auth();

        let mut order = match Self::load_order(env, order_id) {
            Some(o) => o,
            None => return Err(CyloEscrowErrors::InvalidOrderId),
        };

        if order.refunded || order.confirmed {
            return Err(CyloEscrowErrors::AlreadyProcessed);
        }

        if order.buyer != buyer {
            return Err(CyloEscrowErrors::Unauthorized);
        }

        // mark confirmed before transferring (checks-effects-interactions)
        order.confirmed = true;
        save_order_internal(env, &order);

        let token_client = TokenClient::new(env, &order.token);
        let contract_addr = env.current_contract_address();
        token_client
            .try_transfer(&contract_addr, &order.farmer, &order.amount)
            .map_err(|_| CyloEscrowErrors::TransferFailed)?;

        OrderConfirmed { id: order_id }.publish(env);

        Ok(())
    }

    /// refund_order: refund a single expired order back to buyer. Anyone can call.
    pub fn refund_order(env: &Env, order_id: u32) -> Result<(), CyloEscrowErrors> {
        match CyloEscrow::refund_if_expired(env, order_id) {
            Ok(true) => Ok(()),
            Ok(false) => Err(CyloEscrowErrors::NotExpired),
            Err(e) => Err(e),
        }
    }

    /// refund_expired_orders: batched processor for expired orders.
    /// Processes up to `limit` orders starting from `start_id` inclusive.
    /// Returns number processed.
    pub fn refund_expired_orders(env: &Env, start_id: u32, limit: u32) -> Result<u32, CyloEscrowErrors> {
        let last = Self::last_order_id(env);
        if last == 0 || start_id > last || limit == 0 {
            return Ok(0);
        }

        let mut processed: u32 = 0;
        let mut i = start_id;
        let end = if start_id.saturating_add(limit.saturating_sub(1)) > last {
            last
        } else {
            start_id.saturating_add(limit.saturating_sub(1))
        };

        while i <= end {
            if let Ok(refunded) = CyloEscrow::refund_if_expired(env, i) {
                if refunded {
                    processed = processed.saturating_add(1);
                }
            }
            i = i.saturating_add(1);
        }
        Ok(processed)
    }

    /// get_orders_by_buyer: returns list of order IDs associated with buyer
    pub fn get_orders_by_buyer(env: &Env, buyer: Address) -> Vec<u32> {
        Self::get_buyer_orders_internal(env, &buyer)
    }

    /// get_orders_by_farmer: returns list of order IDs associated with farmer
    pub fn get_orders_by_farmer(env: &Env, farmer: Address) -> Vec<u32> {
        Self::get_farmer_orders_internal(env, &farmer)
    }

    /// get_order_details: returns Order struct for the given id
    pub fn get_order_details(env: &Env, order_id: u32) -> Result<Order, CyloEscrowErrors> {
        match Self::load_order(env, order_id) {
            Some(o) => Ok(o),
            None => Err(CyloEscrowErrors::InvalidOrderId),
        }
    }

    /// get_order_status: helper read-only summary
    pub fn get_order_status(env: &Env, order_id: u32) -> Result<OrderStatus, CyloEscrowErrors> {
        let order = match Self::load_order(env, order_id) {
            Some(o) => o,
            None => return Err(CyloEscrowErrors::InvalidOrderId),
        };

        let now = env.ledger().timestamp();
        let expiry = order.timestamp.saturating_add(EXPIRATION_PERIOD);
        let (is_expired, time_remaining) = if now >= expiry {
            (true, 0u64)
        } else {
            (false, expiry.saturating_sub(now))
        };

        Ok(OrderStatus {
            is_confirmed: order.confirmed,
            is_refunded: order.refunded,
            is_expired: is_expired,
            time_remaining,
        })
    }
}
