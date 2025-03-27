#![allow(clippy::unusual_byte_groupings)] // so we can group bits by instruction parts
#![allow(clippy::reversed_empty_ranges)] // We want to use ranges for bis like we have in class (big:small)

mod ops;
pub mod parse;
#[cfg(test)]
mod tests;

use std::ops::Range;

pub use ops::{CpuState, OpCode};

#[derive(Debug, Default, Clone, Copy)]
pub struct EmulatorCell(u16);

impl EmulatorCell {
    pub fn new(value: u16) -> Self {
        Self(value)
    }
    pub fn get(&self) -> u16 {
        self.0
    }

    pub fn set(&mut self, value: u16) {
        self.0 = value;
    }

    /// Sign extend from bit position to 16 bits
    /// bits to the left of pos must be 0
    pub fn sext(&self, bit_pos: u8) -> Self {
        let value = self.0;
        let is_negative = (value >> bit_pos) & 1 == 1;

        if is_negative {
            // Set all bits above bit_pos to 1
            let mask = !((1 << (bit_pos + 1)) - 1);
            Self(value | mask)
        } else {
            *self
        }
    }
}

#[derive(Debug, Clone)]
pub struct Emulator {
    // why non an array? Becuase array sits on stack and takes alot of memory.
    // wasm was unhappy so I put it on the heap using Vec
    pub memory: Vec<EmulatorCell>, // MUST be initialized with 65536 EmulatorCells
    // The alu component (this manages the maths operations)
    pub alu: Alu,

    // Privilege Level
    pub current_privilege_level: PrivilegeLevel,

    // Registers
    pub r: [EmulatorCell; 8],

    // Program Counter
    pub pc: EmulatorCell,

    // memory registers
    pub mar: EmulatorCell,
    pub mdr: EmulatorCell,

    // conditional registers
    pub z: EmulatorCell,
    pub n: EmulatorCell,
    pub p: EmulatorCell,

    // instruction register
    pub ir: EmulatorCell,

    // an interrupt for input.
    pub await_input: Option<bool>,
    pub output: String,

    // CPU state for micro steps
    pub cpu_state: CpuState,
    pub current_op: Option<u16>,

    // Running state
    pub running: bool,

    // write bit (if this is set after the store stage mem[mar] <- mdr)
    pub write_bit: bool,

    // exception
    pub exception: Option<Exception>,
}

#[derive(Debug, Clone)]
pub enum PrivilegeLevel {
    User,
    Supervisor,
}

#[derive(Debug, Clone)]
pub enum Exception {
    PrivilegeViolation,
    IllegalInstruction,
}

impl Exception {
    fn new_privilege_violation() -> Self {
        Exception::PrivilegeViolation
    }

    fn new_illegal_instruction() -> Self {
        Exception::IllegalInstruction
    }
}

#[derive(Debug, Clone, Default)]
pub struct Alu {
    pub op: Option<AluOp>,
    pub alu_out: EmulatorCell,
}

#[derive(Debug, Clone)]
pub enum AluOp {
    Add(EmulatorCell, EmulatorCell),
    And(EmulatorCell, EmulatorCell),
    Not(EmulatorCell),
}

impl AluOp {
    #[allow(dead_code)] // Im boutta use this just wanna get a clean commit of my restruccure
    fn execute(&self) -> EmulatorCell {
        EmulatorCell(match self {
            AluOp::Add(a, b) => a.get() + b.get(),
            AluOp::And(a, b) => a.get() & b.get(),
            AluOp::Not(a) => !a.get(),
        })
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Emulator {
    pub fn new() -> Emulator {
        Self {
            memory: vec![EmulatorCell(0); 65536],
            r: [EmulatorCell(0); 8],
            pc: EmulatorCell(0),
            mar: EmulatorCell(0),
            mdr: EmulatorCell(0),
            z: EmulatorCell(1),
            n: EmulatorCell(0),
            p: EmulatorCell(0),
            ir: EmulatorCell(0),
            await_input: None,
            output: String::new(),
            cpu_state: CpuState::Fetch,
            current_op: None,
            running: false,
            alu: Alu::default(),
            current_privilege_level: PrivilegeLevel::User,
            write_bit: false,
            exception: None,
        }
    }

    pub fn update_flags(&mut self, reg_index: usize) {
        let span = tracing::trace_span!(
            "update_flags",
            register = reg_index,
            value = format!("0x{:04X}", self.r[reg_index].get())
        );
        let _enter = span.enter();

        let value = self.r[reg_index].get();

        // Check if the value is negative (bit 15 is 1)
        let is_negative = (value >> 15) & 1 == 1;

        // Set negative flag
        if is_negative {
            self.n.set(1);
            self.z.set(0);
            self.p.set(0);
            tracing::trace!(n = 1, z = 0, p = 0, "Setting negative flag (N=1)");
        }
        // Set zero flag
        else if value == 0 {
            self.n.set(0);
            self.z.set(1);
            self.p.set(0);
            tracing::trace!(n = 0, z = 1, p = 0, "Setting zero flag (Z=1)");
        }
        // Set positive flag
        else {
            self.n.set(0);
            self.z.set(0);
            self.p.set(1);
            tracing::trace!(n = 0, z = 0, p = 1, "Setting positive flag (P=1)");
        }
    }
}

// emulator logic core
impl Emulator {
    pub fn fetch(&mut self) -> Result<(), &'static str> {
        let span = tracing::debug_span!("fetch");
        let _guard = span.enter();

        // Fetch the instruction
        let pc_value = self.pc.get();
        tracing::trace!("Fetching instruction at PC={:04X}", pc_value);

        self.mar.set(pc_value);
        tracing::trace!("Setting MAR={:04X}", pc_value);

        let instruction = self.memory[pc_value as usize].get();
        self.mdr.set(instruction);
        tracing::trace!(
            "Loaded MDR={:04X} (binary: {:016b})",
            instruction,
            instruction
        );

        // Load instruction into IR
        self.ir.set(self.mdr.get());
        tracing::trace!("Set IR={:04X}", self.mdr.get());

        // Increment PC
        let new_pc = pc_value.wrapping_add(1);
        self.pc.set(new_pc);
        tracing::trace!("Incremented PC to {:04X}", new_pc);

        tracing::debug!("Fetch completed successfully");
        Ok(())
    }

    pub fn decode(&mut self) -> Result<(), &'static str> {
        let span = tracing::debug_span!("decode");
        let _guard = span.enter();

        // Extract opcode (first 4 bits)
        let ir_value = self.ir.get();
        let opcode = ir_value >> 12;
        tracing::trace!("IR={:04X}, extracting opcode={:X}", ir_value, opcode);

        self.current_op = Some(opcode);
        tracing::trace!("Set current_op to {:X}", opcode);

        // Verify the opcode exists
        if let Some(op) = OpCode::from_value(opcode) {
            tracing::debug!("Decoded opcode {:X} as {:?}", opcode, op);
            Ok(())
        } else {
            tracing::error!("Unknown opcode: {:X}", opcode);
            self.exception = Some(Exception::new_illegal_instruction());
            Ok(())
        }
    }

    pub fn read_memory(&mut self) -> Result<(), &'static str> {
        let span = tracing::debug_span!("read_memory");
        let _guard = span.enter();

        if let Some(opcode) = self.current_op {
            tracing::trace!("Current opcode: {:X}", opcode);

            if let Some(op) = OpCode::from_value(opcode) {
                tracing::debug!("Preparing memory access for {:?}", op);

                op.prepare_memory_access(self);
                tracing::trace!(
                    "Memory access preparation completed, MAR={:04X}",
                    self.mar.get()
                );

                // Actually read from memory if MAR was set
                if self.mar.get() != 0 {
                    let mar_value = self.mar.get();
                    let memory_value = self.memory[mar_value as usize].get();
                    tracing::trace!("Reading memory[{:04X}]={:04X}", mar_value, memory_value);
                    self.mdr.set(memory_value);
                    tracing::trace!("Set MDR={:04X}", memory_value);
                } else {
                    tracing::trace!("No memory read needed (MAR=0)");
                }

                tracing::debug!("Memory read completed successfully");
                Ok(())
            } else {
                tracing::error!("Unknown opcode in read_memory: {:X}", opcode);
                Err("Unknown opcode")
            }
        } else {
            tracing::error!("No operation decoded in read_memory");
            Err("No operation decoded")
        }
    }

    pub fn execute(&mut self) -> Result<(), &'static str> {
        let span = tracing::debug_span!("execute");
        let _guard = span.enter();

        if let Some(opcode) = self.current_op {
            tracing::trace!("Current opcode: {:X}", opcode);

            if let Some(op) = OpCode::from_value(opcode) {
                tracing::debug!("Executing operation {:?}", op);

                op.execute(self);

                tracing::debug!("Execution completed successfully");
                tracing::trace!(
                    "Post-execution state: PC={:04X}, R={:?}, N={}, Z={}, P={}",
                    self.pc.get(),
                    self.r.iter().map(|r| r.get()).collect::<Vec<_>>(),
                    self.n.get(),
                    self.z.get(),
                    self.p.get()
                );

                Ok(())
            } else {
                tracing::error!("Unknown opcode in execute: {:X}", opcode);
                Err("Unknown opcode")
            }
        } else {
            tracing::error!("No operation decoded in execute");
            Err("No operation decoded")
        }
    }

    pub fn micro_step(&mut self) -> Result<(), &'static str> {
        let span = tracing::debug_span!("micro_step", state=?self.cpu_state);
        let _guard = span.enter();

        tracing::debug!("Starting micro_step in state {:?}", self.cpu_state);

        let result = match self.cpu_state {
            CpuState::Fetch => {
                tracing::trace!("Executing fetch phase");
                let result = self.fetch();
                if result.is_ok() {
                    self.cpu_state = CpuState::Decode;
                    tracing::trace!("State transition: Fetch -> Decode");
                }
                result
            }
            CpuState::Decode => {
                tracing::trace!("Executing decode phase");
                let result = self.decode();
                if result.is_ok() {
                    self.cpu_state = CpuState::ReadMemory;
                    tracing::trace!("State transition: Decode -> ReadMemory");
                }
                result
            }
            CpuState::ReadMemory => {
                tracing::trace!("Executing read_memory phase");
                let result = self.read_memory();
                if result.is_ok() {
                    self.cpu_state = CpuState::Execute;
                    tracing::trace!("State transition: ReadMemory -> Execute");
                }
                result
            }
            CpuState::Execute => {
                tracing::trace!("Executing execute phase");
                let result = self.execute();
                if result.is_ok() {
                    self.cpu_state = CpuState::Fetch;
                    self.current_op = None;
                    tracing::trace!("State transition: Execute -> Fetch");
                    tracing::trace!("Cleared current_op");
                }
                result
            }
        };

        if let Err(e) = &result {
            tracing::error!("Micro_step failed: {}", e);
        } else {
            tracing::debug!("Micro_step completed successfully");
        }

        result
    }

    pub fn step(&mut self) -> Result<(), &'static str> {
        let span = tracing::info_span!("step");
        let _guard = span.enter();

        tracing::info!("Starting instruction step");
        tracing::debug!(
            "Initial state: PC={:04X}, CPU state={:?}",
            self.pc.get(),
            self.cpu_state
        );

        // Complete a full instruction cycle
        while self.cpu_state != CpuState::Fetch || self.current_op.is_some() {
            tracing::trace!(
                "Cycling to reach Fetch state, current state={:?}, current_op={:?}",
                self.cpu_state,
                self.current_op
            );
            self.micro_step()?;
        }

        // Start the next cycle
        tracing::trace!("Starting next instruction cycle");
        let result = self.micro_step();

        if result.is_err() {
            tracing::error!("Instruction step failed");
        } else {
            tracing::info!("Instruction step completed successfully");
            tracing::debug!(
                "Final state: PC={:04X}, CPU state={:?}",
                self.pc.get(),
                self.cpu_state
            );
        }

        result
    }

    pub fn load_program(&mut self, program: &[u16], start_address: u16) {
        let span = tracing::info_span!(
            "load_program",
            start_address = start_address,
            program_size = program.len()
        );
        let _guard = span.enter();

        tracing::info!(
            "Loading program of {} instructions at address {:04X}",
            program.len(),
            start_address
        );

        for (i, &instruction) in program.iter().enumerate() {
            let addr = start_address.wrapping_add(i as u16);
            tracing::trace!(
                "Setting memory[{:04X}] = {:04X} (binary: {:016b})",
                addr,
                instruction,
                instruction
            );
            self.memory[addr as usize].set(instruction);
        }

        // Set PC to the program's start address
        self.pc.set(start_address);
        tracing::debug!("PC set to start address {:04X}", start_address);

        tracing::info!("Program loaded successfully");
    }

    pub fn run(&mut self, max_steps: Option<usize>) -> Result<(), &'static str> {
        let span = tracing::info_span!("run", max_steps=?max_steps);
        let _guard = span.enter();

        tracing::info!("Starting execution with max_steps={:?}", max_steps);
        let mut steps = 0;

        loop {
            if let Some(max) = max_steps {
                if steps >= max {
                    tracing::info!("Reached maximum steps ({}), stopping execution", max);
                    return Ok(());
                }
                steps += 1;
                tracing::trace!("Step {}/{}", steps, max);
            }

            // Check for input request
            if self.await_input.is_some() {
                tracing::info!("Execution paused, waiting for input");
                return Ok(());
            }

            let step_result = self.step();
            if let Err(e) = step_result {
                tracing::error!("Execution failed: {}", e);
                return Err(e);
            }

            if !self.running {
                tracing::info!("Program terminated (running flag is false)");
                return Ok(());
            }
        }
    }
}

trait BitAddressable {
    fn index(&self, addr: u8) -> Self;
    fn range(&self, slice: Range<u8>) -> Self;
}

impl BitAddressable for EmulatorCell {
    fn index(&self, addr: u8) -> Self {
        Self((self.0 >> addr) & 1)
    }

    fn range(&self, slice: Range<u8>) -> Self {
        // Reversed range: bigger (start) to smaller (end)
        assert!(slice.start >= slice.end, "Invalid range");
        let start = slice.start;
        let end = slice.end;
        let width = (start + 1) - end;
        let mask = ((1 << width) - 1) << end;
        tracing::trace!(
            value = format!("0x{:04X}", self.0),
            mask = format!("0b{:016b}", mask),
            range = format!("{}..{}", start, end),
            width = width,
            "Extracting bits using mask"
        );
        Self((self.0 & mask) >> end)
    }
}
