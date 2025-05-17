#![allow(clippy::unusual_byte_groupings)] // so we can group bits by instruction parts
#![allow(clippy::reversed_empty_ranges)] // We want to use ranges for bis like we have in class (big:small)

pub mod ops;
pub mod parse;
#[cfg(test)]
mod tests;

use std::{collections::HashSet, ops::Range};

pub use ops::{CpuState, OpCode};
use parse::ParseOutput;

// use crate::panes::emulator::machine::BREAKPOINTS; TODO

const STEPS_BETWEEN_FLUSH_GOT_NEW_CHAR: u32 = 30; // 5 full instructions

pub const MAX_OS_STEPS: usize = 1000;

// Device registers at memory addresses xFE00-xFFFF
pub const KBSR_ADDR: usize = 0xFE00;
pub const KBDR_ADDR: usize = 0xFE02;
pub const DSR_ADDR: usize = 0xFE04;
pub const DDR_ADDR: usize = 0xFE06;
pub const PSR_ADDR: usize = 0xFFFC;
pub const MCR_ADDR: usize = 0xFFFE;

#[derive(Debug, Default, Clone, Copy)]
pub struct EmulatorCell(u16, bool);

impl EmulatorCell {
    pub fn new(value: u16) -> Self {
        Self(value, true)
    }
    pub fn get(&self) -> u16 {
        self.0
    }

    pub fn set(&mut self, value: u16) {
        if value != self.0 {
            self.0 = value;
            self.1 = true;
        }
    }

    pub fn changed(&mut self) -> bool {
        let changed = self.1;
        self.1 = false;
        changed
    }

    pub fn changed_peek(&self) -> bool {
        self.1
    }

    /// Sign extend from bit position to 16 bits
    /// bits to the left of pos must be 0
    pub fn sext(&self, bit_pos: u8) -> Self {
        let value = self.0;
        let is_negative = (value >> bit_pos) & 1 == 1;

        if is_negative {
            // Set all bits above bit_pos to 1
            let mask = !((1 << (bit_pos + 1)) - 1);
            Self(value | mask, true)
        } else {
            *self
        }
    }
}

#[derive(Debug, Clone)]
pub struct Emulator {
    // update data
    pub speed: u32,
    pub ticks_between_updates: u32,
    pub tick: u64,
    pub skip_os_emulation: bool,

    // why non an array? Becuase array sits on stack and takes alot of memory.
    // wasm was unhappy so I put it on the heap using Vec
    pub memory: Vec<EmulatorCell>, // MUST be initialized with 65536 EmulatorCells or we will get so many errors
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

    pub output: String,

    // CPU state for micro steps
    pub cpu_state: CpuState,
    pub current_op: Option<u16>, // TODO: move this into cpustate

    // saved stack pointers
    pub saved_ssp: EmulatorCell,
    pub saved_usp: EmulatorCell,

    // Running state
    pub running: bool,

    // write bit (if this is set after the store stage mem[mar] <- mdr)
    pub write_bit: bool,

    // exception
    pub exception: Option<Exception>,

    // last pressed key
    pub last_pressed_key: EmulatorCell,
    pub flush_countdown: u32,
}

impl Emulator {
    pub fn new() -> Emulator {
        let mut emulator = Self {
            speed: 1,
            ticks_between_updates: 2,
            tick: 0,
            skip_os_emulation: true,
            memory: vec![EmulatorCell::new(0); 65536],
            r: [EmulatorCell::new(0); 8],
            pc: EmulatorCell::new(0x200),
            mar: EmulatorCell::new(0),
            mdr: EmulatorCell::new(0),
            z: EmulatorCell::new(1),
            n: EmulatorCell::new(0),
            p: EmulatorCell::new(0),
            ir: EmulatorCell::new(0),
            output: String::new(),
            cpu_state: CpuState::Fetch,
            current_op: None,
            running: false,
            alu: Alu::default(),
            current_privilege_level: PrivilegeLevel::Supervisor,
            write_bit: false,
            exception: None,
            saved_ssp: EmulatorCell::new(0),
            saved_usp: EmulatorCell::new(0),
            last_pressed_key: EmulatorCell::new(0),
            flush_countdown: 0,
        };

        let parse_output = Emulator::parse_program(include_str!("../oses/simpleos.asm"));

        tracing::debug!("OS parse_output: {:?}", parse_output);

        if let Ok(ParseOutput {
            machine_code,
            orig_address,
            ..
        }) = parse_output
        {
            emulator.flash_memory(machine_code, orig_address);
        } else {
            debug_assert!(false, "INVALID DEFAULT OS!!!");
        }

        emulator.memory[DSR_ADDR].set(0x8000); // ready for a char
        emulator.memory[PSR_ADDR].set(0x0002); // Z=1, N=0, P=0
        emulator.memory[MCR_ADDR].set(0x8000); // machine is "running" from the perspective of the contained code

        emulator
    }

    pub fn update_flags(&mut self, reg_index: usize) {
        let value = self.r[reg_index].get();

        // Check if the value is negative (bit 15 is 1)
        let is_negative = (value >> 15) & 1 == 1;

        // Set negative flag
        if is_negative {
            self.n.set(1);
            self.z.set(0);
            self.p.set(0);
        }
        // Set zero flag
        else if value == 0 {
            self.n.set(0);
            self.z.set(1);
            self.p.set(0);
        }
        // Set positive flag
        else {
            self.n.set(0);
            self.z.set(0);
            self.p.set(1);
        }
    }
}

#[derive(Debug, Clone)]
pub enum PrivilegeLevel {
    User,
    Supervisor,
}

pub fn area_from_address(addr: &EmulatorCell) -> MemoryArea {
    match addr.get() {
        0x0000..=0x00FF => MemoryArea::TrapVectorTable,
        0x0100..=0x01FF => MemoryArea::IntrruptVectorTable,
        0x0200..=0x2FFF => MemoryArea::OperatingSystem,
        0x3000..=0xFDFF => MemoryArea::UserSpace,
        0xFE00..=0xFFFF => MemoryArea::DeviceRegisters,
    }
}

pub enum MemoryArea {
    TrapVectorTable,     // x0000 - x00FF (jumpable by userspace (via trap) not storable)
    IntrruptVectorTable, // x0100 - x01FF (No permissions for userspae)
    OperatingSystem,     // x0200 - x2FFF (No permissions for userspae)
    UserSpace,           // x3000 - xFDFF (rwx for userspace)
    DeviceRegisters,     // xFE00 - xFFFF (No permissions for userspae)
}

impl MemoryArea {
    pub fn can_read(&self, level: &PrivilegeLevel) -> bool {
        match level {
            PrivilegeLevel::User => match self {
                MemoryArea::TrapVectorTable => true,
                MemoryArea::IntrruptVectorTable => false,
                MemoryArea::OperatingSystem => false,
                MemoryArea::UserSpace => true,
                MemoryArea::DeviceRegisters => false,
            },
            PrivilegeLevel::Supervisor => true, // Supervisor can read anything
        }
    }

    pub fn can_write(&self, level: &PrivilegeLevel) -> bool {
        match level {
            PrivilegeLevel::User => match self {
                MemoryArea::TrapVectorTable => false,
                MemoryArea::IntrruptVectorTable => false,
                MemoryArea::OperatingSystem => false,
                MemoryArea::UserSpace => true,
                MemoryArea::DeviceRegisters => false,
            },
            // Supervisor generally can write, but maybe some areas mabye read-only even for supervisor (trap table for safety?)
            // Let's assume supervisor can write everywhere for now, adjust if needed.
            PrivilegeLevel::Supervisor => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Exception {
    PrivilegeViolation,
    IllegalInstruction,
    AccessControlViolation,
}

impl Exception {
    fn new_privilege_violation() -> Self {
        Exception::PrivilegeViolation
    }

    fn new_illegal_instruction() -> Self {
        Exception::IllegalInstruction
    }

    fn new_access_control_violation() -> Self {
        Exception::AccessControlViolation
    }

    fn get_handler_address(&self) -> usize {
        // Base address of the Interrupt Vector Table
        const IVT_BASE: usize = 0x0100;
        // TODO: Should we make more?
        match self {
            Exception::PrivilegeViolation => IVT_BASE, // Vector x00 in IVT for Privilege Violation
            Exception::IllegalInstruction => IVT_BASE + 0x01, // Vector x01 in IVT for Illegal Opcode
            Exception::AccessControlViolation => IVT_BASE + 0x02, // Using x02 for Access Control, adjust if standard defines otherwise
        }
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
    fn execute(&self) -> EmulatorCell {
        EmulatorCell::new(match self {
            AluOp::Add(a, b) => a.get().wrapping_add(b.get()),
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

// emulator logic core
impl Emulator {
    // this will be called every frame (bool if emulator state changed)
    pub fn update(&mut self) -> bool {
        let breakpoints: HashSet<usize> = HashSet::new(); // TODO

        let mut os_steps = 0;
        let mut changed = false;

        self.tick = self.tick.wrapping_add(1);

        if self.running {
            if self.skip_os_emulation {
                while self.pc.get() < 0x3000 && os_steps < MAX_OS_STEPS && self.running {
                    changed = true;
                    self.step();
                    os_steps += 1;
                }
            }

            // Automatic stepping logic when running
            if self.tick % self.ticks_between_updates as u64 == 0 {
                let mut i = 0;
                while self.running && i < self.speed {
                    changed = true;
                    self.micro_step();
                    i += 1;

                    // Skip OS code if enabled during running mode
                    // Limit OS skipping to avoid freezing

                    while self.skip_os_emulation
                        && self.pc.get() < 0x3000
                        && os_steps < MAX_OS_STEPS
                        && self.running
                    {
                        self.step();
                        os_steps += 1;
                    }

                    if !self.pc.get() < 0x3000 {
                        // Check for breakpoints
                        let current_pc = self.pc.get() as usize;

                        if breakpoints.contains(&current_pc)
                            && matches!(self.cpu_state, CpuState::Fetch)
                        // Break *before* fetching the instruction at the breakpoint
                        {
                            self.running = false;
                            log::info!("Breakpoint hit at address 0x{:04X}", current_pc);
                            break;
                        }
                    }

                    if i >= self.speed {
                        break;
                    }
                }
            }

            if self.skip_os_emulation {
                while self.pc.get() < 0x3000 && os_steps < MAX_OS_STEPS && self.running {
                    changed = true;
                    self.step();
                    os_steps += 1;
                }
            }
        }
        changed
    }

    // --- Core Instruction Cycle Phases ---

    /// **Fetch Phase:** Read instruction from memory at PC into IR, increment PC.
    fn fetch(&mut self) -> Result<(), String> {
        let pc_value = self.pc.get();
        let memory_area = area_from_address(&self.pc);

        // Check read permission for PC address
        if !memory_area.can_read(&self.current_privilege_level) {
            self.exception = Some(Exception::new_access_control_violation());
            return Err(format!(
                "Fetch Access Violation: Cannot read PC address 0x{:04X}",
                pc_value
            ));
        }

        self.mar.set(pc_value);
        // Implicit memory read: MDR <- Mem[MAR] happens here conceptually
        self.mdr.set(self.memory[self.mar.get() as usize].get());
        self.ir.set(self.mdr.get());

        // Increment PC
        self.pc.set(pc_value.wrapping_add(1));

        Ok(())
    }

    /// **Decode Phase:** Decode instruction in IR, determine OpCode.
    fn decode(&mut self) -> Result<OpCode, String> {
        match OpCode::from_instruction(self.ir) {
            Some(op) => Ok(op),
            None => {
                self.exception = Some(Exception::new_illegal_instruction());
                Err(format!(
                    "Decode Error: Illegal opcode in IR=0x{:04X}",
                    self.ir.get()
                ))
            }
        }
    }

    /// **Evaluate Address Phase:** Calculate effective address for memory access or jump/branch targets.
    fn evaluate_address(&mut self, op: &mut OpCode) -> Result<(), String> {
        op.evaluate_address(self);
        if self.exception.is_some() {
            Err("Exception occurred during address evaluation".to_string())
        } else {
            Ok(())
        }
    }

    pub fn step_read_memory(&mut self) {
        if self.mar.changed() {
            let mar_val = self.mar.get();
            let mem_area = area_from_address(&self.mar);
            // Generally we have already checked the privilege level in the address evaluation phase but to be as
            // thorough as possible, we check again here.
            if mem_area.can_read(&self.current_privilege_level) {
                self.mdr.set(self.memory[mar_val as usize].get());
            } else {
                self.exception = Some(Exception::new_access_control_violation());
            }
        }
    }

    pub fn step_write_memory(&mut self) -> Result<(), String> {
        if self.mdr.changed() && self.write_bit {
            let mar_val = self.mar.get();
            let mem_area = area_from_address(&self.mar);
            // Generally we have already checked the privilege level in the address evaluation phase but to be as
            if mem_area.can_write(&self.current_privilege_level) {
                self.memory[mar_val as usize].set(self.mdr.get());
            } else {
                self.exception = Some(Exception::new_access_control_violation());
                return Err(format!(
                    "Fetch Operands Access Violation: Cannot write to MAR=0x{:04X}",
                    mar_val
                ));
            }
        }
        self.write_bit = false;
        Ok(())
    }

    /// **Fetch Operands Phase:** Read operands from registers or memory (via MAR/MDR).
    fn fetch_operands(&mut self, op: &mut OpCode) -> Result<(), String> {
        op.fetch_operands(self); // becuase this can run multiple times it manags the memory itself

        if self.exception.is_some() {
            Err("Exception occurred during operand fetch".to_string())
        } else {
            Ok(())
        }
    }

    /// **Execute Operation Phase:** Perform the core computation (ALU, PC update, etc.).
    fn execute_operation(&mut self, op: &mut OpCode) -> Result<(), String> {
        op.execute_operation(self);

        // Execute the ALU operation if one was set up by the Op's method
        if let Some(alu_op) = self.alu.op.take() {
            self.alu.alu_out = alu_op.execute();
        }

        if self.exception.is_some() {
            Err("Exception occurred during execution".to_string())
        } else {
            Ok(())
        }
    }

    /// **Store Result Phase:** Write result back to register or set up memory write.
    fn store_result(&mut self, op: &mut OpCode) -> Result<(), String> {
        // Clear write bit before the operation potentially sets it
        self.write_bit = false;
        op.store_result(self);

        // If write_bit was set by op.store_result(), perform the memory write
        if self.write_bit {
            let mar_val = self.mar.get();
            let mem_area = area_from_address(&self.mar);
            if mem_area.can_write(&self.current_privilege_level) {
                self.memory[mar_val as usize].set(self.mdr.get());
            } else {
                self.exception = Some(Exception::new_access_control_violation());
                return Err(format!(
                    "Store Result Access Violation: Cannot write to MAR=0x{:04X}",
                    mar_val
                ));
            }
            self.write_bit = false; // Reset after write
        }

        if self.exception.is_some() {
            Err("Exception occurred during result store".to_string())
        } else {
            Ok(())
        }
    }

    /// **Handle Exception:** Switch to supervisor mode, save state, jump to handler.
    fn handle_exception(&mut self, exception: Exception) {
        tracing::warn!("Handling Exception: {:?}", exception);

        // 1. Get handler address
        let handler_addr = exception.get_handler_address();
        let handler_addr = self.memory[handler_addr];

        // 2. Switch to Supervisor Mode & Stack
        if matches!(self.current_privilege_level, PrivilegeLevel::User) {
            self.saved_usp = self.r[6]; // Save User SP
            self.r[6] = self.saved_ssp; // Load Supervisor SP
        }
        self.current_privilege_level = PrivilegeLevel::Supervisor;

        // 3. Push PSR and PC onto the Supervisor Stack (R6)
        let ssp = self.r[6].get();
        let psr_addr = ssp.wrapping_sub(1);
        let pc_addr = ssp.wrapping_sub(2);

        // Create PSR value (simplified: PLevel=Supervisor, N Z P flags)
        // Note: Actual LC-3 PSR has more bits, this is a basic representation
        let psr_val = (1 << 15) // Supervisor mode
                      | (self.n.get() << 2)
                      | (self.z.get() << 1)
                      | self.p.get();

        // Check stack write permissions (should be writable in Supervisor mode)
        // Basic check: Ensure stack pointer is within valid memory range
        if pc_addr > 1 && pc_addr < (self.memory.len() - 1) as u16 {
            self.memory[psr_addr as usize].set(psr_val);
            self.memory[pc_addr as usize].set(self.pc.get()); // Push PC of *next* instruction
            self.r[6].set(pc_addr); // Update SSP
        } else {
            // Stack Overflow/Underflow - This is a critical error, potentially halt or double fault
            tracing::error!(
                "CRITICAL: Stack pointer R6=0x{:04X} out of bounds during exception handling.",
                ssp
            );
            self.running = false; // Halt on severe stack error
            return;
        }

        // 4. Set PC to handler address
        self.pc = handler_addr;

        // 5. Clear the exception state
        self.exception = None;

        // 6. Reset CPU state to Fetch for the handler routine
        self.cpu_state = CpuState::Fetch;
    }

    /// **Micro Step:** Execute one phase of the instruction cycle.
    pub fn micro_step(&mut self) {
        tracing::trace!(memory_size = self.memory.len(), "Entering micro_step");

        debug_assert!(
            self.memory.len() == 0x10000,
            "Memory size is not initialized with full addressable range"
        );

        // --- Check for and Handle Exceptions First ---
        if let Some(exc) = self.exception.clone() {
            tracing::info!(
                exception = format!("{:?}", exc),
                "Handling exception in micro_step"
            );
            // Clone to avoid borrow checker issues if handle_exception modifies self.exception
            self.handle_exception(exc);
            // After handling, the state is reset, so we can return Ok and the next micro_step will fetch the handler.
            tracing::debug!("Exception handled, returning from micro_step");
        }

        // Give devices a chance to do their things
        tracing::trace!("Updating devices");
        self.update_devices();
        tracing::trace!(
            kbsr = format!("0x{:04X}", self.memory[KBSR_ADDR].get()),
            kbdr = format!("0x{:04X}", self.memory[KBDR_ADDR].get()),
            dsr = format!("0x{:04X}", self.memory[DSR_ADDR].get()),
            ddr = format!("0x{:04X}", self.memory[DDR_ADDR].get()),
            "Device registers after update"
        );

        // --- Execute Current CPU State Phase ---
        let current_state = self.cpu_state.clone(); // Clone to allow modification within match arms
        tracing::debug!(
            state = format!("{:?}", current_state),
            pc = format!("0x{:04X}", self.pc.get()),
            ir = format!("0x{:04X}", self.ir.get()),
            n = self.n.get(),
            z = self.z.get(),
            p = self.p.get(),
            "Executing CPU state phase"
        );

        let result: Result<(), String>;

        match current_state {
            CpuState::Fetch => {
                tracing::debug!("Executing FETCH phase");
                result = self.fetch();
                tracing::trace!(
                    success = result.is_ok(),
                    pc = format!("0x{:04X}", self.pc.get()),
                    mar = format!("0x{:04X}", self.mar.get()),
                    mdr = format!("0x{:04X}", self.mdr.get()),
                    ir = format!("0x{:04X}", self.ir.get()),
                    "Fetch phase complete"
                );
                if result.is_ok() {
                    tracing::debug!("Fetch succeeded, transitioning to DECODE");
                    self.cpu_state = CpuState::Decode;
                } else {
                    tracing::error!(error = result.as_ref().err().unwrap(), "Fetch failed");
                }
            }
            CpuState::Decode => {
                tracing::debug!(
                    ir = format!("0x{:04X}", self.ir.get()),
                    "Executing DECODE phase"
                );
                match self.decode() {
                    Ok(op) => {
                        tracing::debug!(
                            opcode = format!("{:?}", op),
                            "Decode succeeded, transitioning to EVALUATE_ADDRESS"
                        );
                        self.cpu_state = CpuState::EvaluateAddress(op);
                        result = Ok(());
                    }
                    Err(e) => {
                        tracing::error!(
                            error = e,
                            ir = format!("0x{:04X}", self.ir.get()),
                            "Decode failed with illegal instruction"
                        );
                        result = Err(e); // Decode already set the exception
                    }
                }
            }
            CpuState::EvaluateAddress(mut op) => {
                tracing::debug!(
                    opcode = format!("{:?}", op),
                    "Executing EVALUATE_ADDRESS phase"
                );
                result = self.evaluate_address(&mut op);
                tracing::trace!(
                    success = result.is_ok(),
                    mar = format!("0x{:04X}", self.mar.get()),
                    "Address evaluation complete"
                );
                if result.is_ok() {
                    tracing::debug!(
                        "Address evaluation succeeded, transitioning to FETCH_OPERANDS"
                    );
                    self.cpu_state = CpuState::FetchOperands(op);
                } else {
                    tracing::error!(
                        error = result.as_ref().err().unwrap(),
                        "Address evaluation failed"
                    );
                }
            }
            CpuState::FetchOperands(mut op) => {
                tracing::debug!(
                    opcode = format!("{:?}", op),
                    "Executing FETCH_OPERANDS phase"
                );
                match self.fetch_operands(&mut op) {
                    Ok(()) => {
                        tracing::debug!(
                            "Operand fetch succeeded, transitioning to EXECUTE_OPERATION"
                        );
                        tracing::trace!(
                            r0 = format!("0x{:04X}", self.r[0].get()),
                            r1 = format!("0x{:04X}", self.r[1].get()),
                            r2 = format!("0x{:04X}", self.r[2].get()),
                            r3 = format!("0x{:04X}", self.r[3].get()),
                            r4 = format!("0x{:04X}", self.r[4].get()),
                            r5 = format!("0x{:04X}", self.r[5].get()),
                            r6 = format!("0x{:04X}", self.r[6].get()),
                            r7 = format!("0x{:04X}", self.r[7].get()),
                            "Register values after operand fetch"
                        );
                        self.cpu_state = CpuState::ExecuteOperation(op);
                        result = Ok(());
                    }
                    Err(e) => {
                        tracing::error!(error = e, "Operand fetch failed");
                        result = Err(e); // fetch_operands set the exception
                    }
                }
            }
            CpuState::ExecuteOperation(mut op) => {
                tracing::debug!(
                    opcode = format!("{:?}", op),
                    "Executing EXECUTE_OPERATION phase"
                );
                result = self.execute_operation(&mut op);
                if result.is_ok() {
                    tracing::debug!("Operation execution succeeded, transitioning to STORE_RESULT");
                    if let Some(alu_op) = &self.alu.op {
                        tracing::trace!(
                            alu_op = format!("{:?}", alu_op),
                            alu_out = format!("0x{:04X}", self.alu.alu_out.get()),
                            "ALU operation result"
                        );
                    }
                    self.cpu_state = CpuState::StoreResult(op);
                } else {
                    tracing::error!(
                        error = result.as_ref().err().unwrap(),
                        "Operation execution failed"
                    );
                }
            }
            CpuState::StoreResult(mut op) => {
                tracing::debug!(opcode = format!("{:?}", op), "Executing STORE_RESULT phase");
                result = self.store_result(&mut op);
                tracing::trace!(
                    success = result.is_ok(),
                    write_bit = self.write_bit,
                    mar = format!("0x{:04X}", self.mar.get()),
                    mdr = format!("0x{:04X}", self.mdr.get()),
                    "Store result complete"
                );
                if result.is_ok() {
                    // Instruction complete, go back to Fetch
                    tracing::debug!("Result store succeeded, cycling back to FETCH");
                    self.cpu_state = CpuState::Fetch;
                    tracing::trace!(
                        r0 = format!("0x{:04X}", self.r[0].get()),
                        r1 = format!("0x{:04X}", self.r[1].get()),
                        r2 = format!("0x{:04X}", self.r[2].get()),
                        r3 = format!("0x{:04X}", self.r[3].get()),
                        r4 = format!("0x{:04X}", self.r[4].get()),
                        r5 = format!("0x{:04X}", self.r[5].get()),
                        r6 = format!("0x{:04X}", self.r[6].get()),
                        r7 = format!("0x{:04X}", self.r[7].get()),
                        n = self.n.get(),
                        z = self.z.get(),
                        p = self.p.get(),
                        "CPU state after instruction completion"
                    );
                } else {
                    tracing::error!(
                        error = result.as_ref().err().unwrap(),
                        "Result store failed"
                    );
                }

                let write_result = self.step_write_memory();
                tracing::trace!(success = write_result.is_ok(), "Memory write step complete");
                if let Err(e) = &write_result {
                    tracing::error!(error = e, "Memory write step failed");
                }
            }
        }

        // If any phase resulted in an error (and set an exception),
        // the exception check at the start of the *next* micro_step will handle it.
        if let Err(e) = &result {
            // Don't log redundantly if it's just reporting an already set exception
            if self.exception.is_none() {
                tracing::error!("Micro_step failed: {}", e);
                debug_assert!(false, "Micro_step failed: {}", e);
            }
        }
    }

    /// **Step:** Execute one full instruction cycle (multiple micro-steps).
    pub fn step(&mut self) {
        let input_running = self.running;

        self.running = true;

        // Execute micro-steps until we return to the Fetch state, completing one instruction.
        self.micro_step(); // (potentially) Fetch
        while !matches!(self.cpu_state, CpuState::Fetch) && self.running {
            // Continue micro-stepping until Fetch is reached or an exception occurs
            self.micro_step();
        }

        // Check if somehow not running anymore (e.g. HALT)
        if !self.running {
            self.running = false;
            return;
        }

        debug_assert!(matches!(self.cpu_state, CpuState::Fetch), "invalid step");
        self.running = input_running;
    }

    /// **Run:** Execute instructions until HALT, error, input wait, or max_steps.
    pub fn run(&mut self, max_steps: Option<usize>) -> Result<(), String> {
        self.running = true;
        let mut steps = 0;

        loop {
            if !self.running {
                tracing::info!("Execution halted.");
                return Ok(());
            }

            if let Some(max) = max_steps {
                if steps >= max {
                    tracing::info!("Reached maximum steps ({}), stopping execution.", max);
                    self.running = false; // Stop running
                    return Ok(());
                }
            }

            // Execute one full instruction step
            self.step();

            // Step completed successfully (or halted, or paused for input, or exception pending)
            // Check running state again in case step caused HALT
            if !self.running {
                tracing::info!("Execution halted by instruction.");
                return Ok(());
            }
            // Check for pending exception after the step finished
            if self.exception.is_some() {
                tracing::warn!("Exception pending after step, will be handled on next cycle.");
                // Continue loop, exception handler runs at start of next micro_step
            }

            if max_steps.is_some() {
                steps += 1;
            }
        }
    }

    // Custom device ideas:
    // time
    // file-system
    // pixel display
    // For now keep it simple
    fn update_devices(&mut self) {
        self.flush_countdown = if self.flush_countdown == 0 {
            0
        } else {
            self.flush_countdown - 1
        };

        // Check if we're waiting for input
        if self.flush_countdown == 0 || self.last_pressed_key.changed_peek() {
            if self.last_pressed_key.changed() {
                self.memory[KBSR_ADDR].set(0x8000);
                // Update KBDR register with last pressed key
                self.memory[KBDR_ADDR].set(self.last_pressed_key.get());
            } else {
                // If waiting for input, clear ready bit
                self.memory[KBSR_ADDR].set(0x0000);
            }

            self.flush_countdown = STEPS_BETWEEN_FLUSH_GOT_NEW_CHAR;
        }

        // Check if a program is trying to write to display
        let dsr_value = self.memory[DSR_ADDR].get();
        if (dsr_value & 0x8000) != 0 {
            // Display is ready to receive character
            let ddr_value = self.memory[DDR_ADDR].get();
            // Check if a value has been written to DDR that hasn't been processed
            if (ddr_value & 0xFF) != 0 {
                // Extract ASCII character
                let character = (ddr_value & 0xFF) as u8;
                // Convert to character and add to output
                if let Some(c) = char::from_u32(character as u32) {
                    self.output.push(c);
                }
                // Clear DDR after processing
                self.memory[DDR_ADDR].set(0);
                // Set DSR to show we've handled the output
                self.memory[DSR_ADDR].set(dsr_value | 0x8000);
            }
        }

        // Update PSR if in supervisor mode
        let privilege_bit = match self.current_privilege_level {
            PrivilegeLevel::User => 1,
            PrivilegeLevel::Supervisor => 0,
        } << 15;
        // Set condition code bits based on flags
        let n_bit = self.n.get() << 2;
        let z_bit = self.z.get() << 1;
        let p_bit = self.p.get();
        self.memory[PSR_ADDR].set(privilege_bit | n_bit | z_bit | p_bit);

        // Check Machine Control Register (MCR)
        let mcr_value = self.memory[MCR_ADDR].get();
        // If bit 15 (clock enable) is cleared, stop execution
        if (mcr_value & 0x8000) == 0 {
            self.running = false;
        }
    }
}

trait BitAddressable {
    fn index(&self, addr: u8) -> Self;
    fn range(&self, slice: Range<u8>) -> Self;
}

impl BitAddressable for EmulatorCell {
    fn index(&self, addr: u8) -> Self {
        Self((self.0 >> addr) & 1, true)
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
        Self((self.0 & mask) >> end, true)
    }
}
