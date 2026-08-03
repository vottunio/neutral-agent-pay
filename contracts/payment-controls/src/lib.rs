#![no_std]
//! Payment controls for agentic payments on Stellar.
//!
//! Enforces, per agent: a maximum amount per single payment, a maximum amount and a
//! maximum number of payments within a rolling time window, an optional destination
//! allowlist, an optional asset allowlist, and an emergency pause.
//!
//! Authorization is evaluated during transaction simulation, which the x402 facilitator
//! performs before submitting anything on-chain. A violation therefore surfaces as a
//! deterministic pre-settlement rejection that costs the payer nothing.
//!
//! See `docs/architecture.md` section 7 for the design rationale and section 7.4 for the
//! dual-path payer model this contract is designed to support.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    /// Maximum amount for a single payment. Must be > 0.
    pub per_payment_max: i128,
    /// Maximum cumulative amount within one window. Must be > 0.
    pub window_max: i128,
    /// Length of the rolling window, in seconds. Must be > 0.
    pub window_seconds: u64,
    /// Maximum number of payments within one window. Must be > 0.
    pub max_payments_per_window: u32,
    /// Allowed destinations. An empty vector means "no destination restriction".
    pub allowed_destinations: Vec<Address>,
    /// Allowed asset contracts. An empty vector means "no asset restriction".
    pub allowed_assets: Vec<Address>,
    /// When true, all payments from this agent are rejected.
    pub paused: bool,
}

/// Rolling-window accounting for one agent.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowState {
    pub window_start: u64,
    pub spent: i128,
    pub count: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Policy(Address),
    Window(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ControlError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NoPolicy = 3,
    AgentPaused = 4,
    AmountNotPositive = 5,
    PerPaymentMaxExceeded = 6,
    WindowAmountExceeded = 7,
    WindowCountExceeded = 8,
    DestinationNotAllowed = 9,
    AssetNotAllowed = 10,
    InvalidPolicy = 11,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct PaymentControls;

#[contractimpl]
impl PaymentControls {
    /// One-time initialization. `admin` is expected to be a multisig account.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ControlError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ControlError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Replace the policy for `agent`. Admin only.
    pub fn set_policy(env: Env, agent: Address, policy: Policy) -> Result<(), ControlError> {
        Self::require_admin(&env)?;
        Self::validate_policy(&policy)?;

        env.storage()
            .persistent()
            .set(&DataKey::Policy(agent.clone()), &policy);

        env.events()
            .publish((symbol_short!("policy"), agent), policy.window_max);
        Ok(())
    }

    pub fn get_policy(env: Env, agent: Address) -> Result<Policy, ControlError> {
        env.storage()
            .persistent()
            .get(&DataKey::Policy(agent))
            .ok_or(ControlError::NoPolicy)
    }

    pub fn pause(env: Env, agent: Address) -> Result<(), ControlError> {
        Self::set_paused(env, agent, true)
    }

    pub fn resume(env: Env, agent: Address) -> Result<(), ControlError> {
        Self::set_paused(env, agent, false)
    }

    /// Authorize a payment and record it against the agent's rolling window.
    ///
    /// Requires the agent's authorization, so this cannot be invoked on an agent's behalf
    /// by an unrelated party. Returns a typed error on any policy violation.
    pub fn authorize_payment(
        env: Env,
        agent: Address,
        destination: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), ControlError> {
        agent.require_auth();

        if amount <= 0 {
            return Err(ControlError::AmountNotPositive);
        }

        let policy: Policy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(agent.clone()))
            .ok_or(ControlError::NoPolicy)?;

        if policy.paused {
            return Err(ControlError::AgentPaused);
        }
        if amount > policy.per_payment_max {
            return Err(ControlError::PerPaymentMaxExceeded);
        }
        if !Self::allowed(&policy.allowed_destinations, &destination) {
            return Err(ControlError::DestinationNotAllowed);
        }
        if !Self::allowed(&policy.allowed_assets, &asset) {
            return Err(ControlError::AssetNotAllowed);
        }

        let now = env.ledger().timestamp();
        let mut window = Self::current_window(&env, &agent, &policy, now);

        if window.spent + amount > policy.window_max {
            return Err(ControlError::WindowAmountExceeded);
        }
        if window.count + 1 > policy.max_payments_per_window {
            return Err(ControlError::WindowCountExceeded);
        }

        window.spent += amount;
        window.count += 1;
        Self::store_window(&env, &agent, &policy, &window);

        env.events()
            .publish((symbol_short!("authzd"), agent, destination), amount);
        Ok(())
    }

    /// Amount already spent by `agent` in the current window.
    pub fn spent_in_window(env: Env, agent: Address) -> Result<i128, ControlError> {
        let policy: Policy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(agent.clone()))
            .ok_or(ControlError::NoPolicy)?;
        let now = env.ledger().timestamp();
        Ok(Self::current_window(&env, &agent, &policy, now).spent)
    }

    /// Amount `agent` may still spend in the current window.
    pub fn remaining_allowance(env: Env, agent: Address) -> Result<i128, ControlError> {
        let policy: Policy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(agent.clone()))
            .ok_or(ControlError::NoPolicy)?;
        let now = env.ledger().timestamp();
        let window = Self::current_window(&env, &agent, &policy, now);
        Ok(policy.window_max - window.spent)
    }

    // -----------------------------------------------------------------------
    // Internals
    // -----------------------------------------------------------------------

    fn require_admin(env: &Env) -> Result<(), ControlError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ControlError::NotInitialized)?;
        admin.require_auth();
        Ok(())
    }

    fn validate_policy(policy: &Policy) -> Result<(), ControlError> {
        if policy.per_payment_max <= 0
            || policy.window_max <= 0
            || policy.window_seconds == 0
            || policy.max_payments_per_window == 0
            || policy.per_payment_max > policy.window_max
        {
            return Err(ControlError::InvalidPolicy);
        }
        Ok(())
    }

    /// An empty allowlist means unrestricted.
    fn allowed(list: &Vec<Address>, candidate: &Address) -> bool {
        if list.is_empty() {
            return true;
        }
        for entry in list.iter() {
            if entry == *candidate {
                return true;
            }
        }
        false
    }

    /// Return the agent's window state, resetting it if the previous window has elapsed.
    fn current_window(env: &Env, agent: &Address, policy: &Policy, now: u64) -> WindowState {
        let stored: Option<WindowState> = env
            .storage()
            .temporary()
            .get(&DataKey::Window(agent.clone()));

        match stored {
            Some(w) if now < w.window_start + policy.window_seconds => w,
            _ => WindowState {
                window_start: now,
                spent: 0,
                count: 0,
            },
        }
    }

    /// Window counters live in temporary storage with a TTL matched to the policy window,
    /// so expiry is self-cleaning and rent stays bounded.
    fn store_window(env: &Env, agent: &Address, policy: &Policy, window: &WindowState) {
        let key = DataKey::Window(agent.clone());
        env.storage().temporary().set(&key, window);

        // Ledgers close roughly every 5 seconds; keep the entry alive for the remainder
        // of the window plus a safety margin.
        let ttl_ledgers = (policy.window_seconds / 5) as u32 + 120;
        env.storage()
            .temporary()
            .extend_ttl(&key, ttl_ledgers, ttl_ledgers);
    }

    fn set_paused(env: Env, agent: Address, paused: bool) -> Result<(), ControlError> {
        Self::require_admin(&env)?;
        let mut policy: Policy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(agent.clone()))
            .ok_or(ControlError::NoPolicy)?;
        policy.paused = paused;
        env.storage()
            .persistent()
            .set(&DataKey::Policy(agent.clone()), &policy);
        env.events().publish((symbol_short!("paused"), agent), paused);
        Ok(())
    }
}

#[cfg(test)]
mod test;
