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
pub use str_op::StrOp;
pub use trap::TrapOp;

use super::Emulator;

mod add;
mod and;
mod br;
mod jmp;
mod jsr;
mod ld;
mod ldi;
mod ldr;
mod lea;
mod not;
mod rti;
mod st;
mod sti;
mod str_op;
mod trap;

// TODO: redp state system to use each state gone over in class
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CpuState {
    Fetch,
    Decode,
    ReadMemory,
    Execute,
}

#[derive(Debug)]
pub enum OpCode {
    Add(AddOp),
    And(AndOp),
    Br(BrOp),
    Jmp(JmpOp),
    Jsr(JsrOp),
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
    pub fn from_value(value: u16) -> Option<&'static OpCode> {
        match value {
            0x1 => Some(&OpCode::Add(AddOp)),
            0x5 => Some(&OpCode::And(AndOp)),
            0x0 => Some(&OpCode::Br(BrOp)),
            0xC => Some(&OpCode::Jmp(JmpOp)),
            0x4 => Some(&OpCode::Jsr(JsrOp)),
            0x2 => Some(&OpCode::Ld(LdOp)),
            0xA => Some(&OpCode::Ldi(LdiOp)),
            0x6 => Some(&OpCode::Ldr(LdrOp)),
            0xE => Some(&OpCode::Lea(LeaOp)),
            0x9 => Some(&OpCode::Not(NotOp)),
            0x8 => Some(&OpCode::Rti(RtiOp)),
            0x3 => Some(&OpCode::St(StOp)),
            0xB => Some(&OpCode::Sti(StiOp)),
            0x7 => Some(&OpCode::Str(StrOp)),
            0xF => Some(&OpCode::Trap(TrapOp)),
            _ => None,
        }
    }

    pub fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        match self {
            OpCode::Add(op) => op.prepare_memory_access(machine_state),
            OpCode::And(op) => op.prepare_memory_access(machine_state),
            OpCode::Br(op) => op.prepare_memory_access(machine_state),
            OpCode::Jmp(op) => op.prepare_memory_access(machine_state),
            OpCode::Jsr(op) => op.prepare_memory_access(machine_state),
            OpCode::Ld(op) => op.prepare_memory_access(machine_state),
            OpCode::Ldi(op) => op.prepare_memory_access(machine_state),
            OpCode::Ldr(op) => op.prepare_memory_access(machine_state),
            OpCode::Lea(op) => op.prepare_memory_access(machine_state),
            OpCode::Not(op) => op.prepare_memory_access(machine_state),
            OpCode::Rti(op) => op.prepare_memory_access(machine_state),
            OpCode::St(op) => op.prepare_memory_access(machine_state),
            OpCode::Sti(op) => op.prepare_memory_access(machine_state),
            OpCode::Str(op) => op.prepare_memory_access(machine_state),
            OpCode::Trap(op) => op.prepare_memory_access(machine_state),
        }
    }

    pub fn execute(&self, machine_state: &mut Emulator) {
        match self {
            OpCode::Add(op) => op.execute(machine_state),
            OpCode::And(op) => op.execute(machine_state),
            OpCode::Br(op) => op.execute(machine_state),
            OpCode::Jmp(op) => op.execute(machine_state),
            OpCode::Jsr(op) => op.execute(machine_state),
            OpCode::Ld(op) => op.execute(machine_state),
            OpCode::Ldi(op) => op.execute(machine_state),
            OpCode::Ldr(op) => op.execute(machine_state),
            OpCode::Lea(op) => op.execute(machine_state),
            OpCode::Not(op) => op.execute(machine_state),
            OpCode::Rti(op) => op.execute(machine_state),
            OpCode::St(op) => op.execute(machine_state),
            OpCode::Sti(op) => op.execute(machine_state),
            OpCode::Str(op) => op.execute(machine_state),
            OpCode::Trap(op) => op.execute(machine_state),
        }
    }
}

pub trait Op: std::fmt::Debug {
    // Prepare any memory accesses needed for the operation
    fn prepare_memory_access(&self, machine_state: &mut Emulator);

    // Execute the instruction
    fn execute(&self, machine_state: &mut Emulator);
}
