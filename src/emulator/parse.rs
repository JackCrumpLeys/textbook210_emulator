/// This is easily the most botch file in this codebase. We just go line by line
/// and assemble based on string matching. TODO: Make a tokenizer
use std::collections::{HashMap, HashSet};

use super::Emulator;

/// Output structure for the `parse_program` function.
#[derive(Debug)]
pub struct ParseOutput {
    /// Vector containing the generated machine code instructions/data.
    pub machine_code: Vec<u16>,
    /// Map from source code line number (0-based) to the memory address
    /// where the corresponding instruction or data starts.
    pub line_to_address: HashMap<usize, usize>,
    /// Map from label names to their corresponding memory addresses.
    pub labels: HashMap<String, u16>,
    /// The starting memory address specified by the .ORIG directive.
    pub orig_address: u16,
}
// is the return type way too complex and hard to understand? Yes, Am I gonna give a fuck? No
#[allow(clippy::type_complexity)]
// parsing code
impl Emulator {
    /// Parse LC-3 assembly code into machine instructions and related artifacts.
    ///
    /// # Arguments
    /// * `program` - A string slice containing the LC-3 assembly source code.
    ///
    /// # Returns
    /// * `Ok(ParseOutput)` - Contains the generated machine code, line-to-address mapping,
    ///   label map, and origin address if parsing is successful.
    /// * `Err((String, usize))` - An error message and the line number (0-based) where
    ///   the error occurred if parsing fails.
    pub fn parse_program(program: &str) -> Result<ParseOutput, (String, usize)> {
        let span = tracing::info_span!("parse_program", program_length = program.len());
        let _guard = span.enter();

        tracing::info!("Starting to parse program");
        let program_str = program.to_string(); // Keep original for string content

        let mut machine_code = vec![];
        let mut labels = HashMap::new();
        let mut line_to_address: HashMap<usize, usize> = HashMap::new();
        let mut orig_address: u16 = 0x3000; // Default starting address
        let mut address: u16 = 0x3000;
        let mut orig_set = false;
        let mut non_colon_labels = HashSet::new();

        // --- First Pass: Collect labels and determine addresses ---
        tracing::debug!("Starting first pass: collecting labels and directives");
        for (i, line) in program_str.lines().enumerate() {
            let span = tracing::trace_span!("parse_addr_pass1", line = line, address = address);
            let _guard = span.enter();

            let line_trimmed = line.trim();
            let line_uncapped = line_trimmed; // Keep original case for .STRINGZ
            let line_upper = line_uncapped.to_ascii_uppercase();

            // Skip empty lines and comments
            if line_upper.is_empty() || line_upper.starts_with(';') {
                tracing::trace!("Line {}: Skipping empty line or comment", i);
                continue;
            }

            // Remove comments
            let line_no_comment = line_upper.split(';').next().unwrap().trim();
            let line_no_comment_uncapped = line_uncapped.split(';').next().unwrap().trim();

            // Skip if line is still empty
            if line_no_comment.is_empty() {
                tracing::trace!("Line {}: Skipping empty line after comment removal", i);
                continue;
            }

            tracing::trace!("Line {}: Processing '{}'", i, line_no_comment);

            // Helper to get directive size
            fn get_directive_size(
                line: &str,
                line_uncapped: &str,
                i: usize,
            ) -> Result<u16, (String, usize)> {
                if line.starts_with(".ORIG") || line.starts_with(".END") {
                    Ok(0)
                } else if line.starts_with(".FILL") {
                    tracing::trace!("Line {}: Directive .FILL (size 1)", i);
                    Ok(1)
                } else if line.starts_with(".BLKW") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 2 {
                        return Err((format!("Invalid .BLKW directive on line {}", i), i));
                    }
                    match parts[1].trim().parse::<u16>() {
                        Ok(count) => {
                            tracing::trace!(
                                "Line {}: Directive .BLKW {} (size {})",
                                i,
                                count,
                                count
                            );
                            Ok(count)
                        }
                        Err(e) => Err((format!("Invalid block size '{}': {}", parts[1], e), i)),
                    }
                } else if line.starts_with(".STRINGZ") {
                    tracing::trace!("Line {}: Directive .STRINGZ", i);
                    if let Some(start) = line_uncapped.find('"') {
                        if let Some(end) = line_uncapped[start + 1..].find('"') {
                            let content = &line_uncapped[start + 1..start + 1 + end];
                            let mut effective_len = 0;
                            let mut chars_iter = content.chars().peekable();
                            while let Some(c) = chars_iter.next() {
                                if c == '\\' && chars_iter.peek().is_some() {
                                    chars_iter.next(); // Consume escaped character
                                }
                                effective_len += 1;
                            }
                            tracing::trace!(
                                "Line {}: .STRINGZ content '{}', effective length {}",
                                i,
                                content,
                                effective_len
                            );
                            Ok(effective_len + 1) // +1 for null terminator
                        } else {
                            Err((format!("Unterminated string in .STRINGZ on line {}", i), i))
                        }
                    } else {
                        Err((format!("Invalid .STRINGZ directive on line {}", i), i))
                    }
                } else {
                    tracing::trace!("Line {}: Regular instruction (size 1)", i);
                    Ok(1) // Assume regular instruction takes 1 word
                }
            }

            let mut current_line_size = 0;

            if line_no_comment
                .split_whitespace()
                .next()
                .is_some_and(|w| w.contains(':'))
            {
                // Label with colon: LABEL: [instruction/directive]
                let parts: Vec<&str> = line_no_comment.splitn(2, ':').collect();
                let label = parts[0].trim();
                if !label.is_empty() {
                    if labels.contains_key(label) {
                        return Err((format!("Duplicate label '{}' defined", label), i));
                    }
                    tracing::debug!(
                        "Line {}: Found label '{}' (colon) at address {:04X}",
                        i,
                        label,
                        address
                    );
                    labels.insert(label.to_string(), address);
                }
                let instruction_part = parts.get(1).map_or("", |s| s.trim());
                if !instruction_part.is_empty() {
                    current_line_size =
                        get_directive_size(instruction_part, line_no_comment_uncapped, i)?;
                }
            } else if line_no_comment.starts_with(".ORIG") {
                // .ORIG directive
                let parts: Vec<&str> = line_no_comment.split_whitespace().collect();
                if parts.len() < 2 {
                    return Err(("Invalid .ORIG directive".to_string(), i));
                }
                let addr_str = parts[1].trim();
                let new_orig = if addr_str.starts_with('X') {
                    u16::from_str_radix(&addr_str[1..], 16)
                } else {
                    addr_str.parse::<u16>()
                };
                match new_orig {
                    Ok(addr) => {
                        orig_address = addr;
                        address = orig_address; // Update current address
                        orig_set = true;
                        tracing::debug!("Line {}: Set origin address to {:04X}", i, address);
                    }
                    Err(e) => {
                        return Err((format!("Invalid .ORIG address '{}': {}", addr_str, e), i))
                    }
                }
                // .ORIG itself doesn't take space in the final code
            } else if line_no_comment.starts_with('.') {
                // Other directives (.FILL, .BLKW, .STRINGZ, .END)
                current_line_size =
                    get_directive_size(line_no_comment, line_no_comment_uncapped, i)?;
            } else {
                // Potential label without colon OR just an instruction
                let parts: Vec<&str> = line_no_comment.split_whitespace().collect();

                if !parts.is_empty() {
                    let first_word = parts[0];
                    // Check if it's a known opcode/alias or starts with R (register)
                    let is_opcode_or_reg = [
                        "ADD", "AND", "BR", "BRN", "BRZ", "BRP", "BRNZ", "BRNP", "BRZP", "BRNZP",
                        "JMP", "JSR", "JSRR", "LD", "LDI", "LDR", "LEA", "NOT", "RET", "RTI", "ST",
                        "STI", "STR", "TRAP", "GETC", "OUT", "PUTS", "IN", "PUTSP", "HALT",
                    ]
                    .contains(&first_word)
                        || first_word.starts_with('R');

                    if !is_opcode_or_reg && !parts.is_empty() {
                        // Assume it's a label without a colon
                        let label = first_word;
                        if labels.contains_key(label) {
                            // Check if it was defined with a colon before
                            if non_colon_labels.contains(&i) {
                                // Ok, redefinition of no-colon label on same line is fine (though weird)
                            } else {
                                return Err((format!("Duplicate label '{}' defined", label), i));
                            }
                        }
                        tracing::debug!(
                            "Line {}: Found label '{}' (no colon) at address {:04X}",
                            i,
                            label,
                            address
                        );
                        labels.insert(label.to_string(), address);
                        non_colon_labels.insert(i);

                        let instruction_part =
                            line_no_comment.strip_prefix(label).unwrap_or("").trim();
                        if !instruction_part.is_empty() {
                            current_line_size =
                                get_directive_size(instruction_part, line_no_comment_uncapped, i)?;
                        }
                    } else {
                        // Just a regular instruction
                        current_line_size = 1;
                    }
                }
            }

            // Increment address for the next line
            if current_line_size > 0 {
                address = address
                    .checked_add(current_line_size)
                    .ok_or((format!("Address overflow past 0xFFFF on line {}", i), i))?;
            }
        } // End of first pass loop

        if !orig_set {
            return Err(("No .ORIG directive found".to_string(), 0));
        }

        // --- Second Pass: Generate instructions ---
        address = orig_address; // Reset address
        tracing::debug!("First pass completed, {} labels found", labels.len());
        tracing::debug!("Starting second pass with address at {:04X}", address);

        for (i, line) in program_str.lines().enumerate() {
            let line_trimmed = line.trim();
            let line_uncapped = line_trimmed;
            let line_upper = line_uncapped.to_ascii_uppercase();

            // Skip empty/comment lines
            if line_upper.is_empty() || line_upper.starts_with(';') {
                continue;
            }

            // Remove comments
            let line_no_comment = line_upper.split(';').next().unwrap().trim();
            let line_no_comment_uncapped = line_uncapped.split(';').next().unwrap().trim();

            if line_no_comment.is_empty() {
                continue;
            }

            tracing::trace!(
                "Line {}: Processing '{}' at address {:04X}",
                i,
                line_no_comment,
                address
            );

            // --- Record starting address for this line ---
            // Do this *before* processing, so directives map to their start address
            let start_address_for_line = address;
            line_to_address.insert(i, start_address_for_line as usize);

            // Helper to process directives during the second pass
            fn process_directive_second_pass(
                line: &str,
                line_uncapped: &str,
                i: usize,
                address: &mut u16,
                machine_code: &mut Vec<u16>,
                labels: &HashMap<String, u16>,
                line_to_address: &mut HashMap<usize, usize>, // Pass this mutably
                start_address_for_line: u16,                 // Pass the starting address
            ) -> Result<bool, (String, usize)> {
                // Ensure mapping exists even if directive generates no code (like .END)
                // Or if .ORIG changes the address before code generation
                line_to_address
                    .entry(i)
                    .or_insert(start_address_for_line as usize);

                if line.starts_with(".ORIG") {
                    // Update address, but generates no code here
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let addr_str = parts[1].trim();
                    let new_orig = if addr_str.starts_with('X') {
                        u16::from_str_radix(&addr_str[1..], 16)
                    } else {
                        addr_str.parse::<u16>()
                    };
                    match new_orig {
                        Ok(addr) => *address = addr,
                        Err(_) => unreachable!("Parse error should have been caught in first pass"),
                    }
                    return Ok(true); // Directive handled
                } else if line.starts_with(".FILL") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let value_str = parts[1].trim();
                    let value = if let Ok(imm) = Emulator::parse_immediate(value_str, 16) {
                        imm
                    } else if let Some(&label_addr) = labels.get(value_str) {
                        label_addr
                    } else {
                        return Err((format!("Invalid .FILL value '{}'", value_str), i));
                    };
                    tracing::trace!("Line {}: .FILL value {:04X} at {:04X}", i, value, *address);
                    machine_code.push(value);
                    *address += 1;
                    return Ok(true);
                } else if line.starts_with(".BLKW") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let count_str = parts[1].trim();
                    let count = match count_str.parse::<u16>() {
                        Ok(c) => c,
                        Err(_) => unreachable!("Parse error should have been caught in first pass"),
                    };
                    tracing::trace!("Line {}: .BLKW {} at {:04X}", i, count, *address);
                    for _ in 0..count {
                        machine_code.push(0); // Fill with zeros
                        *address += 1;
                    }
                    return Ok(true);
                } else if line.starts_with(".STRINGZ") {
                    tracing::trace!("Line {}: .STRINGZ at {:04X}", i, *address);
                    if let Some(start) = line_uncapped.find('"') {
                        if let Some(end) = line_uncapped[start + 1..].find('"') {
                            let content = &line_uncapped[start + 1..start + 1 + end];
                            let mut chars_iter = content.chars().peekable();
                            while let Some(c) = chars_iter.next() {
                                let char_val = if c == '\\' {
                                    if let Some(escaped_char) = chars_iter.next() {
                                        match escaped_char {
                                            'n' => 10,
                                            't' => 9,
                                            'r' => 13,
                                            '0' => 0,
                                            '\\' => '\\' as u16,
                                            '"' => '"' as u16,
                                            _ => {
                                                // Unrecognized escape, treat literally
                                                machine_code.push('\\' as u16);
                                                *address += 1;
                                                escaped_char as u16
                                            }
                                        }
                                    } else {
                                        '\\' as u16
                                    } // Trailing backslash
                                } else {
                                    c as u16
                                };
                                machine_code.push(char_val);
                                *address += 1;
                            }
                            machine_code.push(0); // Null terminator
                            *address += 1;
                        } // else: unreachable due to first pass check
                    } // else: unreachable due to first pass check
                    return Ok(true);
                } else if line.starts_with(".END") {
                    tracing::trace!("Line {}: Encountered .END", i);
                    // .END generates no code, address doesn't change here
                    return Ok(true); // Indicates directive handled, stop processing this line
                }
                Ok(false) // Not a directive we handle in the second pass code generation
            }

            // --- Process Line Content ---
            let mut instruction_part = line_no_comment;
            let mut instruction_part_uncapped = line_no_comment_uncapped; // Needed for .STRINGZ

            // Handle labels (they don't generate code themselves)
            if line_no_comment
                .split_whitespace()
                .next()
                .is_some_and(|w| w.contains(':'))
            {
                let parts: Vec<&str> = line_no_comment.splitn(2, ':').collect();
                instruction_part = parts.get(1).map_or("", |s| s.trim());
                // Also update the uncapped version if necessary
                let parts_uncapped: Vec<&str> = line_no_comment_uncapped.splitn(2, ':').collect();
                instruction_part_uncapped = parts_uncapped.get(1).map_or("", |s| s.trim());
            } else if non_colon_labels.contains(&i) {
                let parts: Vec<&str> = line_no_comment.split_whitespace().collect();
                if !parts.is_empty() {
                    instruction_part = line_no_comment.strip_prefix(parts[0]).unwrap_or("").trim();
                    // Also update the uncapped version
                    let parts_uncapped: Vec<&str> =
                        line_no_comment_uncapped.split_whitespace().collect();
                    if !parts_uncapped.is_empty() {
                        instruction_part_uncapped = line_no_comment_uncapped
                            .strip_prefix(parts_uncapped[0])
                            .unwrap_or("")
                            .trim();
                    }
                }
            }

            // If instruction_part is empty after label handling, skip to next line
            if instruction_part.is_empty() {
                // Ensure mapping exists even for label-only lines
                line_to_address
                    .entry(i)
                    .or_insert(start_address_for_line as usize);
                continue;
            }

            // Process directive or instruction
            if instruction_part.starts_with('.') {
                // Process directive
                match process_directive_second_pass(
                    instruction_part,
                    instruction_part_uncapped, // Pass the correct case version
                    i,
                    &mut address,
                    &mut machine_code,
                    &labels,
                    &mut line_to_address,
                    start_address_for_line,
                ) {
                    Ok(_) => {} // Address updated inside helper
                    Err(e) => return Err(e),
                }
            } else {
                // Parse instruction
                match Self::parse_instruction(instruction_part, start_address_for_line, &labels) {
                    // Use start address for offset calcs
                    Ok(instruction) => {
                        tracing::trace!(
                            "Line {}: Instruction {:04X} at {:04X}",
                            i,
                            instruction,
                            address
                        );
                        machine_code.push(instruction);
                        address += 1; // Instruction takes one word
                    }
                    Err(e) => return Err((e.0, i)),
                }
            }
        } // End of second pass loop
        Ok(ParseOutput {
            machine_code,
            line_to_address,
            labels,
            orig_address,
        })
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
                    Ok(instruction)
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
                    Ok(instruction)
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
                    Ok(instruction)
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
                    Ok(instruction)
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

                if !(-256..=255).contains(&offset) {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (n << 11) | (z << 10) | (p << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("BR: Generated instruction: {:04X}", instruction);
                Ok(instruction)
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
                Ok(instruction)
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

                if !(-1024..=1023).contains(&offset) {
                    tracing::error!("PCoffset11 out of range: {}", offset);
                    return Err(("PCoffset11 out of range".to_string(), 0));
                }

                let instruction = (0b0100 << 12) | (1 << 11) | (offset as u16 & 0x7FF);
                tracing::debug!("JSR: Generated instruction: {:04X}", instruction);
                Ok(instruction)
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
                Ok(instruction)
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

                if !(-256..=255).contains(&offset) {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b0010 << 12) | (dr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("LD: Generated instruction: {:04X}", instruction);
                Ok(instruction)
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

                if !(-256..=255).contains(&offset) {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b1010 << 12) | (dr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("LDI: Generated instruction: {:04X}", instruction);
                Ok(instruction)
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
                Ok(instruction)
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

                if !(-256..=255).contains(&offset) {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b1110 << 12) | (dr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("LEA: Generated instruction: {:04X}", instruction);
                Ok(instruction)
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
                Ok(instruction)
            }
            "RET" => {
                // RET is an alias for JMP R7
                tracing::debug!("RET: Alias for JMP R7");
                let instruction = (0b1100 << 12) | (7 << 6);
                tracing::debug!("RET: Generated instruction: {:04X}", instruction);
                Ok(instruction)
            }
            "RTI" => {
                tracing::debug!("RTI: Generated instruction: {:04X}", 0b1000 << 12);
                Ok(0b1000 << 12)
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

                if !(-256..=255).contains(&offset) {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b0011 << 12) | (sr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("ST: Generated instruction: {:04X}", instruction);
                Ok(instruction)
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

                if !(-256..=255).contains(&offset) {
                    tracing::error!("PCoffset9 out of range: {}", offset);
                    return Err(("PCoffset9 out of range".to_string(), 0));
                }

                let instruction = (0b1011 << 12) | (sr << 9) | (offset as u16 & 0x1FF);
                tracing::debug!("STI: Generated instruction: {:04X}", instruction);
                Ok(instruction)
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
                Ok(instruction)
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
                Ok(instruction)
            }
            // Trap aliases
            "GETC" => {
                tracing::debug!("GETC: Trap alias for vector 0x20");
                let instruction = (0b1111 << 12) | 0x20;
                tracing::debug!("GETC: Generated instruction: {:04X}", instruction);
                Ok(instruction)
            }
            "OUT" => {
                tracing::debug!("OUT: Trap alias for vector 0x21");
                let instruction = (0b1111 << 12) | 0x21;
                tracing::debug!("OUT: Generated instruction: {:04X}", instruction);
                Ok(instruction)
            }
            "PUTS" => {
                tracing::debug!("PUTS: Trap alias for vector 0x22");
                let instruction = (0b1111 << 12) | 0x22;
                tracing::debug!("PUTS: Generated instruction: {:04X}", instruction);
                Ok(instruction)
            }
            "IN" => {
                tracing::debug!("IN: Trap alias for vector 0x23");
                let instruction = (0b1111 << 12) | 0x23;
                tracing::debug!("IN: Generated instruction: {:04X}", instruction);
                Ok(instruction)
            }
            "PUTSP" => {
                tracing::debug!("PUTSP: Trap alias for vector 0x24");
                let instruction = (0b1111 << 12) | 0x24;
                tracing::debug!("PUTSP: Generated instruction: {:04X}", instruction);
                Ok(instruction)
            }
            "HALT" => {
                tracing::debug!("HALT: Trap alias for vector 0x25");
                let instruction = (0b1111 << 12) | 0x25;
                tracing::debug!("HALT: Generated instruction: {:04X}", instruction);
                Ok(instruction)
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

        if let Some(imm) = imm.strip_prefix("#") {
            // Decimal immediate
            match imm.parse::<i16>() {
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
            match u16::from_str_radix(&imm[1..], 16) {
                Ok(val) => {
                    value = val as i16;
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
        let min_value = (-{ 1 << (width - 1) }) as i16;
        let max_value = ((1 << (width - 1)) - 1) as i16;

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
        Ok(value as u16)
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
    }
}
