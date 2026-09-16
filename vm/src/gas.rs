//! Universal Provenance & Logic Ledger (UPLL) - Dynamic Resource Metering Engine
//! Orchestrates real-time computational resource tracking via instruction weight schedules,
//! memory expansion multipliers, governance runtime limits, and transaction space penalties.

use crate::VmError;
use std::collections::HashMap;

/// Enumerates the performance complexity tiers for transaction micro-ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasTier {
    /// Negligible computations (e.g., HALT, NOP)
    Base,
    /// Standard vector evaluation (e.g., ADD, SUB, stack adjustments)
    VeryLow,
    /// Intermediary operations (e.g., MUL, shifts)
    Low,
    /// High-overhead operations (e.g., DIV, guarded math)
    Mid,
    /// Linear isolate persistence handling (e.g., LOAD, STORE)
    High,
    /// Node boundary interception bridges (e.g., secure SYSCALL entries)
    Extrinsic,
}

impl GasTier {
    /// Returns the initial hardcoded base cost mapping to the specific execution tier.
    #[inline]
    pub const fn base_cost(&self) -> u64 {
        match self {
            GasTier::Base => 1,
            GasTier::VeryLow => 3,
            GasTier::Low => 5,
            GasTier::Mid => 8,
            GasTier::High => 20,
            GasTier::Extrinsic => 150,
        }
    }
}

/// Dynamic ledger resource accounting manager enforcing transaction execution safety bounds.
#[derive(Debug, Clone)]
pub struct GasMeterEngine {
    /// Maximum gas limits allowed for the active contract isolate execution loop.
    pub max_limit: u64,
    /// Absolute total of gas resources currently expended by active processing loops.
    pub spent: u64,
    /// Dynamically updated modification tables tracking gas parameter governance changes.
    pub opcode_weight_matrix: HashMap<u8, GasTier>,
    /// Accumulated runtime memory allocation spaces measured in execution pages.
    pub active_memory_pages: usize,
}

impl GasMeterEngine {
    /// Instantiates a high-fidelity gas allocation accounting engine structure.
    pub fn new(gas_limit: u64) -> Self {
        let mut meter = Self {
            max_limit: gas_limit,
            spent: 0,
            opcode_weight_matrix: HashMap::with_capacity(256),
            active_memory_pages: 0,
        };
        meter.bootstrap_default_weight_matrix();
        meter
    }

    /// Feeds standard operational weights into the mapping index table dynamically.
    fn bootstrap_default_weight_matrix(&mut self) {
        self.opcode_weight_matrix.insert(0x00, GasTier::Base);      // HALT
        self.opcode_weight_matrix.insert(0x0F, GasTier::Base);      // TRAP
        self.opcode_weight_matrix.insert(0x10, GasTier::VeryLow);  // ADD
        self.opcode_weight_matrix.insert(0x11, GasTier::VeryLow);  // SUB
        self.opcode_weight_matrix.insert(0x12, GasTier::Low);      // MUL
        self.opcode_weight_matrix.insert(0x13, GasTier::Mid);      // DIV
        self.opcode_weight_matrix.insert(0x20, GasTier::VeryLow);  // PUSH
        self.opcode_weight_matrix.insert(0x30, GasTier::High);     // LOAD
        self.opcode_weight_matrix.insert(0x31, GasTier::High);     // STORE
        self.opcode_weight_matrix.insert(0x50, GasTier::Extrinsic); // SYSCALL
    }

    /// Evaluates and deducts gas amounts consumed by an input opcode.
    #[inline]
    pub fn consume_resource(&mut self, opcode: u8) -> Result<(), VmError> {
        let tier = self.opcode_weight_matrix.get(&opcode).cloned().unwrap_or(GasTier::Mid);
        let base_fee = tier.base_cost();
        
        // Calculate math bounds safely preventing variable numeric overflows
        let target_expenditure = self.spent.checked_add(base_fee).ok_or(VmError::OutOfGas {
            budget: self.max_limit.saturating_sub(self.spent),
            required: base_fee,
        })?;

        if target_expenditure > self.max_limit {
            return Err(VmError::OutOfGas {
                budget: self.max_limit.saturating_sub(self.spent),
                required: base_fee,
            });
        }

        self.spent = target_expenditure;
        Ok(())
    }

    /// Tracks quadratic memory growth penalties to prevent Out-Of-Memory (OOM) exploits.
    pub fn penalize_memory_expansion(&mut self, target_pages: usize) -> Result<(), VmError> {
        if target_pages <= self.active_memory_pages {
            return Ok();
        }

        let added_pages = target_pages - self.active_memory_pages;
        // Quadratic weight formula modeling state inflation costs: cost = pages^2 * multiplier
        let structural_cost = (added_pages as u64)
            .checked_mul(added_pages as u64)
            .and_then(|square| square.checked_mul(4))
            .ok_or(VmError::OutOfGas {
                budget: self.max_limit.saturating_sub(self.spent),
                required: u64::MAX,
            })?;

        let updated_spend = self.spent.checked_add(structural_cost).ok_or(VmError::OutOfGas {
            budget: self.max_limit.saturating_sub(self.spent),
            required: structural_cost,
        })?;

        if updated_spend > self.max_limit {
            return Err(VmError::OutOfGas {
                budget: self.max_limit.saturating_sub(self.spent),
                required: structural_cost,
            });
        }

        self.spent = updated_spend;
        self.active_memory_pages = target_pages;
        Ok(())
    }

    /// Provides structural parameters tracking total budget limits remaining.
    #[inline]
    pub fn get_remaining_gas(&self) -> u64 {
        self.max_limit.saturating_sub(self.spent)
    }

    /// Updates weight configurations across individual opcodes via self-amending network shifts.
    pub fn mutate_opcode_tier(&mut self, opcode: u8, target_tier: GasTier) {
        self.opcode_weight_matrix.insert(opcode, target_tier);
    }
}
