#![no_std]
//! Settlement registry for Neutral Agent Pay.
//!
//! Records settled x402 payments on-chain and emits the structured events consumed by the
//! indexer. The registry gives the reconciliation layer a stable event contract instead of
//! requiring it to parse transaction bodies, and gives merchants an auditable on-chain
//! record keyed by the payment identifier their middleware issued.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementRecord {
    pub agent: Address,
    pub merchant: Address,
    pub asset: Address,
    pub amount: i128,
    pub settled_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Recorder,
    Payment(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    DuplicatePayment = 3,
    PaymentNotFound = 4,
    AmountNotPositive = 5,
}

#[contract]
pub struct SettlementRegistry;

#[contractimpl]
impl SettlementRegistry {
    /// `recorder` is the platform account permitted to write settlement records.
    pub fn initialize(env: Env, admin: Address, recorder: Address) -> Result<(), RegistryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(RegistryError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Recorder, &recorder);
        Ok(())
    }

    /// Rotate the recorder account. Admin only.
    pub fn set_recorder(env: Env, recorder: Address) -> Result<(), RegistryError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RegistryError::NotInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Recorder, &recorder);
        Ok(())
    }

    /// Record a settled payment. Idempotent by rejection: re-recording the same
    /// `payment_id` is an error rather than a silent overwrite, so a retry storm cannot
    /// corrupt the reconciliation record.
    pub fn record(
        env: Env,
        payment_id: BytesN<32>,
        agent: Address,
        merchant: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), RegistryError> {
        let recorder: Address = env
            .storage()
            .instance()
            .get(&DataKey::Recorder)
            .ok_or(RegistryError::NotInitialized)?;
        recorder.require_auth();

        if amount <= 0 {
            return Err(RegistryError::AmountNotPositive);
        }
        let key = DataKey::Payment(payment_id.clone());
        if env.storage().persistent().has(&key) {
            return Err(RegistryError::DuplicatePayment);
        }

        let record = SettlementRecord {
            agent: agent.clone(),
            merchant: merchant.clone(),
            asset,
            amount,
            settled_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&key, &record);

        env.events()
            .publish((symbol_short!("settled"), merchant, agent), amount);
        Ok(())
    }

    pub fn get(env: Env, payment_id: BytesN<32>) -> Result<SettlementRecord, RegistryError> {
        env.storage()
            .persistent()
            .get(&DataKey::Payment(payment_id))
            .ok_or(RegistryError::PaymentNotFound)
    }

    pub fn exists(env: Env, payment_id: BytesN<32>) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Payment(payment_id))
    }
}

#[cfg(test)]
mod test;
