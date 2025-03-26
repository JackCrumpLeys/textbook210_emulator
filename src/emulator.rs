use std::collections::{HashMap, HashSet};
use std::ops::Range;

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
    pub memory: Vec<EmulatorCell>, // MUST be initialized with 65536 EmulatorCells
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
}

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
    fn from_value(value: u16) -> Option<&'static OpCode> {
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

    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
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

    fn execute(&self, machine_state: &mut Emulator) {
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
            Err("Unknown opcode")
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

// parsing code
impl Emulator {
    /// Parse LC-3 assembly code into machine instructions
    pub fn parse_program(
        program: &str,
    ) -> Result<(Vec<(usize, u16)>, HashMap<String, u16>, u16), (String, usize)> {
        let span = tracing::info_span!("parse_program", program_length = program.len());
        let _guard = span.enter();

        tracing::info!("Starting to parse program");
        let program = program.to_string();

        let mut instructions = vec![];
        let mut labels = HashMap::new();
        let mut orig_address: u16 = 0x3000; // Default starting address for LC-3 programs
        let mut address: u16 = 0x3000;
        let mut orig_set = false;
        let mut non_colon_labels = HashSet::new();

        let mut debug_first_pass_addr = HashMap::new();
        let mut debug_second_pass_addr = HashMap::new();

        // First pass: collect labels and directives
        tracing::debug!("Starting first pass: collecting labels and directives");
        for (i, line) in program.lines().enumerate() {
            let span = tracing::trace_span!("parse_addr_pass1", line = line, address = address);
            let _guard = span.enter();

            let line = line.trim();
            let line_uncapped = line;
            let line = line_uncapped.to_ascii_uppercase();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') {
                tracing::trace!("Line {}: Skipping empty line or comment", i);
                continue;
            }

            // Remove comments from the line
            let line = line.split(';').next().unwrap().trim();

            // Skip if line is still empty after comment removal
            if line.is_empty() {
                tracing::trace!("Line {}: Skipping empty line after comment removal", i);
                continue;
            }

            tracing::trace!("Line {}: Processing '{}'", i, line);

            // Process directives and labels
            // Helper function to get the memory size of a directive
            fn get_directive_size(
                line: &str,
                line_uncapped: &str,
                i: usize,
            ) -> Result<u16, (String, usize)> {
                if line.starts_with(".ORIG") || line.starts_with(".END") {
                    // These don't add to memory size
                    Ok(0)
                } else if line.starts_with(".FILL") {
                    // .FILL takes 1 memory location
                    tracing::trace!("Line {}: Processing .FILL directive", i);
                    Ok(1)
                } else if line.starts_with(".BLKW") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 2 {
                        tracing::error!("Line {}: Invalid .BLKW directive", i);
                        return Err(("Invalid .BLKW directive".to_string(), i));
                    }

                    let count_str = parts[1].trim();
                    tracing::trace!("Line {}: Processing .BLKW with count '{}'", i, count_str);

                    let count = match count_str.parse::<u16>() {
                        Ok(count) => {
                            tracing::debug!("Line {}: Reserving {} memory locations", i, count);
                            count
                        }
                        Err(e) => {
                            tracing::error!(
                                "Line {}: Invalid block size '{}': {}",
                                i,
                                count_str,
                                e
                            );
                            return Err(("Invalid block size".to_string(), i));
                        }
                    };

                    Ok(count)
                } else if line.starts_with(".STRINGZ") {
                    // Find the string between quotes
                    tracing::trace!("Line {}: Processing .STRINGZ directive", i);
                    if let Some(string_content) = line_uncapped.find('"').and_then(|start| {
                        line_uncapped[start + 1..]
                            .find('"')
                            .map(|end| &line_uncapped[start + 1..start + 1 + end])
                    }) {
                        // Count special escape sequences that only take up one character in memory
                        let mut escape_sequences = 0;
                        for i in 0..string_content.len() {
                            if i < string_content.len() - 1 && &string_content[i..i + 2] == "\\n"
                                || i < string_content.len() - 1
                                    && &string_content[i..i + 2] == "\\t"
                                || i < string_content.len() - 1
                                    && &string_content[i..i + 2] == "\\r"
                                || i < string_content.len() - 1
                                    && &string_content[i..i + 2] == "\\0"
                            {
                                escape_sequences += 1;
                            }
                        }
                        // Adjust string length to account for escape sequences
                        let string_len = string_content.len() - escape_sequences;
                        tracing::debug!(
                            "Line {}: String of length {} found: '{}'",
                            i,
                            string_len,
                            string_content
                        );
                        // +1 for null terminator
                        Ok((string_len + 1) as u16)
                    } else {
                        tracing::error!(
                            "Line {}: Invalid .STRINGZ directive, no quoted string found",
                            i
                        );
                        Err(("Invalid .STRINGZ directive".to_string(), i))
                    }
                } else {
                    // Regular instruction
                    tracing::trace!("Line {}: Regular instruction", i);
                    Ok(1)
                }
            }

            if line.contains(':') {
                // Label with colon format: LABEL: instruction
                let parts: Vec<&str> = line.split(':').collect();
                let label = parts[0].trim().to_string();

                // Add label to map
                tracing::debug!(
                    "Line {}: Found label '{}' (with colon) at address {:04X}",
                    i,
                    label,
                    address
                );
                labels.insert(label, address);

                // If there's content after the label, process it
                if parts.len() > 1 && !parts[1].trim().is_empty() {
                    let after_label = parts[1].trim();
                    tracing::trace!("Line {}: Label has content after it", i);

                    if after_label.starts_with(".") {
                        // It's a directive, calculate its size
                        match get_directive_size(after_label, line_uncapped, i) {
                            Ok(size) => address += size,
                            Err(e) => return Err(e),
                        }
                    } else {
                        // Regular instruction
                        address += 1; // Each instruction takes 1 memory location
                    }
                }
            } else if line.starts_with(".ORIG") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    tracing::error!("Line {}: Invalid .ORIG directive", i);
                    return Err(("Invalid .ORIG directive".to_string(), i));
                }

                // Parse origin address (supports hex with x prefix)
                let addr_str = parts[1].trim();
                tracing::trace!("Line {}: Processing .ORIG with address '{}'", i, addr_str);

                if addr_str.starts_with("x") || addr_str.starts_with("X") {
                    match u16::from_str_radix(&addr_str[1..], 16) {
                        Ok(addr) => {
                            orig_address = addr;
                            tracing::debug!(
                                "Line {}: Set origin address to 0x{:04X}",
                                i,
                                orig_address
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "Line {}: Invalid hex address '{}': {}",
                                i,
                                addr_str,
                                e
                            );
                            return Err(("Invalid hex address".to_string(), i));
                        }
                    }
                } else {
                    match addr_str.parse::<u16>() {
                        Ok(addr) => {
                            orig_address = addr;
                            tracing::debug!("Line {}: Set origin address to {}", i, orig_address);
                        }
                        Err(e) => {
                            tracing::error!(
                                "Line {}: Invalid decimal address '{}': {}",
                                i,
                                addr_str,
                                e
                            );
                            return Err(("Invalid address".to_string(), i));
                        }
                    }
                }

                address = orig_address;
                orig_set = true;
            } else if line.starts_with(".") {
                // Handle directives using the helper function
                match get_directive_size(&line, line_uncapped, i) {
                    Ok(size) => address += size,
                    Err(e) => return Err(e),
                }
            } else {
                // Check if this line might be a label without a colon
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 1
                    && !parts[0].starts_with('.')
                    && !parts[0].starts_with('R')
                    && ![
                        "ADD", "AND", "BR", "BRN", "BRZ", "BRP", "BRNZ", "BRNP", "BRZP", "BRNZP",
                        "JMP", "JSR", "JSRR", "LD", "LDI", "LDR", "LEA", "NOT", "RET", "RTI", "ST",
                        "STI", "STR", "TRAP", "GETC", "OUT", "PUTS", "IN", "PUTSP", "HALT",
                    ]
                    .contains(&parts[0])
                {
                    // This looks like a label without a colon: LABEL instruction
                    let label = parts[0].trim().to_string();
                    tracing::debug!(
                        "Line {}: Found label '{}' (without colon) at address {:04X}",
                        i,
                        label,
                        address
                    );
                    labels.insert(label, address);
                    non_colon_labels.insert(i);

                    if parts.len() >= 2 {
                        // Check if there's a directive after the label
                        let after_label = line.strip_prefix(parts[0]).unwrap_or_default().trim();
                        if after_label.starts_with(".") {
                            // It's a directive, calculate its size
                            match get_directive_size(after_label, line_uncapped, i) {
                                Ok(size) => address += size,
                                Err(e) => return Err(e),
                            }
                        } else {
                            // Regular instruction
                            address += 1;
                        }
                    }
                } else {
                    // Regular instruction
                    tracing::trace!("Line {}: Regular instruction", i);
                    address += 1;
                }
            }

            debug_first_pass_addr.insert(address, line_uncapped.clone());
        }

        if !orig_set {
            tracing::error!("No .ORIG directive found in program");
            return Err(("No .ORIG directive found".to_string(), 0));
        }

        // Reset address for second pass
        address = orig_address;
        tracing::debug!("First pass completed, {} labels found", labels.len());
        tracing::debug!("Starting second pass with address at {:04X}", address);
        // Second pass: generate instructions
        tracing::debug!("Starting second pass: generating instructions");
        for (i, line) in program.lines().enumerate() {
            let line = line.trim();
            let line_uncapped = line;
            let line = line_uncapped.to_ascii_uppercase();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Remove comments from the line
            let line = line.split(';').next().unwrap().trim();

            // Skip if line is still empty after comment removal
            if line.is_empty() {
                continue;
            }

            tracing::trace!("Line {}: Processing '{}'", i, line);

            // Helper function to process directives
            fn process_directive(
                line: &str,
                line_uncapped: &str,
                i: usize,
                address: &mut u16,
                instructions: &mut Vec<(usize, u16)>,
                labels: &HashMap<String, u16>,
            ) -> Result<bool, (String, usize)> {
                if line.starts_with(".ORIG") {
                    // Already processed in first pass, just update address
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let addr_str = parts[1].trim();

                    if addr_str.starts_with("x") || addr_str.starts_with("X") {
                        match u16::from_str_radix(&addr_str[1..], 16) {
                            Ok(addr) => {
                                *address = addr;
                                tracing::debug!("Line {}: Updated address to 0x{:04X}", i, address);
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Line {}: Invalid hex address '{}': {}",
                                    i,
                                    addr_str,
                                    e
                                );
                                return Err(("Invalid hex address".to_string(), i));
                            }
                        }
                    } else {
                        match addr_str.parse::<u16>() {
                            Ok(addr) => {
                                *address = addr;
                                tracing::debug!("Line {}: Updated address to {}", i, address);
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Line {}: Invalid decimal address '{}': {}",
                                    i,
                                    addr_str,
                                    e
                                );
                                return Err(("Invalid address".to_string(), i));
                            }
                        }
                    }
                    return Ok(true);
                } else if line.starts_with(".FILL") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let value_str = parts[1].trim();
                    tracing::trace!("Line {}: Processing .FILL with value '{}'", i, value_str);

                    let value: u16;

                    if let Ok(imm) = Emulator::parse_immediate(value_str, 16) {
                        value = imm;
                    } else if labels.contains_key(value_str) {
                        value = *labels.get(value_str).unwrap();
                        tracing::debug!(
                            "Line {}: Using label '{}' value: {:04X}",
                            i,
                            value_str,
                            value
                        );
                    } else {
                        tracing::error!("Line {}: Invalid .FILL value '{}'", i, value_str);
                        return Err((
                            "Invalid .FILL value, please provide a valid immediate value or label"
                                .to_string(),
                            i,
                        ));
                    }

                    instructions.push((i, value));
                    *address += 1;
                    return Ok(true);
                } else if line.starts_with(".BLKW") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let count_str = parts[1].trim();

                    let count = match count_str.parse::<u16>() {
                        Ok(count) => count,
                        Err(e) => {
                            tracing::error!(
                                "Line {}: Invalid block size '{}': {}",
                                i,
                                count_str,
                                e
                            );
                            return Err(("Invalid block size".to_string(), i));
                        }
                    };

                    tracing::debug!(
                        "Line {}: Adding {} zero words at address {:04X}",
                        i,
                        count,
                        address
                    );
                    // Fill with zeros
                    for _ in 0..count {
                        instructions.push((i, 0));
                        *address += 1;
                    }
                    return Ok(true);
                } else if line.starts_with(".STRINGZ") {
                    // Find the string between quotes
                    if let Some(string_content) = line_uncapped.find('"').and_then(|start| {
                        line_uncapped[start + 1..]
                            .find('"')
                            .map(|end| &line_uncapped[start + 1..start + 1 + end])
                    }) {
                        tracing::debug!(
                            "Line {}: Converting string '{}' to ASCII values",
                            i,
                            string_content
                        );
                        // Convert string to ASCII values
                        let mut chars_iter = string_content.chars().peekable();
                        while let Some(c) = chars_iter.next() {
                            // Handle escape sequences
                            if c == '\\' {
                                if let Some(next_char) = chars_iter.next() {
                                    match next_char {
                                        'n' => {
                                            tracing::trace!(
                                                "Line {}: Adding escape character '\\n' (ASCII: 10) at address {:04X}",
                                                i,
                                                address
                                            );
                                            instructions.push((i, 10)); // ASCII newline
                                        }
                                        't' => {
                                            tracing::trace!(
                                                "Line {}: Adding escape character '\\t' (ASCII: 9) at address {:04X}",
                                                i,
                                                address
                                            );
                                            instructions.push((i, 9)); // ASCII tab
                                        }
                                        'r' => {
                                            tracing::trace!(
                                                "Line {}: Adding escape character '\\r' (ASCII: 13) at address {:04X}",
                                                i,
                                                address
                                            );
                                            instructions.push((i, 13)); // ASCII carriage return
                                        }
                                        '0' => {
                                            tracing::trace!(
                                                "Line {}: Adding escape character '\\0' (ASCII: 0) at address {:04X}",
                                                i,
                                                address
                                            );
                                            instructions.push((i, 0)); // ASCII null
                                        }
                                        '\\' => {
                                            tracing::trace!(
                                                "Line {}: Adding escape character '\\\\' (ASCII: 92) at address {:04X}",
                                                i,
                                                address
                                            );
                                            instructions.push((i, 92)); // Backslash character
                                        }
                                        '"' => {
                                            tracing::trace!(
                                                "Line {}: Adding escape character '\\\"' (ASCII: 34) at address {:04X}",
                                                i,
                                                address
                                            );
                                            instructions.push((i, 34)); // Double quote character
                                        }
                                        _ => {
                                            // Unrecognized escape, just include both characters
                                            tracing::trace!(
                                                "Line {}: Unrecognized escape sequence '\\{}', including backslash (ASCII: 92) at address {:04X}",
                                                i,
                                                next_char,
                                                address
                                            );
                                            instructions.push((i, '\\' as u16));
                                            *address += 1;
                                            tracing::trace!(
                                                "Line {}: Adding character '{}' (ASCII: {}) at address {:04X}",
                                                i,
                                                next_char,
                                                next_char as u16,
                                                address
                                            );
                                            instructions.push((i, next_char as u16));
                                        }
                                    }
                                } else {
                                    // Trailing backslash, just include it
                                    tracing::trace!(
                                        "Line {}: Adding trailing backslash (ASCII: 92) at address {:04X}",
                                        i,
                                        address
                                    );
                                    instructions.push((i, '\\' as u16));
                                }
                            } else {
                                // Regular character
                                tracing::trace!(
                                    "Line {}: Adding character '{}' (ASCII: {}) at address {:04X}",
                                    i,
                                    c,
                                    c as u16,
                                    address
                                );
                                instructions.push((i, c as u16));
                            }
                            *address += 1;
                        }
                        // Add null terminator
                        tracing::trace!(
                            "Line {}: Adding null terminator at address {:04X}",
                            i,
                            address
                        );
                        instructions.push((i, 0));
                        *address += 1;
                    } else {
                        tracing::error!(
                            "Line {}: Invalid .STRINGZ directive, no quoted string found",
                            i
                        );
                        return Err(("Invalid .STRINGZ directive".to_string(), i));
                    }
                    return Ok(true);
                } else if line.starts_with(".END") {
                    // End of program, nothing to do
                    tracing::trace!("Line {}: End of program marker (.END)", i);
                    return Ok(true);
                }
                Ok(false)
            }

            // Process directives and instructions
            if line.contains(':') {
                // Label with colon format: LABEL: instruction
                let parts: Vec<&str> = line.split(':').collect();
                let after_label = parts[1].trim();

                // If there's content after the label, process it
                if !after_label.is_empty() {
                    if after_label.starts_with(".") {
                        // Handle directives after labels
                        match process_directive(
                            after_label,
                            line_uncapped,
                            i,
                            &mut address,
                            &mut instructions,
                            &labels,
                        ) {
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        }
                    } else {
                        // Handle regular instructions after labels
                        tracing::trace!(
                            "Line {}: Processing instruction after label: '{}'",
                            i,
                            after_label
                        );
                        match Self::parse_instruction(after_label, address, &labels) {
                            Ok(instruction) => {
                                tracing::debug!("Line {}: Parsed instruction at address {:04X}: {:04X} (binary: {:016b})",
                                               i, address, instruction, instruction);
                                instructions.push((i, instruction));
                                address += 1;
                            }
                            Err(e) => {
                                tracing::error!("Line {}: Failed to parse instruction: {}", i, e.0);
                                return Err((e.0, i));
                            }
                        }
                    }
                }
            } else if non_colon_labels.contains(&i) {
                // Label without colon format: LABEL instruction
                let parts: Vec<&str> = line.split_whitespace().collect();

                // Skip the label and process the remaining instruction
                if parts.len() > 1 {
                    let instruction_part = line.strip_prefix(parts[0]).unwrap_or_default().trim();

                    if instruction_part.starts_with(".") {
                        // Handle directives after labels
                        match process_directive(
                            instruction_part,
                            line_uncapped,
                            i,
                            &mut address,
                            &mut instructions,
                            &labels,
                        ) {
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        }
                    } else {
                        // Handle regular instructions after labels
                        tracing::trace!(
                            "Line {}: Processing instruction after no-colon label: '{}'",
                            i,
                            instruction_part
                        );
                        match Self::parse_instruction(instruction_part, address, &labels) {
                            Ok(instruction) => {
                                tracing::debug!("Line {}: Parsed instruction at address {:04X}: {:04X} (binary: {:016b})",
                                               i, address, instruction, instruction);
                                instructions.push((i, instruction));
                                address += 1;
                            }
                            Err(e) => {
                                tracing::error!("Line {}: Failed to parse instruction: {}", i, e.0);
                                return Err((e.0, i));
                            }
                        }
                    }
                }
            } else if line.starts_with(".") {
                // Process directives not after labels
                match process_directive(
                    line,
                    line_uncapped,
                    i,
                    &mut address,
                    &mut instructions,
                    &labels,
                ) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            } else {
                // Regular instruction
                tracing::trace!("Line {}: Parsing regular instruction: '{}'", i, line);
                match Self::parse_instruction(line, address, &labels) {
                    Ok(instruction) => {
                        tracing::debug!("Line {}: Parsed instruction at address {:04X}: {:04X} (binary: {:016b})",
                                       i, address, instruction, instruction);
                        instructions.push((i, instruction));
                        address += 1;
                    }
                    Err(e) => {
                        tracing::error!("Line {}: Failed to parse instruction: {}", i, e.0);
                        return Err((e.0, i));
                    }
                }
            }

            debug_second_pass_addr.insert(address, line_uncapped.clone());
        }

        // Log addresses from origin to current address
        for addr in orig_address..address {
            if let Some(line) = debug_first_pass_addr.get(&addr) {
                if let Some(second_line) = debug_second_pass_addr.get(&addr) {
                    tracing::debug!(
                        "Address 0x{:04X}: First pass: '{}' | Second pass: '{}'",
                        addr,
                        line,
                        second_line
                    );
                } else {
                    tracing::debug!(
                        "Address 0x{:04X}: First pass: '{}' | No corresponding second pass line",
                        addr,
                        line
                    );
                }
            }
        }

        tracing::info!(
            "Program parsing completed: {} instructions generated",
            instructions.len()
        );
        Ok((instructions, labels, orig_address))
    }
    /// Parse a single instruction into machine code
    fn parse_instruction(
        line: &str,
        current_address: u16,
        labels: &HashMap<String, u16>,
    ) -> Result<u16, (String, usize)> {
        let span =
            tracing::debug_span!("parse_instruction", line = line, address = current_address);
        let _guard = span.enter();

        tracing::debug!("Parsing instruction: '{}'", line);

        let mut parts: Vec<&str> = Vec::new();

        if line.is_empty() {
            tracing::error!("Empty instruction");
            return Err(("Empty instruction".to_string(), 0));
        }

        parts.push(line.split_whitespace().next().unwrap());
        parts.extend(
            line.strip_prefix(parts[0])
                .unwrap_or_default()
                .split(",")
                .map(|s| s.trim())
                .collect::<Vec<&str>>(),
        );

        let opcode = parts[0];
        tracing::trace!("Opcode: '{}'", opcode);

        match opcode {
            "ADD" => {
                if parts.len() < 4 {
                    tracing::error!("Invalid ADD format: not enough arguments");
                    return Err(("Invalid ADD format".to_string(), 0));
                }

                let dr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Destination register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse destination register: {}", e.0);
                        return Err(e);
                    }
                };

                let sr1 = match Self::parse_register(parts[2]) {
                    Ok(reg) => {
                        tracing::trace!("Source register 1: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse source register 1: {}", e.0);
                        return Err(e);
                    }
                };

                // Check mode (register or immediate)
                if parts[3].starts_with("R") || parts[3].starts_with("r") {
                    // Register mode: ADD DR, SR1, SR2
                    let sr2 = match Self::parse_register(parts[3]) {
                        Ok(reg) => {
                            tracing::trace!("Source register 2: R{}", reg);
                            reg
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse source register 2: {}", e.0);
                            return Err(e);
                        }
                    };

                    let instruction = (0b0001 << 12) | (dr << 9) | (sr1 << 6) | sr2;
                    tracing::debug!(
                        "ADD (register mode): Generated instruction: {:04X}",
                        instruction
                    );
                    return Ok(instruction);
                } else {
                    // Immediate mode: ADD DR, SR1, #IMM5
                    let imm5 = match Self::parse_immediate(parts[3], 5) {
                        Ok(imm) => {
                            tracing::trace!("Immediate value (5-bit): {}", imm);
                            imm
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse immediate value: {}", e.0);
                            return Err(e);
                        }
                    };

                    let instruction =
                        (0b0001 << 12) | (dr << 9) | (sr1 << 6) | (1 << 5) | (imm5 & 0x1F);
                    tracing::debug!(
                        "ADD (immediate mode): Generated instruction: {:04X}",
                        instruction
                    );
                    return Ok(instruction);
                }
            }
            "AND" => {
                if parts.len() < 4 {
                    tracing::error!("Invalid AND format: not enough arguments");
                    return Err(("Invalid AND format".to_string(), 0));
                }

                let dr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Destination register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse destination register: {}", e.0);
                        return Err(e);
                    }
                };

                let sr1 = match Self::parse_register(parts[2]) {
                    Ok(reg) => {
                        tracing::trace!("Source register 1: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse source register 1: {}", e.0);
                        return Err(e);
                    }
                };

                // Check mode (register or immediate)
                if parts[3].starts_with("R") || parts[3].starts_with("r") {
                    // Register mode: AND DR, SR1, SR2
                    let sr2 = match Self::parse_register(parts[3]) {
                        Ok(reg) => {
                            tracing::trace!("Source register 2: R{}", reg);
                            reg
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse source register 2: {}", e.0);
                            return Err(e);
                        }
                    };

                    let instruction = (0b0101 << 12) | (dr << 9) | (sr1 << 6) | sr2;
                    tracing::debug!(
                        "AND (register mode): Generated instruction: {:04X}",
                        instruction
                    );
                    return Ok(instruction);
                } else {
                    // Immediate mode: AND DR, SR1, #IMM5
                    let imm5 = match Self::parse_immediate(parts[3], 5) {
                        Ok(imm) => {
                            tracing::trace!("Immediate value (5-bit): {}", imm);
                            imm
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse immediate value: {}", e.0);
                            return Err(e);
                        }
                    };

                    let instruction =
                        (0b0101 << 12) | (dr << 9) | (sr1 << 6) | (1 << 5) | (imm5 & 0x1F);
                    tracing::debug!(
                        "AND (immediate mode): Generated instruction: {:04X}",
                        instruction
                    );
                    return Ok(instruction);
                }
            }
            "BR" | "BRN" | "BRZ" | "BRP" | "BRNZ" | "BRNP" | "BRZP" | "BRNZP" => {
                if parts.len() < 2 {
                    tracing::error!("Invalid {} format: not enough arguments", opcode);
                    return Err((format!("Invalid {} format", opcode), 0));
                }

                let n = opcode.contains('N') as u16;
                let z = opcode.contains('Z') as u16;
                let p = opcode.contains('P') as u16;
                tracing::trace!("Branch condition codes: N={} Z={} P={}", n, z, p);

                // Get the offset (label or PCoffset9)
                let offset: i16;
                if let Some(&label_addr) = labels.get(parts[1]) {
                    tracing::trace!("Using label '{}' address: {:04X}", parts[1], label_addr);
                    offset = (label_addr as i16) - (current_address as i16 + 1);
                    tracing::trace!("Calculated offset from label: {}", offset);
                } else {
                    match Self::parse_immediate(parts[1], 9) {
                        Ok(imm) => {
                            offset = imm as i16;
                            tracing::trace!("Using explicit offset: {}", offset);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse offset: {}", e.0);
                            return Err(e);
                        }
                    }
                }

                if offset < -256 || offset > 255 {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction =
                    (0b0000 << 12) | (n << 11) | (z << 10) | (p << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("BR: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "JMP" => {
                if parts.len() < 2 {
                    tracing::error!("Invalid JMP format: not enough arguments");
                    return Err(("Invalid JMP format".to_string(), 0));
                }

                let base_r = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Base register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse base register: {}", e.0);
                        return Err(e);
                    }
                };

                let instruction = (0b1100 << 12) | (base_r << 6);
                tracing::debug!("JMP: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "JSR" => {
                if parts.len() < 2 {
                    tracing::error!("Invalid JSR format: not enough arguments");
                    return Err(("Invalid JSR format".to_string(), 0));
                }

                // Get the offset (label or PCoffset11)
                let offset: i16;
                if let Some(&label_addr) = labels.get(parts[1]) {
                    tracing::trace!("Using label '{}' address: {:04X}", parts[1], label_addr);
                    offset = (label_addr as i16) - (current_address as i16 + 1);
                    tracing::trace!("Calculated offset from label: {}", offset);
                } else {
                    match Self::parse_immediate(parts[1], 11) {
                        Ok(imm) => {
                            offset = imm as i16;
                            tracing::trace!("Using explicit offset: {}", offset);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse offset: {}", e.0);
                            return Err(e);
                        }
                    }
                }

                if offset < -1024 || offset > 1023 {
                    tracing::error!("PCoffset11 out of range: {}", offset);
                    return Err(("PCoffset11 out of range".to_string(), 0));
                }

                let instruction = (0b0100 << 12) | (1 << 11) | (offset as u16 & 0x7FF);
                tracing::debug!("JSR: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "JSRR" => {
                if parts.len() < 2 {
                    tracing::error!("Invalid JSRR format: not enough arguments");
                    return Err(("Invalid JSRR format".to_string(), 0));
                }

                let base_r = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Base register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse base register: {}", e.0);
                        return Err(e);
                    }
                };

                let instruction = (0b0100 << 12) | (base_r << 6);
                tracing::debug!("JSRR: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "LD" => {
                if parts.len() < 3 {
                    tracing::error!("Invalid LD format: not enough arguments");
                    return Err(("Invalid LD format".to_string(), 0));
                }

                let dr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Destination register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse destination register: {}", e.0);
                        return Err(e);
                    }
                };

                // Get the offset (label or PCoffset9)
                let offset: i16;
                if let Some(&label_addr) = labels.get(parts[2]) {
                    tracing::trace!("Using label '{}' address: {:04X}", parts[2], label_addr);
                    offset = (label_addr as i16) - (current_address as i16 + 1);
                    tracing::trace!("Calculated offset from label: {}", offset);
                } else {
                    match Self::parse_immediate(parts[2], 9) {
                        Ok(imm) => {
                            offset = imm as i16;
                            tracing::trace!("Using explicit offset: {}", offset);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse offset: {}", e.0);
                            return Err(e);
                        }
                    }
                }

                if offset < -256 || offset > 255 {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b0010 << 12) | (dr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("LD: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "LDI" => {
                if parts.len() < 3 {
                    tracing::error!("Invalid LDI format: not enough arguments");
                    return Err(("Invalid LDI format".to_string(), 0));
                }

                let dr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Destination register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse destination register: {}", e.0);
                        return Err(e);
                    }
                };

                // Get the offset (label or PCoffset9)
                let offset: i16;
                if let Some(&label_addr) = labels.get(parts[2]) {
                    tracing::trace!("Using label '{}' address: {:04X}", parts[2], label_addr);
                    offset = (label_addr as i16) - (current_address as i16 + 1);
                    tracing::trace!("Calculated offset from label: {}", offset);
                } else {
                    match Self::parse_immediate(parts[2], 9) {
                        Ok(imm) => {
                            offset = imm as i16;
                            tracing::trace!("Using explicit offset: {}", offset);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse offset: {}", e.0);
                            return Err(e);
                        }
                    }
                }

                if offset < -256 || offset > 255 {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b1010 << 12) | (dr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("LDI: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "LDR" => {
                if parts.len() < 4 {
                    tracing::error!("Invalid LDR format: not enough arguments");
                    return Err(("Invalid LDR format".to_string(), 0));
                }

                let dr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Destination register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse destination register: {}", e.0);
                        return Err(e);
                    }
                };

                let base_r = match Self::parse_register(parts[2]) {
                    Ok(reg) => {
                        tracing::trace!("Base register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse base register: {}", e.0);
                        return Err(e);
                    }
                };

                let offset6 = match Self::parse_immediate(parts[3], 6) {
                    Ok(imm) => {
                        tracing::trace!("Offset6: {}", imm);
                        imm
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse offset: {}", e.0);
                        return Err(e);
                    }
                };

                let instruction = (0b0110 << 12) | (dr << 9) | (base_r << 6) | (offset6 & 0x3F);
                tracing::debug!("LDR: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "LEA" => {
                if parts.len() < 3 {
                    tracing::error!("Invalid LEA format: not enough arguments");
                    return Err(("Invalid LEA format".to_string(), 0));
                }

                let dr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Destination register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse destination register: {}", e.0);
                        return Err(e);
                    }
                };

                // Get the offset (label or PCoffset9)
                let offset: i16;
                if let Some(&label_addr) = labels.get(parts[2]) {
                    tracing::trace!("Using label '{}' address: {:04X}", parts[2], label_addr);
                    offset = (label_addr as i16) - (current_address as i16 + 1);
                    tracing::trace!("Calculated offset from label: {}", offset);
                } else {
                    match Self::parse_immediate(parts[2], 9) {
                        Ok(imm) => {
                            offset = imm as i16;
                            tracing::trace!("Using explicit offset: {}", offset);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse offset: {}", e.0);
                            return Err(e);
                        }
                    }
                }

                if offset < -256 || offset > 255 {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b1110 << 12) | (dr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("LEA: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "NOT" => {
                if parts.len() < 3 {
                    tracing::error!("Invalid NOT format: not enough arguments");
                    return Err(("Invalid NOT format".to_string(), 0));
                }

                let dr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Destination register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse destination register: {}", e.0);
                        return Err(e);
                    }
                };

                let sr = match Self::parse_register(parts[2]) {
                    Ok(reg) => {
                        tracing::trace!("Source register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse source register: {}", e.0);
                        return Err(e);
                    }
                };

                let instruction = (0b1001 << 12) | (dr << 9) | (sr << 6) | 0x3F;
                tracing::debug!("NOT: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "RET" => {
                // RET is an alias for JMP R7
                tracing::debug!("RET: Alias for JMP R7");
                let instruction = (0b1100 << 12) | (7 << 6);
                tracing::debug!("RET: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "RTI" => {
                tracing::debug!("RTI: Generated instruction: {:04X}", 0b1000 << 12);
                return Ok(0b1000 << 12);
            }
            "ST" => {
                if parts.len() < 3 {
                    tracing::error!("Invalid ST format: not enough arguments");
                    return Err(("Invalid ST format".to_string(), 0));
                }

                let sr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Source register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse source register: {}", e.0);
                        return Err(e);
                    }
                };

                // Get the offset (label or PCoffset9)
                let offset: i16;
                if let Some(&label_addr) = labels.get(parts[2]) {
                    tracing::trace!("Using label '{}' address: {:04X}", parts[2], label_addr);
                    offset = (label_addr as i16) - (current_address as i16 + 1);
                    tracing::trace!("Calculated offset from label: {}", offset);
                } else {
                    match Self::parse_immediate(parts[2], 9) {
                        Ok(imm) => {
                            offset = imm as i16;
                            tracing::trace!("Using explicit offset: {}", offset);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse offset: {}", e.0);
                            return Err(e);
                        }
                    }
                }

                if offset < -256 || offset > 255 {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b0011 << 12) | (sr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("ST: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "STI" => {
                if parts.len() < 3 {
                    tracing::error!("Invalid STI format: not enough arguments");
                    return Err(("Invalid STI format".to_string(), 0));
                }

                let sr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Source register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse source register: {}", e.0);
                        return Err(e);
                    }
                };

                // Get the offset (label or PCoffset9)
                let offset: i16;
                if let Some(&label_addr) = labels.get(parts[2]) {
                    tracing::trace!("Using label '{}' address: {:04X}", parts[2], label_addr);
                    offset = (label_addr as i16) - (current_address as i16 + 1);
                    tracing::trace!("Calculated offset from label: {}", offset);
                } else {
                    match Self::parse_immediate(parts[2], 9) {
                        Ok(imm) => {
                            offset = imm as i16;
                            tracing::trace!("Using explicit offset: {}", offset);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse offset: {}", e.0);
                            return Err(e);
                        }
                    }
                }

                if offset < -256 || offset > 255 {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b1011 << 12) | (sr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("STI: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "STR" => {
                if parts.len() < 4 {
                    tracing::error!("Invalid STR format: not enough arguments");
                    return Err(("Invalid STR format".to_string(), 0));
                }

                let sr = match Self::parse_register(parts[1]) {
                    Ok(reg) => {
                        tracing::trace!("Source register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse source register: {}", e.0);
                        return Err(e);
                    }
                };

                let base_r = match Self::parse_register(parts[2]) {
                    Ok(reg) => {
                        tracing::trace!("Base register: R{}", reg);
                        reg
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse base register: {}", e.0);
                        return Err(e);
                    }
                };

                let offset6 = match Self::parse_immediate(parts[3], 6) {
                    Ok(imm) => {
                        tracing::trace!("Offset6: {}", imm);
                        imm
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse offset: {}", e.0);
                        return Err(e);
                    }
                };

                let instruction = (0b0111 << 12) | (sr << 9) | (base_r << 6) | (offset6 & 0x3F);
                tracing::debug!("STR: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "TRAP" => {
                if parts.len() < 2 {
                    tracing::error!("Invalid TRAP format: not enough arguments");
                    return Err(("Invalid TRAP format".to_string(), 0));
                }

                let trapvect8: u16;
                let value_str = parts[1].trim();
                tracing::trace!("TRAP vector: '{}'", value_str);

                if value_str.starts_with("x") || value_str.starts_with("X") {
                    match u16::from_str_radix(&value_str[1..], 16) {
                        Ok(val) => {
                            trapvect8 = val;
                            tracing::debug!("Parsed hex trap vector: 0x{:02X}", trapvect8);
                        }
                        Err(e) => {
                            tracing::error!("Invalid hex trap vector '{}': {}", value_str, e);
                            return Err(("Invalid trap vector".to_string(), 0));
                        }
                    }
                } else {
                    match value_str.parse::<u16>() {
                        Ok(val) => {
                            trapvect8 = val;
                            tracing::debug!("Parsed decimal trap vector: {}", trapvect8);
                        }
                        Err(e) => {
                            tracing::error!("Invalid decimal trap vector '{}': {}", value_str, e);
                            return Err(("Invalid trap vector".to_string(), 0));
                        }
                    }
                }

                if trapvect8 > 0xFF {
                    tracing::error!("Trap vector out of range: 0x{:X}", trapvect8);
                    return Err(("Trap vector out of range".to_string(), 0));
                }

                let instruction = (0b1111 << 12) | trapvect8;
                tracing::debug!("TRAP: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            // Trap aliases
            "GETC" => {
                tracing::debug!("GETC: Trap alias for vector 0x20");
                let instruction = (0b1111 << 12) | 0x20;
                tracing::debug!("GETC: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "OUT" => {
                tracing::debug!("OUT: Trap alias for vector 0x21");
                let instruction = (0b1111 << 12) | 0x21;
                tracing::debug!("OUT: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "PUTS" => {
                tracing::debug!("PUTS: Trap alias for vector 0x22");
                let instruction = (0b1111 << 12) | 0x22;
                tracing::debug!("PUTS: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "IN" => {
                tracing::debug!("IN: Trap alias for vector 0x23");
                let instruction = (0b1111 << 12) | 0x23;
                tracing::debug!("IN: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "PUTSP" => {
                tracing::debug!("PUTSP: Trap alias for vector 0x24");
                let instruction = (0b1111 << 12) | 0x24;
                tracing::debug!("PUTSP: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            "HALT" => {
                tracing::debug!("HALT: Trap alias for vector 0x25");
                let instruction = (0b1111 << 12) | 0x25;
                tracing::debug!("HALT: Generated instruction: {:04X}", instruction);
                return Ok(instruction);
            }
            _ => {
                tracing::error!("Unknown opcode: {}", opcode);
                Err((format!("Unknown opcode: {}", opcode), 0))
            }
        }
    }
    /// Parse a register specifier (R0-R7)
    fn parse_register(reg: &str) -> Result<u16, (String, usize)> {
        let span = tracing::debug_span!("parse_register", reg = reg);
        let _guard = span.enter();

        tracing::debug!("Parsing register: '{}'", reg);

        if reg.len() < 2 || !reg.starts_with('R') {
            tracing::error!("Invalid register format: '{}'", reg);
            return Err((format!("Invalid register: {}", reg), 0));
        }

        match reg[1..].parse::<u16>() {
            Ok(reg_num) => {
                if reg_num > 7 {
                    tracing::error!("Register number out of range: {}", reg_num);
                    return Err((format!("Register number out of range: {}", reg), 0));
                }
                tracing::debug!("Successfully parsed register R{}", reg_num);
                Ok(reg_num)
            }
            Err(e) => {
                tracing::error!("Failed to parse register number '{}': {}", &reg[1..], e);
                Err((format!("Invalid register number: {}", reg), 0))
            }
        }
    }

    /// Parse an immediate value with sign extension to the specified bit width
    fn parse_immediate(imm: &str, width: u8) -> Result<u16, (String, usize)> {
        let span = tracing::debug_span!("parse_immediate", imm = imm, width = width);
        let _guard = span.enter();

        tracing::debug!("Parsing immediate value: '{}' with width {}", imm, width);

        let value: i16;

        if imm.starts_with("#") {
            // Decimal immediate
            match imm[1..].parse::<i16>() {
                Ok(val) => {
                    value = val;
                    tracing::debug!("Parsed decimal immediate: {}", value);
                }
                Err(e) => {
                    tracing::error!("Failed to parse decimal immediate '{}': {}", imm, e);
                    return Err((format!("Invalid decimal immediate: {}", imm), 0));
                }
            }
        } else if imm.starts_with("x") || imm.starts_with("X") {
            // Hex immediate
            match i16::from_str_radix(&imm[1..], 16) {
                Ok(val) => {
                    value = val;
                    tracing::debug!("Parsed hex immediate: {:X} ({})", value, value);
                }
                Err(e) => {
                    tracing::error!("Failed to parse hex immediate '{}': {}", imm, e);
                    return Err((format!("Invalid hex immediate: {}", imm), 0));
                }
            }
        } else {
            // Try parsing as a regular number
            match imm.parse::<i16>() {
                Ok(val) => {
                    value = val;
                    tracing::debug!("Parsed numeric immediate: {}", value);
                }
                Err(e) => {
                    tracing::error!("Failed to parse immediate '{}': {}", imm, e);
                    return Err((format!("Invalid immediate: {}", imm), 0));
                }
            }
        }

        // Check if the immediate fits in the specified bit width
        let min_value = (-((1 << (width - 1)) as i32)) as i16;
        let max_value = ((1 << (width - 1)) as i32 - 1) as i16;

        if value < min_value || value > max_value {
            tracing::error!(
                "Immediate value {} out of range for {}-bit field [{}, {}]",
                value,
                width,
                min_value,
                max_value
            );
            return Err((
                format!(
                    "Immediate value out of range for {}-bit field: {}",
                    width, value
                ),
                0,
            ));
        }

        // Sign extension happens naturally when converting to u16 and masking
        tracing::debug!(
            "Immediate value {} fits in {}-bit field, masked value: {:04X}",
            value,
            width,
            (value as u16)
        );
        Ok((value as u16))
    }

    /// Flash memory with parsed program at the given origin address
    pub fn flash_memory(&mut self, cells: Vec<u16>, start_address: u16) {
        let span = tracing::info_span!(
            "flash_memory",
            cells_count = cells.len(),
            start_address = start_address
        );
        let _guard = span.enter();

        tracing::info!(
            "Flashing {} memory cells starting at address {:04X}",
            cells.len(),
            start_address
        );

        for (i, instruction) in cells.iter().enumerate() {
            let addr = (start_address as usize) + i;
            if addr >= self.memory.len() {
                tracing::error!("Address {:04X} is out of memory bounds", addr);
                break;
            }
            tracing::trace!("Setting memory[{:04X}] = {:04X}", addr, *instruction);
            self.memory[addr].set(*instruction);
        }

        self.pc = EmulatorCell(start_address);
    }
}

pub trait Op: std::fmt::Debug {
    // Prepare any memory accesses needed for the operation
    fn prepare_memory_access(&self, machine_state: &mut Emulator);

    // Execute the instruction
    fn execute(&self, machine_state: &mut Emulator);
}

trait BitAddressable {
    fn index(&self, addr: u8) -> Self;
    fn range(&self, slice: Range<u8>) -> Self;
}

impl BitAddressable for EmulatorCell {
    fn index(&self, addr: u8) -> Self {
        Self(((self.0 >> addr) & 1) as u16)
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

#[derive(Debug)]
pub struct AddOp;

impl Op for AddOp {
    // The ADD operation doesn't need extra memory access, but we implement the method
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // For ADD we don't need to set MAR since we only access registers
        tracing::trace!("ADD: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("ADD_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0001 | DR | SR1 | ImmTBit | (Registor || Immediate)
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let sr1_index = ir.range(8..6).get() as usize;

        tracing::trace!(
            dr = format!("0x{:X}", dr_index),
            sr1 = format!("0x{:X}", sr1_index),
            "Extracted register indices"
        );

        // Check immediate mode (bit[5])
        if ir.index(5).get() == 0x1 {
            // Immediate mode
            let imm5 = ir.range(4..0);
            // Sign extend from 5 bits
            let imm5_val = imm5.sext(4).get();

            tracing::trace!(
                immediate = format!("0x{:X}", imm5_val),
                "Using immediate mode"
            );
            let sr1_val = machine_state.r[sr1_index].get();
            let result = sr1_val.wrapping_add(imm5_val);
            tracing::trace!(
                sr1_value = format!("0x{:X}", sr1_val),
                imm5_value = format!("0x{:X}", imm5_val),
                result = format!("0x{:X}", result),
                "R{:X} = 0x{:X} + 0x{:X} = 0x{:X}",
                dr_index,
                sr1_val,
                imm5_val,
                result
            );

            machine_state.r[dr_index].set(result);
            tracing::trace!(
                register = format!("0x{:X}", dr_index),
                value = format!("0x{:X}", result),
                "Set register value"
            );
        } else {
            // Register mode
            let sr2_index = ir.range(2..0).get() as usize;
            tracing::trace!(sr2 = format!("0x{:X}", sr2_index), "Using register mode");

            let sr1_val = machine_state.r[sr1_index].get();
            let sr2_val = machine_state.r[sr2_index].get();
            let result = sr1_val.wrapping_add(sr2_val);
            tracing::trace!(
                sr1_value = format!("0x{:X}", sr1_val),
                sr2_value = format!("0x{:X}", sr2_val),
                result = format!("0x{:X}", result),
                "R{:X} = 0x{:X} + 0x{:X} = 0x{:X}",
                dr_index,
                sr1_val,
                sr2_val,
                result
            );

            machine_state.r[dr_index].set(result);
            tracing::trace!(
                register = format!("0x{:X}", dr_index),
                value = format!("0x{:X}", result),
                "Set register value"
            );
        }

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = format!("0x{:X}", machine_state.n.get()),
            z = format!("0x{:X}", machine_state.z.get()),
            p = format!("0x{:X}", machine_state.p.get()),
            "Updated condition flags"
        );
    }
}

#[derive(Debug)]
pub struct AndOp;

impl Op for AndOp {
    // The AND operation doesn't need extra memory access
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // For AND we don't need to set MAR since we only access registers
        tracing::trace!("AND: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("AND_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0101 | DR | SR1 | ImmTBit | (Register || Immediate)
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let sr1_index = ir.range(8..6).get() as usize;

        tracing::trace!(
            dr = format!("0x{:X}", dr_index),
            sr1 = format!("0x{:X}", sr1_index),
            "Extracted register indices"
        );

        // Check immediate mode (bit[5])
        if ir.index(5).get() == 0x1 {
            // Immediate mode
            let imm5 = ir.range(4..0);
            // Sign extend from 5 bits
            let imm5_val = imm5.sext(4).get();

            tracing::trace!(
                immediate = format!("0x{:X}", imm5_val),
                "Using immediate mode"
            );
            let sr1_val = machine_state.r[sr1_index].get();
            let result = sr1_val & imm5_val;
            tracing::trace!(
                sr1_value = format!("0x{:X}", sr1_val),
                imm5_value = format!("0x{:X}", imm5_val),
                result = format!("0x{:X}", result),
                "R{:X} = 0x{:X} & 0x{:X} = 0x{:X}",
                dr_index,
                sr1_val,
                imm5_val,
                result
            );

            machine_state.r[dr_index].set(result);
            tracing::trace!(
                register = format!("0x{:X}", dr_index),
                value = format!("0x{:X}", result),
                "Set register value"
            );
        } else {
            // Register mode
            let sr2_index = ir.range(2..0).get() as usize;
            tracing::trace!(sr2 = format!("0x{:X}", sr2_index), "Using register mode");

            let sr1_val = machine_state.r[sr1_index].get();
            let sr2_val = machine_state.r[sr2_index].get();
            let result = sr1_val & sr2_val;
            tracing::trace!(
                sr1_value = format!("0x{:X}", sr1_val),
                sr2_value = format!("0x{:X}", sr2_val),
                result = format!("0x{:X}", result),
                "R{:X} = 0x{:X} & 0x{:X} = 0x{:X}",
                dr_index,
                sr1_val,
                sr2_val,
                result
            );

            machine_state.r[dr_index].set(result);
            tracing::trace!(
                register = format!("0x{:X}", dr_index),
                value = format!("0x{:X}", result),
                "Set register value"
            );
        }

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = format!("0x{:X}", machine_state.n.get()),
            z = format!("0x{:X}", machine_state.z.get()),
            p = format!("0x{:X}", machine_state.p.get()),
            "Updated condition flags"
        );
    }
}

#[derive(Debug)]
pub struct BrOp;

impl Op for BrOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // Branch doesn't need extra memory access preparation
        tracing::trace!("BR: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("BR_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0000 | N | Z | P | PCoffset9
        let ir = machine_state.ir;

        // Extract NZP bits and PCoffset9
        let n_bit = ir.index(11).get();
        let z_bit = ir.index(10).get();
        let p_bit = ir.index(9).get();
        tracing::trace!(
            n = format!("0x{:X}", n_bit),
            z = format!("0x{:X}", z_bit),
            p = format!("0x{:X}", p_bit),
            "Branch condition codes"
        );

        // Extract and sign-extend PCoffset9
        let pc_offset = ir.range(8..0).sext(8).get();
        tracing::trace!(
            offset = format!("0x{:X}", pc_offset),
            "PC offset for branch"
        );

        // Check if condition codes match current state
        let n_match = n_bit == 0x1 && machine_state.n.get() == 0x1;
        let z_match = z_bit == 0x1 && machine_state.z.get() == 0x1;
        let p_match = p_bit == 0x1 && machine_state.p.get() == 0x1;

        tracing::trace!(
            current_n = format!("0x{:X}", machine_state.n.get()),
            current_z = format!("0x{:X}", machine_state.z.get()),
            current_p = format!("0x{:X}", machine_state.p.get()),
            "Current machine condition flags"
        );

        tracing::trace!(
            n_match = n_match,
            z_match = z_match,
            p_match = p_match,
            "Condition code matching results"
        );

        // If any condition matches, branch to the target address
        if n_match || z_match || p_match {
            // PC has already been incremented in fetch, so we add the offset directly
            let old_pc = machine_state.pc.get();
            let new_pc = old_pc.wrapping_add(pc_offset);
            tracing::trace!(
                old_pc = format!("0x{:X}", old_pc),
                new_pc = format!("0x{:X}", new_pc),
                "Taking branch"
            );
            machine_state.pc.set(new_pc);
        } else {
            tracing::trace!("No condition match, not taking branch");
        }
        // If no condition matches, execution continues normally
    }
}

#[derive(Debug)]
pub struct JmpOp;

impl Op for JmpOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // JMP doesn't need extra memory access preparation
        tracing::trace!("JMP: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("JMP_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1100 | 000 | BaseR | 000000
        let ir = machine_state.ir;

        // Extract base register index
        let base_r_index = ir.range(8..6).get() as usize;
        tracing::trace!(
            base_register = format!("0x{:X}", base_r_index),
            "Using base register for jump"
        );

        // Set PC to the value in the base register
        let old_pc = machine_state.pc.get();
        let new_pc = machine_state.r[base_r_index].get();
        tracing::trace!(
            old_pc = format!("0x{:X}", old_pc),
            new_pc = format!("0x{:X}", new_pc),
            from_register = format!("0x{:X}", base_r_index),
            "Jumping to address in register"
        );
        machine_state.pc.set(new_pc);

        // Note: RET is a special case of JMP where BaseR is R7
        if base_r_index == 0x7 {
            tracing::trace!("This is a RET instruction (JMP R7)");
        }
    }
}

#[derive(Debug)]
pub struct JsrOp;

impl Op for JsrOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // JSR doesn't need extra memory access preparation
        tracing::trace!("JSR: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("JSR_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0100 | ToggleBit | rest of bits (PCoffset11 or BaseR)
        let ir = machine_state.ir;
        let curr_pc = machine_state.pc.get();

        // Save return address in R7
        tracing::trace!(
            return_address = format!("0x{:X}", curr_pc),
            "Saving return address in R7"
        );
        machine_state.r[0x7].set(curr_pc);

        // Check if JSR or JSRR
        if ir.index(11).get() == 0x1 {
            // JSR: Use PC-relative addressing
            // Extract and sign-extend PCoffset11
            let pc_offset = ir.range(10..0).sext(10).get();
            tracing::trace!(
                mode = "JSR",
                offset = format!("0x{:X}", pc_offset),
                "PC-relative subroutine jump"
            );

            // PC has already been incremented in fetch, so add the offset directly
            let new_pc = curr_pc.wrapping_add(pc_offset);
            tracing::trace!(
                old_pc = format!("0x{:X}", curr_pc),
                new_pc = format!("0x{:X}", new_pc),
                "Jumping to subroutine"
            );
            machine_state.pc.set(new_pc);
        } else {
            // JSRR: Get address from base register
            let base_r_index = ir.range(8..6).get() as usize;
            tracing::trace!(
                mode = "JSRR",
                base_register = format!("0x{:X}", base_r_index),
                "Register-based subroutine jump"
            );

            let new_pc = machine_state.r[base_r_index].get();
            tracing::trace!(
                old_pc = format!("0x{:X}", curr_pc),
                new_pc = format!("0x{:X}", new_pc),
                from_register = format!("0x{:X}", base_r_index),
                "Jumping to subroutine at address in register"
            );
            machine_state.pc.set(new_pc);
        }
    }
}

#[derive(Debug)]
pub struct LdOp;

impl Op for LdOp {
    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LD_prepare_memory",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0010 | DR | PCoffset9
        let ir = machine_state.ir;

        // Calculate effective address
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let effective_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:X}", curr_pc),
            offset = format!("0x{:X}", pc_offset),
            effective_address = format!("0x{:X}", effective_address),
            "Calculating effective address for load"
        );

        // Set MAR to the effective address
        machine_state.mar.set(effective_address);
        tracing::trace!(
            mar = format!("0x{:X}", effective_address),
            "Setting MAR for memory access"
        );
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LD_execute",
            ir = ?machine_state.ir.get(),
            mar = machine_state.mar.get(),
            mdr = machine_state.mdr.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0010 | DR | PCoffset9
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let effective_address = machine_state.mar.get();

        // MDR was loaded during memory access phase
        let value = machine_state.mdr.get();
        tracing::trace!(
            address = format!("0x{:X}", effective_address),
            value = format!("0x{:X}", value),
            dest_register = format!("0x{:X}", dr_index),
            "Loading value from memory into register"
        );
        machine_state.r[dr_index].set(value);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = format!("0x{:X}", machine_state.n.get()),
            z = format!("0x{:X}", machine_state.z.get()),
            p = format!("0x{:X}", machine_state.p.get()),
            "Updated condition flags after load"
        );
    }
}
#[derive(Debug)]
pub struct LdiOp;

impl Op for LdiOp {
    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LDI_prepare_memory",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1010 | DR | PCoffset9
        let ir = machine_state.ir;

        // Calculate address of pointer
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let pointer_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:04X}", curr_pc),
            offset = format!("0x{:04X}", pc_offset),
            pointer_address = format!("0x{:04X}", pointer_address),
            "Calculating pointer address for indirect load"
        );

        // Set MAR to the pointer address
        machine_state.mar.set(pointer_address);
        tracing::trace!(
            mar = format!("0x{:04X}", pointer_address),
            "Setting MAR to pointer address"
        );

        // Note: The actual memory read (MAR -> MDR) will happen after this function completes
        // The memory system will load the pointer value from machine_state.memory[MAR] into MDR
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LDI_execute",
            ir = ?machine_state.ir.get(),
            mar = machine_state.mar.get(),
            mdr = machine_state.mdr.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1010 | DR | PCoffset9
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;

        // Get the indirect address from MDR and set MAR to it
        let pointer_value = machine_state.mdr.get();
        machine_state.mar.set(pointer_value);
        tracing::trace!(
            pointer_value = format!("0x{:04X}", pointer_value),
            "Setting MAR to indirect address for final load"
        );

        machine_state
            .mdr
            .set(machine_state.memory[machine_state.mar.get() as usize].get());

        let value = machine_state.mdr.get();
        tracing::trace!(
            indirect_address = format!("0x{:04X}", machine_state.mar.get()),
            value = format!("0x{:04X}", value),
            dest_register = dr_index,
            "Loading value from indirect address into register"
        );
        machine_state.r[dr_index].set(value);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Updated condition flags after indirect load"
        );
    }
}

#[derive(Debug)]
pub struct LdrOp;

impl Op for LdrOp {
    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LDR_prepare_memory",
            ir = ?machine_state.ir.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0110 | DR | BaseR | offset6
        let ir = machine_state.ir;

        // Get base register index
        let base_r_index = ir.range(8..6).get() as usize;
        let base_r_value = machine_state.r[base_r_index].get();

        // Calculate effective address: BaseR + offset6
        let offset = ir.range(5..0).sext(5).get();
        let effective_address = base_r_value.wrapping_add(offset);

        tracing::trace!(
            base_register = base_r_index,
            base_value = format!("0x{:04X}", base_r_value),
            offset = format!("0x{:04X}", offset),
            effective_address = format!("0x{:04X}", effective_address),
            "Calculating effective address for register-based load"
        );

        // Set MAR to the effective address
        machine_state.mar.set(effective_address);
        tracing::trace!(
            mar = format!("0x{:04X}", effective_address),
            "Setting MAR for memory access"
        );
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LDR_execute",
            ir = ?machine_state.ir.get(),
            mar = machine_state.mar.get(),
            mdr = machine_state.mdr.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0110 | DR | BaseR | offset6
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let effective_address = machine_state.mar.get();

        // MDR was loaded during memory access phase
        let value = machine_state.mdr.get();
        tracing::trace!(
            address = format!("0x{:04X}", effective_address),
            value = format!("0x{:04X}", value),
            dest_register = dr_index,
            "Loading value from register-relative address into register"
        );
        machine_state.r[dr_index].set(value);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Updated condition flags after register-based load"
        );
    }
}

#[derive(Debug)]
pub struct LeaOp;

impl Op for LeaOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // LEA doesn't need extra memory access preparation
        tracing::trace!("LEA: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("LEA_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1110 | DR | PCoffset9
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;

        // Calculate effective address (PC + PCoffset9)
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let effective_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:04X}", curr_pc),
            offset = format!("0x{:04X}", pc_offset),
            effective_address = format!("0x{:04X}", effective_address),
            "Calculating effective address for load effective address"
        );

        // Load effective address into DR
        tracing::trace!(
            address = format!("0x{:04X}", effective_address),
            dest_register = dr_index,
            "Loading effective address into register"
        );
        machine_state.r[dr_index].set(effective_address);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Updated condition flags after load effective address"
        );
    }
}
#[derive(Debug)]
pub struct NotOp;

impl Op for NotOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // NOT doesn't need extra memory access preparation
        tracing::trace!("NOT: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("NOT_execute",
            ir = ?machine_state.ir.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1001 | DR | SR | 111111
        let ir = machine_state.ir;
        let dr_index = ir.range(11..9).get() as usize;
        let sr_index = ir.range(8..6).get() as usize;

        // Perform bitwise NOT operation
        let sr_value = machine_state.r[sr_index].get();
        let result = !sr_value;
        tracing::trace!(
            source_register = sr_index,
            source_value = format!("0x{:04X}", sr_value),
            result = format!("0x{:04X}", result),
            "Performing bitwise NOT operation"
        );
        machine_state.r[dr_index].set(result);

        // Update condition codes
        machine_state.update_flags(dr_index);
        tracing::trace!(
            n = machine_state.n.get(),
            z = machine_state.z.get(),
            p = machine_state.p.get(),
            "Updated condition flags after NOT"
        );
    }
}

#[derive(Debug)]
pub struct RtiOp;

impl Op for RtiOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // RTI doesn't need extra memory access preparation in this implementation
        tracing::trace!("RTI: No memory access preparation needed");
        tracing::warn!("RTI operation is not fully implemented");
    }

    fn execute(&self, _machine_state: &mut Emulator) {
        let span = tracing::trace_span!("RTI_execute");
        let _enter = span.enter();

        // Simple RTI implementation - in a full emulator this would handle returning from interrupts
        // by popping PC and PSR from the stack
        tracing::trace!("RTI: Execution attempted but not fully implemented");
        tracing::warn!("RTI operation is not fully implemented");
    }
}

#[derive(Debug)]
pub struct StOp;

impl Op for StOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // ST doesn't need to prepare memory access as we handle it in execute
        tracing::trace!("ST: Memory access preparation handled in execute phase");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("ST_execute",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0011 | SR | PCoffset9
        let ir = machine_state.ir;
        let sr_index = ir.range(11..9).get() as usize;

        // Calculate effective address
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let effective_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:04X}", curr_pc),
            offset = format!("0x{:04X}", pc_offset),
            effective_address = format!("0x{:04X}", effective_address),
            "Calculating effective address for store"
        );

        // Set MAR to the effective address and MDR to the value to store
        machine_state.mar.set(effective_address);
        let sr_value = machine_state.r[sr_index].get();
        machine_state.mdr.set(sr_value);
        tracing::trace!(
            mar = format!("0x{:04X}", effective_address),
            mdr = format!("0x{:04X}", sr_value),
            source_register = sr_index,
            "Setting memory registers for store operation"
        );

        // Store value from MDR to memory at address in MAR
        let address = machine_state.mar.get() as usize;
        tracing::trace!(
            address = format!("0x{:04X}", address),
            value = format!("0x{:04X}", sr_value),
            "Storing value in memory"
        );
        machine_state.memory[address].set(machine_state.mdr.get());
    }
}

#[derive(Debug)]
pub struct StiOp;

impl Op for StiOp {
    fn prepare_memory_access(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("STI_prepare_memory",
            ir = ?machine_state.ir.get(),
            pc = machine_state.pc.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1011 | SR | PCoffset9
        let ir = machine_state.ir;

        // Calculate address of pointer
        let pc_offset = ir.range(8..0).sext(8).get();
        let curr_pc = machine_state.pc.get();
        let pointer_address = curr_pc.wrapping_add(pc_offset);

        tracing::trace!(
            pc = format!("0x{:04X}", curr_pc),
            offset = format!("0x{:04X}", pc_offset),
            pointer_address = format!("0x{:04X}", pointer_address),
            "Calculating pointer address for indirect store"
        );

        // Set MAR to the pointer address
        machine_state.mar.set(pointer_address);
        tracing::trace!(
            mar = format!("0x{:04X}", pointer_address),
            "Setting MAR to pointer address"
        );

        // The memory access system will load MDR with the contents at MAR
        // after this function completes
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("STI_execute",
            ir = ?machine_state.ir.get(),
            mar = machine_state.mar.get(),
            mdr = machine_state.mdr.get()
        );
        let _enter = span.enter();

        // At this point, MDR contains the value from memory at the pointer address
        let ir = machine_state.ir;
        let sr_index = ir.range(11..9).get() as usize;
        let pointer_address = machine_state.mar.get();
        let indirect_address = machine_state.mdr.get();

        tracing::trace!(
            pointer_address = format!("0x{:04X}", pointer_address),
            indirect_address = format!("0x{:04X}", indirect_address),
            "Pointer value loaded from memory"
        );

        // Get the indirect address from MDR and set the MAR to it
        machine_state.mar.set(indirect_address);
        tracing::trace!(
            mar = format!("0x{:04X}", indirect_address),
            "Setting MAR to indirect address for store"
        );

        // Set MDR to the value we want to store
        let sr_value = machine_state.r[sr_index].get();
        machine_state.mdr.set(sr_value);
        tracing::trace!(
            source_register = sr_index,
            value = format!("0x{:04X}", sr_value),
            "Setting MDR to value from register"
        );

        // Store value from MDR to memory at the indirect address in MAR
        let address = machine_state.mar.get() as usize;
        tracing::trace!(
            address = format!("0x{:04X}", address),
            value = format!("0x{:04X}", sr_value),
            "Storing value in memory at indirect address"
        );
        machine_state.memory[address].set(machine_state.mdr.get());
    }
}

#[derive(Debug)]
pub struct StrOp;

impl Op for StrOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // STR doesn't need to prepare memory access
        tracing::trace!("STR: Memory access preparation handled in execute phase");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("STR_execute",
            ir = ?machine_state.ir.get()
        );
        let _enter = span.enter();

        // LAYOUT: 0111 | SR | BaseR | offset6
        let ir = machine_state.ir;
        let sr_index = ir.range(11..9).get() as usize;
        let base_r_index = ir.range(8..6).get() as usize;

        // Calculate effective address: BaseR + offset6
        let offset = ir.range(5..0).sext(5).get();
        let base_r_value = machine_state.r[base_r_index].get();
        let effective_address = base_r_value.wrapping_add(offset);

        tracing::trace!(
            base_register = base_r_index,
            base_value = format!("0x{:04X}", base_r_value),
            offset = format!("0x{:04X}", offset),
            effective_address = format!("0x{:04X}", effective_address),
            "Calculating effective address for register-relative store"
        );

        // Set MAR to the effective address and MDR to the value to store
        machine_state.mar.set(effective_address);
        let sr_value = machine_state.r[sr_index].get();
        machine_state.mdr.set(sr_value);
        tracing::trace!(
            mar = format!("0x{:04X}", effective_address),
            mdr = format!("0x{:04X}", sr_value),
            source_register = sr_index,
            "Setting memory registers for store operation"
        );

        // Store value from MDR to memory at address in MAR
        let address = machine_state.mar.get() as usize;
        tracing::trace!(
            address = format!("0x{:04X}", address),
            value = format!("0x{:04X}", sr_value),
            "Storing value in memory at register-relative address"
        );
        machine_state.memory[address].set(machine_state.mdr.get());
    }
}

#[derive(Debug)]
pub struct TrapOp;

impl Op for TrapOp {
    fn prepare_memory_access(&self, _machine_state: &mut Emulator) {
        // TRAP doesn't need extra memory access preparation in this basic implementation
        tracing::trace!("TRAP: No memory access preparation needed");
    }

    fn execute(&self, machine_state: &mut Emulator) {
        let span = tracing::trace_span!("TRAP_execute",
            ir = ?machine_state.ir.get()
        );
        let _enter = span.enter();

        // LAYOUT: 1111 | 0000 | trapvect8
        let ir = machine_state.ir;
        let trap_vector = ir.range(7..0).get();
        tracing::trace!(
            trap_vector = format!("0x{:02X}", trap_vector),
            "TRAP vector"
        );

        // Save the return address in R7
        let curr_pc = machine_state.pc.get();
        machine_state.r[7].set(curr_pc);
        tracing::trace!(
            return_address = format!("0x{:04X}", curr_pc),
            "Saving return address in R7"
        );

        // Basic implementations for common trap vectors
        match trap_vector {
            0x20 => {
                // GETC: Read a character from the keyboard
                tracing::trace!("GETC - Requesting keyboard input");
                machine_state.await_input = Some(false);
            }
            0x21 => {
                // OUT: Output a character to the console
                let char_code = machine_state.r[0].get();
                let char = char_code as u8 as char;
                tracing::trace!(
                    char_code = format!("0x{:04X}", char_code),
                    char = char.to_string(),
                    "OUT - Outputting character"
                );
                machine_state.output.push(char);
            }
            0x22 => {
                // PUTS: Output a null-terminated string starting at address in R0
                let mut string_addr = machine_state.r[0].get() as usize;
                tracing::trace!(
                    start_address = format!("0x{:04X}", string_addr),
                    "PUTS - Outputting null-terminated string"
                );

                let mut output_str = String::new();
                let mut char_count = 0;

                loop {
                    let char_value = machine_state.memory[string_addr].get();
                    if char_value == 0 {
                        break; // Null terminator found
                    }
                    let c = char_value as u8 as char;
                    output_str.push(c);
                    machine_state.output.push(c);
                    string_addr += 1;
                    char_count += 1;
                }

                tracing::trace!(
                    characters = char_count,
                    string = output_str,
                    "PUTS output string"
                );
            }
            0x23 => {
                // IN: Prompt user for input and read character
                tracing::trace!("IN - Prompting for keyboard input and waiting");
                machine_state.output.push_str("\nInput a character> ");
                machine_state.await_input = Some(true);
            }
            0x25 => {
                tracing::trace!("HALT - Halting execution");
                machine_state.running = false;
            }
            _ => {
                // For other trap vectors, in a real implementation we would
                // jump to the trap routine at the specified memory location
                tracing::trace!(
                    vector = format!("0x{:02X}", trap_vector),
                    "Unrecognized trap vector"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_add_op() {
        tracing::info_span!("test_add_op").in_scope(|| {
            tracing::info!("Starting ADD operation test");

            let mut machine_state = Emulator::new();
            machine_state.r[0].set(5);
            tracing::debug!(register = 0, value = 5, "Initialized register");
            machine_state.r[1].set(3);
            tracing::debug!(register = 1, value = 3, "Initialized register");

            // Set instruction register for ADD R0, R0, R1
            // 0001 (ADD) | 000 (DR=R0) | 000 (SR1=R0) | 0 (not immediate) | 00 | 001 (SR2=R1)
            machine_state.ir.set(0b0001_000_000_0_00_001);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                "Set instruction register"
            );

            let add_op = AddOp;
            tracing::debug!("Executing ADD operation");
            add_op.execute(&mut machine_state);
            tracing::debug!("ADD operation executed");

            tracing::debug!(result = machine_state.r[0].get(), "Final R0 value");
            tracing::debug!(
                n = machine_state.n.get(),
                z = machine_state.z.get(),
                p = machine_state.p.get(),
                "Final condition flags"
            );

            assert_eq!(machine_state.r[0].get(), 8);
            assert_eq!(machine_state.n.get(), 0);
            assert_eq!(machine_state.z.get(), 0);
            assert_eq!(machine_state.p.get(), 1);
            tracing::info!("ADD operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_and_op() {
        tracing::info_span!("test_and_op").in_scope(|| {
            tracing::info!("Starting AND operation test");

            let mut machine_state = Emulator::new();
            machine_state.r[0].set(0b1010);
            tracing::debug!(register = 0, value = 0b1010, "Initialized register");
            machine_state.r[1].set(0b1100);
            tracing::debug!(register = 1, value = 0b1100, "Initialized register");

            // Set instruction register for AND R2, R0, R1
            // 0101 (AND) | 010 (DR=R2) | 000 (SR1=R0) | 0 (not immediate) | 00 | 001 (SR2=R1)
            machine_state.ir.set(0b0101_010_000_0_00_001);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                "Set instruction register"
            );

            let and_op = AndOp;
            tracing::debug!("Executing AND operation");
            and_op.execute(&mut machine_state);
            tracing::debug!("AND operation executed");

            tracing::debug!(
                result = format!("0b{:b}", machine_state.r[2].get()),
                "Final R2 value"
            );
            tracing::debug!(
                n = machine_state.n.get(),
                z = machine_state.z.get(),
                p = machine_state.p.get(),
                "Final condition flags"
            );

            assert_eq!(machine_state.r[2].get(), 0b1000);
            assert_eq!(machine_state.n.get(), 0);
            assert_eq!(machine_state.z.get(), 0);
            assert_eq!(machine_state.p.get(), 1);
            tracing::info!("AND operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_br_op() {
        tracing::info_span!("test_br_op").in_scope(|| {
            tracing::info!("Starting BR operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            machine_state.n.set(1);
            machine_state.z.set(0);
            machine_state.p.set(0);
            tracing::debug!(n = 1, z = 0, p = 0, "Set condition codes");

            // Set instruction register for BRn #5 (branch if negative)
            // 0000 (BR) | 1 (n) | 0 (z) | 0 (p) | 000000101 (offset=5)
            machine_state.ir.set(0b0000_100_000000101);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                "Set instruction register"
            );

            let br_op = BrOp;
            tracing::debug!("Executing BR operation");
            br_op.execute(&mut machine_state);
            tracing::debug!("BR operation executed");

            tracing::debug!(
                pc = format!("0x{:04X}", machine_state.pc.get()),
                "Final program counter"
            );
            assert_eq!(machine_state.pc.get(), 0x3005);
            tracing::info!("BR operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_jmp_op() {
        tracing::info_span!("test_jmp_op").in_scope(|| {
            tracing::info!("Starting JMP operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            machine_state.r[2].set(0x4000);
            tracing::debug!(
                register = 2,
                value = format!("0x{:04X}", 0x4000),
                "Initialized register"
            );

            // Set instruction register for JMP R2
            // 1100 (JMP) | 000 | 010 (BaseR=R2) | 000000
            machine_state.ir.set(0b1100_000_010_000000);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                "Set instruction register"
            );

            let jmp_op = JmpOp;
            tracing::debug!("Executing JMP operation");
            jmp_op.execute(&mut machine_state);
            tracing::debug!("JMP operation executed");

            tracing::debug!(
                pc = format!("0x{:04X}", machine_state.pc.get()),
                "Final program counter"
            );
            assert_eq!(machine_state.pc.get(), 0x4000);
            tracing::info!("JMP operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_jsr_op() {
        tracing::info_span!("test_jsr_op").in_scope(|| {
            tracing::info!("Starting JSR operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            // Set instruction register for JSR #10
            // 0100 (JSR) | 1 (JSR mode) | 00000001010 (offset=10)
            machine_state.ir.set(0b0100_1_00000001010);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                "Set instruction register"
            );

            let jsr_op = JsrOp;
            tracing::debug!("Executing JSR operation");
            jsr_op.execute(&mut machine_state);
            tracing::debug!("JSR operation executed");

            tracing::debug!(
                pc = format!("0x{:04X}", machine_state.pc.get()),
                r7 = format!("0x{:04X}", machine_state.r[7].get()),
                "Final state after JSR"
            );

            assert_eq!(machine_state.r[7].get(), 0x3000);
            assert_eq!(machine_state.pc.get(), 0x300A);
            tracing::info!("JSR operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_ld_op() {
        tracing::info_span!("test_ld_op").in_scope(|| {
            tracing::info!("Starting LD operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            machine_state.memory[0x3005].set(0x1234);
            tracing::debug!(
                address = format!("0x{:04X}", 0x3005),
                value = format!("0x{:04X}", 0x1234),
                "Set memory value"
            );

            // Set instruction register for LD R3, #5
            // 0010 (LD) | 011 (DR=R3) | 000000101 (offset=5)
            machine_state.ir.set(0b0010_011_000000101);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                "Set instruction register"
            );

            let ld_op = LdOp;
            tracing::debug!("Preparing memory access for LD operation");
            ld_op.prepare_memory_access(&mut machine_state);
            tracing::debug!(
                mar = format!("0x{:04X}", machine_state.mar.get()),
                "MAR set for memory access"
            );

            machine_state
                .mdr
                .set(machine_state.memory[machine_state.mar.get() as usize].get());
            tracing::debug!(
                mdr = format!("0x{:04X}", machine_state.mdr.get()),
                "MDR loaded with memory value"
            );

            tracing::debug!("Executing LD operation");
            ld_op.execute(&mut machine_state);
            tracing::debug!("LD operation executed");

            tracing::debug!(
                r3 = format!("0x{:04X}", machine_state.r[3].get()),
                "Final register value"
            );
            tracing::debug!(
                n = machine_state.n.get(),
                z = machine_state.z.get(),
                p = machine_state.p.get(),
                "Final condition flags"
            );

            assert_eq!(machine_state.r[3].get(), 0x1234);
            assert_eq!(machine_state.p.get(), 1);
            tracing::info!("LD operation test completed successfully");
        });
    }
    #[traced_test]
    #[test]
    fn test_ldi_op() {
        tracing::info_span!("test_ldi_op").in_scope(|| {
            tracing::info!("Starting LDI operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            machine_state.memory[0x3005].set(0x4000);
            tracing::debug!(
                address = format!("0x{:04X}", 0x3005),
                value = format!("0x{:04X}", 0x4000),
                "Set pointer address in memory"
            );

            machine_state.memory[0x4000].set(0x5678);
            tracing::debug!(
                address = format!("0x{:04X}", 0x4000),
                value = format!("0x{:04X}", 0x5678),
                "Set target value in memory"
            );

            // Set instruction register for LDI R4, #5
            // 1010 (LDI) | 100 (DR=R4) | 000000101 (offset=5)
            machine_state.ir.set(0b1010_100_000000101);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                "Set instruction register"
            );

            let ldi_op = LdiOp;
            tracing::debug!("Preparing memory access for LDI operation");
            ldi_op.prepare_memory_access(&mut machine_state);
            tracing::debug!(
                mar = format!("0x{:04X}", machine_state.mar.get()),
                mdr = format!("0x{:04X}", machine_state.mdr.get()),
                "Memory registers for indirect addressing"
            );

            machine_state
                .mdr
                .set(machine_state.memory[machine_state.mar.get() as usize].get());
            tracing::debug!(
                address = format!("0x{:04X}", machine_state.mar.get()),
                value = format!("0x{:04X}", machine_state.mdr.get()),
                "Loaded indirect value from memory"
            );

            tracing::debug!("Executing LDI operation");
            ldi_op.execute(&mut machine_state);
            tracing::debug!("LDI operation executed");

            tracing::debug!(
                r4 = format!("0x{:04X}", machine_state.r[4].get()),
                "Final register value"
            );
            tracing::debug!(
                n = machine_state.n.get(),
                z = machine_state.z.get(),
                p = machine_state.p.get(),
                "Final condition flags"
            );

            assert_eq!(machine_state.r[4].get(), 0x5678);
            assert_eq!(machine_state.p.get(), 1);
            tracing::info!("LDI operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_ldr_op() {
        tracing::info_span!("test_ldr_op").in_scope(|| {
            tracing::info!("Starting LDR operation test");

            let mut machine_state = Emulator::new();
            machine_state.r[2].set(0x4000);
            tracing::debug!(
                register = 2,
                value = format!("0x{:04X}", 0x4000),
                "Initialized base register"
            );

            machine_state.memory[0x4003].set(0x9ABC);
            tracing::debug!(
                address = format!("0x{:04X}", 0x4003),
                value = format!("0x{:04X}", 0x9ABC),
                "Set memory value"
            );

            // Set instruction register for LDR R5, R2, #3
            // 0110 (LDR) | 101 (DR=R5) | 010 (BaseR=R2) | 000011 (offset=3)
            machine_state.ir.set(0b0110_101_010_000011);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                "Set instruction register"
            );

            let ldr_op = LdrOp;
            tracing::debug!("Preparing memory access for LDR operation");
            ldr_op.prepare_memory_access(&mut machine_state);
            tracing::debug!(
                base_register = 2,
                offset = 3,
                effective_address = format!("0x{:04X}", machine_state.mar.get()),
                "Calculated effective address for LDR"
            );

            machine_state
                .mdr
                .set(machine_state.memory[machine_state.mar.get() as usize].get());
            tracing::debug!(
                address = format!("0x{:04X}", machine_state.mar.get()),
                value = format!("0x{:04X}", machine_state.mdr.get()),
                "Loaded value from memory"
            );

            tracing::debug!("Executing LDR operation");
            ldr_op.execute(&mut machine_state);
            tracing::debug!("LDR operation executed");

            tracing::debug!(
                r5 = format!("0x{:04X}", machine_state.r[5].get()),
                "Final register value"
            );
            tracing::debug!(
                n = machine_state.n.get(),
                z = machine_state.z.get(),
                p = machine_state.p.get(),
                "Final condition flags"
            );

            assert_eq!(machine_state.r[5].get(), 0x9ABC);
            assert_eq!(machine_state.n.get(), 1); // bit 15 is set as 9 = 0b1001
            tracing::info!("LDR operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_lea_op() {
        tracing::info_span!("test_lea_op").in_scope(|| {
            tracing::info!("Starting LEA operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            // Set instruction register for LEA R6, #8
            // 1110 (LEA) | 110 (DR=R6) | 000001000 (offset=8)
            machine_state.ir.set(0b1110_110_000001000);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                dr = 6,
                offset = 8,
                "Set instruction register for LEA"
            );

            let lea_op = LeaOp;
            tracing::debug!("Executing LEA operation");
            lea_op.execute(&mut machine_state);
            tracing::debug!("LEA operation executed");

            tracing::debug!(
                r6 = format!("0x{:04X}", machine_state.r[6].get()),
                expected = format!("0x{:04X}", 0x3008),
                "Final register value"
            );
            tracing::debug!(
                n = machine_state.n.get(),
                z = machine_state.z.get(),
                p = machine_state.p.get(),
                "Final condition flags"
            );

            assert_eq!(machine_state.r[6].get(), 0x3008);
            assert_eq!(machine_state.p.get(), 1);
            tracing::info!("LEA operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_not_op() {
        tracing::info_span!("test_not_op").in_scope(|| {
            tracing::info!("Starting NOT operation test");

            let mut machine_state = Emulator::new();
            machine_state.r[1].set(0xAAAA);
            tracing::debug!(
                register = 1,
                value = format!("0x{:04X}", 0xAAAA),
                binary = format!("0b{:016b}", 0xAAAA),
                "Initialized register with pattern 10101010..."
            );

            // Set instruction register for NOT R2, R1
            // 1001 (NOT) | 010 (DR=R2) | 001 (SR=R1) | 111111
            machine_state.ir.set(0b1001_010_001_111111);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                dr = 2,
                sr = 1,
                "Set instruction register for NOT"
            );

            let not_op = NotOp;
            tracing::debug!("Executing NOT operation");
            not_op.execute(&mut machine_state);
            tracing::debug!("NOT operation executed");

            tracing::debug!(
                r2 = format!("0x{:04X}", machine_state.r[2].get()),
                binary = format!("0b{:016b}", machine_state.r[2].get()),
                "Final register value after NOT"
            );
            tracing::debug!(
                n = machine_state.n.get(),
                z = machine_state.z.get(),
                p = machine_state.p.get(),
                "Final condition flags"
            );

            assert_eq!(machine_state.r[2].get(), 0x5555);
            assert_eq!(machine_state.p.get(), 1);
            assert_eq!(machine_state.n.get(), 0);
            assert_eq!(machine_state.z.get(), 0);
            tracing::info!("NOT operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_st_op() {
        tracing::info_span!("test_st_op").in_scope(|| {
            tracing::info!("Starting ST operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            machine_state.r[3].set(0xDEAD);
            tracing::debug!(
                register = 3,
                value = format!("0x{:04X}", 0xDEAD),
                "Initialized register with value to store"
            );

            // Set instruction register for ST R3, #6
            // 0011 (ST) | 011 (SR=R3) | 000000110 (offset=6)
            machine_state.ir.set(0b0011_011_000000110);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                sr = 3,
                offset = 6,
                "Set instruction register for ST"
            );

            let st_op = StOp;
            tracing::debug!("Executing ST operation");
            st_op.execute(&mut machine_state);
            tracing::debug!("ST operation executed");

            let memory_addr = 0x3006;
            tracing::debug!(
                address = format!("0x{:04X}", memory_addr),
                value = format!("0x{:04X}", machine_state.memory[memory_addr].get()),
                "Memory value after store operation"
            );

            assert_eq!(machine_state.memory[0x3006].get(), 0xDEAD);
            tracing::info!("ST operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_sti_op() {
        tracing::info_span!("test_sti_op").in_scope(|| {
            tracing::info!("Starting STI operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            machine_state.r[4].set(0xBEEF);
            tracing::debug!(
                register = 4,
                value = format!("0x{:04X}", 0xBEEF),
                "Initialized source register"
            );

            machine_state.memory[0x3007].set(0x4000);
            tracing::debug!(
                address = format!("0x{:04X}", 0x3007),
                value = format!("0x{:04X}", 0x4000),
                "Set pointer address in memory"
            );

            // Set instruction register for STI R4, #7
            // 1011 (STI) | 100 (SR=R4) | 000000111 (offset=7)
            machine_state.ir.set(0b1011_100_000000111);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                sr = 4,
                offset = 7,
                "Set instruction register for STI"
            );

            let sti_op = StiOp;
            tracing::debug!("Preparing memory access for STI operation");
            sti_op.prepare_memory_access(&mut machine_state);
            tracing::debug!(
                mar = format!("0x{:04X}", machine_state.mar.get()),
                "MAR set to pointer address"
            );

            machine_state
                .mdr
                .set(machine_state.memory[machine_state.mar.get() as usize].get());
            tracing::debug!(
                mdr = format!("0x{:04X}", machine_state.mdr.get()),
                "MDR loaded with pointer value"
            );

            tracing::debug!("Executing STI operation");
            sti_op.execute(&mut machine_state);
            tracing::debug!("STI operation completed");

            let target_address = 0x4000;
            tracing::debug!(
                address = format!("0x{:04X}", target_address),
                value = format!("0x{:04X}", machine_state.memory[target_address].get()),
                "Final memory value after indirect store"
            );

            assert_eq!(machine_state.memory[0x4000].get(), 0xBEEF);
            tracing::info!("STI operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_str_op() {
        tracing::info_span!("test_str_op").in_scope(|| {
            tracing::info!("Starting STR operation test");

            let mut machine_state = Emulator::new();
            machine_state.r[5].set(0xCAFE);
            tracing::debug!(
                register = 5,
                value = format!("0x{:04X}", 0xCAFE),
                "Initialized source register"
            );

            machine_state.r[1].set(0x5000);
            tracing::debug!(
                register = 1,
                value = format!("0x{:04X}", 0x5000),
                "Initialized base register"
            );

            // Set instruction register for STR R5, R1, #4
            // 0111 (STR) | 101 (SR=R5) | 001 (BaseR=R1) | 000100 (offset=4)
            machine_state.ir.set(0b0111_101_001_000100);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                sr = 5,
                base_register = 1,
                offset = 4,
                "Set instruction register for STR"
            );

            let str_op = StrOp;
            tracing::debug!("Executing STR operation");
            str_op.execute(&mut machine_state);
            tracing::debug!("STR operation completed");

            let target_address = 0x5004;
            tracing::debug!(
                address = format!("0x{:04X}", target_address),
                value = format!("0x{:04X}", machine_state.memory[target_address].get()),
                expected = format!("0x{:04X}", 0xCAFE),
                "Memory value after register-relative store"
            );

            assert_eq!(machine_state.memory[0x5004].get(), 0xCAFE);
            tracing::info!("STR operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_trap_op() {
        tracing::info_span!("test_trap_op").in_scope(|| {
            tracing::info!("Starting TRAP operation test");

            let mut machine_state = Emulator::new();
            machine_state.pc.set(0x3000);
            tracing::debug!(
                pc = format!("0x{:04X}", 0x3000),
                "Initialized program counter"
            );

            machine_state.r[0].set(0x41); // ASCII 'A'
            tracing::debug!(
                register = 0,
                value = format!("0x{:04X}", 0x41),
                "Initialized R0 with ASCII character 'A'"
            );

            // Set instruction register for TRAP x21 (OUT)
            // 1111 (TRAP) | 0000 | 00100001 (trapvect=x21)
            machine_state.ir.set(0b1111_0000_00100001);
            tracing::debug!(
                ir = format!("0b{:016b}", machine_state.ir.get()),
                trap_vector = format!("0x{:02X}", 0x21),
                "Set instruction register for TRAP OUT"
            );

            let trap_op = TrapOp;
            tracing::debug!("Executing TRAP operation");
            trap_op.execute(&mut machine_state);
            tracing::debug!("TRAP operation completed");

            tracing::debug!(
                return_address = format!("0x{:04X}", machine_state.r[7].get()),
                output_size = machine_state.output.len(),
                output_value = format!(
                    "0x{:04X}",
                    machine_state.output.chars().next().unwrap() as u32
                ),
                "State after TRAP operation"
            );

            assert_eq!(machine_state.r[7].get(), 0x3000);
            assert_eq!(machine_state.output.len(), 1);
            assert_eq!(machine_state.output.chars().next(), Some('A'));
            tracing::info!("TRAP operation test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_full_program_execution() {
        tracing::info_span!("test_full_program_execution").in_scope(|| {
            tracing::info!("Starting comprehensive program execution test with all instructions");

            // Create a complex program that exercises all LC-3 instructions:
            // - Uses arithmetic operations (ADD, AND)
            // - Tests branching (BR)
            // - Uses memory operations (LD, LDI, LDR, LEA, ST, STI, STR)
            // - Uses bitwise operations (NOT)
            // - Uses subroutines (JSR, JSRR, RET)
            // - Uses trap routines (TRAP)
            let program = r#"
            .ORIG x3000

            ; Initialize registers
            AND R0, R0, #0      ; Clear R0
            AND R1, R1, #0      ; Clear R1
            AND R2, R2, #0      ; Clear R2
            AND R3, R3, #0      ; Clear R3
            AND R4, R4, #0      ; Clear R4
            AND R5, R5, #0      ; Clear R5
            AND R6, R6, #0      ; Clear R6

            ; Load indirect and set up base register
            LDI R1, POINTER     ; Load R1 indirectly from pointer
            ADD R2, R1, #10     ; Add an immediate to R1

            ; Test branching
            ADD R3, R3, #1      ; Set R3 to 1
            BRp SKIP_SECTION    ; Branch if positive

            ; This section should be skipped
            ADD R3, R3, #15     ; Would set R3 to 16 if executed

            SKIP_SECTION:
            ADD R3, R3, #2      ; R3 now equals 3

            ; Test bitwise operations
            NOT R4, R3          ; Complement R3 into R4
            AND R5, R4, R1      ; Bitwise AND R4 and R1

            ; Load effective address
            LEA R6, SUBROUTINE  ; Load address of subroutine into R6

            ; Test register indirect jump
            JSRR R6             ; Jump to subroutine at address in R6

            ; After return, do a relative jump
            JSR SECOND_SUB      ; Jump to second subroutine using PC-relative

            ; After return from both subroutines, test load/store
            LD R1, DATA_VAL      ; Load address of DATA_VAL
            ADD R0, R0, R1       ; Add DATA_VAL address to R0
            ST R0, DATA_VAL      ; Store directly
            LD R0, DATA_VAL      ; Load value directly
            STR R0, R2, #0       ; Store using register-based addressing

            ; Load register-based
            LDR R3, R2, #0      ; Load the value we just stored

            ; Test NOT and store
            NOT R3, R3          ; Complement the value
            ST R3, RESULT       ; Store directly

            ; Test indirect store
            LEA R1, RESULT_PTR  ; Load address of result pointer
            STI R0, RESULT_PTR  ; Store indirectly


            ; Jump unconditionally past the data
            BRNZP END_PROG

            ; Data section should be skipped
            DATA_VAL: .FILL x0041    ; ASCII 'A' (65)
            RESULT:   .FILL x0000    ; To store direct result
            POINTER:  .FILL PTR_VAL  ; Pointer to a value
            PTR_VAL:  .FILL x00BE    ; Value 190 to load indirectly
            RESULT_PTR: .FILL INDIRECT_RESULT ; Pointer to indirect result
            INDIRECT_RESULT: .FILL x0000 ; To store indirect result

            ; First subroutine
            SUBROUTINE:
                ADD R0, R0, #5      ; Add 5 to R0
                RET                 ; Return using R7

            ; Second subroutine
            SECOND_SUB:
                ADD R0, R0, #3      ; Add 3 to R0
                AND R5, R5, #0      ; Clear R5
                ADD R5, R5, #1      ; Set R5 to 1
                BRnzp RETURN_SUB    ; Branch always to return
                ADD R5, R5, #10     ; Shouldn't be executed

            RETURN_SUB:
                RET                 ; Return using R7

            END_PROG:
                OUT                 ; Output the character in R0
                HALT                ; Stop execution


            REN: .fill #-10

            .END
            "#;

            tracing::debug!(
                program = program,
                "Complex assembled program to test all instructions"
            );

            // Parse the program
            tracing::debug!("Parsing program");
            let parse_result = Emulator::parse_program(program);

            // Check if parsing was successful
            assert!(parse_result.is_ok(), "Program parsing should succeed");

            let (instructions, labels, orig_address) = parse_result.unwrap();

            tracing::debug!(
                instruction_count = instructions.len(),
                origin = format!("0x{:04X}", orig_address),
                "Program parsed successfully"
            );

            // Dump parsed instructions
            for (i, (_line_num, instruction)) in instructions.iter().enumerate() {
                tracing::debug!(
                    address = format!("0x{:04X}", orig_address as usize + i),
                    instruction = format!("0x{:04X}", instruction),
                    "Parsed instruction"
                );
            }

            // Dump labels
            for (name, addr) in labels.iter() {
                tracing::debug!(
                    label = name,
                    address = format!("0x{:04X}", addr),
                    "Label in program"
                );
            }

            // Create an emulator and load the program
            let mut machine_state = Emulator::new();
            tracing::debug!("Loading program into emulator");
            machine_state.flash_memory(
                instructions.into_iter().map(|(_, instr)| instr).collect(),
                orig_address,
            );

            // Verify the program was loaded correctly
            assert_eq!(
                machine_state.pc.get(),
                orig_address,
                "PC should be set to origin address"
            );

            // Execute the program with a maximum number of steps
            tracing::debug!("Beginning program execution");
            let max_steps = 100; // Prevent infinite loops
            machine_state.running = true;
            let result = machine_state.run(Some(max_steps));

            // Verify execution completed successfully
            assert!(result.is_ok(), "Program execution should succeed");

            // Verify the machine halted
            assert_eq!(machine_state.running, false, "Machine should have halted");

            // Verify the results in memory
            let direct_result_address = *labels.get("RESULT").unwrap();
            let expected_direct_result = 0xFFB6;

            // 2. Register-based store:
            let register_store_address = 200;
            let expected_register_result = 0x49;

            // 3. Indirect store: Original value (A=0x41) = 0x41
            let indirect_result_address = *labels.get("INDIRECT_RESULT").unwrap();
            let expected_indirect_result = 0x49;

            tracing::debug!(
                pc = format!("0x{:04X}", machine_state.pc.get()),
                expected_direct = format!("0x{:04X}", expected_direct_result),
                direct_addr = format!("0x{:04X}", direct_result_address),
                actual_direct = format!(
                    "0x{:04X}",
                    machine_state.memory[direct_result_address as usize].get()
                ),
                expected_register = format!("0x{:04X}", expected_register_result),
                register_addr = format!("0x{:04X}", register_store_address),
                actual_register = format!(
                    "0x{:04X}",
                    machine_state.memory[register_store_address].get()
                ),
                expected_indirect = format!("0x{:04X}", expected_indirect_result),
                indirect_addr = format!("0x{:04X}", indirect_result_address),
                actual_indirect = format!(
                    "0x{:04X}",
                    machine_state.memory[indirect_result_address as usize].get()
                ),
                "Final machine state"
            );

            // Verify direct storage result
            assert_eq!(
                machine_state.memory[direct_result_address as usize].get(),
                expected_direct_result,
                "Direct result should be stored correctly in memory"
            );

            // Verify register-based store result
            assert_eq!(
                machine_state.memory[register_store_address].get(),
                expected_register_result,
                "Register-based result should be stored correctly in memory"
            );

            // Verify indirect store result
            assert_eq!(
                machine_state.memory[indirect_result_address as usize].get(),
                expected_indirect_result,
                "Indirect result should be stored correctly in memory"
            );

            // Verify the output
            assert_eq!(
                machine_state.output.len(),
                1,
                "Program should have produced one output"
            );
            assert_eq!(
                machine_state.output.chars().next(),
                Some('I'),
                "Output should match the final value of R0"
            );

            tracing::info!("Comprehensive program execution test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_cell_index_and_range() {
        tracing::info_span!("test_cell_index_and_range").in_scope(|| {
            tracing::info!("Starting test for BitAddressable index and range");

            // Test the index function
            let test_cell = EmulatorCell(0b1010_1100_0011_0101); // 0xAC35

            tracing::debug!(
                cell_value = format!("0x{:04X}", test_cell.get()),
                binary = format!("0b{:016b}", test_cell.get()),
                "Testing index operation on test cell"
            );

            // Test individual bit extraction
            let bit15 = test_cell.index(15).get();
            let bit10 = test_cell.index(10).get();
            let bit5 = test_cell.index(5).get();
            let bit0 = test_cell.index(0).get();

            tracing::debug!(
                bit15 = bit15,
                bit10 = bit10,
                bit5 = bit5,
                bit0 = bit0,
                "Individual bit extraction results"
            );

            // Expected values from binary 1010110000110101
            assert_eq!(bit15, 1, "Bit 15 should be 1");
            assert_eq!(bit10, 1, "Bit 10 should be 1");
            assert_eq!(bit5, 1, "Bit 5 should be 1");
            assert_eq!(bit0, 1, "Bit 0 should be 1");

            // Test the range function
            tracing::debug!(
                cell_value = format!("0x{:04X}", test_cell.get()),
                "Testing range operation on test cell"
            );

            // Extract different ranges
            let range_15_12 = test_cell.range(15..12).get(); // 1010
            let range_11_8 = test_cell.range(11..8).get(); // 1100
            let range_7_4 = test_cell.range(7..4).get(); // 0011
            let range_3_0 = test_cell.range(3..0).get(); // 0101

            tracing::debug!(
                range_15_12 = format!("0x{:X}", range_15_12),
                range_11_8 = format!("0x{:X}", range_11_8),
                range_7_4 = format!("0x{:X}", range_7_4),
                range_3_0 = format!("0x{:X}", range_3_0),
                "Range extraction results"
            );

            assert_eq!(range_15_12, 0xA, "Range 15..12 should be 0xA");
            assert_eq!(range_11_8, 0xC, "Range 11..8 should be 0xC");
            assert_eq!(range_7_4, 0x3, "Range 7..4 should be 0x3");
            assert_eq!(range_3_0, 0x5, "Range 3..0 should be 0x5");

            // Test sign extension with range and sext
            let negative_value = test_cell.range(15..12).sext(3).get(); // Sign-extend 1010
            let positive_value = test_cell.range(3..0).sext(3).get(); // Sign-extend 0101

            tracing::debug!(
                negative_range = format!("0b{:04b}", test_cell.range(15..12).get()),
                negative_extended = format!("0x{:04X}", negative_value),
                positive_range = format!("0b{:04b}", test_cell.range(3..0).get()),
                positive_extended = format!("0x{:04X}", positive_value),
                "Sign extension results"
            );

            // 1010 sign-extended from bit 3 should have 1s in upper bits
            assert_eq!(
                negative_value, 0xFFFA,
                "Negative value sign-extended from bit 3 should be 0xFFFA"
            );
            // 0101 sign-extended from bit 3 should keep 0s in upper bits
            assert_eq!(
                positive_value, 0x0005,
                "Positive value sign-extended from bit 3 should be 0x0005"
            );

            tracing::info!("BitAddressable index and range test completed successfully");
        });
    }

    #[traced_test]
    #[test]
    fn test_c_println_assembly() {
        tracing::info_span!("test_c_println_assembly").in_scope(|| {
            tracing::info!("Starting test for C-generated assembly code");

            // Load assembly file
            let assembly_content = include_str!("../c-println.asm");

            tracing::debug!(
                assembly_size = assembly_content.len(),
                "Loaded assembly file from c-println.asm"
            );

            // Parse the program
            tracing::debug!("Parsing C-generated assembly program");
            let parse_result = Emulator::parse_program(assembly_content);

            // Check if parsing was successful
            assert!(parse_result.is_ok(), "Assembly parsing should succeed");

            let (instructions, labels, orig_address) = parse_result.unwrap();

            tracing::debug!(
                instruction_count = instructions.len(),
                label_count = labels.len(),
                origin = format!("0x{:04X}", orig_address),
                "Assembly parsed successfully"
            );

            // Create an emulator and load the program
            let mut machine_state = Emulator::new();
            tracing::debug!("Loading assembly program into emulator");
            machine_state.flash_memory(
                instructions.into_iter().map(|(_, instr)| instr).collect(),
                orig_address,
            );

            // Execute the program with a maximum number of steps
            tracing::debug!("Beginning C-generated assembly program execution");
            let max_steps = 30; // Prevent infinite loops
            machine_state.running = true;
            let result = machine_state.run(Some(max_steps));

            // Verify execution completed successfully
            assert!(result.is_ok(), "Assembly execution should succeed");

            // Verify the machine halted
            assert_eq!(machine_state.running, false, "Machine should have halted");

            // Log the output
            tracing::debug!(output = machine_state.output, "Assembly program output");

            // Verify some output was produced
            assert!(
                !machine_state.output.is_empty(),
                "Assembly program should produce output"
            );

            tracing::info!("C-generated assembly test completed successfully");
        });
    }
}
