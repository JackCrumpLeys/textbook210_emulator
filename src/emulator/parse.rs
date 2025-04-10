use std::{collections::HashMap, str::FromStr};

use super::Emulator;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpToken {
    Add,
    And,
    Br(bool, bool, bool), // n, z, p
    Jmp,
    Jsr,
    Jsrr,
    Ld,
    Ldi,
    Ldr,
    Lea,
    Not,
    Ret,
    Rti,
    St,
    Sti,
    Str,
    Trap(Option<u8>), // we can use shorthand when lexing
}

impl FromStr for OpToken {
    fn from_str(s: &str) -> std::result::Result<OpToken, ()> {
        match s.to_ascii_uppercase().as_str() {
            "ADD" => Ok(OpToken::Add),
            "AND" => Ok(OpToken::And),
            "BR" => Ok(OpToken::Br(false, false, false)),
            "BRN" => Ok(OpToken::Br(true, false, false)),
            "BRZ" => Ok(OpToken::Br(false, true, false)),
            "BRP" => Ok(OpToken::Br(false, false, true)),
            "BRNZ" => Ok(OpToken::Br(true, true, false)),
            "BRNP" => Ok(OpToken::Br(true, false, true)),
            "BRZP" => Ok(OpToken::Br(false, true, true)),
            "BRNZP" => Ok(OpToken::Br(true, true, true)),
            "JMP" => Ok(OpToken::Jmp),
            "JSR" => Ok(OpToken::Jsr),
            "JSRR" => Ok(OpToken::Jsrr),
            "LD" => Ok(OpToken::Ld),
            "LDI" => Ok(OpToken::Ldi),
            "LDR" => Ok(OpToken::Ldr),
            "LEA" => Ok(OpToken::Lea),
            "NOT" => Ok(OpToken::Not),
            "RET" => Ok(OpToken::Ret),
            "RTI" => Ok(OpToken::Rti),
            "ST" => Ok(OpToken::St),
            "STI" => Ok(OpToken::Sti),
            "STR" => Ok(OpToken::Str),
            "TRAP" => Ok(OpToken::Trap(None)), // Generic trap, vector to be set later
            "GETC" => Ok(OpToken::Trap(Some(0x20))),
            "OUT" => Ok(OpToken::Trap(Some(0x21))),
            "PUTS" => Ok(OpToken::Trap(Some(0x22))),
            "IN" => Ok(OpToken::Trap(Some(0x23))),
            "PUTSP" => Ok(OpToken::Trap(Some(0x24))),
            "HALT" => Ok(OpToken::Trap(Some(0x25))),
            _ => Err(()),
        }
    }

    type Err = ();
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Instructions
    Opcode(OpToken),

    // Registers
    Register(u16),

    // Directives
    Directive(String),

    // Values
    Immediate(u16),
    HexValue(u16),

    // Labels
    Label(String),
    LabelRef(String),

    // String literals
    StringLiteral(String),

    // Delimiters
    Comma,
    Colon,

    // End of line
    EOL,
}

#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    position: usize,
    line: usize,
    column: usize,
    tokens: Vec<TokenSpan>,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            position: 0,
            line: 0,
            column: 0,
            tokens: Vec::new(),
            chars: input.chars().peekable(),
        }
    }

    // Tokenize the entire input
    pub fn tokenize(mut self) -> Result<Vec<TokenSpan>, (String, usize)> {
        while let Some(token) = self.next_token()? {
            self.tokens.push(token);
        }

        Ok(self.tokens)
    }

    // Get the next token
    fn next_token(&mut self) -> Result<Option<TokenSpan>, (String, usize)> {
        self.skip_whitespace();

        if let Some(c) = self.chars.peek() {
            match c {
                // End of line
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 0;
                    Ok(Some(TokenSpan {
                        token: Token::EOL,
                        line: self.line - 1,
                        column: self.column,
                    }))
                }

                // Comment
                ';' => {
                    // Skip the entire line
                    for c in self.chars.by_ref() {
                        self.position += 1;
                        self.column += 1;
                        if c == '\n' {
                            self.line += 1;
                            self.column = 0;
                            break;
                        }
                    }
                    Ok(Some(TokenSpan {
                        token: Token::EOL,
                        line: self.line,
                        column: self.column,
                    }))
                }

                // Comma
                ',' => {
                    self.advance();
                    Ok(Some(TokenSpan {
                        token: Token::Comma,
                        line: self.line,
                        column: self.column - 1,
                    }))
                }

                // Colon
                ':' => {
                    self.advance();
                    Ok(Some(TokenSpan {
                        token: Token::Colon,
                        line: self.line,
                        column: self.column - 1,
                    }))
                }

                // String literal
                '"' => self.tokenize_string(),

                // Numbers or identifiers
                _ => {
                    if c.is_numeric() || *c == '#' || *c == 'x' || *c == 'X' || *c == '-' {
                        self.tokenize_number()
                    } else if c.is_alphabetic() || *c == '.' || *c == '_' {
                        self.tokenize_word()
                    } else {
                        Err((format!("Unexpected character: {}", c), self.line))
                    }
                }
            }
        } else {
            // End of input
            Ok(None)
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() && c != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn advance(&mut self) -> char {
        if let Some(c) = self.chars.next() {
            self.position += 1;
            self.column += 1;
            c
        } else {
            // Return a default value if the character iterator is empty
            '\0'
        }
    }

    fn tokenize_string(&mut self) -> Result<Option<TokenSpan>, (String, usize)> {
        // Skip the opening quote
        self.advance();
        let start_line = self.line;
        let start_column = self.column;
        let mut string_content = String::new();
        let mut escaped = false;

        // Process string characters until closing quote or end of input
        while let Some(&c) = self.chars.peek() {
            if c == '"' && !escaped {
                // End of string - consume the closing quote
                self.advance();
                return Ok(Some(TokenSpan {
                    token: Token::StringLiteral(string_content),
                    line: start_line,
                    column: start_column,
                }));
            } else if c == '\\' && !escaped {
                // Start of escape sequence
                escaped = true;
                self.advance();
            } else {
                if escaped {
                    // Handle escape sequence
                    match c {
                        'n' => string_content.push('\n'),
                        't' => string_content.push('\t'),
                        'r' => string_content.push('\r'),
                        '0' => string_content.push('\0'),
                        '\\' => string_content.push('\\'),
                        '"' => string_content.push('"'),
                        _ => {
                            // Invalid escape sequence - include both backslash and character
                            string_content.push('\\');
                            string_content.push(c);
                        }
                    }
                    escaped = false;
                } else {
                    // Regular character
                    string_content.push(c);
                }
                self.advance();
            }
        }

        // If we get here, string was not properly terminated
        Err(("Unterminated string literal".to_string(), start_line))
    }

    fn tokenize_number(&mut self) -> Result<Option<TokenSpan>, (String, usize)> {
        let start_line = self.line;
        let start_column = self.column;
        let mut result = String::new();
        let mut is_hex = false;

        // Check for # prefix (decimal immediate)
        if Some(&'#') == self.chars.peek() {
            self.advance(); // Skip the #
                            // Omit # from the result as we'll parse as decimal
        } else if self.chars.peek().is_some_and(|c| c == &'x' || c == &'X') {
            is_hex = true;
            self.advance(); // Skip the x or X
            result.push('x'); // Keep for tracking hex format
        }

        // Check for negative sign
        if let Some(&'-') = self.chars.peek() {
            self.advance();
            result.push('-');
        }

        // Collect all digits
        let valid_chars = if is_hex {
            |c: char| c.is_ascii_hexdigit()
        } else {
            |c: char| c.is_ascii_digit()
        };

        let mut has_digits = false;
        while let Some(&c) = self.chars.peek() {
            if valid_chars(c) {
                has_digits = true;
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if !has_digits {
            return Err((
                format!("Invalid number format at line {}", start_line),
                start_line,
            ));
        }

        // Create the appropriate token
        let token = if is_hex {
            match u16::from_str_radix(&result[1..], 16) {
                // Skip the 'x' prefix
                Ok(val) => Token::HexValue(val),
                Err(_) => {
                    return Err((
                        format!(
                            "Invalid hex number format: {} at line {}",
                            result, start_line
                        ),
                        start_line,
                    ));
                }
            }
        } else {
            match result.parse::<i16>() {
                Ok(val) => Token::Immediate(val as u16),
                Err(_) => {
                    return Err((
                        format!(
                            "Invalid decimal number format: {} at line {}",
                            result, start_line
                        ),
                        start_line,
                    ));
                }
            }
        };

        Ok(Some(TokenSpan {
            token,
            line: start_line,
            column: start_column,
        }))
    }

    fn tokenize_word(&mut self) -> Result<Option<TokenSpan>, (String, usize)> {
        let start_line = self.line;
        let start_column = self.column;
        let mut word = String::new();

        // Collect all valid identifier characters
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '.' || c == '_' {
                word.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if word.is_empty() {
            return Err((
                format!("Expected identifier at line {}", start_line),
                start_line,
            ));
        }

        // Check for register pattern
        if word.len() >= 2 && (word.starts_with('R') || word.starts_with('r')) {
            if let Ok(reg_num) = word[1..].parse::<u16>() {
                if reg_num <= 7 {
                    tracing::trace!("Register token: R{}", reg_num);
                    return Ok(Some(TokenSpan {
                        token: Token::Register(reg_num),
                        line: start_line,
                        column: start_column,
                    }));
                }
            }
        }

        // Check for directive
        if word.starts_with('.') {
            tracing::trace!("Directive token: {}", word);
            return Ok(Some(TokenSpan {
                token: Token::Directive(word),
                line: start_line,
                column: start_column,
            }));
        }

        if let Ok(op_token) = OpToken::from_str(word.as_str()) {
            tracing::trace!("Opcode token: {:?}", op_token);
            return Ok(Some(TokenSpan {
                token: Token::Opcode(op_token),
                line: start_line,
                column: start_column,
            }));
        }

        // Default to label/label reference
        tracing::trace!("Label token: {}", word);
        // Check if this might be a label declaration by looking at recent tokens
        let mut is_label = false;
        if let Some(last_token) = self.tokens.last() {
            is_label = matches!(last_token.token, Token::EOL);
        }

        Ok(Some(TokenSpan {
            token: if is_label {
                Token::Label(word)
            } else {
                Token::LabelRef(word)
            },
            line: start_line,
            column: start_column,
        }))
    }
}

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

pub struct Parser {
    tokens: Vec<TokenSpan>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenSpan>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<ParseOutput, (String, usize)> {
        let mut machine_code = vec![];
        let mut labels = HashMap::new();
        let mut line_to_address: HashMap<usize, usize> = HashMap::new();
        let mut orig_address: u16 = 0x3000;
        let mut address: u16 = 0x3000;
        let mut orig_set = false;

        // First pass: collect labels and determine addresses
        self.first_pass(&mut labels, &mut address, &mut orig_address, &mut orig_set)?;

        // Reset for second pass
        self.position = 0;
        address = orig_address;

        // Second pass: generate machine code
        self.second_pass(
            &mut machine_code,
            &labels,
            &mut line_to_address,
            &mut address,
        )?;

        Ok(ParseOutput {
            machine_code,
            line_to_address,
            labels,
            orig_address,
        })
    }

    // First pass: collect labels and calculate addresses
    fn first_pass(
        &mut self,
        labels: &mut HashMap<String, u16>,
        address: &mut u16,
        orig_address: &mut u16,
        orig_set: &mut bool,
    ) -> Result<(), (String, usize)> {
        while self.position < self.tokens.len() {
            let token_span = &self.tokens[self.position];
            let line = token_span.line;

            // Keep track of current line for error reporting
            match &token_span.token {
                Token::Label(label_name) => {
                    // Direct label declaration (already converted from LabelRef+Colon)
                    if labels.contains_key(label_name) {
                        return Err((format!("Duplicate label '{}' defined", label_name), line));
                    }

                    tracing::debug!(
                        "Line {}: Found label '{}' at address {:04X}",
                        line,
                        label_name,
                        *address
                    );

                    labels.insert(label_name.clone(), *address);
                    self.position += 1;
                }

                Token::LabelRef(label_name) => {
                    // Check if next token is a colon to determine if this is a label definition
                    if self.position + 1 < self.tokens.len()
                        && matches!(self.tokens[self.position + 1].token, Token::Colon)
                    {
                        // This is a label definition
                        if labels.contains_key(label_name) {
                            return Err((
                                format!("Duplicate label '{}' defined", label_name),
                                line,
                            ));
                        }

                        tracing::debug!(
                            "Line {}: Found label '{}' (with colon) at address {:04X}",
                            line,
                            label_name,
                            *address
                        );

                        labels.insert(label_name.clone(), *address);

                        // Skip label and colon
                        self.position += 2;
                    } else {
                        // Treat as an opcode or standalone label
                        // For first pass, we just need to calculate address increments
                        *address = address.checked_add(1).ok_or((
                            format!("Address overflow past 0xFFFF on line {}", line),
                            line,
                        ))?;
                        self.position += 1;
                    }
                }

                Token::Directive(dir_name) => {
                    // Handle directives for address calculation
                    match dir_name.to_ascii_uppercase().as_str() {
                        ".ORIG" => {
                            // Check if this is the first directive in the program
                            if *orig_set {
                                return Err((
                                    ".ORIG must be the first directive in the program".to_string(),
                                    line,
                                ));
                            }

                            // Parse .ORIG address
                            if self.position + 1 >= self.tokens.len() {
                                return Err((
                                    "Invalid .ORIG directive: missing address".to_string(),
                                    line,
                                ));
                            }

                            let addr_token = &self.tokens[self.position + 1];
                            match &addr_token.token {
                                Token::HexValue(addr) => {
                                    *orig_address = *addr;
                                    *address = *addr;
                                    *orig_set = true;
                                    tracing::debug!(
                                        "Line {}: Set origin address to {:04X}",
                                        line,
                                        *address
                                    );
                                }
                                Token::Immediate(addr) => {
                                    *orig_address = *addr;
                                    *address = *addr;
                                    *orig_set = true;
                                    tracing::debug!(
                                        "Line {}: Set origin address to {:04X}",
                                        line,
                                        *address
                                    );
                                }
                                _ => {
                                    return Err((
                                        format!("Invalid .ORIG address at line {}", line),
                                        line,
                                    ))
                                }
                            }

                            // Skip past directive and address
                            self.position += 2;
                        }
                        ".END" => {
                            // In the second pass we'll actually validate it's the last directive
                            // No need to track it in first pass except for address calculation
                            self.position += 1;
                        }
                        ".FILL" => {
                            // Ensure .ORIG comes first
                            if !*orig_set {
                                return Err((
                                    ".ORIG must be the first directive in the program".to_string(),
                                    line,
                                ));
                            }

                            // .FILL takes one word
                            *address = address.checked_add(1).ok_or((
                                format!("Address overflow past 0xFFFF on line {}", line),
                                line,
                            ))?;

                            // Skip directive and value
                            self.position += 2;
                        }
                        ".BLKW" => {
                            // Ensure .ORIG comes first
                            if !*orig_set {
                                return Err((
                                    ".ORIG must be the first directive in the program".to_string(),
                                    line,
                                ));
                            }

                            // Parse block size
                            if self.position + 1 >= self.tokens.len() {
                                return Err((
                                    "Invalid .BLKW directive: missing size".to_string(),
                                    line,
                                ));
                            }

                            let size_token = &self.tokens[self.position + 1];
                            let block_size = match &size_token.token {
                                Token::Immediate(size) => {
                                    if *size == 0 {
                                        return Err((
                                            format!(
                                                "Invalid .BLKW size: must be positive, got {}",
                                                size
                                            ),
                                            line,
                                        ));
                                    }
                                    *size
                                }
                                Token::HexValue(size) => *size,
                                _ => {
                                    return Err((
                                        format!("Invalid .BLKW size at line {}", line),
                                        line,
                                    ))
                                }
                            };

                            tracing::trace!(
                                "Line {}: Directive .BLKW {} (size {})",
                                line,
                                block_size,
                                block_size
                            );

                            *address = address.checked_add(block_size).ok_or((
                                format!("Address overflow past 0xFFFF on line {}", line),
                                line,
                            ))?;

                            // Skip directive and size
                            self.position += 2;
                        }
                        ".STRINGZ" => {
                            // Ensure .ORIG comes first
                            if !*orig_set {
                                return Err((
                                    ".ORIG must be the first directive in the program".to_string(),
                                    line,
                                ));
                            }

                            // .STRINGZ takes string length + null terminator
                            if self.position + 1 >= self.tokens.len() {
                                return Err((
                                    "Invalid .STRINGZ directive: missing string".to_string(),
                                    line,
                                ));
                            }

                            let string_token = &self.tokens[self.position + 1];
                            match &string_token.token {
                                Token::StringLiteral(content) => {
                                    let string_size = content.chars().count() as u16 + 1; // +1 for null terminator
                                    tracing::trace!(
                                        "Line {}: Directive .STRINGZ \"{}\" (size {})",
                                        line,
                                        content,
                                        string_size
                                    );

                                    *address = address.checked_add(string_size).ok_or((
                                        format!("Address overflow past 0xFFFF on line {}", line),
                                        line,
                                    ))?;
                                }
                                _ => {
                                    return Err((
                                        format!("Invalid .STRINGZ value at line {}", line),
                                        line,
                                    ))
                                }
                            }

                            // Skip directive and string
                            self.position += 2;
                        }
                        _ => {
                            return Err((
                                format!("Unknown directive: {} at line {}", dir_name, line),
                                line,
                            ))
                        }
                    }
                }
                Token::Opcode(_) => {
                    // Ensure .ORIG comes first
                    if !*orig_set {
                        return Err((
                            ".ORIG must be the first directive in the program".to_string(),
                            line,
                        ));
                    }

                    // Instructions take one word
                    *address = address.checked_add(1).ok_or((
                        format!("Address overflow past 0xFFFF on line {}", line),
                        line,
                    ))?;

                    // Skip past this opcode and its operands (simplified for first pass)
                    let mut op_position = self.position + 1;
                    while op_position < self.tokens.len()
                        && !matches!(self.tokens[op_position].token, Token::EOL)
                    {
                        op_position += 1;
                    }
                    self.position = op_position + 1; // Skip past EOL
                }

                Token::EOL => {
                    // Simply move to next token
                    self.position += 1;
                }

                _ => {
                    // For other tokens, just move forward in first pass
                    self.position += 1;
                }
            }
        }

        if !*orig_set {
            return Err(("No .ORIG directive found".to_string(), 0));
        }

        Ok(())
    }

    // Second pass: generate machine code
    fn second_pass(
        &mut self,
        machine_code: &mut Vec<u16>,
        labels: &HashMap<String, u16>,
        line_to_address: &mut HashMap<usize, usize>,
        address: &mut u16,
    ) -> Result<(), (String, usize)> {
        let mut current_line = 0;
        let mut line_has_address_mapping = false;

        while self.position < self.tokens.len() {
            let token_span = &self.tokens[self.position].clone();
            let line = token_span.line;
            let current_address = *address;

            // Track new line - record the starting address for this line
            if current_line != line {
                current_line = line;
                line_to_address.insert(line, current_address as usize);
                line_has_address_mapping = true;
            } else if !line_has_address_mapping {
                line_to_address.insert(line, current_address as usize);
                line_has_address_mapping = true;
            }

            match &token_span.token {
                Token::Label(_) => {
                    // Labels don't generate code, just skip
                    self.position += 1;

                    // Handle colon suffix on label - if the next token is a colon, skip it
                    if self.position < self.tokens.len()
                        && matches!(self.tokens[self.position].token, Token::Colon)
                    {
                        self.position += 1;
                    }
                }

                Token::Directive(dir_name) => {
                    match dir_name.to_ascii_uppercase().as_str() {
                        ".ORIG" => {
                            // Get the address but don't generate code
                            if self.position + 1 < self.tokens.len() {
                                let addr_token = &self.tokens[self.position + 1];
                                match &addr_token.token {
                                    Token::HexValue(addr) => {
                                        *address = *addr;
                                    }
                                    Token::Immediate(addr) => {
                                        *address = *addr;
                                    }
                                    _ => {} // Already validated in first pass
                                }
                            }
                            self.position += 2; // Skip directive and address
                        }
                        ".END" => {
                            // No code generation needed
                            self.position += 1;
                        }
                        ".FILL" => {
                            if self.position + 1 >= self.tokens.len() {
                                return Err((
                                    "Invalid .FILL directive: missing value".to_string(),
                                    line,
                                ));
                            }

                            let value_token = &self.tokens[self.position + 1];
                            let value = match &value_token.token {
                                Token::Immediate(imm) => *imm,
                                Token::HexValue(hex) => *hex,
                                Token::LabelRef(label) => {
                                    if let Some(&label_addr) = labels.get(label) {
                                        label_addr
                                    } else {
                                        return Err((format!("Unknown label: {}", label), line));
                                    }
                                }
                                _ => {
                                    return Err((
                                        format!("Invalid .FILL value at line {}", line),
                                        line,
                                    ));
                                }
                            };

                            tracing::trace!(
                                "Line {}: .FILL value {:04X} at {:04X}",
                                line,
                                value,
                                *address
                            );
                            machine_code.push(value);
                            *address += 1;
                            self.position += 2; // Skip directive and value
                        }
                        ".BLKW" => {
                            if self.position + 1 >= self.tokens.len() {
                                return Err((
                                    "Invalid .BLKW directive: missing size".to_string(),
                                    line,
                                ));
                            }

                            let size_token = &self.tokens[self.position + 1];
                            let count = match &size_token.token {
                                Token::Immediate(size) => *size,
                                Token::HexValue(size) => *size,
                                _ => {
                                    return Err((
                                        format!("Invalid .BLKW size at line {}", line),
                                        line,
                                    ));
                                }
                            };

                            tracing::trace!("Line {}: .BLKW {} at {:04X}", line, count, *address);
                            for _ in 0..count {
                                machine_code.push(0); // Fill with zeros
                                *address += 1;
                            }
                            self.position += 2; // Skip directive and size
                        }
                        ".STRINGZ" => {
                            if self.position + 1 >= self.tokens.len() {
                                return Err((
                                    "Invalid .STRINGZ directive: missing string".to_string(),
                                    line,
                                ));
                            }

                            let string_token = &self.tokens[self.position + 1];
                            match &string_token.token {
                                Token::StringLiteral(content) => {
                                    tracing::trace!(
                                        "Line {}: .STRINGZ at {:04X}: \"{}\"",
                                        line,
                                        *address,
                                        content
                                    );

                                    // Process each character
                                    for c in content.chars() {
                                        machine_code.push(c as u16);
                                        *address += 1;
                                    }

                                    // Add null terminator
                                    machine_code.push(0);
                                    *address += 1;
                                }
                                _ => {
                                    return Err((
                                        format!("Invalid .STRINGZ value at line {}", line),
                                        line,
                                    ));
                                }
                            }

                            self.position += 2; // Skip directive and string
                        }
                        _ => {
                            return Err((
                                format!("Unknown directive: {} at line {}", dir_name, line),
                                line,
                            ));
                        }
                    }
                }

                Token::Opcode(op) => {
                    // Get instruction operands
                    let mut operands = Vec::new();
                    let mut op_pos = self.position + 1;

                    // Collect operands until EOL
                    while op_pos < self.tokens.len()
                        && !matches!(self.tokens[op_pos].token, Token::EOL)
                    {
                        // Skip commas
                        if !matches!(self.tokens[op_pos].token, Token::Comma) {
                            operands.push(self.tokens[op_pos].clone());
                        }
                        op_pos += 1;
                    }

                    let instruction =
                        self.generate_instruction(op, &operands, current_address, labels, line)?;
                    machine_code.push(instruction);
                    *address += 1;

                    // Skip to next line
                    self.position = op_pos;
                    if op_pos < self.tokens.len() {
                        self.position += 1; // Skip the EOL token
                    }
                }

                Token::EOL => {
                    // Reset the line_has_address_mapping flag for the next line
                    line_has_address_mapping = false;
                    self.position += 1;
                }

                _ => {
                    // For other tokens, report an error - unexpected token
                    return Err((
                        format!("Unexpected token at line {}: {:?}", line, token_span.token),
                        line,
                    ));
                }
            }
        }

        Ok(())
    }

    fn generate_instruction(
        &self,
        op: &OpToken,
        operands: &[TokenSpan],
        current_address: u16,
        labels: &HashMap<String, u16>,
        line: usize,
    ) -> Result<u16, (String, usize)> {
        match op {
            OpToken::Add => {
                if operands.len() < 3 {
                    return Err(("Invalid ADD format: not enough operands".to_string(), line));
                }

                let dr = self.parse_register(&operands[0], line)?;
                let sr1 = self.parse_register(&operands[1], line)?;

                // Check mode (register or immediate)
                match &operands[2].token {
                    Token::Register(sr2) => {
                        // Register mode: ADD DR, SR1, SR2
                        let instruction = (0b0001 << 12) | (dr << 9) | (sr1 << 6) | *sr2;
                        Ok(instruction)
                    }
                    Token::Immediate(imm5) | Token::HexValue(imm5) => {
                        // Immediate mode: ADD DR, SR1, #IMM5
                        let imm5_val = self.check_immediate_range(*imm5 as i16, 5, line)?;
                        let instruction =
                            (0b0001 << 12) | (dr << 9) | (sr1 << 6) | (1 << 5) | (imm5_val & 0x1F);
                        Ok(instruction)
                    }
                    _ => Err((format!("Invalid ADD operand at line {}", line), line)),
                }
            }

            OpToken::And => {
                if operands.len() < 3 {
                    return Err(("Invalid AND format: not enough operands".to_string(), line));
                }

                let dr = self.parse_register(&operands[0], line)?;
                let sr1 = self.parse_register(&operands[1], line)?;

                // Check mode (register or immediate)
                match &operands[2].token {
                    Token::Register(sr2) => {
                        // Register mode: AND DR, SR1, SR2
                        let instruction = (0b0101 << 12) | (dr << 9) | (sr1 << 6) | *sr2;
                        Ok(instruction)
                    }
                    Token::Immediate(imm5) | Token::HexValue(imm5) => {
                        // Immediate mode: AND DR, SR1, #IMM5
                        let imm5_val = self.check_immediate_range(*imm5 as i16, 5, line)?;
                        let instruction =
                            (0b0101 << 12) | (dr << 9) | (sr1 << 6) | (1 << 5) | (imm5_val & 0x1F);
                        Ok(instruction)
                    }
                    _ => Err((format!("Invalid AND operand at line {}", line), line)),
                }
            }

            OpToken::Br(n, z, p) => {
                if operands.is_empty() {
                    return Err(("Invalid BR format: missing target".to_string(), line));
                }

                let offset = self.parse_offset(&operands[0], current_address, labels, 9, line)?;
                let n_bit = (*n as u16) << 11;
                let z_bit = (*z as u16) << 10;
                let p_bit = (*p as u16) << 9;

                let instruction = n_bit | z_bit | p_bit | (offset & 0x1FF);
                Ok(instruction)
            }

            OpToken::Jmp => {
                if operands.is_empty() {
                    return Err(("Invalid JMP format: missing register".to_string(), line));
                }

                let base_r = self.parse_register(&operands[0], line)?;
                let instruction = (0b1100 << 12) | (base_r << 6);
                Ok(instruction)
            }

            OpToken::Jsr => {
                if operands.is_empty() {
                    return Err(("Invalid JSR format: missing target".to_string(), line));
                }

                let offset = self.parse_offset(&operands[0], current_address, labels, 11, line)?;
                let instruction = (0b0100 << 12) | (1 << 11) | (offset & 0x7FF);
                Ok(instruction)
            }

            OpToken::Jsrr => {
                if operands.is_empty() {
                    return Err(("Invalid JSRR format: missing register".to_string(), line));
                }

                let base_r = self.parse_register(&operands[0], line)?;
                let instruction = (0b0100 << 12) | (base_r << 6);
                Ok(instruction)
            }

            OpToken::Ld => {
                if operands.len() < 2 {
                    return Err(("Invalid LD format: not enough operands".to_string(), line));
                }

                let dr = self.parse_register(&operands[0], line)?;
                let offset = self.parse_offset(&operands[1], current_address, labels, 9, line)?;

                let instruction = (0b0010 << 12) | (dr << 9) | (offset & 0x1FF);
                Ok(instruction)
            }

            OpToken::Ldi => {
                if operands.len() < 2 {
                    return Err(("Invalid LDI format: not enough operands".to_string(), line));
                }

                let dr = self.parse_register(&operands[0], line)?;
                let offset = self.parse_offset(&operands[1], current_address, labels, 9, line)?;

                let instruction = (0b1010 << 12) | (dr << 9) | (offset & 0x1FF);
                Ok(instruction)
            }

            OpToken::Ldr => {
                if operands.len() < 3 {
                    return Err(("Invalid LDR format: not enough operands".to_string(), line));
                }

                let dr = self.parse_register(&operands[0], line)?;
                let base_r = self.parse_register(&operands[1], line)?;
                let offset = self.parse_immediate(&operands[2], 6, line)?;

                let instruction = (0b0110 << 12) | (dr << 9) | (base_r << 6) | (offset & 0x3F);
                Ok(instruction)
            }

            OpToken::Lea => {
                if operands.len() < 2 {
                    return Err(("Invalid LEA format: not enough operands".to_string(), line));
                }

                let dr = self.parse_register(&operands[0], line)?;
                let offset = self.parse_offset(&operands[1], current_address, labels, 9, line)?;

                let instruction = (0b1110 << 12) | (dr << 9) | (offset & 0x1FF);
                Ok(instruction)
            }

            OpToken::Not => {
                if operands.len() < 2 {
                    return Err(("Invalid NOT format: not enough operands".to_string(), line));
                }

                let dr = self.parse_register(&operands[0], line)?;
                let sr = self.parse_register(&operands[1], line)?;

                let instruction = (0b1001 << 12) | (dr << 9) | (sr << 6) | 0x3F;
                Ok(instruction)
            }

            OpToken::Ret => {
                // RET is an alias for JMP R7
                let instruction = (0b1100 << 12) | (7 << 6);
                Ok(instruction)
            }

            OpToken::Rti => {
                // RTI has no operands
                let instruction = 0b1000 << 12;
                Ok(instruction)
            }

            OpToken::St => {
                if operands.len() < 2 {
                    return Err(("Invalid ST format: not enough operands".to_string(), line));
                }

                let sr = self.parse_register(&operands[0], line)?;
                let offset = self.parse_offset(&operands[1], current_address, labels, 9, line)?;

                let instruction = (0b0011 << 12) | (sr << 9) | (offset & 0x1FF);
                Ok(instruction)
            }

            OpToken::Sti => {
                if operands.len() < 2 {
                    return Err(("Invalid STI format: not enough operands".to_string(), line));
                }

                let sr = self.parse_register(&operands[0], line)?;
                let offset = self.parse_offset(&operands[1], current_address, labels, 9, line)?;

                let instruction = (0b1011 << 12) | (sr << 9) | (offset & 0x1FF);
                Ok(instruction)
            }

            OpToken::Str => {
                if operands.len() < 3 {
                    return Err(("Invalid STR format: not enough operands".to_string(), line));
                }

                let sr = self.parse_register(&operands[0], line)?;
                let base_r = self.parse_register(&operands[1], line)?;
                let offset = self.parse_immediate(&operands[2], 6, line)?;

                let instruction = (0b0111 << 12) | (sr << 9) | (base_r << 6) | (offset & 0x3F);
                Ok(instruction)
            }

            OpToken::Trap(trap_vector) => {
                let trapvect8 = if let Some(vector) = trap_vector {
                    // Use the predefined trap vector for trap aliases
                    *vector as u16
                } else {
                    // Parse custom trap vector
                    if operands.is_empty() {
                        return Err(("Invalid TRAP format: missing vector".to_string(), line));
                    }

                    match &operands[0].token {
                        Token::HexValue(vector) => {
                            if *vector > 0xFF {
                                return Err(("Trap vector out of range (0-255)".to_string(), line));
                            }
                            *vector
                        }
                        Token::Immediate(vector) => {
                            if *vector > 255 {
                                return Err(("Trap vector out of range (0-255)".to_string(), line));
                            }
                            *vector
                        }
                        _ => return Err(("Invalid trap vector format".to_string(), line)),
                    }
                };

                let instruction = (0b1111 << 12) | trapvect8;
                Ok(instruction)
            }
        }
    }

    fn parse_register(&self, token: &TokenSpan, line: usize) -> Result<u16, (String, usize)> {
        match &token.token {
            Token::Register(reg) => {
                if *reg <= 7 {
                    Ok(*reg)
                } else {
                    Err((format!("Register number out of range: {}", reg), line))
                }
            }
            _ => Err((format!("Expected register at line {}", line), line)),
        }
    }

    fn parse_immediate(
        &self,
        token: &TokenSpan,
        width: u8,
        line: usize,
    ) -> Result<u16, (String, usize)> {
        match &token.token {
            Token::Immediate(imm) => self.check_immediate_range(*imm as i16, width, line),
            Token::HexValue(hex) => {
                let signed_value = if *hex & (1 << (width - 1)) != 0 {
                    // Value would be negative when sign-extended
                    -(((!*hex + 1) & ((1 << width) - 1)) as i16)
                } else {
                    // Value is positive
                    *hex as i16
                };
                self.check_immediate_range(signed_value, width, line)
            }
            _ => Err((format!("Expected immediate value at line {}", line), line)),
        }
    }

    fn parse_offset(
        &self,
        token: &TokenSpan,
        current_address: u16,
        labels: &HashMap<String, u16>,
        width: u8,
        line: usize,
    ) -> Result<u16, (String, usize)> {
        match &token.token {
            Token::LabelRef(label) => {
                if let Some(&label_addr) = labels.get(label) {
                    let offset = (label_addr as i16) - (current_address as i16 + 1);
                    self.check_immediate_range(offset, width, line)
                } else {
                    Err((format!("Unknown label: {}", label), line))
                }
            }
            Token::Immediate(imm) => self.check_immediate_range(*imm as i16, width, line),
            Token::HexValue(hex) => {
                // Convert to signed value based on bit width
                let signed_value = if *hex & (1 << (width - 1)) != 0 {
                    // Value would be negative when sign-extended
                    -(((!*hex + 1) & ((1 << width) - 1)) as i16)
                } else {
                    // Value is positive
                    *hex as i16
                };
                self.check_immediate_range(signed_value, width, line)
            }
            _ => Err((format!("Expected label or offset at line {}", line), line)),
        }
    }

    fn check_immediate_range(
        &self,
        value: i16,
        width: u8,
        line: usize,
    ) -> Result<u16, (String, usize)> {
        let min_value = -(1 << (width - 1));
        let max_value = (1 << (width - 1)) - 1;

        if value < min_value || value > max_value {
            Err((
                format!(
                    "Immediate value {} out of range for {}-bit field [{}, {}]",
                    value, width, min_value, max_value
                ),
                line,
            ))
        } else {
            // For negative values, we need to properly represent in 2's complement
            if value < 0 {
                Ok(((1 << width) + value as i32) as u16)
            } else {
                Ok(value as u16)
            }
        }
    }
}

impl Emulator {
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

    pub fn parse_program(program: &str) -> Result<ParseOutput, (String, usize)> {
        let span = tracing::info_span!("parse_program", program_length = program.len());
        let _guard = span.enter();

        tracing::info!("starting to parse program");

        // step 1: tokenize the input
        let lexer = Lexer::new(program);
        let tokens = lexer.tokenize()?;

        tracing::debug!("tokenization complete: {} tokens", tokens.len());
        tracing::trace!("tokens: {:?}", tokens);

        // step 2: parse the tokens
        let mut parser = Parser::new(tokens);
        let out = parser.parse();

        tracing::trace!("parsed output: {:?}", out);

        out
    }
}
