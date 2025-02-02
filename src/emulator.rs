use std::{collections::HashMap, error::Error};

use log::warn;

#[derive(Debug, Clone)]
pub struct Emulator {
    pub memory: [u16; 4096],
    // Register
    pub r: i16,
    // Program Counter
    pub pc: u16,
    pub gt: u16,
    pub lt: u16,
    pub eq: u16,
    pub ir: u16,

    pub await_input: Option<u16>,
    pub output: Vec<u16>,
}
impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Instructions:
/// op code - operation - meaning
/// 0000      LOAD X      Load the value at memory address X into the register r. CON(X) -> r
/// 0001      STORE X     Store the value in the register r into memory address X. r -> CON(X)
/// 0010      CLear X     Store 0 in the cell at memory address X. 0 -> CON(X)
/// 0011      ADD X       Add the value at memory address X to the register r. r + CON(X) -> r
/// 0100      INCREMENT X Increment the value at memory address X. CON(X) + 1 -> CON(X)
/// 0101      SUBTRACT X  Subtract the value at memory address X from the register r. r - CON(X) -> r
/// 0110      DECREMENT X Decrement the value at memory address X. CON(X) - 1 -> CON(X)
/// 0111      COMPARE X   Compare the value at memory address X with the register r.
///                       if CON(X) > r, gt=1, else gt=0
///                       if CON(X) < r, lt=1, else lt=0
///                       if CON(X) = r, eq=1, else eq=0
/// 1000      JUMP X      Jump to the instruction at memory address X. PC -> X
/// 1001      JUMPGT X    Jump to the instruction at memory address X if gt=1. if gt=1, PC -> X
/// 1010      JUMPEQ X    Jump to the instruction at memory address X if eq=1. if eq=1, PC -> X
/// 1011      JUMPLT X    Jump to the instruction at memory address X if lt=1. if lt=1, PC -> X
/// 1100      JUMPNEQ X   Jump to the instruction at memory address X if eq=0. if eq=0, PC -> X
/// 1101      IN X        Read a int from the input device and store it in the cell at memory address X. IN -> CON(X)
/// 1110      OUT X       Write the value at memory address X to the output device. CON(X) -> OUT
/// 1111      HALT        Halt the program. Stop execution.
impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            memory: [0; 4096],
            r: 0,
            pc: 0,
            gt: 0,
            lt: 0,
            eq: 0,
            ir: 0,

            await_input: None,
            output: Vec::new(),
        }
    }

    /// example program:
    ///          .begin
    ///          LOAD X -- load the value at memory address X into the register r
    ///          ADD 2
    ///          STORE Y
    ///          LOAD X
    ///          INCREMENT X -- increment the value at memory address X
    ///          ADD Y
    ///          STORE RESULT
    ///          JUMP RETURN
    /// RETURN:  OUT RESULT
    /// X:       .data 5
    /// Y:       .data 10 -- memory address Y will contain 10
    /// RESULT:  .data 0
    pub fn parse_program(
        program: &str,
    ) -> Result<(Vec<(usize, u16)>, HashMap<String, u16>), (String, usize)> {
        let program = program.to_string().to_ascii_uppercase();

        let mut instructions = vec![];
        // first pass will get the address of the labels and split the program and data segment and remove the labels
        let mut labels = HashMap::new();
        let mut address = 0;
        let mut data_segment = Vec::new();
        let mut code_segement = Vec::new();
        for (i, line) in program.lines().enumerate() {
            let line = line.trim();
            let line = line.split("--").next().unwrap();
            if line.starts_with(".") {
                continue;
            }

            if line == "" {
                continue;
            }

            let line_split: Vec<&str> = line.split(":").filter(|x| !x.is_empty()).collect();
            if line.contains(":") && line_split.len() == 1 {
                return Err((format!("No instruction after label."), i));
            } else if line_split.len() == 2 {
                let label = line_split[0];
                labels.insert(label.trim().to_string(), address);
                if line_split[1].trim().starts_with(".DATA") {
                    data_segment.push((
                        i,
                        line_split[1].trim().strip_prefix(".DATA").unwrap().trim(),
                    ));
                } else {
                    code_segement.push((i, line_split[1].trim()));
                }
            } else if line_split.len() == 3 {
                return Err((format!("too many colon characters (':')"), i));
            } else {
                if line.starts_with(".DATA") {
                    data_segment.push((i, line.strip_prefix(".DATA").unwrap().trim()));
                } else {
                    code_segement.push((i, line.trim()));
                }
            }
            address += 1;
        }
        if code_segement.is_empty() {
            return Err(("No code entered!".to_string(), 0));
        }
        // make sure halt instruction is at the end of the program
        if !code_segement.last().unwrap().1.starts_with("HALT") {
            println!("{}", code_segement.len());
            for (_, address) in labels.iter_mut() {
                println!("{}", *address);
                if *address >= code_segement.len() as u16 {
                    // if the label is after the halt instruction, we need to adjust the address
                    *address += 1;
                }
            }
            code_segement.push((usize::MAX, "HALT"));
        }

        // second pass will convert the instructions to machine code and replace the labels with addresses

        for (i, (ii, line)) in code_segement.iter().enumerate() {
            let instruction = line.split_whitespace().collect::<Vec<&str>>();
            let op = match instruction[0] {
                "LOAD" => 0,
                "STORE" => 1,
                "CLEAR" => 2,
                "ADD" => 3,
                "INCREMENT" => 4,
                "SUBTRACT" => 5,
                "DECREMENT" => 6,
                "COMPARE" => 7,
                "JUMP" => 8,
                "JUMPGT" => 9,
                "JUMPEQ" => 10,
                "JUMPLT" => 11,
                "JUMPNEQ" => 12,
                "IN" => 13,
                "OUT" => 14,
                "HALT" => 15,
                _ => {
                    return Err((format!("Invalid instruction: {}", instruction[0]), *ii));
                }
            };
            if instruction[0] != "HALT" {
                let x = match instruction[1].parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => {
                        let label = instruction[1];
                        let address = labels
                            .get(label)
                            .ok_or((format!("Label not found: {}", label), *ii))?;
                        *address
                    }
                };
                let machine_code = (op << 12) | x;
                instructions.push((*ii, machine_code));
            } else {
                instructions.push((*ii, op << 12))
            }
        }

        for (ii, line) in data_segment {
            if labels.contains_key(line) {
                instructions.push((ii, *labels.get(line).unwrap()));
                continue;
            }
            instructions.push((
                ii,
                line.parse::<i16>().map_err(|e| {
                    (
                        format!("Invalid number: {}, error: {:?}. Line : {}", line, e, ii),
                        0,
                    )
                })? as u16,
            ));
        }

        Ok((instructions, labels))
    }

    fn set_memory(&mut self, memory: Vec<u16>) {
        self.memory[..memory.len()].copy_from_slice(&memory);
    }

    pub fn step(&mut self) -> bool {
        if self.await_input.is_some() {
            warn!("I was stepped without input");
            return true;
        }

        self.ir = self.memory[self.pc as usize];
        self.pc += 1;
        let op = self.ir >> 12;
        let x = self.ir & 0x0FFF;
        match op {
            0 => self.load(x),
            1 => self.store(x),
            2 => self.clear(x),
            3 => self.add(x),
            4 => self.increment(x),
            5 => self.subtract(x),
            6 => self.decrement(x),
            7 => self.compare(x),
            8 => self.jump(x),
            9 => self.jump_gt(x),
            10 => self.jump_eq(x),
            11 => self.jump_lt(x),
            12 => self.jump_neq(x),
            13 => self.input(x),
            14 => self.output(x),
            15 => return false,
            _ => panic!("Invalid instruction"),
        }
        if self.pc >= 4096 {
            return false;
        }
        true
    }

    fn load(&mut self, x: u16) {
        self.r = self.memory[x as usize] as i16;
    }

    fn store(&mut self, x: u16) {
        self.memory[x as usize] = self.r as u16;
    }

    fn clear(&mut self, x: u16) {
        self.memory[x as usize] = 0;
    }

    fn add(&mut self, x: u16) {
        self.r = self.r.wrapping_add(self.memory[x as usize] as i16);
    }

    fn increment(&mut self, x: u16) {
        self.memory[x as usize] = 1i16.wrapping_add(self.memory[x as usize] as i16) as u16;
    }

    fn subtract(&mut self, x: u16) {
        self.r = self.r.wrapping_sub(self.memory[x as usize] as i16);
    }

    fn decrement(&mut self, x: u16) {
        self.memory[x as usize] = (self.memory[x as usize] as i16).wrapping_sub(1) as u16;
    }

    fn compare(&mut self, x: u16) {
        if self.memory[x as usize] as i16 > self.r as i16 {
            self.gt = 1;
            self.lt = 0;
            self.eq = 0;
        } else if (self.memory[x as usize] as i16) < self.r as i16 {
            self.gt = 0;
            self.lt = 1;
            self.eq = 0;
        } else {
            self.gt = 0;
            self.lt = 0;
            self.eq = 1;
        }
    }

    fn jump(&mut self, x: u16) {
        self.pc = x;
    }

    fn jump_gt(&mut self, x: u16) {
        if self.gt == 1 {
            self.jump(x)
        }
    }

    fn jump_eq(&mut self, x: u16) {
        if self.eq == 1 {
            self.jump(x)
        }
    }

    fn jump_lt(&mut self, x: u16) {
        if self.lt == 1 {
            self.jump(x)
        }
    }

    fn jump_neq(&mut self, x: u16) {
        if self.eq == 0 {
            self.jump(x)
        }
    }

    fn input(&mut self, x: u16) {
        self.await_input = Some(x);
    }

    fn output(&mut self, x: u16) {
        self.output.push(self.memory[x as usize]);
    }

    pub fn set_input(&mut self, input: u16) {
        if let Some(x) = self.await_input {
            self.memory[x as usize] = input;
            self.await_input = None;
        }
    }

    pub fn flash_memory(&mut self, cells: Vec<u16>) {
        self.memory[..cells.len()].copy_from_slice(&cells);
    }

    pub fn get_output(&self) -> Vec<u16> {
        self.output.clone()
    }
}

fn util_instruction_to_string(instruction: u16) -> String {
    let op = instruction >> 12;
    let x = instruction & 0x0FFF;
    match op {
        0 => format!("LOAD {}", x),
        1 => format!("STORE {}", x),
        2 => format!("CLEAR {}", x),
        3 => format!("ADD {}", x),
        4 => format!("INCREMENT {}", x),
        5 => format!("SUBTRACT {}", x),
        6 => format!("DECREMENT {}", x),
        7 => format!("COMPARE {}", x),
        8 => format!("JUMP {}", x),
        9 => format!("JUMPGT {}", x),
        10 => format!("JUMPEQ {}", x),
        11 => format!("JUMPLT {}", x),
        12 => format!("JUMPNEQ {}", x),
        13 => format!("IN {}", x),
        14 => format!("OUT {}", x),
        15 => "HALT".to_string(),
        _ => panic!("Invalid instruction"),
    }
}

#[cfg(test)]
mod tests {
    use egui::debug_text::print;

    use super::*;

    #[test]
    fn test_parse_program() {
        let program = r#"
            .begin

            -- get magnitude of B and A

            LOAD A               -- Load the value at memory address A into register r
            COMPARE ZERO         -- Compare register r with memory[ZERO]
            STORE A_MAGNITUDE    -- Store the magnitude of A
            JUMPLT LOAD_B        -- If A is NOT negative, load B
            SUBTRACT A       -- Negate to get to 0
            SUBTRACT A       -- Negate to get to 0
            STORE A_MAGNITUDE -- Store the magnitude of A

            LOAD_B: LOAD B           -- Load the value at memory address B into register r
            COMPARE ZERO         -- Compare register r with memory[ZERO]
            JUMPLT SWAP      -- If B is NOT negative, skip flipping the signs
            SUBTRACT B       -- Negate to get to 0
            SUBTRACT B       -- Negate to get to 0


            SWAP: COMPARE A_MAGNITUDE        -- Compare B_MAGNITUDE with A_MAGNITUDE
            JUMPGT LOOP      -- If A is greater than B, jump to LOOP
            LOAD B
            STORE TEMP       -- Store the value of B in TEMP
            LOAD A           -- Load the value at memory address A into register r
            STORE B          -- Store the value of A in B
            LOAD TEMP        -- Load the value at memory address TEMP into register r
            STORE A          -- Store the value of TEMP in A

            LOOP: LOAD B     -- Load the value at memory address B into register r
            COMPARE ZERO     -- Compare register r with memory[ZERO]
            JUMPEQ DONE      -- If B is zero, jump to DONE
            JUMPGT SUB       -- If B is negative, subtract A from PRODUCT B timees

            LOAD PRODUCT     -- Load current PRODUCT value into register r
            ADD A           -- Add the value at memory[A] to register r
            STORE PRODUCT    -- Store the new r back into memory[PRODUCT]

            DECREMENT B      -- Decrement memory[B]
            JUMP LOOP        -- Repeat until B becomes zero

            SUB: LOAD PRODUCT -- Load the value at memory[PRODUCT] into register r
            SUBTRACT A       -- Subtract the value at memory[A] from register r
            STORE PRODUCT    -- Store the new r back into memory[PRODUCT]

            INCREMENT B      -- Increment memory[B]
            JUMP LOOP        -- Repeat until B becomes zero


            DONE: OUT PRODUCT -- Output the final PRODUCT
            HALT             -- Halt execution
            .end

            A: .data -10       -- First factor
            B: .data -200        -- Second factor
            A_MAGNITUDE: .data 0 -- Magnitude of A
            ZERO: .data 0    -- Used for comparison
            TEMP: .data 0    -- Temporary storage
            PRODUCT: .data 0 -- Will hold the result
        "#;
        // print each line in binary with spaces between each 4 bits
        let output = Emulator::parse_program(program).unwrap();
        let mut emulator = Emulator::new();
        let mut emulator_one_step_behind = Emulator::new();
        emulator.memory[..output.0.len()]
            .copy_from_slice(&output.0.iter().map(|(x, y)| *y).collect::<Vec<_>>());
        emulator_one_step_behind.memory[..output.0.len()]
            .copy_from_slice(&output.0.into_iter().map(|(x, y)| y).collect::<Vec<_>>());
        let mut step = 0;

        let address_label_map = output
            .1
            .iter()
            .map(|(k, v)| (v, k))
            .collect::<HashMap<&u16, &String>>();
        while emulator.step() {
            println!("=== step {} ===", step);
            if emulator_one_step_behind.r != emulator.r {
                println!("[ r: {} ]", emulator.r);
            } else {
                println!("r: {}", emulator.r);
            }
            if emulator_one_step_behind.pc != emulator.pc {
                println!("[ pc: {} ]", emulator.pc);
            } else {
                println!("pc: {}", emulator.pc);
            }
            if emulator_one_step_behind.gt != emulator.gt {
                println!("[ gt: {} ]", emulator.gt);
            } else {
                println!("gt: {}", emulator.gt);
            }
            if emulator_one_step_behind.lt != emulator.lt {
                println!("[ lt: {} ]", emulator.lt);
            } else {
                println!("lt: {}", emulator.lt);
            }
            if emulator_one_step_behind.eq != emulator.eq {
                println!("[ eq: {} ]", emulator.eq);
            } else {
                println!("eq: {}", emulator.eq);
            }
            for output in &emulator.output {
                println!("> {}", output);
            }
            println!();
            for (i, x) in emulator.memory.iter().enumerate() {
                if *x != 0 || address_label_map.contains_key(&(i as u16)) {
                    if let Some(label) = address_label_map.get(&(i as u16)) {
                        print!("{}:\t\t", label);
                    } else {
                        print!("\t\t");
                    }

                    if i == emulator.pc as usize {
                        println!(
                            "{}: {:016b} ({}) <- PC",
                            i,
                            x,
                            util_instruction_to_string(*x)
                        );
                    } else {
                        if emulator_one_step_behind.memory[i] != *x {
                            println!(
                                "[ {} : {:016b} ({}) ]",
                                i,
                                x,
                                util_instruction_to_string(*x)
                            );
                        } else {
                            println!("{}: {:016b} ({})", i, x, util_instruction_to_string(*x));
                        }
                    }
                }
            }
            println!();
            step += 1;
            emulator_one_step_behind = emulator.clone();
        }
        println!("=== end state ===");

        println!("r: {}", emulator.r);
        println!("pc: {}", emulator.pc);
        for output in &emulator.output {
            println!("> {}", *output as i16);
        }

        assert_eq!(emulator.r, 0);
        assert_eq!(emulator.pc, 36);
        assert_eq!(emulator.output, vec![2000]);
    }
}
