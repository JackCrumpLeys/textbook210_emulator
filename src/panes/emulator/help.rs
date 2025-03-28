use crate::panes::{Pane, PaneDisplay, PaneTree};
use egui::RichText;
use serde::{Deserialize, Serialize};

use super::EmulatorPane;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpPane {
    instruction_fields: InstructionFields,
}

impl Default for HelpPane {
    fn default() -> Self {
        Self {
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
                trapvector: 0x25,
                imm_mode: false,
                jsr_mode: true,
            },
        }
    }
}

impl PaneDisplay for HelpPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("LC-3 Emulator Help").show(ui, render_help_ui);
        egui::CollapsingHeader::new("LC-3 Instruction Reference")
            .show(ui, |ui| self.render_reference(ui));
        egui::CollapsingHeader::new("LC-3 Cheatsheet and Examples")
            .show(ui, render_cheatsheet_examples);
    }

    fn title(&self) -> String {
        "Help".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Help".to_string(),
            Pane::EmulatorPanes(Box::new(EmulatorPane::Help(HelpPane::default()))),
        )
    }
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

impl HelpPane {
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
}
