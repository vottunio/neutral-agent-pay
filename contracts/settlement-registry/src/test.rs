#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::BytesN as _, Env};

fn setup() -> (Env, SettlementRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(SettlementRegistry, ());
    let client = SettlementRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);
    let recorder = Address::generate(&env);
    client.initialize(&admin, &recorder);
    (env, client)
}

#[test]
fn records_and_reads_back_a_payment() {
    let (env, client) = setup();
    let payment_id = BytesN::random(&env);
    let agent = Address::generate(&env);
    let merchant = Address::generate(&env);
    let asset = Address::generate(&env);

    client.record(&payment_id, &agent, &merchant, &asset, &2_500);

    let record = client.get(&payment_id);
    assert_eq!(record.amount, 2_500);
    assert_eq!(record.agent, agent);
    assert_eq!(record.merchant, merchant);
    assert!(client.exists(&payment_id));
}

#[test]
fn duplicate_payment_ids_are_rejected() {
    let (env, client) = setup();
    let payment_id = BytesN::random(&env);
    let agent = Address::generate(&env);
    let merchant = Address::generate(&env);
    let asset = Address::generate(&env);

    client.record(&payment_id, &agent, &merchant, &asset, &10);
    let err = client
        .try_record(&payment_id, &agent, &merchant, &asset, &10)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, RegistryError::DuplicatePayment);
}

#[test]
fn unknown_payment_returns_not_found() {
    let (env, client) = setup();
    let err = client
        .try_get(&BytesN::random(&env))
        .unwrap_err()
        .unwrap();
    assert_eq!(err, RegistryError::PaymentNotFound);
}

#[test]
fn non_positive_amounts_are_rejected() {
    let (env, client) = setup();
    let err = client
        .try_record(
            &BytesN::random(&env),
            &Address::generate(&env),
            &Address::generate(&env),
            &Address::generate(&env),
            &0,
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, RegistryError::AmountNotPositive);
}
