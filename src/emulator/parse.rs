use std::collections::{HashMap, HashSet};

use crate::emulator::EmulatorCell;

use super::Emulator;

// is the return type way too complex and hard to understand? Yes, Am I gonna give a fuck? No
#[allow(clippy::type_complexity)]
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
                match get_directive_size(line, line_uncapped, i) {
                    Ok(size) => address += size,
                    Err(e) => return Err(e),
                }
            } else {
                // Check if this line might be a label without a colon
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty()
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

            debug_first_pass_addr.insert(address, line_uncapped);
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

            debug_second_pass_addr.insert(address, line_uncapped);
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

        self.pc = EmulatorCell(start_address);
    }
}
