use egui::ahash::HashMapExt;
use rustc_hash::FxHashMap as HashMap;

/// For the tape 00 is 0 11 is 1 and 10 is b (blank) with 01 being an invalid state
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TuringMachine {
    pub tape: Vec<u8>,
    /// 30 bits for the tape page and 2 bits for the tape index
    pub head: u32,
    pub state: u32,
    /// The rules are stored in a hashmap with the key being the state and the tape status
    pub rules: HashMap<(u32, u8), Rule>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Rule {
    pub write: u8,
    pub move_right: bool,
    pub next_state: u32,
}

impl TuringMachine {
    pub fn new() -> Self {
        Self {
            tape: vec![0xAA],
            head: 0,
            state: 0,
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(mut self, state: u32, status: u8, rule: Rule) {
        self.rules.insert((state, status), rule);
    }

    // outputs if halted
    pub fn step(&mut self) -> bool {
        let low_offset = (self.head & 0x00000003) << 1;
        let low_index_mask = (0b11000000 >> low_offset);
        // let rule = self.rules.iter().find(|rule| rule.state == self.state && rule.tape == self.tape[self.head >> 2 as usize] & low_index_mask);
        let rule = self.rules.get(&(
            self.state,
            (self.tape[(self.head >> 2) as usize] & low_index_mask) >> (6 - low_offset),
        ));
        if let Some(rule) = rule {
            self.tape[(self.head >> 2) as usize] = (self.tape[(self.head >> 2) as usize]
                & !low_index_mask)
                | (rule.write << (6 - low_offset));
            if rule.move_right == true {
                self.head += 1;
                if (self.head >> 2) as usize >= self.tape.len() {
                    self.tape.push(0xAA); // Blank (bbbb)
                }
            } else {
                if self.head == 0 {
                    self.tape.insert(0, 0xAA);
                    self.head = 3;
                } else {
                    self.head -= 1;
                }
            }
            self.state = rule.next_state;
            false
        } else {
            true
        }
    }
}
