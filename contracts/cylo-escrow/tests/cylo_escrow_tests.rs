// #![cfg(test)]

// use soroban_sdk::{
//     testutils::{Address as _, Ledger, Token as _},
//     token::Client as TokenClient,
//     Address, Env, String, Vec,
// };

// use cylo_escrow::{
//     CyloEscrow, CyloEscrowClient, cylo_escrow_types::*, errors::CyloEscrowErrors, EXPIRATION_PERIOD,
// };

// /// Test setup helper
// fn setup() -> (Env, CyloEscrowClient<'static>, Address, Address, Address, Address) {
//     let env = Env::default();
//     env.mock_all_auths();

//     let contract_id = env.register_contract(None, CyloEscrow);
//     let client = CyloEscrowClient::new(&env, &contract_id);

//     let admin = Address::generate(&env);
//     let buyer = Address::generate(&env);
//     let farmer = Address::generate(&env);

//     // Create mock token (simulating XLM or custom asset)
//     let token_id = env.register_stellar_asset_contract(admin.clone());
//     let token = TokenClient::new(&env, &token_id);
//     token.mint(&buyer, &1_000_000);

//     (env, client, admin, buyer, farmer, token_id)
// }

// /// Helper to advance time
// fn advance_time(env: &Env, seconds: u64) {
//     let mut ledger = env.ledger().get();
//     ledger.timestamp += seconds;
//     env.ledger().set(ledger);
// }

// #[test]
// fn test_init() {
//     let (env, client, admin, _, _, _) = setup();

//     // Initialize contract
//     let res = client.__init(&admin);
//     assert_eq!(res, Ok(()));

//     // Double init should fail
//     let res = client.__init(&admin);
//     assert_eq!(res, Err(CyloEscrowErrors::AlreadyInitialized));
// }

// #[test]
// fn test_create_order() {
//     let (env, client, admin, buyer, farmer, token) = setup();
//     client.__init(&admin).unwrap();

//     // Successful order
//     let order_id = client.create_order(
//         &buyer,
//         &farmer,
//         &token,
//         &100,
//         &String::from_str(&env, "Test Order"),
//     ).unwrap();
//     assert_eq!(order_id, 1);

//     // Zero amount should fail
//     let err = client.create_order(
//         &buyer,
//         &farmer,
//         &token,
//         &0,
//         &String::from_str(&env, "Invalid"),
//     );
//     assert_eq!(err, Err(CyloEscrowErrors::InvalidInput));

//     // Buyer == Farmer should fail
//     let err = client.create_order(
//         &buyer,
//         &buyer,
//         &token,
//         &50,
//         &String::from_str(&env, "Self"),
//     );
//     assert_eq!(err, Err(CyloEscrowErrors::InvalidInput));
// }

// #[test]
// fn test_confirm_receipt() {
//     let (env, client, admin, buyer, farmer, token) = setup();
//     client.__init(&admin).unwrap();

//     let order_id = client.create_order(
//         &buyer,
//         &farmer,
//         &token,
//         &100,
//         &String::from_str(&env, "Order"),
//     ).unwrap();

//     // Buyer confirms
//     client.confirm_receipt(&buyer, &order_id).unwrap();

//     // Confirming again should fail
//     let err = client.confirm_receipt(&buyer, &order_id);
//     assert_eq!(err, Err(CyloEscrowErrors::AlreadyProcessed));

//     // Wrong buyer cannot confirm
//     let wrong = Address::generate(&env);
//     let err = client.confirm_receipt(&wrong, &order_id);
//     assert_eq!(err, Err(CyloEscrowErrors::Unauthorized));
// }

// #[test]
// fn test_refund_order() {
//     let (env, client, admin, buyer, farmer, token) = setup();
//     client.__init(&admin).unwrap();

//     let order_id = client.create_order(
//         &buyer,
//         &farmer,
//         &token,
//         &150,
//         &String::from_str(&env, "Refund Test"),
//     ).unwrap();

//     // Refund before expiry → NotExpired
//     let res = client.refund_order(&order_id);
//     assert_eq!(res, Err(CyloEscrowErrors::NotExpired));

//     // Advance past expiry
//     advance_time(&env, EXPIRATION_PERIOD + 10);

//     // Should succeed
//     client.refund_order(&order_id).unwrap();

//     // Trying again → NotExpired (already refunded)
//     let res = client.refund_order(&order_id);
//     assert_eq!(res, Err(CyloEscrowErrors::NotExpired));
// }

// #[test]
// fn test_refund_expired_orders() {
//     let (env, client, admin, buyer, farmer, token) = setup();
//     client.__init(&admin).unwrap();

//     // Create 3 orders
//     for _ in 0..3 {
//         client.create_order(
//             &buyer,
//             &farmer,
//             &token,
//             &100,
//             &String::from_str(&env, "Batch"),
//         ).unwrap();
//     }

//     advance_time(&env, EXPIRATION_PERIOD + 1);

//     // Refund all
//     let processed = client.refund_expired_orders(&1, &5).unwrap();
//     assert_eq!(processed, 3);
// }

// #[test]
// fn test_get_orders_by_buyer_and_farmer() {
//     let (env, client, admin, buyer, farmer, token) = setup();
//     client.__init(&admin).unwrap();

//     let id1 = client.create_order(
//         &buyer,
//         &farmer,
//         &token,
//         &100,
//         &String::from_str(&env, "O1"),
//     ).unwrap();
//     let id2 = client.create_order(
//         &buyer,
//         &farmer,
//         &token,
//         &200,
//         &String::from_str(&env, "O2"),
//     ).unwrap();

//     let buyer_orders = client.get_orders_by_buyer(&buyer);
//     assert!(buyer_orders.contains(&id1) && buyer_orders.contains(&id2));

//     let farmer_orders = client.get_orders_by_farmer(&farmer);
//     assert!(farmer_orders.contains(&id1) && farmer_orders.contains(&id2));
// }

// #[test]
// fn test_get_order_details_and_status() {
//     let (env, client, admin, buyer, farmer, token) = setup();
//     client.__init(&admin).unwrap();

//     let order_id = client.create_order(
//         &buyer,
//         &farmer,
//         &token,
//         &500,
//         &String::from_str(&env, "Details"),
//     ).unwrap();

//     let order = client.get_order_details(&order_id).unwrap();
//     assert_eq!(order.amount, 500);
//     assert!(!order.confirmed && !order.refunded);

//     let status = client.get_order_status(&order_id).unwrap();
//     assert!(!status.is_confirmed);
//     assert!(!status.is_refunded);
//     assert!(!status.is_expired);
//     assert!(status.time_remaining > 0);

//     // After expiry
//     advance_time(&env, EXPIRATION_PERIOD + 1);
//     let expired_status = client.get_order_status(&order_id).unwrap();
//     assert!(expired_status.is_expired);
// }
