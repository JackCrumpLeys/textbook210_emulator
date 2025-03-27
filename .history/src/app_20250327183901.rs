use std::{collections::BTreeSet, fmt::Display, vec::IntoIter};

use egui::{
    ahash::{HashMap, HashMapExt},
    OutputCommand, RichText,
};
use egui_tiles::SimplificationOptions;

use crate::emulator::{CpuState, Emulator, EmulatorCell};

pub trait Window: Default {
    fn render(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum Pane {
    BaseConverter(BaseConverter),
    Emulator(Box<EmulatorPane>),
}

#[derive(Debug, Clone, Default)]
pub struct BaseConverter {
    input: String,
    output_hist: Vec<String>,
    alphabet: String,
    base_in: u32,
    base_out: u32,
    case_sensitive: bool,
    uppercase: bool,
}

impl Window for BaseConverter {
    fn render(&mut self, ui: &mut egui::Ui) {
        ui.label("This is a base converter. Enter a number, select the input and output bases, adjust the alphabet, and click 'Convert' to see the result. You can also toggle case sensitivity and choose between uppercase and lowercase conversion.");

        if self.case_sensitive {
            ui.label(RichText::new("⚠ Note: Case sensitivity is enabled. ⚠")
                            .small()
                            .color(ui.visuals().warn_fg_color)).on_hover_text("Case sensitivity is enabled.  You can change this behavior by toggling the 'Case Sensitive' checkbox.");
        }
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.input);
            ui.label("->");
            if let Some(most_recent_output) = self.output_hist.last() {
                ui.label(most_recent_output);
            } else {
                ui.label("");
            }
            if ui.button("Convert").clicked() {
                // Call the stub function base_to_base
                if !self.case_sensitive {
                    if self.uppercase {
                        self.input = self.input.to_uppercase();
                    } else {
                        self.input = self.input.to_lowercase();
                    }
                }
                let output = base_to_base(self.base_in, self.base_out, &self.input, &self.alphabet);
                self.output_hist.push(output);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Alphabet:");
            ui.text_edit_singleline(&mut self.alphabet);
        });

        let max_base = self.alphabet.len() as u32;

        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.base_in, 2..=max_base));
            ui.add(egui::Slider::new(&mut self.base_out, 2..=max_base));
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.case_sensitive, "Case Sensitive");
            if !self.case_sensitive {
                ui.checkbox(&mut self.uppercase, "Uppercase");
            }
        });

        ui.separator();

        egui::CollapsingHeader::new("History")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for line in self.output_hist.iter() {
                        ui.label(line);
                    }
                });
            });
    }

    fn title(&self) -> String {
        "Base Converter".to_owned()
    }
}

fn base_to_base(base_in: u32, base_out: u32, input: &str, alphabet: impl Into<String>) -> String {
    let alphabet: String = alphabet.into();
    let mut output = String::new();
    let mut num = 0;
    let mut place = 1;
    for c in input.chars().rev() {
        let digit = match alphabet.find(c) {
            Some(d) => d as u32,
            None => {
                return "Invalid input".to_owned();
            }
        };
        num += digit * place;
        place *= base_in;
    }
    while num > 0 {
        let digit = num % base_out;
        num /= base_out;
        let c = match alphabet.chars().nth(digit as usize) {
            Some(c) => c,
            None => {
                return "Invalid input".to_owned();
            }
        };
        output.push(c);
    }
    if output == String::new() {
        output = alphabet.chars().next().unwrap().to_string();
    }
    output.chars().rev().collect()
}

#[derive(Debug, Clone)]
pub struct EmulatorPane {
    program: String,
    last_compiled: String,
    breakpoints: Vec<usize>,
    error: Option<(String, usize)>,
    emulator: Emulator,
    line_to_address: HashMap<usize, usize>,
    show_machine_code: bool,
    speed: u32,
    ticks_between_updates: u32,
    tick: u64,
    display_base: u32,
    instruction_fields: InstructionFields,
    input_stack: String, // getc is ripped off the start of the string
    shell_input: String,
    machine_code_base: u32,
}

impl Default for EmulatorPane {
    fn default() -> Self {
        Self {
            program: String::new(),
            last_compiled: String::new(),
            breakpoints: Vec::new(),
            error: None,
            emulator: Emulator::new(),
            line_to_address: HashMap::new(),
            show_machine_code: false,
            speed: 1,
            ticks_between_updates: 2,
            tick: 0,
            display_base: 16,
            instruction_fields: InstructionFields {
                dr: 0,
                sr1: 1,
                sr2: 2,
                imm5: 5,
                offset6: 6,
                offset9: 9,
                offset11: 11,
                base_r: 3,
                n_bit: true,
                z_bit: false,
                p_bit: true,
                trapvector: 0x25, // HALT by default
                imm_mode: false,
                jsr_mode: true,
            },
            input_stack: String::new(),
            shell_input: String::new(),
            machine_code_base: 16,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct InstructionFields {
    dr: u8,
    sr1: u8,
    sr2: u8,
    imm5: i8,
    offset6: i8,
    offset9: i16,
    offset11: i16,
    base_r: u8,
    n_bit: bool,
    z_bit: bool,
    p_bit: bool,
    trapvector: u8,
    imm_mode: bool,
    jsr_mode: bool,
}
impl Window for EmulatorPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        self.tick = self.tick.wrapping_add(1);
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::CollapsingHeader::new("LC-3 Emulator Help").show(ui, render_help_ui);
            egui::CollapsingHeader::new("LC-3 Instruction Reference")
                .show(ui, |ui| self.render_reference(ui));
            egui::CollapsingHeader::new("LC-3 Cheatsheet and Examples")
                .show(ui, render_cheatsheet_examples);

            ui.separator();

            // TEXT EDITOR
            egui::CollapsingHeader::new("Code Editor")
                .default_open(true)
                .show(ui, |ui| {
                    egui_code_editor::CodeEditor::default()
                        .with_syntax(
                            egui_code_editor::Syntax::new("lc3_assembly")
                                .with_comment(";")
                                .with_keywords(BTreeSet::from([
                                    "ADD", "AND", "BR", "BRN", "BRZ", "BRP", "BRNZ", "BRNP",
                                    "BRZP", "BRNZP", "JMP", "JSR", "JSRR", "LD", "LDI", "LDR",
                                    "LEA", "NOT", "RET", "RTI", "ST", "STI", "STR", "TRAP", "GETC",
                                    "OUT", "PUTS", "IN", "HALT",
                                ]))
                                .with_special(BTreeSet::from([
                                    ":", ".ORIG", ".FILL", ".BLKW", ".STRINGZ", ".END",
                                ]))
                                .with_case_sensitive(false),
                        )
                        .vscroll(false)
                        .with_theme(egui_code_editor::ColorTheme::SONOKAI)
                        .show(ui, &mut self.program);
                });

            ui.group(|ui| {
                ui.label("Execution Speed");
                ui.horizontal(|ui| {
                    ui.label("Clocks per update:");
                    ui.add(egui::Slider::new(&mut self.speed, 1..=1000).logarithmic(true));
                });
                ui.horizontal(|ui| {
                    ui.label("Update frequency:");
                    ui.add(
                        egui::Slider::new(&mut self.ticks_between_updates, 1..=100)
                            .text("ticks between updates")
                            .logarithmic(true),
                    );
                });
                ui.label("Higher speed values execute more instructions per update cycle.");
            });

            ui.horizontal(|ui| {
                ui.label("Input:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.input_stack)
                        .hint_text("Enter getc input stack here."),
                );
            });

            // BUTTONS
            // STEP - RUN - RESET - SUBMIT INPUT
            ui.horizontal(|ui| self.render_control_buttons_and_run_emulator(ui));

            // STATE
            // Show LC-3 registers and flags
            ui.horizontal(|ui| {
                ui.label("Display Base:");
                ui.add(egui::Slider::new(&mut self.display_base, 2..=36).integer());
            });

            egui::CollapsingHeader::new("Registers")
                .default_open(false)
                .show(ui, |ui| {
                    self.render_register_editor(ui);
                });

            egui::CollapsingHeader::new("Flags")
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.emulator.n.get() == 1, "N")
                            .clicked()
                        {
                            self.emulator.n.set(1);
                            self.emulator.z.set(0);
                            self.emulator.p.set(0);
                        }

                        if ui
                            .selectable_label(self.emulator.z.get() == 1, "Z")
                            .clicked()
                        {
                            self.emulator.z.set(1);
                            self.emulator.n.set(0);
                            self.emulator.p.set(0);
                        }

                        if ui
                            .selectable_label(self.emulator.p.get() == 1, "P")
                            .clicked()
                        {
                            self.emulator.p.set(1);
                            self.emulator.n.set(0);
                            self.emulator.z.set(0);
                        }
                    });
                });

            egui::CollapsingHeader::new("Processor Cycle")
                .default_open(false)
                .show(ui, |ui| self.render_cycle_view(ui));

            ui.separator();

            egui::CollapsingHeader::new("Output")
                .default_open(true)
                .show(ui, |ui| {
                    self.render_output(ui);
                });

            if self.emulator.await_input.is_some() && self.emulator.await_input.unwrap() {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("TRAP IN Waiting for input:")
                            .strong()
                            .color(egui::Color32::YELLOW),
                    );
                    ui.horizontal(|ui| {
                        let mut input = self.shell_input.clone();
                        if ui.text_edit_singleline(&mut input).changed() {
                            self.shell_input = input;
                        }

                        if ui.button("Submit").clicked() && !self.shell_input.is_empty() {
                            let c = self.shell_input.chars().next().unwrap();
                            self.emulator.r[0].set(c as u16);
                            self.emulator.output.push(c); // Echo the character
                            self.emulator.await_input = None;
                            self.shell_input.clear();
                        }
                    });
                    ui.label(
                        "Type a character and click Submit. The character will be stored in R0.",
                    );
                });
            }

            self.compiled_view(ui);
        });
    }

    fn title(&self) -> String {
        "Emulator".to_owned()
    }
}

fn render_cheatsheet_examples(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Instruction Reference")
                .heading()
                .strong()
                .color(egui::Color32::LIGHT_YELLOW),
        );
        ui.label("Quick reference for common LC-3 instructions and syntax.");

        ui.label(RichText::new("Arithmetic/Logic:").strong());
        ui.label("ADD R1, R2, R3    ; R1 = R2 + R3");
        ui.label("ADD R1, R2, #5    ; R1 = R2 + 5 (immediate)");
        ui.label("AND R1, R2, R3    ; R1 = R2 & R3 (bitwise)");
        ui.label("NOT R1, R2        ; R1 = ~R2 (1's complement)");

        ui.add_space(4.0);
        ui.label(RichText::new("Data Movement:").strong());
        ui.label("LD  R1, LABEL     ; R1 = Mem[PC+offset]");
        ui.label("LDI R1, LABEL     ; R1 = Mem[Mem[PC+offset]]");
        ui.label("LDR R1, R2, #5    ; R1 = Mem[R2+5]");
        ui.label("LEA R1, LABEL     ; R1 = PC+offset");
        ui.label("ST  R1, LABEL     ; Mem[PC+offset] = R1");
        ui.label("STI R1, LABEL     ; Mem[Mem[PC+offset]] = R1");
        ui.label("STR R1, R2, #5    ; Mem[R2+5] = R1");

        ui.add_space(4.0);
        ui.label(RichText::new("Control Flow:").strong());
        ui.label("BR  LABEL         ; Branch always");
        ui.label("BRn LABEL         ; Branch if negative");
        ui.label("BRz LABEL         ; Branch if zero");
        ui.label("BRp LABEL         ; Branch if positive");
        ui.label("JMP R1            ; PC = R1");
        ui.label("JSR LABEL         ; Jump to subroutine (PC+offset)");
        ui.label("JSRR R1           ; Jump to subroutine (R1)");
        ui.label("RET               ; Return (PC = R7)");

        ui.add_space(4.0);
        ui.label(RichText::new("System Operations:").strong());
        ui.label("TRAP x20          ; GETC (char -> R0)");
        ui.label("TRAP x21          ; OUT (R0 -> display)");
        ui.label("TRAP x22          ; PUTS (string at R0)");
        ui.label("TRAP x23          ; IN (prompt & input -> R0)");
        ui.label("TRAP x25          ; HALT (stop program)");

        ui.add_space(4.0);
        ui.label(RichText::new("Directives:").strong());
        ui.label(".ORIG x3000       ; Program origin (starting address)");
        ui.label(".FILL #10         ; Insert value (decimal, hex, or label)");
        ui.label(".BLKW 5           ; Reserve 5 memory locations");
        ui.label(".STRINGZ \"Text\"  ; Null-terminated string");
        ui.label(".END              ; End of program");

        ui.add_space(4.0);
        ui.label(RichText::new("Number Formats:").strong());
        ui.label("#10               ; Decimal");
        ui.label("x10A2             ; Hexadecimal");
        ui.label("LABEL             ; Label reference");
    });
    ui.add_space(8.0);
    ui.separator();
    ui.label(
        RichText::new("LC-3 Sample Programs")
            .heading()
            .strong()
            .color(egui::Color32::KHAKI),
    );
    ui.label("Common patterns and examples for LC-3 assembly programming.");
    ui.group(|ui| {
        ui.label(
            RichText::new("Hello World")
                .heading()
                .color(egui::Color32::GOLD),
        );
        ui.code(
            r#"; Simple Hello World program
.ORIG x3000
LEA R0, MESSAGE    ; Load the address of the message
PUTS               ; Output the string
HALT               ; Halt the program

MESSAGE: .STRINGZ "Hello, World!"
.END"#,
        );
    });
    ui.add_space(4.0);
    ui.group(|ui| {
        ui.label(
            RichText::new("Input and Echo")
                .heading()
                .color(egui::Color32::GOLD),
        );
        ui.code(
            r#"; Program that gets a character and echoes it
.ORIG x3000
LOOP:   GETC                ; Read a character from keyboard
        OUT                 ; Echo the character
        BRnzp LOOP          ; Repeat
.END"#,
        );
    });
    ui.add_space(4.0);
    ui.group(|ui| {
        ui.label(
            RichText::new("Counter Loop")
                .heading()
                .color(egui::Color32::GOLD),
        );
        ui.code(
            r#"; Simple counter from 0 to 9
.ORIG x3000
        AND R2, R2, #0      ; Clear R2 (our counter)
        LD R1, TEN          ; Load the value 10 into R1
        LD R4, ZERO_ASCII   ; Load the ASCII value for '0' (so we can print nums)

LOOP:   ADD R0, R2, R4      ; Convert to ASCII
        OUT                 ; Print digit
        ADD R2, R2, #1      ; Increment counter
        ADD R3, R2, R1      ; Compare with 10
        BRn LOOP            ; Loop if not 10
        HALT                ; Stop when done

TEN:    .FILL #-10          ; Negative 10 for comparison
ZERO_ASCII: .FILL #48       ; ASCII value for '0'
.END"#,
        );
    });
    ui.add_space(4.0);
    ui.group(|ui| {
        ui.label(
            RichText::new("Subroutine Example")
                .heading()
                .color(egui::Color32::GOLD),
        );
        ui.code(
            r#"; Program using a subroutine
.ORIG x3000
        JSR SUBROUTINE      ; Call subroutine
        HALT                ; End program

SUBROUTINE:                 ; Label with colon
        ST R7, SAVE_R7      ; Save return address
        LEA R0, MESSAGE     ; Load message
        PUTS                ; Print it
        LD R7, SAVE_R7      ; Restore return address
        RET                 ; Return from subroutine (using the address in R7)

SAVE_R7: .BLKW 1            ; Storage for R7 (note the colon)
MESSAGE: .STRINGZ "Called from subroutine!"  ; Note the colon
.END"#,
        );
    });
    ui.add_space(4.0);
    ui.group(|ui| {
        ui.label(
            RichText::new("Array Manipulation")
                .heading()
                .color(egui::Color32::GOLD),
        );
        ui.code(
            r#"; Program that initializes an array and sums it
.ORIG x3000
        AND R0, R0, #0      ; Clear R0 (sum)
        LEA R1, ARRAY       ; R1 points to start of array
        LD R2, COUNT        ; R2 holds counter

LOOP:   LDR R3, R1, #0      ; Load array element
        ADD R0, R0, R3      ; Add to sum
        ADD R1, R1, #1      ; Point to next element
        ADD R2, R2, #-1     ; Decrement counter
        BRp LOOP            ; Repeat if more elements
        HALT                ; End program

COUNT:  .FILL #5            ; Array size
ARRAY:  .FILL #10           ; Array values
        .FILL #20
        .FILL #30
        .FILL #40
        .FILL #50
.END"#,
        );
    });
    ui.add_space(4.0);
}

impl EmulatorPane {
    fn render_reference(&mut self, ui: &mut egui::Ui) {
        let instruction_fields = &mut self.instruction_fields;
        let format_binary =
            |value: u16, width: usize| -> String { format!("{:0width$b}", value, width = width) };
        let format_hex = |value: u16| -> String { format!("0x{:04X}", value) };
        let register_selector = |ui: &mut egui::Ui, value: &mut u8, label: &str| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).strong());
                ui.add(egui::DragValue::new(value).range(0..=7).speed(0.1))
                    .on_hover_text("Register value (0-7)");
                ui.label(format!("R{}", value));
            })
        };
        let immediate_selector = |ui: &mut egui::Ui, value: &mut i8, bits: u8, label: &str| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).strong());
                let min = -(1 << (bits - 1));
                let max = (1 << (bits - 1)) - 1;
                ui.add(egui::DragValue::new(value).range(min..=max).speed(0.1))
                    .on_hover_text(format!("{}-bit immediate value", bits));
                ui.label(format!("#{}", value));
            })
        };
        let offset_selector = |ui: &mut egui::Ui, value: &mut i16, bits: u8, label: &str| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).strong());
                let min = -(1 << (bits - 1));
                let max = (1 << (bits - 1)) - 1;
                ui.add(egui::DragValue::new(value).range(min..=max).speed(0.1))
                    .on_hover_text(format!("{}-bit offset value", bits));
                ui.label(format!("PC+{}", value));
            })
        };
        let instruction_layout = |ui: &mut egui::Ui, desc: &str, color: egui::Color32| {
            ui.label(
                RichText::new(desc)
                    .monospace()
                    .background_color(egui::Color32::from_black_alpha(180))
                    .color(color),
            );
        };
        ui.add(egui::Label::new(
            RichText::new("LC-3 Assembly Instructions")
                .heading()
                .strong()
                .color(ui.visuals().text_color()),
        ));
        ui.label("Select an instruction category to explore. Adjust instruction fields to see how they affect binary representation.");
        ui.add_space(4.0);
        egui::CollapsingHeader::new("Arithmetic & Logic")
            .id_salt("arithmetic_logic")
            .show(ui, |ui| {
                // ADD
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("ADD - Addition")
                        .heading()
                        .strong()
                        .color(egui::Color32::LIGHT_BLUE),
                );
                ui.label("Adds two values and stores the result in a destination register.");

                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut instruction_fields.imm_mode,
                        RichText::new("Immediate mode").strong(),
                    );
                });

                ui.group(|ui| {
                    if instruction_fields.imm_mode {
                        ui.label(
                            RichText::new("ADD DR, SR1, #imm5")
                                .monospace()
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                        ui.label(
                            RichText::new("Adds SR1 and immediate value, stores in DR").italics(),
                        );

                        register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                        register_selector(ui, &mut instruction_fields.sr1, "Source Reg 1:");
                        immediate_selector(ui, &mut instruction_fields.imm5, 5, "Immediate Value:");

                        let pseudo_code = format!(
                            "DR = R{} = R{} + {} = {}",
                            instruction_fields.dr,
                            instruction_fields.sr1,
                            instruction_fields.imm5,
                            format_hex(
                                (instruction_fields.sr1 as i16 + instruction_fields.imm5 as i16)
                                    as u16
                            )
                        );

                        ui.label(
                            RichText::new(pseudo_code)
                                .monospace()
                                .color(egui::Color32::YELLOW),
                        );

                        let add_imm = (0b0001 << 12) | // Opcode
                                     ((instruction_fields.dr as u16) << 9) | // DR
                                     ((instruction_fields.sr1 as u16) << 6) | // SR1
                                     (1 << 5) | // Immediate mode bit
                                     (instruction_fields.imm5 as u16 & 0x1F); // IMM5

                        egui::CollapsingHeader::new("Binary Representation")
                            .id_salt("add_binary")
                            .show(ui, |ui| {
                                instruction_layout(
                                    ui,
                                    "Layout: 0001 | DR | SR1 | 1 | IMM5",
                                    egui::Color32::GREEN,
                                );
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("0001").monospace().color(egui::Color32::RED),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.dr as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::from_rgb(33, 78, 211)),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.sr1 as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::GREEN),
                                    );
                                    ui.label(
                                        RichText::new("1").monospace().color(egui::Color32::YELLOW),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            instruction_fields.imm5 as u16 & 0x1F,
                                            5,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::LIGHT_BLUE),
                                    );
                                });
                                ui.label(
                                    RichText::new(format!(
                                        "Binary: {}",
                                        format_binary(add_imm, 16)
                                    ))
                                    .monospace(),
                                );
                                ui.label(
                                    RichText::new(format!("Hex: {}", format_hex(add_imm)))
                                        .monospace(),
                                );
                            });
                    } else {
                        ui.label(
                            RichText::new("ADD DR, SR1, SR2")
                                .monospace()
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                        ui.label(RichText::new("Adds SR1 and SR2, stores in DR").italics());

                        register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                        register_selector(ui, &mut instruction_fields.sr1, "Source Reg 1:");
                        register_selector(ui, &mut instruction_fields.sr2, "Source Reg 2:");

                        let pseudo_code = format!(
                            "DR = R{} = R{} + R{}",
                            instruction_fields.dr, instruction_fields.sr1, instruction_fields.sr2
                        );

                        ui.label(
                            RichText::new(pseudo_code)
                                .monospace()
                                .color(egui::Color32::YELLOW),
                        );

                        let add_reg = ((0b0001 << 12) | // Opcode
                                     ((instruction_fields.dr as u16) << 9) | // DR
                                     ((instruction_fields.sr1 as u16) << 6)) | // Unused bits
                                     (instruction_fields.sr2 as u16 & 0x7); // SR2

                        egui::CollapsingHeader::new("Binary Representation")
                            .id_salt("add_binary")
                            .show(ui, |ui| {
                                instruction_layout(
                                    ui,
                                    "Layout: 0001 | DR | SR1 | 0 | 00 | SR2",
                                    egui::Color32::GREEN,
                                );
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("0001").monospace().color(egui::Color32::RED),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.dr as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::from_rgb(33, 78, 211)),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.sr1 as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::GREEN),
                                    );
                                    ui.label(
                                        RichText::new("0").monospace().color(egui::Color32::YELLOW),
                                    );
                                    ui.label(
                                        RichText::new("00")
                                            .monospace()
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.sr2 as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::LIGHT_BLUE),
                                    );
                                });
                                ui.label(
                                    RichText::new(format!(
                                        "Binary: {}",
                                        format_binary(add_reg, 16)
                                    ))
                                    .monospace(),
                                );
                                ui.label(
                                    RichText::new(format!("Hex: {}", format_hex(add_reg)))
                                        .monospace(),
                                );
                            });
                    }
                });

                ui.label(
                    RichText::new("Sets condition codes: N, Z, P")
                        .small()
                        .italics(),
                );

                // AND
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("AND - Bitwise AND")
                        .heading()
                        .strong()
                        .color(egui::Color32::LIGHT_BLUE),
                );
                ui.label("Performs bitwise AND of two values and stores the result.");

                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut instruction_fields.imm_mode,
                        RichText::new("Immediate mode").strong(),
                    );
                });

                ui.group(|ui| {
                    if instruction_fields.imm_mode {
                        ui.label(
                            RichText::new("AND DR, SR1, #imm5")
                                .monospace()
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                        ui.label(
                            RichText::new("Bitwise ANDs SR1 and immediate value, stores in DR")
                                .italics(),
                        );

                        register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                        register_selector(ui, &mut instruction_fields.sr1, "Source Reg 1:");
                        immediate_selector(ui, &mut instruction_fields.imm5, 5, "Immediate Value:");

                        let pseudo_code = format!(
                            "DR = R{} = R{} & {} = {}",
                            instruction_fields.dr,
                            instruction_fields.sr1,
                            instruction_fields.imm5,
                            format_hex(
                                (instruction_fields.sr1 as u16) & (instruction_fields.imm5 as u16)
                            )
                        );

                        ui.label(
                            RichText::new(pseudo_code)
                                .monospace()
                                .color(egui::Color32::YELLOW),
                        );

                        let and_imm = (0b0101 << 12) | // Opcode
                                     ((instruction_fields.dr as u16) << 9) | // DR
                                     ((instruction_fields.sr1 as u16) << 6) | // SR1
                                     (1 << 5) | // Immediate mode bit
                                     (instruction_fields.imm5 as u16 & 0x1F); // IMM5

                        egui::CollapsingHeader::new("Binary Representation")
                            .id_salt("and_binary")
                            .show(ui, |ui| {
                                instruction_layout(
                                    ui,
                                    "Layout: 0101 | DR | SR1 | 1 | IMM5",
                                    egui::Color32::GREEN,
                                );
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("0101").monospace().color(egui::Color32::RED),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.dr as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::from_rgb(33, 78, 211)),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.sr1 as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::GREEN),
                                    );
                                    ui.label(
                                        RichText::new("1").monospace().color(egui::Color32::YELLOW),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            instruction_fields.imm5 as u16 & 0x1F,
                                            5,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::LIGHT_BLUE),
                                    );
                                });
                                ui.label(
                                    RichText::new(format!(
                                        "Binary: {}",
                                        format_binary(and_imm, 16)
                                    ))
                                    .monospace(),
                                );
                                ui.label(
                                    RichText::new(format!("Hex: {}", format_hex(and_imm)))
                                        .monospace(),
                                );
                            });
                    } else {
                        ui.label(
                            RichText::new("AND DR, SR1, SR2")
                                .monospace()
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                        ui.label(RichText::new("Bitwise ANDs SR1 and SR2, stores in DR").italics());

                        register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                        register_selector(ui, &mut instruction_fields.sr1, "Source Reg 1:");
                        register_selector(ui, &mut instruction_fields.sr2, "Source Reg 2:");

                        let pseudo_code = format!(
                            "DR = R{} = R{} & R{}",
                            instruction_fields.dr, instruction_fields.sr1, instruction_fields.sr2
                        );

                        ui.label(
                            RichText::new(pseudo_code)
                                .monospace()
                                .color(egui::Color32::YELLOW),
                        );

                        let and_reg = ((0b0101 << 12) | // Opcode
                                     ((instruction_fields.dr as u16) << 9) | // DR
                                     ((instruction_fields.sr1 as u16) << 6)) | // Unused bits
                                     (instruction_fields.sr2 as u16 & 0x7); // SR2

                        egui::CollapsingHeader::new("Binary Representation")
                            .id_salt("and_binary")
                            .show(ui, |ui| {
                                instruction_layout(
                                    ui,
                                    "Layout: 0101 | DR | SR1 | 0 | 00 | SR2",
                                    egui::Color32::GREEN,
                                );
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("0101").monospace().color(egui::Color32::RED),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.dr as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::from_rgb(33, 78, 211)),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.sr1 as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::GREEN),
                                    );
                                    ui.label(
                                        RichText::new("0").monospace().color(egui::Color32::YELLOW),
                                    );
                                    ui.label(
                                        RichText::new("00")
                                            .monospace()
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.sr2 as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::LIGHT_BLUE),
                                    );
                                });
                                ui.label(
                                    RichText::new(format!(
                                        "Binary: {}",
                                        format_binary(and_reg, 16)
                                    ))
                                    .monospace(),
                                );
                                ui.label(
                                    RichText::new(format!("Hex: {}", format_hex(and_reg)))
                                        .monospace(),
                                );
                            });
                    }
                });

                ui.label(
                    RichText::new("Sets condition codes: N, Z, P")
                        .small()
                        .italics(),
                );

                // NOT
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("NOT - Bitwise NOT")
                        .heading()
                        .strong()
                        .color(egui::Color32::LIGHT_BLUE),
                );
                ui.label("Performs bitwise NOT (complement) of a value.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("NOT DR, SR")
                            .monospace()
                            .color(egui::Color32::LIGHT_BLUE),
                    );
                    ui.label(RichText::new("Bitwise NOTs SR, stores in DR").italics());

                    register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                    register_selector(ui, &mut instruction_fields.sr1, "Source Reg:");

                    let pseudo_code = format!(
                        "DR = R{} = ~R{}",
                        instruction_fields.dr, instruction_fields.sr1
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let not_instr = (0b1001 << 12) | // Opcode
                                 ((instruction_fields.dr as u16) << 9) | // DR
                                 ((instruction_fields.sr1 as u16) << 6) | // SR
                                 0x3F; // Constant field (111111)

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("not_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 1001 | DR | SR | 111111",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("1001").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.dr as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.sr1 as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                                ui.label(
                                    RichText::new("111111")
                                        .monospace()
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(not_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(not_instr)))
                                    .monospace(),
                            );
                        });
                });

                ui.label(
                    RichText::new("Sets condition codes: N, Z, P")
                        .small()
                        .italics(),
                );
            });
        egui::CollapsingHeader::new("Data Movement")
            .id_salt("data_movement")
            .show(ui, |ui| {
                // LD
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("LD - Load")
                        .heading()
                        .strong()
                        .color(egui::Color32::GOLD),
                );
                ui.label("Loads a value from memory into a register.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("LD DR, LABEL")
                            .monospace()
                            .color(egui::Color32::GOLD),
                    );
                    ui.label(
                        RichText::new(
                            "PC-relative addressing: Loads from memory at PC+offset into DR",
                        )
                        .italics(),
                    );

                    register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                    offset_selector(ui, &mut instruction_fields.offset9, 9, "PC Offset:");

                    let pseudo_code = format!(
                        "DR = R{} = MEM[PC + {}]",
                        instruction_fields.dr, instruction_fields.offset9
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let ld_instr = (0b0010 << 12) | // Opcode
                                 ((instruction_fields.dr as u16) << 9) | // DR
                                 (instruction_fields.offset9 as u16 & 0x1FF); // PCoffset9

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("ld_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 0010 | DR | PCoffset9",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("0010").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.dr as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        instruction_fields.offset9 as u16 & 0x1FF,
                                        9,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(ld_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(ld_instr))).monospace(),
                            );
                        });
                });

                ui.label(
                    RichText::new("Sets condition codes: N, Z, P")
                        .small()
                        .italics(),
                );

                // LDI
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("LDI - Load Indirect")
                        .heading()
                        .strong()
                        .color(egui::Color32::GOLD),
                );
                ui.label("Loads a value using a pointer stored in memory.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("LDI DR, LABEL")
                            .monospace()
                            .color(egui::Color32::GOLD),
                    );
                    ui.label(
                        RichText::new(
                            "Loads value from memory at address stored at PC+offset into DR",
                        )
                        .italics(),
                    );

                    register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                    offset_selector(ui, &mut instruction_fields.offset9, 9, "PC Offset:");

                    let pseudo_code = format!(
                        "DR = R{} = MEM[MEM[PC + {}]]",
                        instruction_fields.dr, instruction_fields.offset9
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let ldi_instr = (0b1010 << 12) | // Opcode
                                  ((instruction_fields.dr as u16) << 9) | // DR
                                  (instruction_fields.offset9 as u16 & 0x1FF); // PCoffset9

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("ldi_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 1010 | DR | PCoffset9",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("1010").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.dr as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        instruction_fields.offset9 as u16 & 0x1FF,
                                        9,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(ldi_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(ldi_instr)))
                                    .monospace(),
                            );
                        });
                });

                ui.label(
                    RichText::new("Sets condition codes: N, Z, P")
                        .small()
                        .italics(),
                );

                // LDR
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("LDR - Load Register")
                        .heading()
                        .strong()
                        .color(egui::Color32::GOLD),
                );
                ui.label("Loads a value using base register + offset addressing.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("LDR DR, BaseR, #offset6")
                            .monospace()
                            .color(egui::Color32::GOLD),
                    );
                    ui.label(RichText::new("Loads from memory at BaseR+offset into DR").italics());

                    register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                    register_selector(ui, &mut instruction_fields.base_r, "Base Reg:");
                    immediate_selector(ui, &mut instruction_fields.offset6, 6, "Offset:");

                    let pseudo_code = format!(
                        "DR = R{} = MEM[R{} + {}]",
                        instruction_fields.dr,
                        instruction_fields.base_r,
                        instruction_fields.offset6
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let ldr_instr = (0b0110 << 12) | // Opcode
                                  ((instruction_fields.dr as u16) << 9) | // DR
                                  ((instruction_fields.base_r as u16) << 6) | // BaseR
                                  (instruction_fields.offset6 as u16 & 0x3F); // offset6

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("ldr_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 0110 | DR | BaseR | offset6",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("0110").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.dr as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.base_r as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        instruction_fields.offset6 as u16 & 0x3F,
                                        6,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::YELLOW),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(ldr_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(ldr_instr)))
                                    .monospace(),
                            );
                        });
                });

                ui.label(
                    RichText::new("Sets condition codes: N, Z, P")
                        .small()
                        .italics(),
                );

                // LEA
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("LEA - Load Effective Address")
                        .heading()
                        .strong()
                        .color(egui::Color32::GOLD),
                );
                ui.label("Loads the address of a label into a register.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("LEA DR, LABEL")
                            .monospace()
                            .color(egui::Color32::GOLD),
                    );
                    ui.label(RichText::new("Loads effective address PC+offset into DR").italics());

                    register_selector(ui, &mut instruction_fields.dr, "Destination Reg:");
                    offset_selector(ui, &mut instruction_fields.offset9, 9, "PC Offset:");

                    let pseudo_code = format!(
                        "DR = R{} = PC + {}",
                        instruction_fields.dr, instruction_fields.offset9
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let lea_instr = (0b1110 << 12) | // Opcode
                                  ((instruction_fields.dr as u16) << 9) | // DR
                                  (instruction_fields.offset9 as u16 & 0x1FF); // PCoffset9

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("lea_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 1110 | DR | PCoffset9",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("1110").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.dr as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        instruction_fields.offset9 as u16 & 0x1FF,
                                        9,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(lea_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(lea_instr)))
                                    .monospace(),
                            );
                        });
                });

                ui.label(
                    RichText::new("Sets condition codes: N, Z, P")
                        .small()
                        .italics(),
                );

                // ST
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("ST - Store")
                        .heading()
                        .strong()
                        .color(egui::Color32::GOLD),
                );
                ui.label("Stores a register value into memory.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("ST SR, LABEL")
                            .monospace()
                            .color(egui::Color32::GOLD),
                    );
                    ui.label(RichText::new("Stores SR into memory at PC+offset").italics());

                    register_selector(ui, &mut instruction_fields.sr1, "Source Reg:");
                    offset_selector(ui, &mut instruction_fields.offset9, 9, "PC Offset:");

                    let pseudo_code = format!(
                        "MEM[PC + {}] = SR = R{}",
                        instruction_fields.offset9, instruction_fields.sr1
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let st_instr = (0b0011 << 12) | // Opcode
                                 ((instruction_fields.sr1 as u16) << 9) | // SR
                                 (instruction_fields.offset9 as u16 & 0x1FF); // PCoffset9

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("st_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 0011 | SR | PCoffset9",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("0011").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.sr1 as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        instruction_fields.offset9 as u16 & 0x1FF,
                                        9,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(st_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(st_instr))).monospace(),
                            );
                        });
                });

                // STI
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("STI - Store Indirect")
                        .heading()
                        .strong()
                        .color(egui::Color32::GOLD),
                );
                ui.label("Stores a register value using a pointer in memory.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("STI SR, LABEL")
                            .monospace()
                            .color(egui::Color32::GOLD),
                    );
                    ui.label(
                        RichText::new("Stores SR into memory at address stored at PC+offset")
                            .italics(),
                    );

                    register_selector(ui, &mut instruction_fields.sr1, "Source Reg:");
                    offset_selector(ui, &mut instruction_fields.offset9, 9, "PC Offset:");

                    let pseudo_code = format!(
                        "MEM[MEM[PC + {}]] = SR = R{}",
                        instruction_fields.offset9, instruction_fields.sr1
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let sti_instr = (0b1011 << 12) | // Opcode
                                  ((instruction_fields.sr1 as u16) << 9) | // SR
                                  (instruction_fields.offset9 as u16 & 0x1FF); // PCoffset9

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("sti_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 1011 | SR | PCoffset9",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("1011").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.sr1 as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        instruction_fields.offset9 as u16 & 0x1FF,
                                        9,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(sti_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(sti_instr)))
                                    .monospace(),
                            );
                        });
                });

                // STR
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("STR - Store Register")
                        .heading()
                        .strong()
                        .color(egui::Color32::GOLD),
                );
                ui.label("Stores a value using base register + offset addressing.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("STR SR, BaseR, #offset6")
                            .monospace()
                            .color(egui::Color32::GOLD),
                    );
                    ui.label(RichText::new("Stores SR into memory at BaseR+offset").italics());

                    register_selector(ui, &mut instruction_fields.sr1, "Source Reg:");
                    register_selector(ui, &mut instruction_fields.base_r, "Base Reg:");
                    immediate_selector(ui, &mut instruction_fields.offset6, 6, "Offset:");

                    let pseudo_code = format!(
                        "MEM[R{} + {}] = SR = R{}",
                        instruction_fields.base_r,
                        instruction_fields.offset6,
                        instruction_fields.sr1
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let str_instr = (0b0111 << 12) | // Opcode
                                  ((instruction_fields.sr1 as u16) << 9) | // SR
                                  ((instruction_fields.base_r as u16) << 6) | // BaseR
                                  (instruction_fields.offset6 as u16 & 0x3F); // offset6

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("str_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 0111 | SR | BaseR | offset6",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("0111").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.sr1 as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.base_r as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        instruction_fields.offset6 as u16 & 0x3F,
                                        6,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::YELLOW),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(str_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(str_instr)))
                                    .monospace(),
                            );
                        });
                });
            });
        egui::CollapsingHeader::new("Control Flow")
            .id_salt("control_flow")
            .show(ui, |ui| {
                // BR
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("BR/BRn/BRz/BRp - Conditional Branch")
                        .heading()
                        .strong()
                        .color(egui::Color32::LIGHT_RED),
                );
                ui.label("Branches to a label if condition codes match.");

                ui.group(|ui| {
                    ui.label(
                        RichText::new("BRnzp LABEL")
                            .monospace()
                            .color(egui::Color32::LIGHT_RED),
                    );
                    ui.label(
                        RichText::new("Branches to PC+offset if specified condition codes match")
                            .italics(),
                    );

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Condition Flags:").strong());
                        ui.checkbox(&mut instruction_fields.n_bit, "N (negative)");
                        ui.checkbox(&mut instruction_fields.z_bit, "Z (zero)");
                        ui.checkbox(&mut instruction_fields.p_bit, "P (positive)");
                    });

                    offset_selector(ui, &mut instruction_fields.offset9, 9, "PC Offset:");

                    let pseudo_code = format!(
                        "if ({}{}{}) PC = PC + {}",
                        if instruction_fields.n_bit { "N=1 " } else { "" },
                        if instruction_fields.z_bit { "Z=1 " } else { "" },
                        if instruction_fields.p_bit { "P=1" } else { "" },
                        instruction_fields.offset9
                    );

                    ui.label(
                        RichText::new(pseudo_code)
                            .monospace()
                            .color(egui::Color32::YELLOW),
                    );

                    let br_instr = ((instruction_fields.n_bit as u16) << 11) | // N flag
                                 ((instruction_fields.z_bit as u16) << 10) | // Z flag
                                 ((instruction_fields.p_bit as u16) << 9) | // P flag
                                 (instruction_fields.offset9 as u16 & 0x1FF); // PCoffset9

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("br_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 0000 | n | z | p | PCoffset9",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("0000").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new(
                                        (if instruction_fields.n_bit { "1" } else { "0" })
                                            .to_string(),
                                    )
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new(
                                        (if instruction_fields.z_bit { "1" } else { "0" })
                                            .to_string(),
                                    )
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                                );
                                ui.label(
                                    RichText::new(
                                        (if instruction_fields.p_bit { "1" } else { "0" })
                                            .to_string(),
                                    )
                                    .monospace()
                                    .color(egui::Color32::YELLOW),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        instruction_fields.offset9 as u16 & 0x1FF,
                                        9,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::LIGHT_BLUE),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(br_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(br_instr))).monospace(),
                            );
                        });
                });

                // JMP/RET
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("JMP/RET - Jump")
                        .heading()
                        .strong()
                        .color(egui::Color32::LIGHT_RED),
                );
                ui.label("Jumps to address in a register.");

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let mut ret_mode = instruction_fields.base_r == 7;
                        if ui.checkbox(&mut ret_mode, "RET").clicked() {
                            if ret_mode {
                                instruction_fields.base_r = 7; // Make it RET
                            } else {
                                instruction_fields.base_r = 0; // Make it JMP
                            }
                        }
                    });

                    if instruction_fields.base_r == 7 {
                        ui.label(
                            RichText::new("RET")
                                .monospace()
                                .color(egui::Color32::LIGHT_RED),
                        );
                        ui.label(
                            RichText::new("Returns from subroutine - jumps to address in R7")
                                .italics(),
                        );

                        let pseudo_code = "PC = R7";
                        ui.label(
                            RichText::new(pseudo_code)
                                .monospace()
                                .color(egui::Color32::YELLOW),
                        );
                    } else {
                        ui.label(
                            RichText::new("JMP BaseR")
                                .monospace()
                                .color(egui::Color32::LIGHT_RED),
                        );
                        ui.label(RichText::new("Jumps to address in BaseR").italics());

                        register_selector(ui, &mut instruction_fields.base_r, "Base Reg:");

                        let pseudo_code = format!("PC = BaseR = R{}", instruction_fields.base_r);
                        ui.label(
                            RichText::new(pseudo_code)
                                .monospace()
                                .color(egui::Color32::YELLOW),
                        );
                    }

                    let jmp_instr = (0b1100 << 12) | // Unused bits
                                 ((instruction_fields.base_r as u16) << 6); // Unused bits

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("jmp_binary")
                        .show(ui, |ui| {
                            instruction_layout(
                                ui,
                                "Layout: 1100 | 000 | BaseR | 000000",
                                egui::Color32::GREEN,
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("1100").monospace().color(egui::Color32::RED),
                                );
                                ui.label(
                                    RichText::new("000").monospace().color(egui::Color32::GRAY),
                                );
                                ui.label(
                                    RichText::new(format_binary(
                                        (instruction_fields.base_r as u16) & 0x7,
                                        3,
                                    ))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(33, 78, 211)),
                                );
                                ui.label(
                                    RichText::new("000000")
                                        .monospace()
                                        .color(egui::Color32::GRAY),
                                );
                            });
                            ui.label(
                                RichText::new(format!("Binary: {}", format_binary(jmp_instr, 16)))
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("Hex: {}", format_hex(jmp_instr)))
                                    .monospace(),
                            );
                        });
                });

                // JSR/JSRR
                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("JSR/JSRR - Jump to Subroutine")
                        .heading()
                        .strong()
                        .color(egui::Color32::LIGHT_RED),
                );
                ui.label("Jumps to a subroutine, saving return address in R7.");

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut instruction_fields.jsr_mode, "JSR mode (vs JSRR)");
                    });

                    if instruction_fields.jsr_mode {
                        ui.label(
                            RichText::new("JSR LABEL")
                                .monospace()
                                .color(egui::Color32::LIGHT_RED),
                        );
                        ui.label(
                            RichText::new("Jumps to PC+offset, saving return address in R7")
                                .italics(),
                        );

                        offset_selector(ui, &mut instruction_fields.offset11, 11, "PC Offset:");

                        let pseudo_code =
                            format!("R7 = PC\nPC = PC + {}", instruction_fields.offset11);
                        ui.label(
                            RichText::new(pseudo_code)
                                .monospace()
                                .color(egui::Color32::YELLOW),
                        );

                        let jsr_instr = (0b0100 << 12) | // Opcode
                                     (1 << 11) | // JSR bit = 1
                                     (instruction_fields.offset11 as u16 & 0x7FF); // PCoffset11

                        egui::CollapsingHeader::new("Binary Representation")
                            .id_salt("jsr_binary")
                            .show(ui, |ui| {
                                instruction_layout(
                                    ui,
                                    "Layout: 0100 | 1 | PCoffset11",
                                    egui::Color32::GREEN,
                                );
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("0100").monospace().color(egui::Color32::RED),
                                    );
                                    ui.label(
                                        RichText::new("1")
                                            .monospace()
                                            .color(egui::Color32::from_rgb(33, 78, 211)),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            instruction_fields.offset11 as u16 & 0x7FF,
                                            11,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::GREEN),
                                    );
                                });
                                ui.label(
                                    RichText::new(format!(
                                        "Binary: {}",
                                        format_binary(jsr_instr, 16)
                                    ))
                                    .monospace(),
                                );
                                ui.label(
                                    RichText::new(format!("Hex: {}", format_hex(jsr_instr)))
                                        .monospace(),
                                );
                            });
                    } else {
                        ui.label(
                            RichText::new("JSRR BaseR")
                                .monospace()
                                .color(egui::Color32::LIGHT_RED),
                        );
                        ui.label(
                            RichText::new("Jumps to address in BaseR, saving return address in R7")
                                .italics(),
                        );

                        register_selector(ui, &mut instruction_fields.base_r, "Base Reg:");

                        let pseudo_code =
                            format!("R7 = PC\nPC = BaseR = R{}", instruction_fields.base_r);
                        ui.label(
                            RichText::new(pseudo_code)
                                .monospace()
                                .color(egui::Color32::YELLOW),
                        );

                        let jsrr_instr = (0b0100 << 12) | // Unused bits
                                      ((instruction_fields.base_r as u16) << 6); // Unused bits

                        egui::CollapsingHeader::new("Binary Representation")
                            .id_salt("jsrr_binary")
                            .show(ui, |ui| {
                                instruction_layout(
                                    ui,
                                    "Layout: 0100 | 0 | 00 | BaseR | 000000",
                                    egui::Color32::GREEN,
                                );
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("0100").monospace().color(egui::Color32::RED),
                                    );
                                    ui.label(
                                        RichText::new("0")
                                            .monospace()
                                            .color(egui::Color32::from_rgb(33, 78, 211)),
                                    );
                                    ui.label(
                                        RichText::new("00").monospace().color(egui::Color32::GRAY),
                                    );
                                    ui.label(
                                        RichText::new(format_binary(
                                            (instruction_fields.base_r as u16) & 0x7,
                                            3,
                                        ))
                                        .monospace()
                                        .color(egui::Color32::GREEN),
                                    );
                                    ui.label(
                                        RichText::new("000000")
                                            .monospace()
                                            .color(egui::Color32::GRAY),
                                    );
                                });
                                ui.label(
                                    RichText::new(format!(
                                        "Binary: {}",
                                        format_binary(jsrr_instr, 16)
                                    ))
                                    .monospace(),
                                );
                                ui.label(
                                    RichText::new(format!("Hex: {}", format_hex(jsrr_instr)))
                                        .monospace(),
                                );
                            });
                    }
                });
            });
        egui::CollapsingHeader::new("System Operations")
            .id_salt("system_ops")
            .show(ui, |ui| {
                // TRAP
                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("TRAP - System Call").heading().strong().color(egui::Color32::LIGHT_GREEN));
                ui.label("Performs a system call based on the trap vector.");

                ui.group(|ui| {
                    ui.label(RichText::new("TRAP trapvect8").monospace().color(egui::Color32::LIGHT_GREEN));
                    ui.label(RichText::new("System call to vector specified by trapvect8").italics());

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Trap Vector:").strong());

                        let mut trap_hex = format!("0x{:02X}", instruction_fields.trapvector);
                        if ui.text_edit_singleline(&mut trap_hex).changed() {
                            if let Ok(value) = u8::from_str_radix(trap_hex.trim_start_matches("0x"), 16) {
                                instruction_fields.trapvector = value;
                            }
                        }

                        ui.add(egui::DragValue::new(&mut instruction_fields.trapvector)
                            .range(0..=0xFF)
                            .speed(0.1))
                            .on_hover_text("Trap vector (0-255)");
                    });

                    ui.separator();
                    ui.label(RichText::new("Common TRAP vectors:").strong());

                    ui.horizontal(|ui| {
                        if ui.selectable_label(instruction_fields.trapvector == 0x20, "GETC (x20)").clicked() {
                            instruction_fields.trapvector = 0x20;
                        }
                        ui.label("Read character from keyboard -> R0");
                    });

                    ui.horizontal(|ui| {
                        if ui.selectable_label(instruction_fields.trapvector == 0x21, "OUT (x21)").clicked() {
                            instruction_fields.trapvector = 0x21;
                        }
                        ui.label("Write character in R0 to console");
                    });

                    ui.horizontal(|ui| {
                        if ui.selectable_label(instruction_fields.trapvector == 0x22, "PUTS (x22)").clicked() {
                            instruction_fields.trapvector = 0x22;
                        }
                        ui.label("Output null-terminated string pointed to by R0");
                    });

                    ui.horizontal(|ui| {
                        if ui.selectable_label(instruction_fields.trapvector == 0x23, "IN (x23)").clicked() {
                            instruction_fields.trapvector = 0x23;
                        }
                        ui.label("Print prompt and read character -> R0");
                    });


                    ui.horizontal(|ui| {
                        if ui.selectable_label(instruction_fields.trapvector == 0x25, "HALT (x25)").clicked() {
                            instruction_fields.trapvector = 0x25;
                        }
                        ui.label("Halt execution");
                    });

                    let pseudo_code = format!("R7 = PC\nPC = MEM[x{:02X}]", instruction_fields.trapvector);
                    ui.label(RichText::new(pseudo_code).monospace().color(egui::Color32::YELLOW));

                    let trap_instr = (0b1111 << 12) | // Unused bits
                                   (instruction_fields.trapvector as u16); // trapvect8

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("trap_binary")
                        .show(ui, |ui| {
                            instruction_layout(ui, "Layout: 1111 | 0000 | trapvect8", egui::Color32::GREEN);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("1111").monospace().color(egui::Color32::RED));
                                ui.label(RichText::new("0000").monospace().color(egui::Color32::GRAY));
                                ui.label(RichText::new(format_binary(instruction_fields.trapvector as u16, 8)).monospace().color(egui::Color32::from_rgb(33,78,211)));
                            });
                            ui.label(RichText::new(format!("Binary: {}", format_binary(trap_instr, 16))).monospace());
                            ui.label(RichText::new(format!("Hex: {}", format_hex(trap_instr))).monospace());
                        });
                });

                // RTI
                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("RTI - Return from Interrupt").heading().strong().color(egui::Color32::LIGHT_GREEN));
                ui.label("Returns from an interrupt service routine.");

                ui.group(|ui| {
                    ui.label(RichText::new("RTI").monospace().color(egui::Color32::LIGHT_GREEN));
                    ui.label(RichText::new("Return from interrupt - restore PC and PSR from stack").italics());

                    let pseudo_code = "if (Privilege Mode)\n    PC = MEM[R6]\n    PSR = MEM[R6+1]\n    R6 = R6 + 2\nelse\n    Privilege Mode Exception";
                    ui.label(RichText::new(pseudo_code).monospace().color(egui::Color32::YELLOW));

                    let rti_instr = 0b1000 << 12; // Opcode, all other bits are unused

                    egui::CollapsingHeader::new("Binary Representation")
                        .id_salt("rti_binary")
                        .show(ui, |ui| {
                            instruction_layout(ui, "Layout: 1000 | 000000000000", egui::Color32::GREEN);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("000000000000").monospace().color(egui::Color32::GRAY));
                            });
                            ui.label(RichText::new(format!("Hex: {}", format_hex(rti_instr))).monospace());
                });
            });
        });
    }

    fn render_control_buttons_and_run_emulator(&mut self, ui: &mut egui::Ui) {
        if ui.button("Small Step").clicked() {
            let _ = self.emulator.micro_step();
        }
        if ui.button("Step").clicked() {
            let _ = self.emulator.step();
        }
        if self.emulator.running {
            if self.emulator.await_input.is_none()
                && self.tick % self.ticks_between_updates as u64 == 0
            {
                let mut i = 0;
                while self.emulator.await_input.is_none() && self.emulator.running && i < self.speed
                {
                    match self.emulator.micro_step() {
                        Ok(_) => {}
                        Err(_) => {
                            self.emulator.running = false;
                            break;
                        }
                    }
                    i += 1;
                    if self
                        .breakpoints
                        .contains(&(self.emulator.pc.get() as usize))
                        && self.emulator.cpu_state == CpuState::Decode
                    {
                        self.emulator.running = false;
                    }

                    if self.emulator.await_input.is_some()
                        && !self.emulator.await_input.unwrap()
                        && !self.input_stack.is_empty()
                    {
                        self.emulator.r[0].set(self.input_stack.remove(0) as u16);
                        self.emulator.await_input = None;
                    }
                }
            }
            if ui.button("Pause").clicked() {
                self.emulator.running = false;
            }
        } else if ui.button("Run").clicked() {
            self.emulator.running = true;
        }
        if self.last_compiled.is_empty() {
            // compile
            if ui.button("Compile").clicked() {
                let data_to_load = Emulator::parse_program(&self.program);
                if let Ok((instructions, _, orig_address)) = data_to_load {
                    self.line_to_address = instructions
                        .iter()
                        .enumerate()
                        .map(|(i, (x, _))| (*x, i + orig_address as usize))
                        .collect();
                    self.emulator.flash_memory(
                        instructions.into_iter().map(|(_, y)| y).collect(),
                        orig_address,
                    );
                    self.error = None;
                } else {
                    self.error = Some(data_to_load.unwrap_err());
                }
                self.last_compiled = self.program.clone();
            }
        } else if ui.button("Reset & compile").clicked() {
            self.emulator = Emulator::new();
            let data_to_load = Emulator::parse_program(&self.program);
            if let Ok((instructions, _, orig_address)) = data_to_load {
                self.line_to_address = instructions
                    .iter()
                    .enumerate()
                    .map(|(i, (x, _))| (*x, i + orig_address as usize))
                    .collect();
                self.emulator.flash_memory(
                    instructions.into_iter().map(|(_, y)| y).collect(),
                    orig_address,
                );
                self.error = None;
            } else {
                self.line_to_address = HashMap::new();
                self.error = Some(data_to_load.unwrap_err());
            }
            self.last_compiled = self.program.clone();
        }
        ui.separator();
        if self.emulator.await_input.is_some() && !self.emulator.await_input.unwrap() {
            if self.input_stack.is_empty() {
                ui.label("No input available pls enter some");
            } else {
                self.emulator.r[0].set(self.input_stack.remove(0) as u16);
                self.emulator.await_input = None;
            }
        }
    }

    fn render_register_editor(&mut self, ui: &mut egui::Ui) {
        for i in 0..8 {
            ui.horizontal(|ui| {
                ui.label(format!("R{}:", i));
                register_view(ui, &mut self.emulator.r[i], self.display_base);
            });
        }

        ui.horizontal(|ui| {
            ui.label("PC:");
            register_view(ui, &mut self.emulator.pc, self.display_base);
        });

        ui.horizontal(|ui| {
            ui.label("MDR:");
            register_view(ui, &mut self.emulator.mdr, self.display_base);
        });

        ui.horizontal(|ui| {
            ui.label("MAR:");
            register_view(ui, &mut self.emulator.mar, self.display_base);
        });

        ui.horizontal(|ui| {
            ui.label("IR:");
            register_view(ui, &mut self.emulator.ir, self.display_base);
        });
    }

    fn render_cycle_view(&mut self, ui: &mut egui::Ui) {
        let cycles = ["Fetch", "Decode", "Get memory", "Execute"];
        let current_cycle = self.emulator.cpu_state as usize;
        let instruction_text = match self.emulator.ir.get() {
            0x1000..=0x1FFF => "ADD - Addition Operation",
            0x5000..=0x5FFF => "AND - Bitwise AND Operation",
            0x0000..=0x0FFF => "BR - Branch Operation",
            0x2000..=0x2FFF => "LD - Load Operation",
            0x6000..=0x6FFF => "LDR - Load Register Operation",
            0xA000..=0xAFFF => "LDI - Load Indirect Operation",
            0xE000..=0xEFFF => "LEA - Load Effective Address",
            0x9000..=0x9FFF => "NOT - Bitwise NOT Operation",
            0x3000..=0x3FFF => "ST - Store Operation",
            0x7000..=0x7FFF => "STR - Store Register Operation",
            0xB000..=0xBFFF => "STI - Store Indirect Operation",
            0xC000..=0xCFFF => "JMP/RET - Jump/Return Operation",
            0x4000..=0x4FFF => "JSR/JSRR - Jump to Subroutine Operation",
            0x8000..=0x8FFF => "RTI - Return from Interrupt",
            0xF000..=0xFFFF => "TRAP - System Call",
            _ => "Unknown Instruction",
        };
        // Find the corresponding source line if available
        let source_line = self
            .line_to_address
            .get(&(self.emulator.pc.get() as usize))
            .and_then(|&line_num| self.last_compiled.lines().nth(line_num))
            .unwrap_or("Unknown instruction");
        let mut description = RichText::new("NO CURRENT CYCLE");
        // Display cycle information
        for (i, cycle) in cycles.iter().enumerate() {
            if i == current_cycle {
                ui.label(
                    RichText::new(format!("-> {}", cycle))
                        .strong()
                        .color(egui::Color32::GREEN),
                );

                // Provide specific description based on the current cycle
                // Create detailed descriptions for each processor cycle
                description = match current_cycle {
                    0 => RichText::new(format!(
                        "FETCH: The processor is fetching the instruction at address {:#06x} from memory. \
                                    The PC (Program Counter) is used to determine which instruction to fetch. \
                                    The MAR is loaded with the PC value, and the MDR will receive the instruction from memory. \
                                    After fetching, PC is incremented to point to the next instruction."
                        , self.emulator.pc.get())).color(egui::Color32::LIGHT_GREEN),

                    1 => RichText::new(format!(
                        "DECODE: The processor is analyzing instruction {:#06x} to determine what operation to perform. \
                                    The IR (Instruction Register) contains the fetched instruction, with the 4 most significant bits \
                                    identifying the operation type. Current instruction appears to be {}. \
                                    Source and destination registers are being identified.",
                        self.emulator.ir.get(), instruction_text)).color(egui::Color32::LIGHT_YELLOW),

                    2 => RichText::new(format!(
                        "MEMORY ACCESS: The processor is accessing memory to read required for execution. \
                                    The MAR contains address {:#06x} to be accessed and loaded into the MDR (the operation will change the value if it needs to read memory). \
                                    This step is necessary for instructions like LDI, ST, STI, and STR that read memory.",
                        self.emulator.mar.get())).color(egui::Color32::LIGHT_BLUE),

                    3 => RichText::new(format!(
                        "EXECUTE: The processor is performing the actual operation specified by the instruction. \
                                    Instruction '{:#06x}' ({}), which corresponds to '{}' is being executed. \
                                    This may involve arithmetic/logic operations, updating registers, or modifying condition codes (N={}, Z={}, P={}).",
                        self.emulator.ir.get(), instruction_text, source_line.trim(),
                        self.emulator.n.get(), self.emulator.z.get(), self.emulator.p.get())).color(egui::Color32::GOLD),

                    _ => RichText::new("UNKNOWN CYCLE").color(egui::Color32::RED),
                };

                // Show relevant flags for the current cycle
            } else {
                ui.label(RichText::new(format!("  {}", cycle)).color(egui::Color32::GRAY));
            }
        }
        ui.label(description);
        ui.separator();
        ui.horizontal(|ui| {
            // Always show condition flags
            ui.label("Condition Flags:");
            let n_text = format!("N={}", self.emulator.n.get());
            let z_text = format!("Z={}", self.emulator.z.get());
            let p_text = format!("P={}", self.emulator.p.get());

            // Highlight flags modified in the last cycle
            if current_cycle == 3 {
                // Execute cycle may have modified flags
                ui.label(RichText::new(n_text).color(egui::Color32::LIGHT_GREEN));
                ui.label(RichText::new(z_text).color(egui::Color32::LIGHT_GREEN));
                ui.label(RichText::new(p_text).color(egui::Color32::LIGHT_GREEN));
            } else {
                ui.label(n_text);
                ui.label(z_text);
                ui.label(p_text);
            }
        });
        ui.horizontal(|ui| {
            // Always show memory access registers
            ui.label("Memory Access:");
            let mar_text = format!("MAR={:#06x}", self.emulator.mar.get());
            let mdr_text = format!("MDR={:#06x}", self.emulator.mdr.get());

            if current_cycle == 0 || current_cycle == 2 {
                // Fetch or Memory Access cycle
                ui.label(RichText::new(mar_text).color(egui::Color32::YELLOW));
                ui.label(RichText::new(mdr_text).color(egui::Color32::YELLOW));
            } else if current_cycle == 3 {
                // Execute may have modified MAR/MDR for next cycle
                ui.label(RichText::new(mar_text).color(egui::Color32::LIGHT_GREEN));
                ui.label(RichText::new(mdr_text).color(egui::Color32::LIGHT_GREEN));
            } else {
                ui.label(mar_text);
                ui.label(mdr_text);
            }
        });
        ui.horizontal(|ui| {
            // Always show instruction register
            ui.label("Instruction:");
            let ir_text = format!("IR={:#06x}", self.emulator.ir.get());

            // Highlight IR when it's being actively used
            if current_cycle == 0 {
                // IR will be used in next cycle
                ui.label(RichText::new(ir_text).color(egui::Color32::YELLOW));
            } else if current_cycle == 1 {
                // Decode is actively using IR
                ui.label(RichText::new(ir_text).color(egui::Color32::LIGHT_GREEN));
            } else {
                ui.label(ir_text);
            }

            // Show PC as well
            let pc_text = format!("PC={:#06x}", self.emulator.pc.get());
            if current_cycle == 0 {
                // Fetch uses PC
                ui.label(RichText::new(pc_text).color(egui::Color32::LIGHT_GREEN));
            } else {
                ui.label(pc_text);
            }
        });
    }

    fn render_output(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Output from program:").strong());

        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                if self.emulator.output.is_empty() {
                    ui.label(
                        RichText::new("No output yet")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.emulator.output.clone())
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace)
                            .interactive(false),
                    );
                }
            });

        if !self.emulator.output.is_empty() {
            ui.horizontal(|ui| {
                if ui.button("Clear Output").clicked() {
                    self.emulator.output.clear();
                }

                if ui.button("Copy to Clipboard").clicked() {
                    ui.output_mut(|o| {
                        o.commands
                            .push(OutputCommand::CopyText(self.emulator.output.clone()))
                    });
                }
            });
        }
    }

    fn compiled_view(&mut self, ui: &mut egui::Ui) {
        if let Some((error, line)) = &self.error {
            ui.label(
                RichText::new(format!("Error on line {}: {}", line, error))
                    .small()
                    .color(ui.visuals().warn_fg_color),
            );
        }

        ui.separator();

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_machine_code, "Show Machine Code");
                if self.show_machine_code {
                    ui.label("Base:");
                    ui.radio_value(&mut self.machine_code_base, 2, "Binary");
                    ui.radio_value(&mut self.machine_code_base, 16, "Hex");
                    ui.radio_value(&mut self.machine_code_base, 10, "Decimal");
                }
            });
        });

        let mut longest_label = 0;
        let mut longest_body = 0;
        let mut longest_operand = 0;
        for (i, line) in self.last_compiled.lines().enumerate() {
            if !self.line_to_address.contains_key(&i) || line.is_empty() {
                continue;
            }
            // Check if line has a comment and split it off
            let code_part = if line.contains(';') {
                line.split(';').next().unwrap().trim()
            } else {
                line.trim()
            };

            if code_part.is_empty() {
                continue;
            }

            // Parse LC-3 assembly line
            let mut split = code_part.split_whitespace();
            let (mut len_lab, mut len_body, mut len_op) = (0, 0, 0);

            match split.clone().count() {
                0 => {
                    log::error!("Empty line somehow passed through the compiler");
                }
                1 => {
                    // Single word - could be an instruction with no operands (like RET)
                    len_body = split.next().unwrap().len();
                }
                2 => {
                    // Could be LABEL: INSTRUCTION or INSTRUCTION OPERAND
                    let first = split.next().unwrap();
                    if first.ends_with(':') {
                        len_lab = first.len();
                        len_body = split.next().unwrap().len();
                    } else {
                        len_body = first.len();
                        len_op = split.next().unwrap().len();
                    }
                }
                3 => {
                    // Could be LABEL: INSTRUCTION OPERAND or INSTRUCTION OPERAND, OPERAND
                    let first = split.next().unwrap();
                    if first.ends_with(':') {
                        len_lab = first.len();
                        len_body = split.next().unwrap().len();
                        len_op = split.next().unwrap().len();
                    } else {
                        len_body = first.len();
                        len_op = split.collect::<Vec<&str>>().join(" ").len();
                    }
                }
                _ => {
                    // Multiple operands or more complex format
                    let first = split.next().unwrap();
                    if first.ends_with(':') {
                        len_lab = first.len();
                        len_body = split.next().unwrap().len();
                        len_op = split.collect::<Vec<&str>>().join(" ").len();
                    } else {
                        len_body = first.len();
                        len_op = split.collect::<Vec<&str>>().join(" ").len();
                    }
                }
            }

            if len_lab > longest_label {
                longest_label = len_lab;
            }
            if len_body > longest_body {
                longest_body = len_body;
            }
            if len_op > longest_operand {
                longest_operand = len_op;
            }
        }

        // Display the program
        for (i, line) in self.last_compiled.lines().enumerate() {
            let original_line = line.trim().to_string();
            // Split the line into code and comment parts
            let (code_part, comment_part) = if original_line.contains(';') {
                let parts: Vec<&str> = original_line.split(';').collect();
                (
                    parts[0].trim().to_ascii_uppercase(),
                    format!("; {}", parts[1..].join(";")),
                )
            } else {
                (original_line.to_ascii_uppercase(), String::new())
            };

            let mut label = code_part.clone();
            if let Some((error, line)) = &self.error {
                if *line == i {
                    label = format!("{} (error: {})", label, error);
                }
            }

            if self.breakpoints.contains(&i) {
                label = format!("{} (breakpoint)", label);
            }

            let label_capitalized = label.as_str().to_uppercase();

            // Handle LC-3 directives
            let is_directive = label_capitalized.contains(".ORIG")
                || label_capitalized.contains(".FILL")
                || label_capitalized.contains(".BLKW")
                || label_capitalized.contains(".STRINGZ")
                || label_capitalized.contains(".END");

            if let Some(address) = self.line_to_address.get(&(i)) {
                // Format based on LC-3 assembly
                let label_parts: Vec<&str> = code_part.split_whitespace().collect();
                let formatted_label = match label_parts.len() {
                    1 => format!(
                        "{:<width1$} {:<width2$}",
                        "",
                        label_parts[0],
                        width1 = longest_label,
                        width2 = longest_body
                    ),
                    2 => {
                        if label_parts[0].ends_with(':') {
                            format!(
                                "{:<width1$} {:<width2$}",
                                label_parts[0],
                                label_parts[1],
                                width1 = longest_label,
                                width2 = longest_body
                            )
                        } else {
                            format!(
                                "{:<width1$} {:<width2$} {:<width3$}",
                                "",
                                label_parts[0],
                                label_parts[1],
                                width1 = longest_label,
                                width2 = longest_body,
                                width3 = longest_operand
                            )
                        }
                    }
                    3 => {
                        if label_parts[0].ends_with(':') {
                            format!(
                                "{:<width1$} {:<width2$} {:<width3$}",
                                label_parts[0],
                                label_parts[1],
                                label_parts[2],
                                width1 = longest_label,
                                width2 = longest_body,
                                width3 = longest_operand
                            )
                        } else {
                            format!(
                                "{:<width1$} {:<width2$} {:<width3$}",
                                "",
                                label_parts[0],
                                format!("{} {}", label_parts[1], label_parts[2]),
                                width1 = longest_label,
                                width2 = longest_body,
                                width3 = longest_operand
                            )
                        }
                    }
                    _ => {
                        if label_parts[0].ends_with(':') {
                            let ops = label_parts[2..].join(" ");
                            format!(
                                "{:<width1$} {:<width2$} {:<width3$}",
                                label_parts[0],
                                label_parts[1],
                                ops,
                                width1 = longest_label,
                                width2 = longest_body,
                                width3 = longest_operand
                            )
                        } else {
                            let ops = label_parts[1..].join(" ");
                            format!(
                                "{:<width1$} {:<width2$} {:<width3$}",
                                "",
                                label_parts[0],
                                ops,
                                width1 = longest_label,
                                width2 = longest_body,
                                width3 = longest_operand
                            )
                        }
                    }
                };
                label = formatted_label;
                log::info!("label: {}", label);

                if self.show_machine_code {
                    if let Some(instruction) = self.emulator.memory.get(*address) {
                        match self.machine_code_base {
                            2 => label = format!("{:016b}", instruction.get()),
                            16 => label = format!("0x{:04X}", instruction.get()),
                            10 => label = format!("{}", instruction.get()),
                            _ => {
                                label = base_to_base(
                                    10,
                                    self.display_base,
                                    &(instruction.get() as u32).to_string(),
                                    "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                                )
                            }
                        }
                    }
                }

                if *address == 1 + self.emulator.pc.get() as usize {
                    label = format!("0x{:04X}: {} (pc)", address, label);
                } else {
                    label = format!("0x{:04X}: {}", address, label);
                }

                ui.horizontal(|ui| {
                    if ui.button("🛑").clicked() {
                        if self.breakpoints.contains(address) {
                            self.breakpoints.retain(|x| *x != *address);
                        } else {
                            self.breakpoints.push(*address);
                        }
                    }
                    // Color coding
                    if let Some(_instruction) = self.emulator.memory.get(*address) {
                        if self.emulator.pc.get() as usize == *address {
                            ui.label(
                                RichText::new(label)
                                    .background_color(egui::Color32::GREEN)
                                    .color(egui::Color32::BLACK)
                                    .monospace(),
                            );

                            // Add the comment if it exists
                            if !comment_part.is_empty() {
                                ui.label(
                                    RichText::new(comment_part)
                                        .color(egui::Color32::GRAY)
                                        .monospace(),
                                );
                            }
                        } else if self.error.as_ref().is_some_and(|(_, line)| *line == i) {
                            // Extract the error text to highlight just that portion
                            if let Some((error_msg, _)) = &self.error {
                                let parts: Vec<&str> =
                                    label.split(&format!("(error: {})", error_msg)).collect();
                                if parts.len() > 1 {
                                    ui.horizontal(|ui| {
                                        // Base code with yellow text, no background
                                        ui.label(
                                            RichText::new(parts[0])
                                                .color(egui::Color32::YELLOW)
                                                .monospace(),
                                        );

                                        // Error part with more faded red background
                                        ui.label(
                                            RichText::new(format!("(error: {})", error_msg))
                                                .background_color(
                                                    egui::Color32::from_rgba_premultiplied(
                                                        255, 150, 150, 120,
                                                    ),
                                                )
                                                .color(egui::Color32::DARK_RED)
                                                .monospace(),
                                        );

                                        // Any remaining text
                                        if parts.len() > 1 && !parts[1].is_empty() {
                                            ui.label(RichText::new(parts[1]).monospace());
                                        }

                                        // Add the comment if it exists
                                        if !comment_part.is_empty() {
                                            ui.label(
                                                RichText::new(comment_part)
                                                    .color(egui::Color32::GRAY)
                                                    .monospace(),
                                            );
                                        }
                                    });
                                } else {
                                    // Fallback if splitting didn't work
                                    ui.label(
                                        RichText::new(label)
                                            .color(egui::Color32::YELLOW)
                                            .monospace(),
                                    );

                                    // Add the comment if it exists
                                    if !comment_part.is_empty() {
                                        ui.label(
                                            RichText::new(comment_part)
                                                .color(egui::Color32::GRAY)
                                                .monospace(),
                                        );
                                    }
                                }
                            } else {
                                // Fallback if error message is not available
                                ui.label(
                                    RichText::new(label)
                                        .color(egui::Color32::YELLOW)
                                        .monospace(),
                                );

                                // Add the comment if it exists
                                if !comment_part.is_empty() {
                                    ui.label(
                                        RichText::new(comment_part)
                                            .color(egui::Color32::GRAY)
                                            .monospace(),
                                    );
                                }
                            }
                        } else if self.breakpoints.contains(address) {
                            ui.label(
                                RichText::new(label)
                                    .background_color(egui::Color32::LIGHT_RED)
                                    .color(egui::Color32::BLACK)
                                    .monospace(),
                            );

                            // Add the comment if it exists
                            if !comment_part.is_empty() {
                                ui.label(
                                    RichText::new(comment_part)
                                        .color(egui::Color32::GRAY)
                                        .monospace(),
                                );
                            }
                        } else if is_directive {
                            ui.label(
                                RichText::new(label)
                                    .background_color(egui::Color32::LIGHT_BLUE)
                                    .color(egui::Color32::BLACK)
                                    .monospace(),
                            );

                            // Add the comment if it exists
                            if !comment_part.is_empty() {
                                ui.label(
                                    RichText::new(comment_part)
                                        .color(egui::Color32::GRAY)
                                        .monospace(),
                                );
                            }
                        } else {
                            ui.label(RichText::new(label).monospace());

                            // Add the comment if it exists
                            if !comment_part.is_empty() {
                                ui.label(
                                    RichText::new(comment_part)
                                        .color(egui::Color32::GRAY)
                                        .monospace(),
                                );
                            }
                        }
                    }

                    // Allow editing memory values
                    if is_directive || code_part.contains(".FILL") {
                        let value_u16 = self.emulator.memory[*address].get();
                        let mut value = value_u16 as i16;
                        if ui.add(egui::DragValue::new(&mut value)).changed() {
                            self.emulator.memory[*address].set(value as u16);
                        }
                    }
                });
            } else if self.error.as_ref().is_some_and(|(_, line)| *line == i) {
                // For error lines that aren't in the address map
                if let Some((error_msg, _)) = &self.error {
                    let parts: Vec<&str> =
                        label.split(&format!("(error: {})", error_msg)).collect();
                    if parts.len() > 1 {
                        ui.horizontal(|ui| {
                            // Base code with yellow text, no background
                            ui.label(
                                RichText::new(parts[0])
                                    .color(egui::Color32::YELLOW)
                                    .monospace(),
                            );

                            // Error part with more faded red background
                            ui.label(
                                RichText::new(format!("(error: {})", error_msg))
                                    .background_color(egui::Color32::from_rgba_premultiplied(
                                        255, 20, 20, 60,
                                    ))
                                    .color(egui::Color32::DARK_RED)
                                    .monospace(),
                            );

                            // Any remaining text
                            if !parts[1].is_empty() {
                                ui.label(RichText::new(parts[1]).monospace());
                            }

                            // Add the comment if it exists
                            if !comment_part.is_empty() {
                                ui.label(
                                    RichText::new(comment_part)
                                        .color(egui::Color32::GRAY)
                                        .monospace(),
                                );
                            }
                        });
                    } else {
                        // Fallback if splitting didn't work
                        ui.label(
                            RichText::new(label)
                                .color(egui::Color32::YELLOW)
                                .monospace(),
                        );

                        // Add the comment if it exists
                        if !comment_part.is_empty() {
                            ui.label(
                                RichText::new(comment_part)
                                    .color(egui::Color32::GRAY)
                                    .monospace(),
                            );
                        }
                    }
                } else {
                    // Fallback if error message is not available
                    ui.label(
                        RichText::new(label)
                            .color(egui::Color32::YELLOW)
                            .monospace(),
                    );

                    // Add the comment if it exists
                    if !comment_part.is_empty() {
                        ui.label(
                            RichText::new(comment_part)
                                .color(egui::Color32::GRAY)
                                .monospace(),
                        );
                    }
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).monospace());

                    // Add the comment if it exists
                    if !comment_part.is_empty() {
                        ui.label(
                            RichText::new(comment_part)
                                .color(egui::Color32::GRAY)
                                .monospace(),
                        );
                    }
                });
            }
        }
    }
}

fn register_view(ui: &mut egui::Ui, value_cell: &mut EmulatorCell, base: u32) {
    let mut value = value_cell.get() as i16;
    ui.add(egui::DragValue::new(&mut value));
    value_cell.set(value as u16);
    ui.label(base_to_base(
        10,
        base,
        &(value_cell.get() as u32).to_string(),
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    ));
}

fn render_help_ui(ui: &mut egui::Ui) {
    ui.add(egui::Label::new(
        RichText::new("Welcome to the LC-3 Emulator")
            .heading()
            .strong()
            .color(ui.visuals().text_color()),
    ));
    ui.add_space(4.0);
    ui.label("This emulator allows you to write, compile, and execute LC-3 assembly programs. Below is a guide to using the various features:");
    egui::CollapsingHeader::new("Basic Usage")
        .id_salt("basic_usage")
        .show(ui, |ui| {
            ui.label(RichText::new("1. Code Editor").strong());
            ui.label("Write your LC-3 assembly code in the editor below. The editor supports syntax highlighting and basic formatting.");

            ui.label(RichText::new("2. Compilation").strong());
            ui.label("Click 'Compile' to assemble your code. If there are errors, they will be displayed below the editor.");

            ui.label(RichText::new("3. Execution").strong());
            ui.label("- Use 'Run' to begin continuous execution");
            ui.label("- Use 'Pause' to stop execution");
            ui.label("- Use 'Step' to execute one full instruction");
            ui.label("- Use 'Small Step' to execute one phase of the CPU cycle");

            ui.label(RichText::new("4. Input").strong());
            ui.label("There are two ways to provide input to LC-3 programs:");
            ui.label("- Use the 'Input' text field below the editor to pre-load input for GETC instructions");
            ui.label("- When a TRAP IN instruction executes, you'll be prompted to enter a character");
            ui.label("Characters typed into the input field will be consumed one at a time by GETC instructions from the start of the input");

            ui.label(RichText::new("5. Debugging").strong());
            ui.label("- Set breakpoints by clicking the 🛑 button next to a line");
            ui.label("- Examine registers, memory, and flags in the collapsible sections");
            ui.label("- Monitor the processor cycle for detailed execution information");
        });
    egui::CollapsingHeader::new("Execution Controls")
        .id_salt("execution_controls")
        .show(ui, |ui| {
            ui.label(RichText::new("Execution Speed").strong());
            ui.label("Control how quickly the program executes:");
            ui.label("- 'Clocks per update': How many instructions to process in each update cycle");
            ui.label("- 'Update frequency': How often to process instructions");
            ui.label("Higher values mean faster execution but less responsive UI.");

            ui.label(RichText::new("Display Options").strong());
            ui.label("- 'Display Base': Change how register and memory values are displayed");
            ui.label("- 'Show Machine Code': Toggle between assembly and binary representation");

            ui.label(RichText::new("Memory Editing").strong());
            ui.label("You can directly edit memory values for directives like .FILL by using the numeric controls next to them.");
        });
    egui::CollapsingHeader::new("Registers and Flags")
        .id_salt("registers_flags")
        .show(ui, |ui| {
            ui.label(RichText::new("General Purpose Registers").strong());
            ui.label("R0-R7: Eight general-purpose registers for computation");

            ui.label(RichText::new("Special Registers").strong());
            ui.label("PC: Program Counter - Points to the next instruction to fetch");
            ui.label("IR: Instruction Register - Holds the current instruction being executed");
            ui.label("MAR: Memory Address Register - Holds the address for memory access");
            ui.label("MDR: Memory Data Register - Holds data being read from or written to memory");

            ui.label(RichText::new("Condition Flags").strong());
            ui.label("N: Negative flag - Set when the result of an operation is negative");
            ui.label("Z: Zero flag - Set when the result of an operation is zero");
            ui.label("P: Positive flag - Set when the result of an operation is positive");
        });
}

impl Default for Pane {
    fn default() -> Self {
        Pane::Emulator(Box::default())
    }
}

impl From<Pane> for String {
    fn from(pane: Pane) -> String {
        match pane {
            Pane::BaseConverter(a) => a.title(),
            Pane::Emulator(a) => a.title(),
        }
    }
}

impl Pane {
    fn render(&mut self, ui: &mut egui::Ui, _tile_id: egui_tiles::TileId) {
        match self {
            Pane::BaseConverter(a) => a.render(ui),
            Pane::Emulator(a) => a.render(ui),
        }
    }

    fn iter_default() -> IntoIter<Pane> {
        vec![Pane::Emulator(Box::default())].into_iter()
    }
}

impl Display for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(String::from(self.clone()).as_str())
    }
}

#[derive(Default)]
struct TreeBehavior {
    add_child_to: Option<(egui_tiles::TileId, Pane)>,
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        let pane_name = pane.to_string();
        pane_name.to_string().into()
    }

    fn is_tab_closable(
        &self,
        _tiles: &egui_tiles::Tiles<Pane>,
        _tile_id: egui_tiles::TileId,
    ) -> bool {
        true
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        pane.render(ui, tile_id);

        egui_tiles::UiResponse::None
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<Pane>,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        let combobox_span = tracing::info_span!("add_pane_combobox", tile_id = tile_id.0);
        let _combobox_guard = combobox_span.enter();

        egui::ComboBox::from_label("")
            .selected_text("➕")
            .show_ui(ui, |ui| {
                for pane in Pane::iter_default() {
                    let pane_name = pane.to_string();

                    if ui.button(pane_name.clone()).clicked() {
                        self.add_child_to = Some((tile_id, pane));
                    }
                }
            });
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        tracing::trace!("Returning tile simplification options");

        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TemplateApp {
    tree: egui_tiles::Tree<Pane>,
    tree_behavior: TreeBehavior,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let span = tracing::info_span!("TemplateApp::default");
        let _guard = span.enter();

        tracing::info!("Creating new TemplateApp with default settings");

        let mut next_view_nr = 0;
        let mut gen_pane = || {
            tracing::debug!("Generating pane #{}", next_view_nr);
            let pane = Pane::default();
            next_view_nr += 1;
            pane
        };

        tracing::debug!("Initializing tile system");
        let mut tiles = egui_tiles::Tiles::default();

        let mut tabs = vec![];
        tracing::debug!("Creating initial pane");
        tabs.push(tiles.insert_pane(gen_pane()));

        tracing::debug!("Setting up root tab tile");
        let root = tiles.insert_tab_tile(tabs);

        tracing::trace!("Creating tree with root ID: {}", root.0);
        let tree = egui_tiles::Tree::new("my_tree", root, tiles);

        tracing::info!("TemplateApp default initialization complete");
        Self {
            tree,
            tree_behavior: TreeBehavior::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let span = tracing::info_span!("TemplateApp::new");
        let _guard = span.enter();

        tracing::info!("Creating new TemplateApp instance");
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        tracing::info!("No persistent storage found, creating default instance");
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let update_span = tracing::info_span!("TemplateApp::update");
        let _update_guard = update_span.enter();

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if let Some((tile_id, pane)) = self.tree_behavior.add_child_to.take() {
            let new_pane = self.tree.tiles.insert_pane(pane);

            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
                self.tree.tiles.get_mut(tile_id)
            {
                tabs.add_child(new_pane);

                tabs.set_active(new_pane);
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");

                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                ui.menu_button("windows", |ui| {
                    if ui.button("Reset layout").clicked() {
                        self.tree = Self::default().tree;
                    }
                });

                ui.menu_button("ui", |ui| {
                    // slider for ui scale
                    let mut scale = ctx.zoom_factor();
                    tracing::trace!("Current UI scale: {}", scale);

                    let dragging = ui
                        .add(egui::Slider::new(&mut scale, 0.5..=5.0).text("UI scale"))
                        .dragged();

                    if dragging {
                        tracing::trace!("User dragging UI scale slider: {}", scale);
                    } else if scale != ctx.zoom_factor() {
                        tracing::info!("Setting new UI scale: {}", scale);
                        ctx.set_zoom_factor(scale);
                    }
                });

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            let tile_ui_span = tracing::info_span!("tile_tree_ui");
            let _tile_ui_guard = tile_ui_span.enter();

            self.tree.ui(&mut self.tree_behavior, ui);
        });

        // tracing::trace!("Rendering central panel");
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     // The central panel the region left after adding TopPanel's and SidePanel's
        //     tracing::trace!("Emulator UI");
        //     let tile_ui_span = tracing::info_span!("tile_tree_ui");
        //     let _tile_ui_guard = tile_ui_span.enter();

        //     self.emulator.render(ui);
        //     tracing::trace!("Tile tree UI render complete");
        // });

        ctx.request_repaint();
    }
}
