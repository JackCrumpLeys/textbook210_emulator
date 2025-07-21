use std::collections::HashMap;
use std::fmt;

use crate::emulator::{Emulator, Exception};

// Re-export EmulatorCell for convenience
pub use super::EmulatorCell;

/// The 5+1 phases of the instruction cycle, for display purposes.
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

type CustomMicroOpFunction = Box<dyn Fn(&mut Emulator) -> Result<(), Exception>>;

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
    Custom(CustomMicroOpFunction, String),
}

impl MicroOp {
    pub fn new_custom<F>(f: F, display_code: String) -> Self
    where
        F: Fn(&mut Emulator) -> Result<(), Exception> + 'static,
    {
        MicroOp::Custom(Box::new(f), display_code)
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

#[cfg(test)]
mod op_equivalence_tests {
    use tracing_test::traced_test;

    use crate::{
        emulator::{executor::CpuPhaseState, ops::OpCode, Emulator, EmulatorCell, PSR_ADDR},
        micro_op,
    };

    use super::{CycleState, MicroOpGenerator};

    /// Simulates the execution of a single instruction using the legacy `Op` trait methods.
    /// This function attempts to replicate the behavior of the old state machine.
    fn execute_legacy(emu: &mut Emulator, instruction: u16) {
        let pc = emu.pc.get() as usize;
        emu.memory[pc].set(instruction);

        emu.step();
    }

    /// Executes a single instruction by generating and running its micro-op plan.
    fn execute_micro_op(emu: &mut Emulator, instruction: u16) {
        // The micro-op plan needs the instruction in memory for the fetch cycle.
        let pc = emu.pc.get() as usize;
        emu.memory[pc].set(instruction);

        // Get the micro-op generator for the instruction
        let opcode = OpCode::from_instruction(EmulatorCell::new(instruction))
            .expect("Failed to decode instruction");
        let micro_op_gen: &dyn MicroOpGenerator = match &opcode {
            OpCode::Add(op) => op,
            OpCode::And(op) => op,
            OpCode::Br(op) => op,
            OpCode::Jmp(op) => op,
            OpCode::Jsr(op) => op,
            OpCode::Ld(op) => op,
            OpCode::Ldi(op) => op,
            OpCode::Ldr(op) => op,
            OpCode::Lea(op) => op,
            OpCode::Not(op) => op,
            OpCode::Rti(op) => op,
            OpCode::St(op) => op,
            OpCode::Sti(op) => op,
            OpCode::Str(op) => op,
            OpCode::Trap(op) => op,
        };

        // Get the plan for the specific instruction phases
        let mut op_plan_map = micro_op_gen.generate_plan();

        // Create the full 6-phase execution plan
        let full_plan = vec![
            // Phase 0: Fetch
            vec![
                micro_op!(-> Fetch),
                micro_op!(MAR <- PC),
                micro_op!(ALU_OUT <- PC + C(1)),
                micro_op!(PC <- AluOut),
            ],
            // Phase 1: Decode
            vec![
                micro_op!(-> Decode),
                micro_op!(IR <- MDR),
                micro_op!(MSG format!("Instruction decoded: {}", opcode)),
            ],
            // Phase 2: EvaluateAddress
            {
                let mut v = vec![micro_op!(-> EvaluateAddress)];
                v.extend(
                    op_plan_map
                        .remove(&CycleState::EvaluateAddress)
                        .unwrap_or_default(),
                );
                v
            },
            // Phase 3: FetchOperands
            {
                let mut v = vec![micro_op!(-> FetchOperands)];
                v.extend(
                    op_plan_map
                        .remove(&CycleState::FetchOperands)
                        .unwrap_or_default(),
                );
                v
            },
            // Phase 4: Execute
            {
                let mut v = vec![micro_op!(-> Execute)];
                v.extend(op_plan_map.remove(&CycleState::Execute).unwrap_or_default());
                v
            },
            // Phase 5: StoreResult
            {
                let mut v = vec![micro_op!(-> StoreResult)];
                v.extend(
                    op_plan_map
                        .remove(&CycleState::StoreResult)
                        .unwrap_or_default(),
                );
                v
            },
        ];

        let mut executor = CpuPhaseState::new(full_plan);
        executor
            .step_instruction(emu)
            .expect("Micro-op execution failed");
    }

    /// A helper to test that legacy and micro-op execution paths yield the same result.
    fn test_op_equivalence(
        instruction: u16,
        pc: u16,
        setup: impl Fn(&mut Emulator),
        check: impl Fn(&Emulator, &Emulator),
    ) {
        // --- Setup ---
        let mut emu_legacy = Emulator::new();
        emu_legacy.pc.set(pc);
        setup(&mut emu_legacy);

        let mut emu_micro = Emulator::new();
        emu_micro.pc.set(pc);
        setup(&mut emu_micro);

        // --- Execute ---
        execute_legacy(&mut emu_legacy, instruction);
        execute_micro_op(&mut emu_micro, instruction);

        // --- Compare ---
        for i in 0..8 {
            assert_eq!(
                emu_legacy.r[i].get(),
                emu_micro.r[i].get(),
                "Register R{} differs (legacy: {:04X}, micro: {:04X})",
                i,
                emu_legacy.r[i].get(),
                emu_micro.r[i].get()
            );
        }
        assert_eq!(
            emu_legacy.pc.get(),
            emu_micro.pc.get(),
            "PC differs (legacy: {:04X}, micro: {:04X})",
            emu_legacy.pc.get(),
            emu_micro.pc.get()
        );
        assert_eq!(
            emu_legacy.get_nzp(),
            emu_micro.get_nzp(),
            "Flags (NZP) differ (legacy: {:?}, micro: {:?})",
            emu_legacy.get_nzp(),
            emu_micro.get_nzp()
        );
        check(&emu_legacy, &emu_micro);
    }

    #[traced_test]
    #[test]
    fn test_and_reg_equivalence() {
        test_op_equivalence(
            0b0101_001_010_0_00_011, // AND R1, R2, R3
            0x3000,
            |emu| {
                emu.r[2].set(0xF0F0);
                emu.r[3].set(0x0F0F);
            },
            |_, _| {
                // No extra checks needed, register state is enough
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_ldr_equivalence() {
        test_op_equivalence(
            0b0110_001_010_000101, // LDR R1, R2, #5
            0x3000,
            |emu| {
                emu.r[2].set(0x4000); // Base register
                emu.memory[0x4005].set(0xABCD);
            },
            |_, _| {},
        );
    }

    #[traced_test]
    #[test]
    fn test_st_equivalence() {
        test_op_equivalence(
            0b0011_001_000000000, // ST R1, #0
            0x3050,
            |emu| {
                emu.r[1].set(0xBEEF);
            },
            |legacy, micro| {
                // Target address = PC + 1 + offset = 0x3050 + 1 + 0 = 0x3051
                let addr = 0x3051;
                assert_eq!(
                    legacy.memory[addr].get(),
                    0xBEEF,
                    "Legacy memory not written correctly"
                );
                assert_eq!(
                    micro.memory[addr].get(),
                    0xBEEF,
                    "Micro-op memory not written correctly"
                );
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_br_taken_equivalence() {
        test_op_equivalence(
            0b0000_001_0_00001010, // BRP #10
            0x3000,
            |emu| {
                // Set positive flag
                emu.r[0].set(1);
                emu.update_flags(0);
            },
            |_, _| {}, // PC check is sufficient
        );
    }

    #[traced_test]
    #[test]
    fn test_br_not_taken_equivalence() {
        test_op_equivalence(
            0b0000_100_0_00001010, // BRN #10
            0x3000,
            |emu| {
                // Set positive flag
                emu.r[0].set(1);
                emu.update_flags(0);
            },
            |_, _| {}, // PC check is sufficient
        );
    }
    #[traced_test]
    #[test]
    fn test_add_reg_equivalence() {
        test_op_equivalence(
            0b0001_001_010_0_00_011, // ADD R1, R2, R3
            0x3000,
            |emu| {
                emu.r[2].set(10);
                emu.r[3].set(20);
            },
            |_, _| {
                // Register and flag checks are sufficient
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_add_imm_equivalence() {
        test_op_equivalence(
            0b0001_001_010_1_11111, // ADD R1, R2, #-1
            0x3000,
            |emu| {
                emu.r[2].set(10);
            },
            |_, _| {
                // Register and flag checks are sufficient
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_jmp_equivalence() {
        test_op_equivalence(
            0b1100_000_011_000000, // JMP R3
            0x3000,
            |emu| {
                emu.r[3].set(0x4000);
            },
            |_, _| {
                // PC check is sufficient
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_ret_equivalence() {
        test_op_equivalence(
            0b1100_000_111_000000, // RET (JMP R7)
            0x3000,
            |emu| {
                emu.r[7].set(0x5000);
            },
            |_, _| {
                // PC check is sufficient
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_jsr_equivalence() {
        test_op_equivalence(
            0b0100_1_00000001010, // JSR #10
            0x3000,
            |_| {},
            |legacy, micro| {
                // PC should be 0x3000 + 1 + 10 = 0x300B
                // R7 should be 0x3001
                assert_eq!(legacy.r[7].get(), 0x3001, "Legacy R7 incorrect");
                assert_eq!(micro.r[7].get(), 0x3001, "Micro-op R7 incorrect");
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_jsrr_equivalence() {
        test_op_equivalence(
            0b0100_0_00_011_000000, // JSRR R3
            0x3000,
            |emu| {
                emu.r[3].set(0x5000);
            },
            |legacy, micro| {
                // R7 should be 0x3001
                assert_eq!(legacy.r[7].get(), 0x3001, "Legacy R7 incorrect");
                assert_eq!(micro.r[7].get(), 0x3001, "Micro-op R7 incorrect");
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_ld_equivalence() {
        test_op_equivalence(
            0b0010_001_000000000, // LD R1, #0
            0x3050,
            |emu| {
                // Target address is 0x3050 + 1 = 0x3051
                emu.memory[0x3051].set(0xABCD);
            },
            |_, _| {
                // Register check is sufficient
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_ldi_equivalence() {
        test_op_equivalence(
            0b1010_001_000000000, // LDI R1, #0
            0x3050,
            |emu| {
                // Pointer address = 0x3050 + 1 = 0x3051
                // Final address is stored at 0x3051
                emu.memory[0x3051].set(0x4000);
                // Value to load is at 0x4000
                emu.memory[0x4000].set(0xABCD);
            },
            |_, _| {
                // Register check is sufficient
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_lea_equivalence() {
        test_op_equivalence(
            0b1110_001_111111111, // LEA R1, #-1
            0x3050,
            |_| {},
            |_, _| {
                // R1 should contain address 0x3050 + 1 - 1 = 0x3050
                // This is checked by the main register comparison loop.
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_not_equivalence() {
        test_op_equivalence(
            0b1001_001_010_111111, // NOT R1, R2
            0x3000,
            |emu| {
                emu.r[2].set(0b0101_0101_0101_0101);
            },
            |_, _| {
                // Register check is sufficient
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_sti_equivalence() {
        test_op_equivalence(
            0b1011_001_000000000, // STI R1, #0
            0x3050,
            |emu| {
                emu.r[1].set(0xBEEF);
                // Pointer address = 0x3050 + 1 + 0 = 0x3051
                // Final address is stored at 0x3051
                emu.memory[0x3051].set(0x4000);
            },
            |legacy, micro| {
                let addr = 0x4000;
                assert_eq!(
                    legacy.memory[addr].get(),
                    0xBEEF,
                    "Legacy memory not written correctly"
                );
                assert_eq!(
                    micro.memory[addr].get(),
                    0xBEEF,
                    "Micro-op memory not written correctly"
                );
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_str_equivalence() {
        test_op_equivalence(
            0b0111_001_010_000101, // STR R1, R2, #5
            0x3000,
            |emu| {
                emu.r[1].set(0xBEEF);
                emu.r[2].set(0x4000); // Base register
            },
            |legacy, micro| {
                // Target address = 0x4000 + 5 = 0x4005
                let addr = 0x4005;
                assert_eq!(
                    legacy.memory[addr].get(),
                    0xBEEF,
                    "Legacy memory not written correctly"
                );
                assert_eq!(
                    micro.memory[addr].get(),
                    0xBEEF,
                    "Micro-op memory not written correctly"
                );
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_rti_equivalence() {
        test_op_equivalence(
            0x8000, // RTI
            0x4000, // PC where RTI is
            |emu| {
                // PSR_ADDR is 0xFFFE. Start in supervisor mode. Bit 15 of PSR = 0.
                emu.memory[0xFFFE].set(0x0000);

                // Set up supervisor stack pointer (R6) and saved user stack pointer
                emu.r[6].set(0x2FF0);
                emu.saved_usp.set(0xF000);

                // Push PC and PSR onto the stack to be popped by RTI
                let new_pc = 0x5000;
                let new_psr = 0x8002; // Return to User mode, with P flag set
                emu.memory[0x2FF0].set(new_pc);
                emu.memory[0x2FF1].set(new_psr);
            },
            |legacy, _micro| {
                // The main `test_op_equivalence` will fail on R6 because the micro-op
                // version doesn't handle the USP/SSP swap.
                // We do an extra check here on the legacy emulator's internal state.
                assert_eq!(
                    legacy.saved_ssp.get(),
                    0x2FF2,
                    "Legacy saved_ssp not updated correctly"
                );
            },
        );
    }

    #[traced_test]
    #[test]
    fn test_trap_equivalence() {
        test_op_equivalence(
            0xF023, // TRAP x23 (IN)
            0x3000,
            |emu| {
                // Start in user mode with P flag set. PSR_ADDR is 0xFFFE.
                emu.memory[PSR_ADDR].set(0x8001);

                // Set user stack pointer (initial R6) and supervisor stack pointer
                emu.r[6].set(0xF000);
                emu.saved_ssp.set(0x2FF8);

                // Set up Trap Vector Table entry for x23
                let handler_addr = 0x1000;
                emu.memory[0x0023].set(handler_addr);
            },
            |legacy, micro| {
                let new_ssp = 0x2FF8 - 2;

                // Check stack contents (PC and PSR pushed)
                let old_pc = 0x3001;
                let old_psr = 0x8001;
                assert_eq!(
                    legacy.memory[new_ssp as usize].get(),
                    old_pc,
                    "Legacy PC not on stack"
                );
                assert_eq!(
                    micro.memory[new_ssp as usize].get(),
                    old_pc,
                    "Micro-op PC not on stack"
                );
                assert_eq!(
                    legacy.memory[(new_ssp + 1) as usize].get(),
                    old_psr,
                    "Legacy PSR not on stack"
                );
                assert_eq!(
                    micro.memory[(new_ssp + 1) as usize].get(),
                    old_psr,
                    "Micro-op PSR not on stack"
                );

                // Check that PSR now indicates supervisor mode
                assert_eq!(
                    legacy.memory[0xFFFE].get() & 0x8000,
                    0,
                    "Legacy not in supervisor mode"
                );
                assert_eq!(
                    micro.memory[0xFFFE].get() & 0x8000,
                    0,
                    "Micro-op not in supervisor mode"
                );
            },
        );
    }
}
