#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, vec, Env};

struct Fixture {
    env: Env,
    client: PaymentControlsClient<'static>,
    agent: Address,
    merchant: Address,
    asset: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentControls, ());
    let client = PaymentControlsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    Fixture {
        agent: Address::generate(&env),
        merchant: Address::generate(&env),
        asset: Address::generate(&env),
        env,
        client,
    }
}

fn open_policy(env: &Env) -> Policy {
    Policy {
        per_payment_max: 1_000,
        window_max: 5_000,
        window_seconds: 3_600,
        max_payments_per_window: 10,
        allowed_destinations: vec![env],
        allowed_assets: vec![env],
        paused: false,
    }
}

#[test]
fn payment_within_limits_is_authorized() {
    let f = setup();
    f.client.set_policy(&f.agent, &open_policy(&f.env));

    f.client
        .authorize_payment(&f.agent, &f.merchant, &f.asset, &100);

    assert_eq!(f.client.spent_in_window(&f.agent), 100);
    assert_eq!(f.client.remaining_allowance(&f.agent), 4_900);
}

#[test]
fn per_payment_cap_is_enforced() {
    let f = setup();
    f.client.set_policy(&f.agent, &open_policy(&f.env));

    let err = f
        .client
        .try_authorize_payment(&f.agent, &f.merchant, &f.asset, &1_001)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, ControlError::PerPaymentMaxExceeded);
}

#[test]
fn window_amount_cap_is_enforced() {
    let f = setup();
    f.client.set_policy(&f.agent, &open_policy(&f.env));

    for _ in 0..5 {
        f.client
            .authorize_payment(&f.agent, &f.merchant, &f.asset, &1_000);
    }

    let err = f
        .client
        .try_authorize_payment(&f.agent, &f.merchant, &f.asset, &1)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, ControlError::WindowAmountExceeded);
}

#[test]
fn window_count_cap_is_enforced() {
    let f = setup();
    let mut policy = open_policy(&f.env);
    policy.max_payments_per_window = 2;
    f.client.set_policy(&f.agent, &policy);

    f.client
        .authorize_payment(&f.agent, &f.merchant, &f.asset, &1);
    f.client
        .authorize_payment(&f.agent, &f.merchant, &f.asset, &1);

    let err = f
        .client
        .try_authorize_payment(&f.agent, &f.merchant, &f.asset, &1)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, ControlError::WindowCountExceeded);
}

#[test]
fn window_resets_after_it_elapses() {
    let f = setup();
    f.client.set_policy(&f.agent, &open_policy(&f.env));

    f.client
        .authorize_payment(&f.agent, &f.merchant, &f.asset, &1_000);
    assert_eq!(f.client.spent_in_window(&f.agent), 1_000);

    f.env.ledger().with_mut(|l| l.timestamp += 3_601);

    assert_eq!(f.client.spent_in_window(&f.agent), 0);
    f.client
        .authorize_payment(&f.agent, &f.merchant, &f.asset, &1_000);
    assert_eq!(f.client.spent_in_window(&f.agent), 1_000);
}

#[test]
fn destination_allowlist_is_enforced() {
    let f = setup();
    let allowed = Address::generate(&f.env);
    let mut policy = open_policy(&f.env);
    policy.allowed_destinations = vec![&f.env, allowed.clone()];
    f.client.set_policy(&f.agent, &policy);

    f.client.authorize_payment(&f.agent, &allowed, &f.asset, &10);

    let err = f
        .client
        .try_authorize_payment(&f.agent, &f.merchant, &f.asset, &10)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, ControlError::DestinationNotAllowed);
}

#[test]
fn asset_allowlist_is_enforced() {
    let f = setup();
    let allowed_asset = Address::generate(&f.env);
    let mut policy = open_policy(&f.env);
    policy.allowed_assets = vec![&f.env, allowed_asset];
    f.client.set_policy(&f.agent, &policy);

    let err = f
        .client
        .try_authorize_payment(&f.agent, &f.merchant, &f.asset, &10)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, ControlError::AssetNotAllowed);
}

#[test]
fn pause_blocks_and_resume_restores() {
    let f = setup();
    f.client.set_policy(&f.agent, &open_policy(&f.env));

    f.client.pause(&f.agent);
    let err = f
        .client
        .try_authorize_payment(&f.agent, &f.merchant, &f.asset, &10)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ControlError::AgentPaused);

    f.client.resume(&f.agent);
    f.client
        .authorize_payment(&f.agent, &f.merchant, &f.asset, &10);
    assert_eq!(f.client.spent_in_window(&f.agent), 10);
}

#[test]
fn agent_without_policy_cannot_pay() {
    let f = setup();
    let err = f
        .client
        .try_authorize_payment(&f.agent, &f.merchant, &f.asset, &10)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ControlError::NoPolicy);
}

#[test]
fn non_positive_amounts_are_rejected() {
    let f = setup();
    f.client.set_policy(&f.agent, &open_policy(&f.env));

    for amount in [0i128, -1i128] {
        let err = f
            .client
            .try_authorize_payment(&f.agent, &f.merchant, &f.asset, &amount)
            .unwrap_err()
            .unwrap();
        assert_eq!(err, ControlError::AmountNotPositive);
    }
}

#[test]
fn invalid_policies_are_rejected() {
    let f = setup();
    let mut policy = open_policy(&f.env);
    policy.per_payment_max = policy.window_max + 1;

    let err = f
        .client
        .try_set_policy(&f.agent, &policy)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, ControlError::InvalidPolicy);
}
