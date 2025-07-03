use std::fmt::Display;

pub use add::AddOp;
pub use and::AndOp;
pub use br::BrOp;
pub use jmp::JmpOp;
pub use jsr::JsrOp;
pub use ld::LdOp;
pub use ldi::LdiOp;
pub use ldr::LdrOp;
pub use lea::LeaOp;
pub use not::NotOp;
pub use rti::RtiOp;
pub use st::StOp;
pub use sti::StiOp;
pub use str::StrOp;
pub use trap::TrapOp;

use super::{BitAddressable, Emulator, EmulatorCell};

mod add;
mod and;
mod br;
mod jmp;
pub mod jsr;
mod ld;
mod ldi;
mod ldr;
mod lea;
mod not;
mod rti;
mod st;
mod sti;
mod str;
mod trap;

#[derive(Debug, Clone)]
/// This encodes the key data used in the state machine to decide the next action to take
pub enum CpuState {
    Fetch,                    // Fetch instruction from memory location pointed by PC into IR
    Decode,                   // Decode instruction in IR, identify opcode and operands
    EvaluateAddress(OpCode),  // Calculate memory address for operands or target (if needed)
    FetchOperands(OpCode),    // Fetch operands from registers or memory
    ExecuteOperation(OpCode), // Perform the operation (ALU, branch check, PC update, etc.)
    StoreResult(OpCode),      // Write the result back to register or memory (if needed)
}

/// Represents the decoded operation type.
#[derive(Debug, Clone)]
pub enum OpCode {
    Add(AddOp),
    And(AndOp),
    Br(BrOp),
    Jmp(JmpOp), // Includes RET
    Jsr(JsrOp), // Includes JSRR
    Ld(LdOp),
    Ldi(LdiOp),
    Ldr(LdrOp),
    Lea(LeaOp),
    Not(NotOp),
    Rti(RtiOp),
    St(StOp),
    Sti(StiOp),
    Str(StrOp),
    Trap(TrapOp),
}

impl OpCode {
    /// Decodes an instruction from the machine state (usually from IR)
    /// and returns the corresponding OpCode variant containing the decoded operation details.
    pub fn from_instruction(instruction: EmulatorCell) -> Option<OpCode> {
        let opcode_val = instruction.range(15..12).get();

        debug_assert!(opcode_val != 0xd, "Tried to decode non-existant op");
        match opcode_val {
            // Call the specific decode method for each opcode
            0x1 => Some(OpCode::Add(AddOp::decode(instruction))),
            0x5 => Some(OpCode::And(AndOp::decode(instruction))),
            0x0 => Some(OpCode::Br(BrOp::decode(instruction))),
            0xC => Some(OpCode::Jmp(JmpOp::decode(instruction))),
            0x4 => Some(OpCode::Jsr(JsrOp::decode(instruction))),
            0x2 => Some(OpCode::Ld(LdOp::decode(instruction))),
            0xA => Some(OpCode::Ldi(LdiOp::decode(instruction))),
            0x6 => Some(OpCode::Ldr(LdrOp::decode(instruction))),
            0xE => Some(OpCode::Lea(LeaOp::decode(instruction))),
            0x9 => Some(OpCode::Not(NotOp::decode(instruction))),
            0x8 => Some(OpCode::Rti(RtiOp::decode(instruction))),
            0x3 => Some(OpCode::St(StOp::decode(instruction))),
            0xB => Some(OpCode::Sti(StiOp::decode(instruction))),
            0x7 => Some(OpCode::Str(StrOp::decode(instruction))),
            0xF => Some(OpCode::Trap(TrapOp::decode(instruction))),
            // Opcode 13 (0xD) is unused/reserved in standard LC-3.
            _ => None, // Return None for invalid/unused opcode
        }
    }

    /// Dispatch methods for each phase, calling the appropriate method on the specific Op struct.
    /// The main emulator loop will call these based on the current CpuState.
    pub fn evaluate_address(&mut self, machine_state: &mut Emulator) {
        match self {
            OpCode::Add(op) => op.evaluate_address(machine_state),
            OpCode::And(op) => op.evaluate_address(machine_state),
            OpCode::Br(op) => op.evaluate_address(machine_state),
            OpCode::Jmp(op) => op.evaluate_address(machine_state),
            OpCode::Jsr(op) => op.evaluate_address(machine_state),
            OpCode::Ld(op) => op.evaluate_address(machine_state),
            OpCode::Ldi(op) => op.evaluate_address(machine_state),
            OpCode::Ldr(op) => op.evaluate_address(machine_state),
            OpCode::Lea(op) => op.evaluate_address(machine_state),
            OpCode::Not(op) => op.evaluate_address(machine_state),
            OpCode::Rti(op) => op.evaluate_address(machine_state),
            OpCode::St(op) => op.evaluate_address(machine_state),
            OpCode::Sti(op) => op.evaluate_address(machine_state),
            OpCode::Str(op) => op.evaluate_address(machine_state),
            OpCode::Trap(op) => op.evaluate_address(machine_state),
        }
    }

    /// On this cyle instructions whos operands are at a given adress will fetch them.
    pub fn fetch_operands(&mut self, machine_state: &mut Emulator) {
        if match self {
            OpCode::Add(op) => op.fetch_operands(machine_state),
            OpCode::And(op) => op.fetch_operands(machine_state),
            OpCode::Br(op) => op.fetch_operands(machine_state),
            OpCode::Jmp(op) => op.fetch_operands(machine_state),
            OpCode::Jsr(op) => op.fetch_operands(machine_state),
            OpCode::Ld(op) => op.fetch_operands(machine_state),
            OpCode::Ldi(op) => op.fetch_operands(machine_state),
            OpCode::Ldr(op) => op.fetch_operands(machine_state),
            OpCode::Lea(op) => op.fetch_operands(machine_state),
            OpCode::Not(op) => op.fetch_operands(machine_state),
            OpCode::Rti(op) => op.fetch_operands(machine_state),
            OpCode::St(op) => op.fetch_operands(machine_state),
            OpCode::Sti(op) => op.fetch_operands(machine_state),
            OpCode::Str(op) => op.fetch_operands(machine_state),
            OpCode::Trap(op) => op.fetch_operands(machine_state),
        } {
            machine_state.step_read_memory();

            self.fetch_operands(machine_state);
        }
        machine_state.step_read_memory();
    }

    /// This is the meat of most instruction. This is when SHIT GOES DOWN
    pub fn execute_operation(&mut self, machine_state: &mut Emulator) {
        match self {
            OpCode::Add(op) => op.execute_operation(machine_state),
            OpCode::And(op) => op.execute_operation(machine_state),
            OpCode::Br(op) => op.execute_operation(machine_state),
            OpCode::Jmp(op) => op.execute_operation(machine_state),
            OpCode::Jsr(op) => op.execute_operation(machine_state),
            OpCode::Ld(op) => op.execute_operation(machine_state),
            OpCode::Ldi(op) => op.execute_operation(machine_state),
            OpCode::Ldr(op) => op.execute_operation(machine_state),
            OpCode::Lea(op) => op.execute_operation(machine_state),
            OpCode::Not(op) => op.execute_operation(machine_state),
            OpCode::Rti(op) => op.execute_operation(machine_state),
            OpCode::St(op) => op.execute_operation(machine_state),
            OpCode::Sti(op) => op.execute_operation(machine_state),
            OpCode::Str(op) => op.execute_operation(machine_state),
            OpCode::Trap(op) => op.execute_operation(machine_state),
        }
    }

    /// Store the result of the execute cyvle wherever it needs to be stored
    pub fn store_result(&mut self, machine_state: &mut Emulator) {
        match self {
            OpCode::Add(op) => op.store_result(machine_state),
            OpCode::And(op) => op.store_result(machine_state),
            OpCode::Br(op) => op.store_result(machine_state),
            OpCode::Jmp(op) => op.store_result(machine_state),
            OpCode::Jsr(op) => op.store_result(machine_state),
            OpCode::Ld(op) => op.store_result(machine_state),
            OpCode::Ldi(op) => op.store_result(machine_state),
            OpCode::Ldr(op) => op.store_result(machine_state),
            OpCode::Lea(op) => op.store_result(machine_state),
            OpCode::Not(op) => op.store_result(machine_state),
            OpCode::Rti(op) => op.store_result(machine_state),
            OpCode::St(op) => op.store_result(machine_state),
            OpCode::Sti(op) => op.store_result(machine_state),
            OpCode::Str(op) => op.store_result(machine_state),
            OpCode::Trap(op) => op.store_result(machine_state),
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Add(op) => write!(f, "{op}"),
            OpCode::And(op) => write!(f, "{op}"),
            OpCode::Br(op) => write!(f, "{op}"),
            OpCode::Jmp(op) => write!(f, "{op}"),
            OpCode::Jsr(op) => write!(f, "{op}"),
            OpCode::Ld(op) => write!(f, "{op}"),
            OpCode::Ldi(op) => write!(f, "{op}"),
            OpCode::Ldr(op) => write!(f, "{op}"),
            OpCode::Lea(op) => write!(f, "{op}"),
            OpCode::Not(op) => write!(f, "{op}"),
            OpCode::Rti(op) => write!(f, "{op}"),
            OpCode::St(op) => write!(f, "{op}"),
            OpCode::Sti(op) => write!(f, "{op}"),
            OpCode::Str(op) => write!(f, "{op}"),
            OpCode::Trap(op) => write!(f, "{op}"),
        }
    }
}

// Trait defining the interface for LC-3 operations during different phases of the instruction cycle.
// Individual Op structs (AddOp, LdOp, etc.) will implement this trait.
// Methods default to doing nothing, as not all instructions perform actions in every phase.
pub trait Op: std::fmt::Debug + Display {
    /// **Decode Phase:**
    /// Extract operand specifiers (register numbers, immediate values, offsets)
    /// from the instruction stored in `machine_state.ir`.
    fn decode(instruction: EmulatorCell) -> Self;
    /// **Evaluate Address Phase:**
    /// Calculate the effective memory address needed for operand fetch or result store.
    /// For instructions like `LDI` or `STI`, this might involve an initial memory read.
    /// For branch/jump instructions, this might calculate the target address.
    /// The result (e.g., address) should typically be stored within the `Emulator` state
    /// (e.g., in a temporary address register like MAR) for use in later phases.
    fn evaluate_address(&mut self, _machine_state: &mut Emulator) {
        // Default: No address evaluation needed for this instruction phase.
    }

    /// **Fetch Operands Phase:**
    /// Retrieve operands required for the execution phase.
    /// Operands might come from registers or from memory (using an address
    /// potentially calculated in the `evaluate_address` phase).
    /// Fetched values should be stored within the `Emulator` state (e.g., in temporary
    /// data registers or specific fields) for use in the `execute_operation` phase.
    /// For store instructions (`ST`, `STI`, `STR`), this phase fetches the *data* to be stored.
    /// Bool for if we need a second phase (Only ldi and rti need this)
    fn fetch_operands(&mut self, _machine_state: &mut Emulator) -> bool {
        // Default: No operands need fetching for this instruction phase.
        false
    }

    /// **Execute Operation Phase:**
    /// Perform the core computation or action of the instruction using fetched operands.
    /// Examples: ALU operations (ADD, AND, NOT), condition checking (BR),
    /// updating the PC (JMP, JSR, BR taken), handling traps (TRAP).
    /// Side effects like updating condition codes occur here.
    fn execute_operation(&mut self, _machine_state: &mut Emulator) {
        // Default: No execution action needed for this instruction phase.
    }

    /// **Store Result Phase:**
    /// Write the final result of the operation (if any) to its destination.
    /// The destination could be a register or a memory location (using an address
    /// potentially calculated in the `evaluate_address` phase).
    /// Instructions like `BR`, `JMP`, or `ST`/`STR`/`STI` might not have a conventional result
    /// to store in a *destination register*, but this phase handles the memory write for stores.
    fn store_result(&mut self, _machine_state: &mut Emulator) {
        // Default: No result needs storing for this instruction phase.
    }
}
