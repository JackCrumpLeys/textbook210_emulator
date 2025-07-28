use std::fmt;
use std::{collections::HashMap, sync::Arc};

use crate::emulator::{Emulator, Exception};

// Re-export EmulatorCell for convenience
pub use super::EmulatorCell;

/// The 6 phases of the instruction cycle, for display purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CycleState {
    Fetch,
    Decode,
    EvaluateAddress,
    FetchOperands,
    Execute,
    StoreResult,
}

impl fmt::Display for CycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CycleState::Fetch => write!(f, "Fetch"),
            CycleState::Decode => write!(f, "Decode"),
            CycleState::EvaluateAddress => write!(f, "Evaluate Address"),
            CycleState::FetchOperands => write!(f, "Fetch Operands"),
            CycleState::Execute => write!(f, "Execute"),
            CycleState::StoreResult => write!(f, "Store Result"),
        }
    }
}

/// The operations the ALU can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MAluOp {
    Add,
    And,
    Not,
}

impl fmt::Display for MAluOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MAluOp::Add => write!(f, "+"),
            MAluOp::And => write!(f, "&"),
            MAluOp::Not => write!(f, "NOT"),
        }
    }
}

/// Represents a source of data for an operation.
#[derive(Debug, Clone)]
pub enum DataSource {
    Register(u16),
    PC,
    IR,
    MAR,
    MDR,
    PSR,
    AluOut,
    Temp,

    Immediate(i16),
    PCOffset(i16),
    TrapVector(u8),
    Constant(u16),
}

impl fmt::Display for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataSource::Register(n) => write!(f, "R{n}"),
            DataSource::PC => write!(f, "PC"),
            DataSource::IR => write!(f, "IR"),
            DataSource::MAR => write!(f, "MAR"),
            DataSource::MDR => write!(f, "MDR"),
            DataSource::PSR => write!(f, "PSR"),
            DataSource::AluOut => write!(f, "ALU_OUT"),
            DataSource::Temp => write!(f, "TEMP"),

            DataSource::Immediate(val) => write!(f, "#{val}"),
            DataSource::PCOffset(val) => write!(f, "#{val}"),
            DataSource::TrapVector(val) => write!(f, "x{val:02X}"),
            DataSource::Constant(val) => write!(f, "x{val:04X}"),
        }
    }
}

/// Represents a destination for data.
#[derive(Debug, Clone)]
pub enum DataDestination {
    Register(u16),
    PC,
    IR,
    MAR,
    MDR,
    PSR,
    AluOut,
    Temp,
}

impl fmt::Display for DataDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataDestination::Register(n) => write!(f, "R{n}"),
            DataDestination::PC => write!(f, "PC"),
            DataDestination::IR => write!(f, "IR"),
            DataDestination::MAR => write!(f, "MAR"),
            DataDestination::MDR => write!(f, "MDR"),
            DataDestination::PSR => write!(f, "PSR"),
            DataDestination::AluOut => write!(f, "ALU_OUT"),
            DataDestination::Temp => write!(f, "TEMP"),
        }
    }
}

/// Flags that can be set by a micro-op.
#[derive(Debug, Clone, Copy)]
pub enum MachineFlag {
    UpdateCondCodes(u16),
    WriteMemory,
}

impl fmt::Display for MachineFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MachineFlag::UpdateCondCodes(reg) => write!(f, "SET_CC(R{reg})"),
            MachineFlag::WriteMemory => write!(f, "WRITE_MEM"),
        }
    }
}

type CustomMicroOpFunction = Box<dyn Fn(&mut Emulator) -> Result<(), Exception> + Send + Sync>;

/// A single, atomic CPU operation.
pub enum MicroOp {
    /// Transfer data from source to destination
    Transfer {
        source: DataSource,
        destination: DataDestination,
    },
    /// transfer op1 and op2 to the alu populating the alu out with the result of the operation
    Alu {
        operation: MAluOp,
        operand1: DataSource,
        operand2: DataSource,
    },
    /// Run memory writes and reads for the given phase
    PhaseTransition(CycleState),
    /// Set flags that can be read by the machine (Write flag for memory bus etc)
    SetFlag(MachineFlag),
    /// special op to provide metadata on ops.
    Message(String),
    /// Do somthing not covered by other ops (if statement or messing with the psr)
    Custom(Arc<CustomMicroOpFunction>, String),
}

impl MicroOp {
    pub fn new_custom<F>(f: F, display_code: String) -> Self
    where
        F: Fn(&mut Emulator) -> Result<(), Exception> + Send + Sync + 'static,
    {
        MicroOp::Custom(Arc::new(Box::new(f)), display_code)
    }
}

impl fmt::Display for MicroOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MicroOp::Transfer {
                source,
                destination,
            } => {
                write!(f, "{destination} <- {source}")
            }
            MicroOp::Alu {
                operation,
                operand1,
                operand2,
            } => match operation {
                MAluOp::Not => write!(f, "ALU_OUT <- NOT {operand1}"),
                _ => write!(f, "ALU_OUT <- {operand1} {operation} {operand2}"),
            },
            MicroOp::PhaseTransition(phase) => write!(f, "-> {phase}"),
            MicroOp::SetFlag(flag) => write!(f, "{flag}"),
            MicroOp::Message(msg) => write!(f, "[{msg}]"),
            MicroOp::Custom(_, s) => write!(f, "{s}"),
        }
    }
}

/// Trait for operations that can generate micro-op execution plans
pub trait MicroOpGenerator {
    /// Generate the complete execution plan for this operation
    /// Returns a HashMap mapping each phase to its micro-ops
    fn generate_plan(&self) -> HashMap<CycleState, Vec<MicroOp>>;
}

/// Macro for creating micro-ops with a readable syntax
#[macro_export]
macro_rules! micro_op {
    // Phase transition
    (-> $phase:ident) => {
        $crate::emulator::micro_op::MicroOp::PhaseTransition(
            $crate::emulator::micro_op::CycleState::$phase,
        )
    };

    // Register transfers
    (R($dst:expr) <- R($src:expr)) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::Register($src),
            destination: $crate::emulator::micro_op::DataDestination::Register($dst),
        }
    };

    // Register to component
    ($dst:ident <- R($src:expr)) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::Register($src),
            destination: $crate::emulator::micro_op::DataDestination::$dst,
        }
    };

    // Component to register
    (R($dst:expr) <- $src:ident) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::$src,
            destination: $crate::emulator::micro_op::DataDestination::Register($dst),
        }
    };

    // Component to component
    ($dst:ident <- $src:ident) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::$src,
            destination: $crate::emulator::micro_op::DataDestination::$dst,
        }
    };

    // TEMP register patterns
    (TEMP <- R($src:expr)) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::Register($src),
            destination: $crate::emulator::micro_op::DataDestination::Temp,
        }
    };

    (R($dst:expr) <- TEMP) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::Temp,
            destination: $crate::emulator::micro_op::DataDestination::Register($dst),
        }
    };

    (TEMP <- $src:ident) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::$src,
            destination: $crate::emulator::micro_op::DataDestination::Temp,
        }
    };

    ($dst:ident <- TEMP) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::Temp,
            destination: $crate::emulator::micro_op::DataDestination::$dst,
        }
    };

    // Immediate values
    ($dst:ident <- IMM($val:expr)) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::Immediate($val),
            destination: $crate::emulator::micro_op::DataDestination::$dst,
        }
    };

    // PC Offset
    ($dst:ident <- PCOFFSET($val:expr)) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::PCOffset($val),
            destination: $crate::emulator::micro_op::DataDestination::$dst,
        }
    };

    // Constants
    ($dst:ident <- C($val:expr)) => {
        $crate::emulator::micro_op::MicroOp::Transfer {
            source: $crate::emulator::micro_op::DataSource::Constant($val),
            destination: $crate::emulator::micro_op::DataDestination::$dst,
        }
    };

    // ALU operations - ADD with registers
    (ALU_OUT <- R($src1:expr) + R($src2:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::Add,
            operand1: $crate::emulator::micro_op::DataSource::Register($src1),
            operand2: $crate::emulator::micro_op::DataSource::Register($src2),
        }
    };

    // ALU ADD with immediate
    (ALU_OUT <- R($src:expr) + IMM($val:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::Add,
            operand1: $crate::emulator::micro_op::DataSource::Register($src),
            operand2: $crate::emulator::micro_op::DataSource::Immediate($val),
        }
    };

    // ALU ADD with PC offset
    (ALU_OUT <- PC + PCOFFSET($val:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::Add,
            operand1: $crate::emulator::micro_op::DataSource::PC,
            operand2: $crate::emulator::micro_op::DataSource::PCOffset($val),
        }
    };

    // ALU ADD with constants
    (ALU_OUT <- $src:ident + C($val:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::Add,
            operand1: $crate::emulator::micro_op::DataSource::$src,
            operand2: $crate::emulator::micro_op::DataSource::Constant($val),
        }
    };

    // ALU ADD constant to constant
    (ALU_OUT <- C($val1:expr) + C($val2:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::Add,
            operand1: $crate::emulator::micro_op::DataSource::Constant($val1),
            operand2: $crate::emulator::micro_op::DataSource::Constant($val2),
        }
    };

    // ALU ADD register to constant
    (ALU_OUT <- R($src:expr) + C($val:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::Add,
            operand1: $crate::emulator::micro_op::DataSource::Register($src),
            operand2: $crate::emulator::micro_op::DataSource::Constant($val),
        }
    };

    // ALU AND operations
    (ALU_OUT <- R($src1:expr) & R($src2:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::And,
            operand1: $crate::emulator::micro_op::DataSource::Register($src1),
            operand2: $crate::emulator::micro_op::DataSource::Register($src2),
        }
    };

    // ALU AND with immediate
    (ALU_OUT <- R($src:expr) & IMM($val:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::And,
            operand1: $crate::emulator::micro_op::DataSource::Register($src),
            operand2: $crate::emulator::micro_op::DataSource::Immediate($val),
        }
    };

    // ALU NOT
    (ALU_OUT <- NOT R($src:expr)) => {
        $crate::emulator::micro_op::MicroOp::Alu {
            operation: $crate::emulator::micro_op::MAluOp::Not,
            operand1: $crate::emulator::micro_op::DataSource::Register($src),
            operand2: $crate::emulator::micro_op::DataSource::Constant(0),
        }
    };

    // Flags
    (SET_CC($reg:expr)) => {
        $crate::emulator::micro_op::MicroOp::SetFlag(
            $crate::emulator::micro_op::MachineFlag::UpdateCondCodes($reg),
        )
    };

    (SET_FLAG(WriteMemory)) => {
        $crate::emulator::micro_op::MicroOp::SetFlag(
            $crate::emulator::micro_op::MachineFlag::WriteMemory,
        )
    };

    // Messages
    (MSG $msg:expr) => {
        $crate::emulator::micro_op::MicroOp::Message($msg.to_string())
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::micro_op;

    #[test]
    fn test_micro_op_macro() {
        // Test basic phase transition
        let phase_op = micro_op!(-> Execute);
        match phase_op {
            MicroOp::PhaseTransition(CycleState::Execute) => (),
            _ => panic!("Expected PhaseTransition(Execute)"),
        }

        // Test register transfer
        let reg_transfer = micro_op!(R(1) <- R(2));
        match reg_transfer {
            MicroOp::Transfer {
                source: DataSource::Register(2),
                destination: DataDestination::Register(1),
            } => (),
            _ => panic!("Expected register transfer R1 <- R2"),
        }

        // Test ALU operation
        let alu_op = micro_op!(ALU_OUT <- R(1) + R(2));
        match alu_op {
            MicroOp::Alu {
                operation: MAluOp::Add,
                operand1: DataSource::Register(1),
                operand2: DataSource::Register(2),
            } => (),
            _ => panic!("Expected ALU ADD operation"),
        }

        // Test condition code setting
        let cc_op = micro_op!(SET_CC(3));
        match cc_op {
            MicroOp::SetFlag(MachineFlag::UpdateCondCodes(3)) => (),
            _ => panic!("Expected SET_CC(3)"),
        }
    }

    #[test]
    fn test_micro_op_display() {
        // Test that micro-ops display correctly
        let transfer = micro_op!(PC <- MAR);
        assert_eq!(format!("{transfer}"), "PC <- MAR");

        let alu_add = micro_op!(ALU_OUT <- R(1) + R(2));
        assert_eq!(format!("{alu_add}"), "ALU_OUT <- R1 + R2");

        let alu_not = micro_op!(ALU_OUT <- NOT R(3));
        assert_eq!(format!("{alu_not}"), "ALU_OUT <- NOT R3");

        let phase = micro_op!(-> Execute);
        assert_eq!(format!("{phase}"), "-> Execute");

        let cc = micro_op!(SET_CC(5));
        assert_eq!(format!("{cc}"), "SET_CC(R5)");

        let msg = micro_op!(MSG "Testing");
        assert_eq!(format!("{msg}"), "[Testing]");
    }
}
