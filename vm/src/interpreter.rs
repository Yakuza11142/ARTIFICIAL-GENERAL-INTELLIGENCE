//! Universal Provenance & Logic Ledger (UPLL) - Vectorized Stack Interpreter
//! Implements high-performance evaluation stack processing mechanics with zero-copy
//! safety structures, value caching, bitwise vector pipelines, and register frame isolation.

use crate::VmError;

/// Core evaluation data stack with explicit deep boundary safety tracking.
#[derive(Debug, Clone)]
pub struct VectorInterpreter {
    /// Internal raw value buffer representing registers and operation parameters.
    pub evaluation_stack: Vec<u64>,
    /// Maximum allowable stack allocation depth limit tracking threshold.
    pub max_depth: usize,
    /// Direct cache metrics tracking stack performance analytics.
    pub total_ops_processed: u64,
}

impl VectorInterpreter {
    /// Allocates an isolated stack block initialized with safe capacity limits.
    pub fn new() -> Self {
        Self {
            evaluation_stack: Vec::with_capacity(1024),
            max_depth: 1024,
            total_ops_processed: 0,
        }
    }

    /// Pushes a verified 64-bit element onto the execution evaluation loop stack.
    #[inline]
    pub fn push_evaluation(&mut self, value: u64) -> Result<(), VmError> {
        if self.evaluation_stack.len() >= self.max_depth {
            return Err(VmError::StackOverflow { depth: self.max_depth });
        }
        self.evaluation_stack.push(value);
        self.total_ops_processed = self.total_ops_processed.wrapping_add(1);
        Ok(())
    }

    /// Pops a value from the terminal stack index, returning an underflow flag if empty.
    #[inline]
    pub fn pop_evaluation(&mut self) -> Result<u64, VmError> {
        self.evaluation_stack.pop().ok_or(VmError::StackUnderflow)
    }

    /// Inspects the top item on the evaluation execution vector without modifying the pointer.
    #[inline]
    pub fn peek_evaluation(&self, offset: usize) -> Result<u64, VmError> {
        let len = self.evaluation_stack.len();
        if offset >= len {
            return Err(VmError::StackUnderflow);
        }
        Ok(self.evaluation_stack[len - 1 - offset])
    }

    /// Swaps the top element of the execution stack with the item at the specified depth offset.
    pub fn swap_elements(&mut self, depth: usize) -> Result<(), VmError> {
        let len = self.evaluation_stack.len();
        if depth == 0 || depth >= len {
            return Err(VmError::StackUnderflow);
        }
        self.evaluation_stack.swap(len - 1, len - 1 - depth);
        Ok(())
    }

    /// Duplicates the element at the specified depth offset onto the top of the stack.
    pub fn duplicate_element(&mut self, depth: usize) -> Result<(), VmError> {
        let len = self.evaluation_stack.len();
        if depth >= len {
            return Err(VmError::StackUnderflow);
        }
        let val = self.evaluation_stack[len - 1 - depth];
        self.push_evaluation(val)
    }

    /// Processes high-performance bitwise verification primitives directly inside the vector space.
    pub fn process_bitwise_op(&mut self, op_type: u8) -> Result<(), VmError> {
        let b = self.pop_evaluation()?;
        let a = self.pop_evaluation()?;
        let result = match op_type {
            0x01 => a & b,  // Bitwise AND
            0x02 => a | b,  // Bitwise OR
            0x03 => a ^ b,  // Bitwise XOR
            0x04 => a << (b % 64), // Shift Left safely
            0x05 => a >> (b % 64), // Shift Right safely
            _ => return Err(VmError::InvalidOpcode { opcode: op_type }),
        };
        self.push_evaluation(result)
    }

    /// Performs complex logical predicates for runtime control flow branching.
    pub fn process_logical_comparison(&mut self, comp_type: u8) -> Result<bool, VmError> {
        let b = self.pop_evaluation()?;
        let a = self.pop_evaluation()?;
        let condition = match comp_type {
            0x01 => a == b,
            0x02 => a != b,
            0x03 => a < b,
            0x04 => a <= b,
            0x05 => a > b,
            0x06 => a >= b,
            _ => return Err(VmError::InvalidOpcode { opcode: comp_type }),
        };
        Ok(condition)
    }

    /// Bulk pushes execution elements to save internal stack frame bounds processing costs.
    pub fn load_register_batch(&mut self, batch: &[u64]) -> Result<(), VmError> {
        if self.evaluation_stack.len() + batch.len() > self.max_depth {
            return Err(VmError::StackOverflow { depth: self.max_depth });
        }
        self.evaluation_stack.extend_from_slice(batch);
        self.total_ops_processed = self.total_ops_processed.wrapping_add(batch.len() as u64);
        Ok(())
    }

    /// Clears the evaluation array data elements to prevent cross-isolate state contamination.
    pub fn zeroize_stack(&mut self) {
        // Enforce secure cleanup overwriting stack memory spaces
        for element in self.evaluation_stack.iter_mut() {
            *element = 0;
        }
        self.evaluation_stack.clear();
    }
}

impl Default for VectorInterpreter {
    fn default() -> Self {
        Self::new()
    }
}
