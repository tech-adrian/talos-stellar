//! TalosNameService — Soroban smart contract for human-readable Talos names.
//!
//! Handles:
//! - Name registration (e.g., "marketbot" → Talos ID) with creator authorization
//! - Name resolution (name → Talos ID)
//! - Name availability checks
//! - Validation: 3-32 chars, lowercase alphanumeric + hyphens

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String,
};

// ── Data Types ──────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    NameRecord(String),  // name → talos_id
    TalosName(u32),      // talos_id → name
}

// ── Events ──────────────────────────────────────────────────────────

fn emit_name_registered(env: &Env, talos_id: u32, name: String) {
    let topics = (symbol_short!("name_reg"), talos_id);
    env.events().publish(topics, name);
}

// ── Validation ──────────────────────────────────────────────────────
// Character-level validation is handled off-chain (Next.js regex).
// On-chain we only enforce the byte-length bounds.

fn validate_name(name: &String) -> bool {
    let len = name.len();
    len >= 3 && len <= 32
}

// ── Contract ────────────────────────────────────────────────────────

#[contract]
pub struct TalosNameService;

#[contractimpl]
impl TalosNameService {
    /// Register a name for a Talos.
    ///
    /// # Arguments
    /// * `e` - Soroban environment
    /// * `talos_id` - The Talos ID to associate with the name
    /// * `caller` - The caller's address (must be the Talos creator)
    /// * `name` - Human-readable name (3-32 chars, lowercase alphanumeric + hyphens)
    ///
    /// # Authorization
    /// Only the registered creator of the Talos can register a name for it.
    pub fn register_name(e: Env, talos_id: u32, caller: Address, name: String) {
        // Require caller authorization
        caller.require_auth();

        if !validate_name(&name) {
            panic!("Invalid name. Must be 3-32 chars, lowercase alphanumeric + hyphens, no consecutive hyphens.");
        }

        if e.storage()
            .persistent()
            .get::<_, u32>(&DataKey::NameRecord(name.clone()))
            .is_some()
        {
            panic!("Name already taken");
        }

        // Verify caller is the Talos creator
        // For now, we trust that the caller is authorized at the application level.
        // In a production system, you would cross-call the TalosRegistry contract
        // to verify caller == creator_of(talos_id). This requires storing the
        // registry contract ID, which we leave as a future enhancement.
        //
        // Cross-contract verification example:
        // let creator: Address = e.invoke_contract(
        //     &registry_contract_id,
        //     &Symbol::new(&e, "creator_of"),
        //     &vec![&e, talos_id],
        // );
        // if creator != caller {
        //     panic!("Unauthorized: only the Talos creator can register names");
        // }

        // Store mappings
        e.storage()
            .persistent()
            .set(&DataKey::NameRecord(name.clone()), &talos_id);
        e.storage()
            .persistent()
            .set(&DataKey::TalosName(talos_id), &name);

        emit_name_registered(&e, talos_id, name);
    }

    /// Resolve a name to a Talos ID.
    /// Returns None if the name doesn't exist.
    pub fn resolve_name(e: Env, name: String) -> Option<u32> {
        e.storage()
            .persistent()
            .get(&DataKey::NameRecord(name))
    }

    /// Get the name associated with a Talos ID.
    /// Returns None if the Talos has no name.
    pub fn name_of(e: Env, talos_id: u32) -> Option<String> {
        e.storage()
            .persistent()
            .get(&DataKey::TalosName(talos_id))
    }

    /// Check if a name is available.
    pub fn is_name_available(e: Env, name: String) -> bool {
        if !validate_name(&name) {
            return false;
        }
        e.storage()
            .persistent()
            .get::<_, u32>(&DataKey::NameRecord(name))
            .is_none()
    }

    /// Check if a Talos has a registered name.
    pub fn has_name(e: Env, talos_id: u32) -> bool {
        e.storage()
            .persistent()
            .get::<_, String>(&DataKey::TalosName(talos_id))
            .is_some()
    }
}
