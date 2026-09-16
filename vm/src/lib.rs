//! Universal Provenance & Logic Ledger (UPLL) - Virtual Machine Isolate Subsystem
//! Fully production-scaled sandboxed container managing register environments,
//! memory boundaries, dynamic gas structures, and telemetry interception contexts.

pub mod interpreter;
pub mod gas;
pub mod memory;
pub mod syscall;

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Systemic execution failures and boundaries within the deterministic runtime environment.
#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmError {
    #[error("Isolate runtime out of gas. Limit: {budget}, Requested: {required}")]
    OutOfGas { budget: u64, required: u64 },
    
    #[error("Linear isolation memory boundary check violation at address: {address:#010x}")]
    MemoryAccessViolation { address: usize },
    
    #[error("Evaluation execution stack underflow condition encountered")]
    StackUnderflow,
    
    #[error("Evaluation execution stack overflow condition encountered. Max Depth: {depth}")]
    StackOverflow { depth: usize },
    
    #[error("Illegal execution instruction opcode encountered: {opcode:#04x}")]
    InvalidOpcode { opcode: u8 },
    
    #[error("Mathematical zero division arithmetic system check failure")]
    DivisionByZero,
    
    #[error("Secure system call execution route failure for code: {call_code:#06x}")]
    SyscallExecutionFailure { call_code: u32 },
    
    #[error("Context frame call stack overflow reached maximum capacity")]
    CallStackOverflow,
    
    #[error("Context frame call stack underflow condition encountered")]
    CallStackUnderflow,
    
    #[error("Execution trap signaled explicitly by code engine instruction")]
    ExecutionTrap,
}

/// An isolated call stack frame capturing local register offsets and safe returns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFrameContext {
    /// The exact byte address where execution returns when this frame pops.
    pub return_address: usize,
    /// Fast-access register slots allocated localized to the active context block.
    pub registers: Vec<u64>,
    /// Base pointer index inside the global linear memory array bounds.
    pub memory_base: usize,
}

/// The top-level core virtual machine coordinator orchestrating execution components.
pub struct VirtualMachineIsolate {
    pub interpreter: interpreter::VectorInterpreter,
    pub memory: memory::LinearMemoryIsolate,
    pub gas_meter: gas::GasMeterEngine,
    pub syscall_router: syscall::SyscallRouterTable,
    pub call_stack: Vec<ExecutionFrameContext>,
    pub max_call_depth: usize,
    pub active: bool,
}

impl VirtualMachineIsolate {
    /// Constructs a brand new, sandboxed runtime container environment.
    pub fn new(gas_limit: u64, memory_pages: usize) -> Self {
        Self {
            interpreter: interpreter::VectorInterpreter::new(),
            memory: memory::LinearMemoryIsolate::new(memory_pages),
            gas_meter: gas_meter::GasMeterEngine::new(gas_limit),
            syscall_router: syscall::SyscallRouterTable::new(),
            call_stack: Vec::with_capacity(512),
            max_call_depth: 512,
            active: true,
        }
    }

    /// Primary execution sandbox engine loop processing zero-copy transaction slices.
    pub fn execute_program(&mut self, bytecode: &[u8]) -> Result<(), VmError> {
        let mut instruction_pointer: usize = 0;
        
        while instruction_pointer < bytecode.len() && self.active {
            let raw_opcode = bytecode[instruction_pointer];
            
            // Meter performance invariants using weight schedules
            self.gas_meter.consume_resource(raw_opcode)?;
            
            match raw_opcode {
                0x00 => { // HALT: Cleanly cease isolate operations
                    self.active = false;
                    instruction_pointer += 1;
                }
                0x0F => { // TRAP: Force abort processing pipeline
                    return Err(VmError::ExecutionTrap);
                }
                0x10 => { // ADD: Wrap arithmetic sum into state
                    let b = self.interpreter.pop_evaluation()?;
                    let a = self.interpreter.pop_evaluation()?;
                    self.interpreter.push_evaluation(a.wrapping_add(b))?;
                    instruction_pointer += 1;
                }
                0x11 => { // SUB: Wrapped subtraction processing
                    let b = self.interpreter.pop_evaluation()?;
                    let a = self.interpreter.pop_evaluation()?;
                    self.interpreter.push_evaluation(a.wrapping_sub(b))?;
                    instruction_pointer += 1;
                }
                0x12 => { // MUL: Wrapped product calculation
                    let b = self.interpreter.pop_evaluation()?;
                    let a = self.interpreter.pop_evaluation()?;
                    self.interpreter.push_evaluation(a.wrapping_mul(b))?;
                    instruction_pointer += 1;
                }
                0x13 => { // DIV: Guarded non-zero division safety path
                    let b = self.interpreter.pop_evaluation()?;
                    if b == 0 {
                        return Err(VmError::DivisionByZero);
                    }
                    let a = self.interpreter.pop_evaluation()?;
                    self.interpreter.push_evaluation(a / b)?;
                    instruction_pointer += 1;
                }
                0x20 => { // PUSH: Read static 8-byte network parameters directly
                    if instruction_pointer + 8 >= bytecode.len() {
                        return Err(VmError::MemoryAccessViolation { address: instruction_pointer });
                    }
                    let mut raw_bytes = [0u8; 8];
                    raw_bytes.copy_from_slice(&bytecode[instruction_pointer + 1..instruction_pointer + 9]);
                    let parsed_val = u64::from_be_bytes(raw_bytes);
                    self.interpreter.push_evaluation(parsed_val)?;
                    instruction_pointer += 9;
                }
                0x30 => { // LOAD: Pull values out of structural linear memory
                    let addr = self.interpreter.pop_evaluation()? as usize;
                    let mut read_buf = [0u8; 8];
                    self.memory.read_bytes(addr, &mut read_buf)?;
                    self.interpreter.push_evaluation(u64::from_be_bytes(read_buf))?;
                    instruction_pointer += 1;
                }
                0x31 => { // STORE: Commit parameters back down to linear persistence
                    let addr = self.interpreter.pop_evaluation()? as usize;
                    let val = self.interpreter.pop_evaluation()?;
                    self.memory.write_bytes(addr, &val.to_be_bytes())?;
                    instruction_pointer += 1;
                }
                0x40 => { // CALL: Pivot frame context pointers to code segments
                    if self.call_stack.len() >= self.max_call_depth {
                        return Err(VmError::CallStackOverflow);
                    }
                    let dest_addr = self.interpreter.pop_evaluation()? as usize;
                    let frame = ExecutionFrameContext {
                        return_address: instruction_pointer + 1,
                        registers: vec![0; 16],
                        memory_base: 0,
                    };
                    self.call_stack.push(frame);
                    instruction_pointer = dest_addr;
                }
                0x41 => { // RET: Pop frame structures cleanly
                    let frame = self.call_stack.pop().ok_or(VmError::CallStackUnderflow)?;
                    instruction_pointer = frame.return_address;
                }
                0x50 => { // SYSCALL: Interface with external node architectures via dispatch
                    let code = self.interpreter.pop_evaluation()? as u32;
                    self.syscall_router.dispatch(code, &mut self.interpreter.evaluation_stack)?;
                    instruction_pointer += 1;
                }
                _ => return Err(VmError::InvalidOpcode { opcode: raw_opcode }),
            }
        }
        Ok(())
    }
}
