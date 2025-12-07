use crate::emulator::micro_op::{
    CycleState, DataDestination, DataSource, MAluOp, MachineFlag, MicroOp,
};
use crate::emulator::{
    AluOp, CpuState, Emulator, EmulatorCell, Exception, KBDR_ADDR, KBSR_ADDR, MCR_ADDR, OpCode,
    PSR_ADDR, area_from_address,
};
use std::fmt::{self};

/// Manages the execution state and flow of micro-operations within an instruction cycle
pub struct CpuPhaseState {
    /// The complete execution plan for the current instruction (6 phases)
    execution_plan: Vec<Vec<MicroOp>>,
    /// Current phase index (0-5)
    pub current_phase: usize,
    /// Current micro-op index within the current phase
    pub micro_op_index: usize,
    /// Flag indicating if the instruction is complete
    instruction_complete: bool,
    /// Flag indicating if a memory read is pending between phases
    pub memory_read_pending: bool,
    /// Flag indicating if a memory write is pending between phases
    pub memory_write_pending: bool,
    temp_register: EmulatorCell,
}

impl CpuPhaseState {
    /// Create a new phase state with the given execution plan
    pub fn new(execution_plan: Vec<Vec<MicroOp>>) -> Self {
        let span = tracing::trace_span!("CpuPhaseState::new", plan_phases = execution_plan.len());
        let _enter = span.enter();

        // Ensure we have exactly 6 phases
        if execution_plan.len() != 6 {
            tracing::warn!(
                "Execution plan has {} phases, expected 6",
                execution_plan.len()
            );
        }

        tracing::trace!(
            "Created new phase state with {} phases",
            execution_plan.len()
        );

        Self {
            execution_plan,
            current_phase: 0,
            micro_op_index: 0,
            instruction_complete: false,
            memory_read_pending: false,
            memory_write_pending: false,
            temp_register: EmulatorCell::new(0),
        }
    }

    /// Get the micro-ops for the current phase
    pub fn current_phase_ops(&self) -> &[MicroOp] {
        self.phase_ops(self.current_phase)
    }

    /// Get the micro-ops for a phase
    pub fn phase_ops(&self, i: usize) -> &[MicroOp] {
        if i < self.execution_plan.len() {
            &self.execution_plan[i]
        } else {
            &[]
        }
    }

    /// Check if the instruction is complete
    pub fn is_instruction_complete(&self) -> bool {
        self.instruction_complete
    }
}

impl Emulator {
    /// Execute a single micro-operation
    pub fn step_micro_op(&mut self) -> Result<(), String> {
        let span = tracing::trace_span!(
            "step_micro_op",
            phase = self.execute_state.current_phase,
            micro_op = self.execute_state.micro_op_index
        );
        let _enter = span.enter();

        if self.execute_state.current_phase >= self.execute_state.execution_plan.len() {
            self.execute_state.instruction_complete = true;
            return Ok(());
        }

        let current_phase_ops =
            &self.execute_state.execution_plan[self.execute_state.current_phase];

        if self.execute_state.micro_op_index >= current_phase_ops.len() {
            // Move to next phase
            self.execute_state.current_phase += 1;
            self.execute_state.micro_op_index = 0;

            if self.execute_state.current_phase >= self.execute_state.execution_plan.len() {
                self.execute_state.instruction_complete = true;
                tracing::trace!("Instruction execution complete");
                return Ok(());
            }

            // Update display phase
            if let Some(MicroOp::PhaseTransition(phase)) =
                self.execute_state.execution_plan[self.execute_state.current_phase].first()
            {
                self.cpu_state = match phase {
                    CycleState::Fetch => CpuState::Fetch,
                    CycleState::Decode => CpuState::Decode,
                    CycleState::EvaluateAddress => CpuState::EvaluateAddress(
                        self.cpu_state
                            .to_instruction()
                            .unwrap_or(OpCode::from_instruction(self.ir).unwrap()),
                    ),
                    CycleState::FetchOperands => CpuState::FetchOperands(
                        self.cpu_state
                            .to_instruction()
                            .unwrap_or(OpCode::from_instruction(self.ir).unwrap()),
                    ),
                    CycleState::Execute => CpuState::ExecuteOperation(
                        self.cpu_state
                            .to_instruction()
                            .unwrap_or(OpCode::from_instruction(self.ir).unwrap()),
                    ),
                    CycleState::StoreResult => CpuState::StoreResult(
                        self.cpu_state
                            .to_instruction()
                            .unwrap_or(OpCode::from_instruction(self.ir).unwrap()),
                    ),
                };
                tracing::trace!("transitioned to phase: {:?}", self.cpu_state);
            }

            return self.step_micro_op();
        }

        self.execute_micro_op()?;
        self.execute_state.micro_op_index += 1;

        Ok(())
    }

    /// Execute all micro-ops in the current phase
    pub fn step_phase(&mut self) -> Result<(), String> {
        let span = tracing::trace_span!("step_phase", phase = self.execute_state.current_phase);
        let _enter = span.enter();

        if self.execute_state.current_phase >= self.execute_state.execution_plan.len() {
            self.execute_state.instruction_complete = true;
            return Ok(());
        }

        let ops_in_phase =
            self.execute_state.execution_plan[self.execute_state.current_phase].len();
        tracing::trace!("Executing {} micro-ops in current phase", ops_in_phase);

        while self.execute_state.micro_op_index < ops_in_phase {
            self.step_micro_op()?;
        }

        // Perform implicit memory operations between phases
        self.handle_implicit_memory_operations()?;

        // Move to next phase
        self.execute_state.current_phase += 1;
        self.execute_state.micro_op_index = 0;

        if self.execute_state.current_phase >= self.execute_state.execution_plan.len() {
            self.execute_state.instruction_complete = true;
            tracing::trace!("Instruction execution complete");
        } else if let Some(MicroOp::PhaseTransition(phase)) =
            self.execute_state.execution_plan[self.execute_state.current_phase].first()
        {
            self.cpu_state = match phase {
                CycleState::Fetch => CpuState::Fetch,
                CycleState::Decode => CpuState::Decode,
                CycleState::EvaluateAddress => CpuState::EvaluateAddress(
                    self.cpu_state
                        .to_instruction()
                        .unwrap_or(OpCode::from_instruction(self.ir).unwrap()),
                ),
                CycleState::FetchOperands => CpuState::FetchOperands(
                    self.cpu_state
                        .to_instruction()
                        .unwrap_or(OpCode::from_instruction(self.ir).unwrap()),
                ),
                CycleState::Execute => CpuState::ExecuteOperation(
                    self.cpu_state
                        .to_instruction()
                        .unwrap_or(OpCode::from_instruction(self.ir).unwrap()),
                ),
                CycleState::StoreResult => CpuState::StoreResult(
                    self.cpu_state
                        .to_instruction()
                        .unwrap_or(OpCode::from_instruction(self.ir).unwrap()),
                ),
            };
        }

        Ok(())
    }

    /// Handle implicit memory operations that occur between phases
    fn handle_implicit_memory_operations(&mut self) -> Result<(), String> {
        let span = tracing::trace_span!("implicit_memory_ops");
        let _enter = span.enter();

        // Check if there's a pending memory write
        if self.execute_state.memory_write_pending {
            let addr = self.mar.get() as usize;
            let value = self.mdr.get();

            // Check write permissions
            let addr_cell = EmulatorCell::new(addr as u16);
            let area = area_from_address(&addr_cell);
            if !area.can_write(&self.priv_level()) {
                self.exception = Some(Exception::AccessControlViolation);
                return Err("Access control violation during memory write".to_string());
            }

            if addr < self.memory.len() {
                self.memory[addr].set(value);
                tracing::trace!("Implicit memory write: [0x{:04X}] <- 0x{:04X}", addr, value);
                if value == 0 && addr == MCR_ADDR {
                    self.halted = true;
                }
            } else {
                return Err(format!("Memory write address out of bounds: 0x{addr:04X}"));
            }

            self.execute_state.memory_write_pending = false;
            self.execute_state.memory_read_pending = false;
            return Ok(());
        }

        // Check if there's a pending memory read (MAR was set in previous phase)
        if self.execute_state.memory_read_pending {
            let addr = self.mar.get() as usize;

            // Check read permissions
            let addr_cell = EmulatorCell::new(addr as u16);
            let area = area_from_address(&addr_cell);
            if !area.can_read(&self.priv_level()) {
                self.exception = Some(Exception::AccessControlViolation);
                return Err("Access control violation during memory read".to_string());
            }

            if addr < self.memory.len() {
                let value = self.memory[addr].get();
                self.mdr.set(value);
                tracing::trace!(
                    "Implicit memory read: [0x{:04X}] -> MDR = 0x{:04X}",
                    addr,
                    value
                );
            } else {
                return Err(format!("Memory read address out of bounds: 0x{addr:04X}"));
            }

            if addr == KBDR_ADDR {
                self.memory[KBSR_ADDR].set(0x0000);
            }

            self.execute_state.memory_read_pending = false;
        }

        Ok(())
    }

    /// Execute the entire instruction
    pub fn step_instruction(&mut self) -> Result<(), String> {
        let span = tracing::trace_span!("step_instruction");
        let _enter = span.enter();

        while !self.execute_state.instruction_complete {
            self.step_phase()?;
        }

        tracing::trace!("Instruction execution complete");
        Ok(())
    }

    /// Execute a single micro-operation on the emulator state
    fn execute_micro_op(&mut self) -> Result<(), String> {
        let span = tracing::trace_span!("execute_micro_op");
        let _enter = span.enter();

        match &(self.execute_state.execution_plan[self.execute_state.current_phase])
            [self.execute_state.micro_op_index]
        {
            MicroOp::Transfer {
                source,
                destination,
            } => {
                let value = self.get_source_value(source)?;
                tracing::trace!(
                    "Transfer: {} -> {} (value: 0x{:04X})",
                    source,
                    destination,
                    value.get()
                );
                self.set_destination_value(&destination.clone(), value.get())?;
            }

            MicroOp::Alu {
                operation,
                operand1,
                operand2,
            } => {
                let val1 = self.get_source_value(operand1)?;
                let val2 = self.get_source_value(operand2)?;
                self.alu.op = Some(match operation {
                    MAluOp::Add => AluOp::Add(val1, val2),
                    MAluOp::And => AluOp::And(val1, val2),
                    MAluOp::Not => AluOp::Not(val1),
                });
                if let Some(alu_op) = self.alu.op.take() {
                    self.alu.alu_out = alu_op.execute();
                }
            }

            MicroOp::PhaseTransition(phase) => {
                tracing::trace!("Phase transition: {}", phase);
                self.handle_implicit_memory_operations()?;
            }

            MicroOp::SetFlag(flag) => {
                match flag {
                    MachineFlag::UpdateCondCodes(reg_num) => {
                        tracing::trace!("Updated condition codes for R{}", reg_num);
                        self.update_flags(*reg_num as usize);
                    }
                    MachineFlag::WriteMemory => {
                        self.execute_state.memory_write_pending = true;
                        // Perform the memory write if MAR and MDR are set
                        if self.mar.get() != 0 {
                            let addr = self.mar.get() as usize;
                            let value = self.mdr.get();

                            // Check write permissions
                            // let area = area_from_address(&self.mar);
                            // if !area.can_write(&self.priv_level()) {
                            //     self.exception = Some(Exception::AccessControlViolation);
                            //     return Err(
                            //         "Access control violation during memory write".to_string()
                            //     );
                            // }

                            if addr < self.memory.len() {
                                // self.memory[addr].set(value);
                                tracing::trace!("Memory write: [0x{:04X}] = 0x{:04X}", addr, value);
                            } else {
                                return Err(format!(
                                    "Memory write address out of bounds: 0x{addr:04X}"
                                ));
                            }
                        }
                    }
                }
            }

            MicroOp::Message(msg) => {
                tracing::trace!("Message: {}", msg);
            }

            MicroOp::Custom(f, _) => match f.clone()(self) {
                Ok(()) => (),
                Err(err) => {
                    self.exception = Some(err.clone());
                    return Err(format!("{err:?}"));
                }
            },
        }

        Ok(())
    }

    /// Set a value to a data destination
    fn set_destination_value(
        &mut self,
        destination: &DataDestination,
        value: u16,
    ) -> Result<(), String> {
        let span =
            tracing::trace_span!("set_destination_value", dest = %destination, value = value);
        let _enter = span.enter();

        match destination {
            DataDestination::Register(reg_num) => {
                if *reg_num > 7 {
                    return Err(format!("Invalid register number: {reg_num}"));
                }
                self.r[*reg_num as usize].set(value);
                tracing::trace!("Write R{} <- 0x{:04X}", reg_num, value);
            }

            DataDestination::PC => {
                self.pc.set(value);
                tracing::trace!("Write PC <- 0x{:04X}", value);
            }

            DataDestination::IR => {
                self.ir.set(value);
                tracing::trace!("Write IR <- 0x{:04X}", value);
            }

            DataDestination::MAR => {
                self.mar.set(value);
                // Setting MAR triggers a memory read that will happen between phases
                self.execute_state.memory_read_pending = true;
                tracing::trace!("Write MAR <- 0x{:04X} (memory read pending)", value);
            }

            DataDestination::MDR => {
                self.mdr.set(value);
                tracing::trace!("Write MDR <- 0x{:04X}", value);
            }

            DataDestination::PSR => {
                self.memory[PSR_ADDR].set(value);
                tracing::trace!("Write PSR <- 0x{:04X}", value);
            }

            DataDestination::AluOut => {
                self.alu.alu_out.set(value);
                tracing::trace!("Write ALU_OUT <- 0x{:04X}", value);
            }

            DataDestination::Temp => {
                self.execute_state.temp_register.set(value);
                tracing::trace!("Write TEMP <- 0x{:04X}", value);
            }
        }

        Ok(())
    }
    /// Get the value from a data source
    fn get_source_value(&self, source: &DataSource) -> Result<EmulatorCell, String> {
        let span = tracing::trace_span!("get_source_value", source = %source);
        let _enter = span.enter();

        let value = match source {
            DataSource::Register(reg_num) => {
                if *reg_num > 7 {
                    return Err(format!("Invalid register number: {reg_num}"));
                }
                let value = self.r[*reg_num as usize].get();
                tracing::trace!("Read R{} = 0x{:04X}", reg_num, value);
                value
            }

            DataSource::PC => {
                let value = self.pc.get();
                tracing::trace!("Read PC = 0x{:04X}", value);
                value
            }

            DataSource::IR => {
                let value = self.ir.get();
                tracing::trace!("Read IR = 0x{:04X}", value);
                value
            }

            DataSource::MAR => {
                let value = self.mar.get();
                tracing::trace!("Read MAR = 0x{:04X}", value);
                value
            }

            DataSource::MDR => {
                let value = self.mdr.get();
                tracing::trace!("Read MDR = 0x{:04X}", value);
                value
            }

            DataSource::PSR => {
                let value = self.memory[PSR_ADDR].get();
                tracing::trace!("Read PSR = 0x{:04X}", value);
                value
            }

            DataSource::AluOut => {
                let value = self.alu.alu_out.get();
                tracing::trace!("Read ALU_OUT = 0x{:04X}", value);
                value
            }

            DataSource::Temp => {
                let value = self.execute_state.temp_register.get();
                tracing::trace!("Read TEMP = 0x{:04X}", value);
                value
            }

            DataSource::Immediate(imm) => {
                let value = *imm as u16;
                tracing::trace!("Immediate value: 0x{:04X}", value);
                value
            }

            DataSource::PCOffset(offset) => {
                let value = *offset as u16;
                tracing::trace!("PC offset: 0x{:04X}", value);
                value
            }

            DataSource::TrapVector(vector) => {
                let value = *vector as u16;
                tracing::trace!("Trap vector: 0x{:02X}", vector);
                value
            }

            DataSource::Constant(constant) => {
                tracing::trace!("Constant: 0x{:04X}", constant);
                *constant
            }
        };

        Ok(EmulatorCell::new(value))
    }
}

impl fmt::Display for CpuPhaseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Phase {}/6, Op {}/{})",
            self.current_phase + 1,
            self.micro_op_index + 1,
            if self.current_phase < self.execution_plan.len() {
                self.execution_plan[self.current_phase].len()
            } else {
                0
            }
        )
    }
}
#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    use crate::micro_op;

    fn create_test_emulator() -> Emulator {
        let mut emulator = Emulator::new();
        // Set up some initial state for testing
        emulator.r[1].set(0x1000);
        emulator.r[2].set(0x2000);
        emulator.pc.set(0x3000);
        emulator
    }

    #[test]
    fn test_simple_transfer() {
        let mut emulator = create_test_emulator();

        let plan = vec![vec![micro_op!(-> Execute), micro_op!(R(0) <- R(1))]];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;

        // Step through all micro-ops
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.r[0].get(), 0x1000);
    }

    #[test]
    fn test_alu_operations() {
        let mut emulator = create_test_emulator();
        emulator.r[1].set(5);
        emulator.r[2].set(3);

        let plan = vec![vec![
            micro_op!(-> Execute),
            micro_op!(ALU_OUT <- R(1) + R(2)),
            micro_op!(R(0) <- AluOut),
        ]];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.r[0].get(), 8);
        assert_eq!(emulator.alu.alu_out.get(), 8);
    }

    #[test]
    fn test_alu_and_operation() {
        let mut emulator = create_test_emulator();
        emulator.r[1].set(0b1100);
        emulator.r[2].set(0b1010);

        let plan = vec![vec![
            micro_op!(-> Execute),
            micro_op!(ALU_OUT <- R(1) & R(2)),
            micro_op!(R(0) <- AluOut),
        ]];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.r[0].get(), 0b1000);
    }

    #[test]
    fn test_alu_not_operation() {
        let mut emulator = create_test_emulator();
        emulator.r[1].set(0b0000_1111_0000_1111);

        let plan = vec![vec![
            micro_op!(-> Execute),
            micro_op!(ALU_OUT <- NOT R(1)),
            micro_op!(R(0) <- AluOut),
        ]];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.r[0].get(), 0b1111_0000_1111_0000);
    }

    #[test]
    fn test_immediate_values() {
        let mut emulator = create_test_emulator();
        emulator.r[1].set(10);

        let plan = vec![vec![
            micro_op!(-> Execute),
            micro_op!(ALU_OUT <- R(1) + IMM(5)),
            micro_op!(R(0) <- AluOut),
        ]];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.r[0].get(), 15);
    }

    #[test]
    fn test_pc_operations() {
        let mut emulator = create_test_emulator();

        let plan = vec![vec![
            micro_op!(-> Execute),
            micro_op!(ALU_OUT <- PC + C(1)),
            micro_op!(PC <- AluOut),
        ]];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.pc.get(), 0x3001);
    }

    #[traced_test]
    #[test]
    fn test_memory_operations() {
        let mut emulator = create_test_emulator();
        emulator.memory[0x4000].set(0xABCD);
        emulator.mar.set(0x4000);

        let plan = vec![
            vec![micro_op!(-> Execute)], // implicit load
            vec![micro_op!(-> Execute), micro_op!(R(0) <- MDR)],
        ];

        let mut phase_state = CpuPhaseState::new(plan);
        phase_state.memory_read_pending = true; // simulate modifying mar
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.r[0].get(), 0xABCD);
    }

    #[test]
    fn test_condition_codes() {
        let mut emulator = create_test_emulator();
        emulator.r[1].set(0x8000); // Negative value

        let plan = vec![vec![
            micro_op!(-> Execute),
            micro_op!(R(0) <- R(1)),
            micro_op!(SET_CC(0)),
        ]];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        let (n, z, p) = emulator.get_nzp();
        assert!(n, "Negative flag should be set");
        assert!(!z, "Zero flag should not be set");
        assert!(!p, "Positive flag should not be set");
    }

    #[test]
    fn test_memory_write() {
        let mut emulator = create_test_emulator();
        emulator.mar.set(0x4000);
        emulator.mdr.set(0xBEEF);

        let plan = vec![vec![
            micro_op!(-> Execute),
            micro_op!(SET_FLAG(WriteMemory)),
            micro_op!(-> Execute), // write
        ]];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.memory[0x4000].get(), 0xBEEF);
    }

    #[test]
    fn test_step_micro_op() {
        let mut emulator = create_test_emulator();
        emulator.r[1].set(42);

        let plan = vec![
            vec![
                micro_op!(-> Execute),
                micro_op!(R(0) <- R(1)),
                micro_op!(ALU_OUT <- R(0) + IMM(1)),
                micro_op!(R(2) <- AluOut),
            ],
            vec![micro_op!(-> Decode)],
            vec![micro_op!(-> EvaluateAddress)],
            vec![micro_op!(-> FetchOperands)],
            vec![micro_op!(-> Execute)],
            vec![micro_op!(-> StoreResult)],
        ];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;

        // Test micro-op stepping within the first phase
        emulator.step_micro_op().unwrap(); // Phase transition
        emulator.step_micro_op().unwrap(); // R0 <- R1
        assert_eq!(emulator.r[0].get(), 42);

        emulator.step_micro_op().unwrap(); // ALU operation
        assert_eq!(emulator.alu.alu_out.get(), 43);

        emulator.step_micro_op().unwrap(); // R2 <- ALU_OUT
        assert_eq!(emulator.r[2].get(), 43);

        // Complete the remaining phases
        while !emulator.execute_state.is_instruction_complete() {
            emulator.step_phase().unwrap();
        }

        assert!(emulator.execute_state.is_instruction_complete());
    }

    #[test]
    #[traced_test]
    fn test_complex_add_instruction() {
        let mut emulator = create_test_emulator();
        emulator.r[1].set(10);
        emulator.r[2].set(5);

        // Simulate a complete ADD R0, R1, R2 instruction
        let plan = vec![
            vec![
                micro_op!(-> Fetch),
                micro_op!(MAR <- PC),
                micro_op!(ALU_OUT <- PC + C(1)),
                micro_op!(PC <- AluOut),
            ],
            vec![
                micro_op!(-> Decode),
                micro_op!(IR <- MDR),
                micro_op!(MSG "ADD instruction decoded"),
            ],
            vec![
                micro_op!(-> EvaluateAddress),
                micro_op!(MSG "No address evaluation needed for ADD"),
            ],
            vec![
                micro_op!(-> FetchOperands),
                micro_op!(MSG "Operands already in registers"),
            ],
            vec![micro_op!(-> Execute), micro_op!(ALU_OUT <- R(1) + R(2))],
            vec![
                micro_op!(-> StoreResult),
                micro_op!(R(0) <- AluOut),
                micro_op!(SET_CC(0)),
            ],
        ];

        let phase_state = CpuPhaseState::new(plan);
        emulator.execute_state = phase_state;
        emulator.step_instruction().unwrap();

        assert_eq!(emulator.r[0].get(), 15);
        assert_eq!(emulator.pc.get(), 0x3001);

        let (n, z, p) = emulator.get_nzp();
        assert!(p, "Positive flag should be set");
        assert!(!z && !n, "Zero and negative flags should not be set");
    }
}
