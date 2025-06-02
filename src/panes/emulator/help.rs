use crate::panes::{Pane, PaneDisplay, PaneTree, RealPane};
use crate::theme::CURRENT_THEME_SETTINGS;
use egui::{Color32, RichText, TextWrapMode, Ui};
use serde::{Deserialize, Serialize};

use super::EmulatorPane;

// --- Data Structures ---

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

// --- PaneDisplay Implementation ---

impl PaneDisplay for HelpPane {
    fn render(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            render_collapsible_section_with_id(
                ui,
                "LC-3 Emulator Help",
                "help_main",
                render_general_help_ui,
            );
            render_collapsible_section_with_id(
                ui,
                "LC-3 Instruction Reference",
                "help_instruction_reference",
                |ui| self.render_instruction_reference_ui(ui),
            );
            render_collapsible_section_with_id(
                ui,
                "LC-3 Cheatsheet and Examples",
                "help_cheatsheet_examples",
                render_cheatsheet_examples_ui,
            );
        });
    }

    fn title(&self) -> String {
        "Help".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Help".to_string(),
            Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Help(
                HelpPane::default(),
            )))),
        )
    }
}

// --- UI Helper Functions ---

fn ui_main_title(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .heading()
            .strong()
            .color(ui.visuals().text_color()),
    );
}

fn ui_section_heading(ui: &mut Ui, text: &str, color: Color32) {
    ui.label(RichText::new(text).heading().strong().color(color));
}

fn ui_sub_heading(ui: &mut Ui, text: &str, color: Color32) {
    ui.label(RichText::new(text).heading().strong().color(color));
}

fn ui_strong_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).strong());
}

fn ui_simple_label(ui: &mut Ui, text: &str) {
    ui.label(text);
}

fn ui_italic_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).italics());
}

fn ui_small_italic_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().italics());
}

fn ui_monospace_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).monospace());
}

fn ui_monospace_label_with_color(ui: &mut Ui, text: &str, color: Color32) {
    ui.label(RichText::new(text).monospace().color(color));
}

fn ui_code_block(ui: &mut Ui, code: &str) {
    // Using a frame to give a slight background, similar to original ui.code
    egui::Frame::group(ui.style())
        .fill(ui.visuals().extreme_bg_color) // Use theme's extreme_bg_color for code blocks
        .inner_margin(egui::Margin::same(5))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(RichText::new(code).monospace()).wrap_mode(TextWrapMode::Extend),
            );
        });
}

fn render_collapsible_section_with_id(
    ui: &mut Ui,
    title: &str,
    id_salt: &str,
    add_contents: impl FnOnce(&mut Ui),
) {
    egui::CollapsingHeader::new(title)
        .id_salt(id_salt)
        .show(ui, add_contents);
}

fn render_info_list_item(ui: &mut Ui, text: &str) {
    // For items like "- Use 'Run'..."
    ui.label(format!("- {}", text));
}

// --- General Help UI ---

fn render_general_help_ui(ui: &mut Ui) {
    ui_main_title(ui, "Welcome to the LC-3 Emulator");
    ui.add_space(4.0);
    ui_simple_label(ui, "This emulator allows you to write, compile, and execute LC-3 assembly programs. Below is a guide to using the various features:");

    render_collapsible_section_with_id(ui, "Basic Usage", "basic_usage", |ui| {
        ui_strong_label(ui, "1. Code Editor");
        ui_simple_label(ui, "Write your LC-3 assembly code in the editor below. The editor supports syntax highlighting and basic formatting.");

        ui_strong_label(ui, "2. Compilation");
        ui_simple_label(ui, "Click 'Compile' to assemble your code. If there are errors, they will be displayed below the editor.");

        ui_strong_label(ui, "3. Execution");
        render_info_list_item(ui, "Use 'Run' to begin continuous execution");
        render_info_list_item(ui, "Use 'Pause' to stop execution");
        render_info_list_item(ui, "Use 'Step' to execute one full instruction");
        render_info_list_item(ui, "Use 'Small Step' to execute one phase of the CPU cycle");

        ui_strong_label(ui, "4. Input");
        ui_simple_label(ui, "There are two ways to provide input to LC-3 programs:");
        render_info_list_item(
            ui,
            "Use the 'Input' text field below the editor to pre-load input for GETC instructions",
        );
        render_info_list_item(
            ui,
            "When a TRAP IN instruction executes, you'll be prompted to enter a character",
        );
        ui_simple_label(ui, "Characters typed into the input field will be consumed one at a time by GETC instructions from the start of the input");

        ui_strong_label(ui, "5. Debugging");
        render_info_list_item(
            ui,
            "Set breakpoints by clicking the 🛑 button next to a line",
        );
        render_info_list_item(
            ui,
            "Examine registers, memory, and flags in the collapsible sections",
        );
        render_info_list_item(
            ui,
            "Monitor the processor cycle for detailed execution information",
        );
    });

    render_collapsible_section_with_id(ui, "Execution Controls", "execution_controls", |ui| {
        ui_strong_label(ui, "Execution Speed");
        ui_simple_label(ui, "Control how quickly the program executes:");
        render_info_list_item(
            ui,
            "'Clocks per update': How many instructions to process in each update cycle",
        );
        render_info_list_item(ui, "'Update frequency': How often to process instructions");
        ui_simple_label(
            ui,
            "Higher values mean faster execution but less responsive UI.",
        );

        ui_strong_label(ui, "Display Options");
        render_info_list_item(
            ui,
            "'Display Base': Change how register and memory values are displayed",
        );
        render_info_list_item(
            ui,
            "'Show Machine Code': Toggle between assembly and binary representation",
        );

        ui_strong_label(ui, "Memory Editing");
        ui_simple_label(ui, "You can directly edit memory values for directives like .FILL by using the numeric controls next to them.");
    });

    render_collapsible_section_with_id(ui, "Registers and Flags", "registers_flags", |ui| {
        ui_strong_label(ui, "General Purpose Registers");
        ui_simple_label(ui, "R0-R7: Eight general-purpose registers for computation");

        ui_strong_label(ui, "Special Registers");
        ui_simple_label(
            ui,
            "PC: Program Counter - Points to the next instruction to fetch",
        );
        ui_simple_label(
            ui,
            "IR: Instruction Register - Holds the current instruction being executed",
        );
        ui_simple_label(
            ui,
            "MAR: Memory Address Register - Holds the address for memory access",
        );
        ui_simple_label(
            ui,
            "MDR: Memory Data Register - Holds data being read from or written to memory",
        );

        ui_strong_label(ui, "Condition Flags");
        ui_simple_label(
            ui,
            "N: Negative flag - Set when the result of an operation is negative",
        );
        ui_simple_label(
            ui,
            "Z: Zero flag - Set when the result of an operation is zero",
        );
        ui_simple_label(
            ui,
            "P: Positive flag - Set when the result of an operation is positive",
        );
    });
}

// --- Cheatsheet and Examples UI ---

fn render_cheatsheet_category(ui: &mut Ui, title: &str, items: &[&str]) {
    ui_strong_label(ui, title);
    for item in items {
        ui_simple_label(ui, item);
    }
    ui.add_space(4.0);
}

fn render_sample_program_card(ui: &mut Ui, title: &str, title_color: Color32, code: &str) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui_sub_heading(ui, title, title_color);
        ui_code_block(ui, code);
    });
    ui.add_space(4.0);
}

fn render_cheatsheet_examples_ui(ui: &mut Ui) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.add_space(8.0);
        // Keep these hardcoded as they are in a different section and the prompt only
        // asked to theme the instruction reference part.
        ui_section_heading(ui, "Instruction Reference", egui::Color32::LIGHT_YELLOW);
        ui_simple_label(
            ui,
            "Quick reference for common LC-3 instructions and syntax.",
        );

        render_cheatsheet_category(
            ui,
            "Arithmetic/Logic:",
            &[
                "ADD R1, R2, R3    ; R1 = R2 + R3",
                "ADD R1, R2, #5    ; R1 = R2 + 5 (immediate)",
                "AND R1, R2, R3    ; R1 = R2 & R3 (bitwise)",
                "NOT R1, R2        ; R1 = ~R2 (1's complement)",
            ],
        );
        render_cheatsheet_category(
            ui,
            "Data Movement:",
            &[
                "LD  R1, LABEL     ; R1 = Mem[PC+offset]",
                "LDI R1, LABEL     ; R1 = Mem[Mem[PC+offset]]",
                "LDR R1, R2, #5    ; R1 = Mem[R2+5]",
                "LEA R1, LABEL     ; R1 = PC+offset",
                "ST  R1, LABEL     ; Mem[PC+offset] = R1",
                "STI R1, LABEL     ; Mem[Mem[PC+offset]] = R1",
                "STR R1, R2, #5    ; Mem[R2+5] = R1",
            ],
        );
        render_cheatsheet_category(
            ui,
            "Control Flow:",
            &[
                "BR  LABEL         ; Branch always",
                "BRn LABEL         ; Branch if negative",
                "BRz LABEL         ; Branch if zero",
                "BRp LABEL         ; Branch if positive",
                "JMP R1            ; PC = R1",
                "JSR LABEL         ; Jump to subroutine (PC+offset)",
                "JSRR R1           ; Jump to subroutine (R1)",
                "RET               ; Return (PC = R7)",
            ],
        );
        render_cheatsheet_category(
            ui,
            "System Operations:",
            &[
                "TRAP x20          ; GETC (char -> R0)",
                "TRAP x21          ; OUT (R0 -> display)",
                "TRAP x22          ; PUTS (string at R0)",
                "TRAP x23          ; IN (prompt & input -> R0)",
                "TRAP x25          ; HALT (stop program)",
            ],
        );
        render_cheatsheet_category(
            ui,
            "Directives:",
            &[
                ".ORIG x3000       ; Program origin (starting address)",
                ".FILL #10         ; Insert value (decimal, hex, or label)",
                ".BLKW 5           ; Reserve 5 memory locations",
                ".STRINGZ \"Text\"  ; Null-terminated string",
                ".END              ; End of program",
            ],
        );
        render_cheatsheet_category(
            ui,
            "Number Formats:",
            &[
                "#10               ; Decimal",
                "x10A2             ; Hexadecimal",
                "LABEL             ; Label reference",
            ],
        );
    });
    ui.add_space(8.0);
    ui.separator();
    // Keep these hardcoded as they are in a different section and the prompt only
    // asked to theme the instruction reference part.
    ui_section_heading(ui, "LC-3 Sample Programs", egui::Color32::KHAKI);
    ui_simple_label(
        ui,
        "Common patterns and examples for LC-3 assembly programming.",
    );

    // Keep sample program card colors hardcoded as they are outside the instruction reference section.
    render_sample_program_card(
        ui,
        "Hello World",
        egui::Color32::GOLD,
        r#"; Simple Hello World program
.ORIG x3000
LEA R0, MESSAGE    ; Load the address of the message
PUTS               ; Output the string
HALT               ; Halt the program

MESSAGE: .STRINGZ "Hello, World!"
.END"#,
    );
    render_sample_program_card(
        ui,
        "Input and Echo",
        egui::Color32::GOLD,
        r#"; Program that gets a character and echoes it
.ORIG x3000
LOOP:   GETC                ; Read a character from keyboard
        OUT                 ; Echo the character
        BRnzp LOOP          ; Repeat
.END"#,
    );
    render_sample_program_card(
        ui,
        "Counter Loop",
        egui::Color32::GOLD,
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
    render_sample_program_card(
        ui,
        "Subroutine Example",
        egui::Color32::GOLD,
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
    render_sample_program_card(
        ui,
        "Array Manipulation",
        egui::Color32::GOLD,
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
}
// --- Instruction Reference UI ---

// Helper for formatting binary numbers
fn format_binary(value: u16, width: usize) -> String {
    format!("{:0width$b}", value, width = width)
}

// Helper for formatting hexadecimal numbers
fn format_hex(value: u16) -> String {
    format!("0x{:04X}", value)
}

// Input field for selecting a register
fn ui_register_selector(ui: &mut Ui, value: &mut u8, label: &str) {
    ui.horizontal(|ui| {
        ui_strong_label(ui, label);
        ui.add(egui::DragValue::new(value).range(0..=7).speed(0.1))
            .on_hover_text("Register value (0-7)");
        ui_simple_label(ui, &format!("R{}", value));
    });
}

// Input field for an immediate value
fn ui_immediate_selector(ui: &mut Ui, value: &mut i8, bits: u8, label: &str) {
    ui.horizontal(|ui| {
        ui_strong_label(ui, label);
        let min_val = -(1 << (bits - 1));
        let max_val = (1 << (bits - 1)) - 1;
        ui.add(
            egui::DragValue::new(value)
                .range(min_val..=max_val)
                .speed(0.1),
        )
        .on_hover_text(format!("{}-bit immediate value", bits));
        ui_simple_label(ui, &format!("#{}", value));
    });
}

// Input field for an offset value
fn ui_offset_selector(ui: &mut Ui, value: &mut i16, bits: u8, label: &str) {
    ui.horizontal(|ui| {
        ui_strong_label(ui, label);
        let min_val = -(1 << (bits - 1));
        let max_val = (1 << (bits - 1)) - 1;
        ui.add(
            egui::DragValue::new(value)
                .range(min_val..=max_val)
                .speed(0.1),
        )
        .on_hover_text(format!("{}-bit offset value", bits));
        ui_simple_label(ui, &format!("PC+{}", value));
    });
}

// Displays the "Layout: ..." part of binary representation
fn ui_binary_layout_display(ui: &mut Ui, desc: &str, color: Color32) {
    ui.label(
        RichText::new(desc)
            .monospace()
            .background_color(ui.visuals().extreme_bg_color)
            .color(color),
    );
}

struct BinarySegment {
    text: String,
    color: Color32,
}

fn render_binary_representation_view(
    ui: &mut Ui,
    id_salt_suffix: &str,
    layout_str: &str,
    layout_color: Color32,
    segments: Vec<BinarySegment>,
    binary_value: u16,
) {
    render_collapsible_section_with_id(ui, "Binary Representation", id_salt_suffix, |ui| {
        ui_binary_layout_display(ui, layout_str, layout_color);
        ui.horizontal(|ui| {
            for segment in segments {
                ui_monospace_label_with_color(ui, &segment.text, segment.color);
            }
        });
        ui_monospace_label(ui, &format!("Binary: {}", format_binary(binary_value, 16)));
        ui_monospace_label(ui, &format!("Hex: {}", format_hex(binary_value)));
    });
}

fn render_instruction_card_content(
    ui: &mut Ui,
    title: &str,
    title_color: Color32,
    description: &str,
    content_fn: impl FnOnce(&mut Ui, &mut InstructionFields),
    fields: &mut InstructionFields,
    condition_codes_note: Option<&str>,
) {
    ui.add_space(8.0);
    ui.separator();
    ui_sub_heading(ui, title, title_color);
    ui_simple_label(ui, description);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        content_fn(ui, fields);
    });

    if let Some(note) = condition_codes_note {
        ui_small_italic_label(ui, note);
    }
}

impl HelpPane {
    fn render_instruction_reference_ui(&mut self, ui: &mut Ui) {
        ui_main_title(ui, "LC-3 Assembly Instructions");
        ui_simple_label(ui, "Select an instruction category to explore. Adjust instruction fields to see how they affect binary representation.");
        ui.add_space(4.0);

        render_collapsible_section_with_id(ui, "Arithmetic & Logic", "arithmetic_logic", |ui| {
            self.render_add_instruction(ui);
            self.render_and_instruction(ui);
            self.render_not_instruction(ui);
        });

        render_collapsible_section_with_id(ui, "Data Movement", "data_movement", |ui| {
            self.render_ld_instruction(ui);
            self.render_ldi_instruction(ui);
            self.render_ldr_instruction(ui);
            self.render_lea_instruction(ui);
            self.render_st_instruction(ui);
            self.render_sti_instruction(ui);
            self.render_str_instruction(ui);
        });

        render_collapsible_section_with_id(ui, "Control Flow", "control_flow", |ui| {
            self.render_br_instruction(ui);
            self.render_jmp_ret_instruction(ui);
            self.render_jsr_jsrr_instruction(ui);
        });

        render_collapsible_section_with_id(ui, "System Operations", "system_ops", |ui| {
            self.render_trap_instruction(ui);
            self.render_rti_instruction(ui);
        });
    }

    // --- Individual Instruction Rendering Functions ---

    fn render_add_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "ADD - Addition",
            theme_settings.opcode_color,
            "Adds two values and stores the result in a destination register.",
            |ui, fields| {
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut fields.imm_mode,
                        RichText::new("Immediate mode")
                            .strong()
                            .color(theme_settings.strong_text_color),
                    );
                });

                if fields.imm_mode {
                    ui_monospace_label_with_color(
                        ui,
                        "ADD DR, SR1, #imm5",
                        theme_settings.opcode_color,
                    );
                    ui_italic_label(ui, "Adds SR1 and immediate value, stores in DR");
                    ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                    ui_register_selector(ui, &mut fields.sr1, "Source Reg 1:");
                    ui_immediate_selector(ui, &mut fields.imm5, 5, "Immediate Value:");
                    let pseudo_code = format!(
                        "DR = R{} = R{} + {} = {}",
                        fields.dr,
                        fields.sr1,
                        fields.imm5,
                        format_hex((fields.sr1 as i16 + fields.imm5 as i16) as u16)
                    );
                    ui_monospace_label_with_color(
                        ui,
                        &pseudo_code,
                        theme_settings.help_pseudo_code_color,
                    );

                    let add_imm = (0b0001 << 12)
                        | ((fields.dr as u16) << 9)
                        | ((fields.sr1 as u16) << 6)
                        | (1 << 5) // Mode bit
                        | (fields.imm5 as u16 & 0x1F);

                    render_binary_representation_view(
                        ui,
                        "add_binary_imm",
                        "Layout: 0001 | DR | SR1 | 1 | IMM5",
                        theme_settings.help_binary_layout_fixed_bits_color,
                        vec![
                            BinarySegment {
                                text: "0001".to_string(),
                                color: theme_settings.opcode_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.dr as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.sr1 as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: "1".to_string(), // Mode bit
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.imm5 as u16 & 0x1F, 5),
                                color: theme_settings.help_immediate_color,
                            },
                        ],
                        add_imm,
                    );
                } else {
                    ui_monospace_label_with_color(
                        ui,
                        "ADD DR, SR1, SR2",
                        theme_settings.opcode_color,
                    );
                    ui_italic_label(ui, "Adds SR1 and SR2, stores in DR");
                    ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                    ui_register_selector(ui, &mut fields.sr1, "Source Reg 1:");
                    ui_register_selector(ui, &mut fields.sr2, "Source Reg 2:");
                    let pseudo_code =
                        format!("DR = R{} = R{} + R{}", fields.dr, fields.sr1, fields.sr2);
                    ui_monospace_label_with_color(
                        ui,
                        &pseudo_code,
                        theme_settings.help_pseudo_code_color,
                    );

                    let add_reg = ((0b0001 << 12)
                        | ((fields.dr as u16) << 9)
                        | ((fields.sr1 as u16) << 6)) // Unused bits
                        | (fields.sr2 as u16 & 0x7);

                    render_binary_representation_view(
                        ui,
                        "add_binary_reg",
                        "Layout: 0001 | DR | SR1 | 0 | 000 | SR2",
                        theme_settings.help_binary_layout_fixed_bits_color,
                        vec![
                            BinarySegment {
                                text: "0001".to_string(),
                                color: theme_settings.opcode_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.dr as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.sr1 as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: "0".to_string(), // Mode bit
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: "000".to_string(), // Unused bits
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.sr2 as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                        ],
                        add_reg,
                    );
                }
            },
            fields,
            Some("Sets condition codes: N, Z, P"),
        );
    }

    fn render_and_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "AND - Bitwise AND",
            theme_settings.opcode_color,
            "Performs bitwise AND of two values and stores the result.",
            |ui, fields| {
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut fields.imm_mode,
                        RichText::new("Immediate mode")
                            .strong()
                            .color(theme_settings.strong_text_color),
                    );
                });

                if fields.imm_mode {
                    ui_monospace_label_with_color(
                        ui,
                        "AND DR, SR1, #imm5",
                        theme_settings.opcode_color,
                    );
                    ui_italic_label(ui, "Bitwise ANDs SR1 and immediate value, stores in DR");
                    ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                    ui_register_selector(ui, &mut fields.sr1, "Source Reg 1:");
                    ui_immediate_selector(ui, &mut fields.imm5, 5, "Immediate Value:");
                    let pseudo_code = format!(
                        "DR = R{} = R{} & {} = {}",
                        fields.dr,
                        fields.sr1,
                        fields.imm5,
                        format_hex((fields.sr1 as u16) & (fields.imm5 as u16))
                    );
                    ui_monospace_label_with_color(
                        ui,
                        &pseudo_code,
                        theme_settings.help_pseudo_code_color,
                    );

                    let and_imm = (0b0101 << 12)
                        | ((fields.dr as u16) << 9)
                        | ((fields.sr1 as u16) << 6)
                        | (1 << 5) // Mode bit
                        | (fields.imm5 as u16 & 0x1F);

                    render_binary_representation_view(
                        ui,
                        "and_binary_imm",
                        "Layout: 0101 | DR | SR1 | 1 | IMM5",
                        theme_settings.help_binary_layout_fixed_bits_color,
                        vec![
                            BinarySegment {
                                text: "0101".to_string(),
                                color: theme_settings.opcode_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.dr as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.sr1 as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: "1".to_string(), // Mode bit
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.imm5 as u16 & 0x1F, 5),
                                color: theme_settings.help_immediate_color,
                            },
                        ],
                        and_imm,
                    );
                } else {
                    ui_monospace_label_with_color(
                        ui,
                        "AND DR, SR1, SR2",
                        theme_settings.opcode_color,
                    );
                    ui_italic_label(ui, "Bitwise ANDs SR1 and SR2, stores in DR");
                    ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                    ui_register_selector(ui, &mut fields.sr1, "Source Reg 1:");
                    ui_register_selector(ui, &mut fields.sr2, "Source Reg 2:");
                    let pseudo_code =
                        format!("DR = R{} = R{} & R{}", fields.dr, fields.sr1, fields.sr2);
                    ui_monospace_label_with_color(
                        ui,
                        &pseudo_code,
                        theme_settings.help_pseudo_code_color,
                    );

                    let and_reg = ((0b0101 << 12)
                        | ((fields.dr as u16) << 9)
                        | ((fields.sr1 as u16) << 6)) // Unused bits
                        | (fields.sr2 as u16 & 0x7);

                    render_binary_representation_view(
                        ui,
                        "and_binary_reg",
                        "Layout: 0101 | DR | SR1 | 0 | 000 | SR2",
                        theme_settings.help_binary_layout_fixed_bits_color,
                        vec![
                            BinarySegment {
                                text: "0101".to_string(),
                                color: theme_settings.opcode_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.dr as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.sr1 as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: "0".to_string(), // Mode bit
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: "000".to_string(), // Unused bits
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.sr2 as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                        ],
                        and_reg,
                    );
                }
            },
            fields,
            Some("Sets condition codes: N, Z, P"),
        );
    }

    fn render_not_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "NOT - Bitwise NOT",
            theme_settings.opcode_color,
            "Performs bitwise NOT (complement) of a value.",
            |ui, fields| {
                ui_monospace_label_with_color(ui, "NOT DR, SR", theme_settings.opcode_color);
                ui_italic_label(ui, "Bitwise NOTs SR, stores in DR");
                ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                ui_register_selector(ui, &mut fields.sr1, "Source Reg:");
                let pseudo_code = format!("DR = R{} = ~R{}", fields.dr, fields.sr1);
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let not_instr =
                    (0b1001 << 12) | ((fields.dr as u16) << 9) | ((fields.sr1 as u16) << 6) | 0x3F; // bits 5-0 are 111111

                render_binary_representation_view(
                    ui,
                    "not_binary",
                    "Layout: 1001 | DR | SR | 111111",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "1001".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.dr as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.sr1 as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: "111111".to_string(), // Fixed bits for NOT
                            color: theme_settings.help_binary_layout_fixed_bits_color,
                        },
                    ],
                    not_instr,
                );
            },
            fields,
            Some("Sets condition codes: N, Z, P"),
        );
    }
    fn render_ld_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "LD - Load",
            theme_settings.opcode_color,
            "Loads a value from memory into a register.",
            |ui, fields| {
                ui_monospace_label_with_color(ui, "LD DR, LABEL", theme_settings.opcode_color);
                ui_italic_label(
                    ui,
                    "PC-relative addressing: Loads from memory at PC+offset into DR",
                );
                ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                ui_offset_selector(ui, &mut fields.offset9, 9, "PC Offset:");
                let pseudo_code = format!("DR = R{} = MEM[PC + {}]", fields.dr, fields.offset9);
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let ld_instr =
                    (0b0010 << 12) | ((fields.dr as u16) << 9) | (fields.offset9 as u16 & 0x1FF);

                render_binary_representation_view(
                    ui,
                    "ld_binary",
                    "Layout: 0010 | DR | PCoffset9",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "0010".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.dr as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.offset9 as u16 & 0x1FF, 9),
                            color: theme_settings.help_offset_color,
                        },
                    ],
                    ld_instr,
                );
            },
            fields,
            Some("Sets condition codes: N, Z, P"),
        );
    }
    fn render_ldi_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "LDI - Load Indirect",
            theme_settings.opcode_color,
            "Loads a value using a pointer stored in memory.",
            |ui, fields| {
                ui_monospace_label_with_color(ui, "LDI DR, LABEL", theme_settings.opcode_color);
                ui_italic_label(
                    ui,
                    "Loads value from memory at address stored at PC+offset into DR",
                );
                ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                ui_offset_selector(ui, &mut fields.offset9, 9, "PC Offset:");
                let pseudo_code =
                    format!("DR = R{} = MEM[MEM[PC + {}]]", fields.dr, fields.offset9);
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let ldi_instr =
                    (0b1010 << 12) | ((fields.dr as u16) << 9) | (fields.offset9 as u16 & 0x1FF);

                render_binary_representation_view(
                    ui,
                    "ldi_binary",
                    "Layout: 1010 | DR | PCoffset9",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "1010".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.dr as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.offset9 as u16 & 0x1FF, 9),
                            color: theme_settings.help_offset_color,
                        },
                    ],
                    ldi_instr,
                );
            },
            fields,
            Some("Sets condition codes: N, Z, P"),
        );
    }

    fn render_ldr_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "LDR - Load Register",
            theme_settings.opcode_color,
            "Loads a value using base register + offset addressing.",
            |ui, fields| {
                ui_monospace_label_with_color(
                    ui,
                    "LDR DR, BaseR, #offset6",
                    theme_settings.opcode_color,
                );
                ui_italic_label(ui, "Loads from memory at BaseR+offset into DR");
                ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                ui_register_selector(ui, &mut fields.base_r, "Base Reg:");
                ui_immediate_selector(ui, &mut fields.offset6, 6, "Offset:");
                let pseudo_code = format!(
                    "DR = R{} = MEM[R{} + {}]",
                    fields.dr, fields.base_r, fields.offset6
                );
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let ldr_instr = (0b0110 << 12)
                    | ((fields.dr as u16) << 9)
                    | ((fields.base_r as u16) << 6)
                    | (fields.offset6 as u16 & 0x3F);

                render_binary_representation_view(
                    ui,
                    "ldr_binary",
                    "Layout: 0110 | DR | BaseR | offset6",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "0110".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.dr as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.base_r as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.offset6 as u16 & 0x3F, 6),
                            color: theme_settings.help_offset_color,
                        },
                    ],
                    ldr_instr,
                );
            },
            fields,
            Some("Sets condition codes: N, Z, P"),
        );
    }

    fn render_lea_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "LEA - Load Effective Address",
            theme_settings.opcode_color,
            "Loads the address of a label into a register.",
            |ui, fields| {
                ui_monospace_label_with_color(ui, "LEA DR, LABEL", theme_settings.opcode_color);
                ui_italic_label(ui, "Loads effective address PC+offset into DR");
                ui_register_selector(ui, &mut fields.dr, "Destination Reg:");
                ui_offset_selector(ui, &mut fields.offset9, 9, "PC Offset:");
                let pseudo_code = format!("DR = R{} = PC + {}", fields.dr, fields.offset9);
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let lea_instr =
                    (0b1110 << 12) | ((fields.dr as u16) << 9) | (fields.offset9 as u16 & 0x1FF);

                render_binary_representation_view(
                    ui,
                    "lea_binary",
                    "Layout: 1110 | DR | PCoffset9",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "1110".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.dr as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.offset9 as u16 & 0x1FF, 9),
                            color: theme_settings.help_offset_color,
                        },
                    ],
                    lea_instr,
                );
            },
            fields,
            Some("Sets condition codes: N, Z, P"),
        );
    }

    fn render_st_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "ST - Store",
            theme_settings.opcode_color,
            "Stores a register value into memory.",
            |ui, fields| {
                ui_monospace_label_with_color(ui, "ST SR, LABEL", theme_settings.opcode_color);
                ui_italic_label(ui, "Stores SR into memory at PC+offset");
                ui_register_selector(ui, &mut fields.sr1, "Source Reg:");
                ui_offset_selector(ui, &mut fields.offset9, 9, "PC Offset:");
                let pseudo_code = format!("MEM[PC + {}] = SR = R{}", fields.offset9, fields.sr1);
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let st_instr =
                    (0b0011 << 12) | ((fields.sr1 as u16) << 9) | (fields.offset9 as u16 & 0x1FF);

                render_binary_representation_view(
                    ui,
                    "st_binary",
                    "Layout: 0011 | SR | PCoffset9",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "0011".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.sr1 as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.offset9 as u16 & 0x1FF, 9),
                            color: theme_settings.help_offset_color,
                        },
                    ],
                    st_instr,
                );
            },
            fields,
            None,
        );
    }
    fn render_sti_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "STI - Store Indirect",
            theme_settings.opcode_color,
            "Stores a register value using a pointer in memory.",
            |ui, fields| {
                ui_monospace_label_with_color(ui, "STI SR, LABEL", theme_settings.opcode_color);
                ui_italic_label(ui, "Stores SR into memory at address stored at PC+offset");
                ui_register_selector(ui, &mut fields.sr1, "Source Reg:");
                ui_offset_selector(ui, &mut fields.offset9, 9, "PC Offset:");
                let pseudo_code =
                    format!("MEM[MEM[PC + {}]] = SR = R{}", fields.offset9, fields.sr1);
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let sti_instr =
                    (0b1011 << 12) | ((fields.sr1 as u16) << 9) | (fields.offset9 as u16 & 0x1FF);

                render_binary_representation_view(
                    ui,
                    "sti_binary",
                    "Layout: 1011 | SR | PCoffset9",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "1011".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.sr1 as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.offset9 as u16 & 0x1FF, 9),
                            color: theme_settings.help_offset_color,
                        },
                    ],
                    sti_instr,
                );
            },
            fields,
            None,
        );
    }
    fn render_str_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "STR - Store Register",
            theme_settings.opcode_color,
            "Stores a value using base register + offset addressing.",
            |ui, fields| {
                ui_monospace_label_with_color(
                    ui,
                    "STR SR, BaseR, #offset6",
                    theme_settings.opcode_color,
                );
                ui_italic_label(ui, "Stores SR into memory at BaseR+offset");
                ui_register_selector(ui, &mut fields.sr1, "Source Reg:");
                ui_register_selector(ui, &mut fields.base_r, "Base Reg:");
                ui_immediate_selector(ui, &mut fields.offset6, 6, "Offset:");
                let pseudo_code = format!(
                    "MEM[R{} + {}] = SR = R{}",
                    fields.base_r, fields.offset6, fields.sr1
                );
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let str_instr = (0b0111 << 12)
                    | ((fields.sr1 as u16) << 9)
                    | ((fields.base_r as u16) << 6)
                    | (fields.offset6 as u16 & 0x3F);

                render_binary_representation_view(
                    ui,
                    "str_binary",
                    "Layout: 0111 | SR | BaseR | offset6",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "0111".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.sr1 as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.base_r as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.offset6 as u16 & 0x3F, 6),
                            color: theme_settings.help_offset_color,
                        },
                    ],
                    str_instr,
                );
            },
            fields,
            None,
        );
    }
    fn render_br_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "BR/BRn/BRz/BRp - Conditional Branch",
            theme_settings.opcode_color,
            "Branches to a label if condition codes match.",
            |ui, fields| {
                ui_monospace_label_with_color(ui, "BRnzp LABEL", theme_settings.opcode_color);
                ui_italic_label(
                    ui,
                    "Branches to PC+offset if specified condition codes match",
                );

                ui.horizontal(|ui| {
                    ui_strong_label(ui, "Condition Flags:");
                    ui.checkbox(
                        &mut fields.n_bit,
                        RichText::new("N (negative)").color(theme_settings.help_strong_label_color),
                    );
                    ui.checkbox(
                        &mut fields.z_bit,
                        RichText::new("Z (zero)").color(theme_settings.help_strong_label_color),
                    );
                    ui.checkbox(
                        &mut fields.p_bit,
                        RichText::new("P (positive)").color(theme_settings.help_strong_label_color),
                    );
                });

                ui_offset_selector(ui, &mut fields.offset9, 9, "PC Offset:");
                let nzp_str = format!(
                    "{}{}{}",
                    if fields.n_bit { "N=1 " } else { "" },
                    if fields.z_bit { "Z=1 " } else { "" },
                    if fields.p_bit { "P=1" } else { "" }
                )
                .trim_end()
                .to_string();
                let pseudo_code = format!("if ({}) PC = PC + {}", nzp_str, fields.offset9);
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let br_instr = ((fields.n_bit as u16) << 11)
                    | ((fields.z_bit as u16) << 10)
                    | ((fields.p_bit as u16) << 9)
                    | (fields.offset9 as u16 & 0x1FF);

                render_binary_representation_view(
                    ui,
                    "br_binary",
                    "Layout: 0000 | n | z | p | PCoffset9",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "0000".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: (if fields.n_bit { "1" } else { "0" }).to_string(),
                            color: theme_settings.help_strong_label_color,
                        },
                        BinarySegment {
                            text: (if fields.z_bit { "1" } else { "0" }).to_string(),
                            color: theme_settings.help_strong_label_color,
                        },
                        BinarySegment {
                            text: (if fields.p_bit { "1" } else { "0" }).to_string(),
                            color: theme_settings.help_strong_label_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.offset9 as u16 & 0x1FF, 9),
                            color: theme_settings.help_offset_color,
                        },
                    ],
                    br_instr,
                );
            },
            fields,
            None,
        );
    }
    fn render_jmp_ret_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "JMP/RET - Jump",
            theme_settings.opcode_color,
            "Jumps to address in a register.",
            |ui, fields| {
                ui.horizontal(|ui| {
                    let mut ret_mode = fields.base_r == 7;
                    if ui
                        .checkbox(
                            &mut ret_mode,
                            RichText::new("RET").color(theme_settings.strong_text_color),
                        )
                        .clicked()
                    {
                        fields.base_r = if ret_mode { 7 } else { 0 };
                    }
                });

                if fields.base_r == 7 {
                    ui_monospace_label_with_color(ui, "RET", theme_settings.opcode_color);
                    ui_italic_label(ui, "Returns from subroutine - jumps to address in R7");
                    ui_monospace_label_with_color(
                        ui,
                        "PC = R7",
                        theme_settings.help_pseudo_code_color,
                    );
                } else {
                    ui_monospace_label_with_color(ui, "JMP BaseR", theme_settings.opcode_color);
                    ui_italic_label(ui, "Jumps to address in BaseR");
                    ui_register_selector(ui, &mut fields.base_r, "Base Reg:");
                    let pseudo_code = format!("PC = BaseR = R{}", fields.base_r);
                    ui_monospace_label_with_color(
                        ui,
                        &pseudo_code,
                        theme_settings.help_pseudo_code_color,
                    );
                }

                let jmp_instr = (0b1100 << 12) | ((fields.base_r as u16) << 6);

                render_binary_representation_view(
                    ui,
                    "jmp_binary",
                    "Layout: 1100 | 000 | BaseR | 000000",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "1100".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: "000".to_string(),
                            color: theme_settings.help_binary_layout_fixed_bits_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.base_r as u16 & 0x7, 3),
                            color: theme_settings.help_operand_color,
                        },
                        BinarySegment {
                            text: "000000".to_string(),
                            color: theme_settings.help_binary_layout_fixed_bits_color,
                        },
                    ],
                    jmp_instr,
                );
            },
            fields,
            None,
        );
    }
    fn render_jsr_jsrr_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "JSR/JSRR - Jump to Subroutine",
            theme_settings.opcode_color,
            "Jumps to a subroutine, saving return address in R7.",
            |ui, fields| {
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut fields.jsr_mode,
                        RichText::new("JSR mode (vs JSRR)").color(theme_settings.strong_text_color),
                    );
                });

                if fields.jsr_mode {
                    ui_monospace_label_with_color(ui, "JSR LABEL", theme_settings.opcode_color);
                    ui_italic_label(ui, "Jumps to PC+offset, saving return address in R7");
                    ui_offset_selector(ui, &mut fields.offset11, 11, "PC Offset:");
                    let pseudo_code = format!("R7 = PC\nPC = PC + {}", fields.offset11);
                    ui_monospace_label_with_color(
                        ui,
                        &pseudo_code,
                        theme_settings.help_pseudo_code_color,
                    );

                    let jsr_instr = (0b0100 << 12) | (1 << 11) | (fields.offset11 as u16 & 0x7FF);

                    render_binary_representation_view(
                        ui,
                        "jsr_binary",
                        "Layout: 0100 | 1 | PCoffset11",
                        theme_settings.help_binary_layout_fixed_bits_color,
                        vec![
                            BinarySegment {
                                text: "0100".to_string(),
                                color: theme_settings.opcode_color,
                            },
                            BinarySegment {
                                text: "1".to_string(), // JSR bit
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.offset11 as u16 & 0x7FF, 11),
                                color: theme_settings.help_offset_color,
                            },
                        ],
                        jsr_instr,
                    );
                } else {
                    ui_monospace_label_with_color(ui, "JSRR BaseR", theme_settings.opcode_color);
                    ui_italic_label(ui, "Jumps to address in BaseR, saving return address in R7");
                    ui_register_selector(ui, &mut fields.base_r, "Base Reg:");
                    let pseudo_code = format!("R7 = PC\nPC = BaseR = R{}", fields.base_r);
                    ui_monospace_label_with_color(
                        ui,
                        &pseudo_code,
                        theme_settings.help_pseudo_code_color,
                    );

                    let jsrr_instr = (0b0100 << 12) | ((fields.base_r as u16) << 6);

                    render_binary_representation_view(
                        ui,
                        "jsrr_binary",
                        "Layout: 0100 | 0 | 00 | BaseR | 000000",
                        theme_settings.help_binary_layout_fixed_bits_color,
                        vec![
                            BinarySegment {
                                text: "0100".to_string(),
                                color: theme_settings.opcode_color,
                            },
                            BinarySegment {
                                text: "0".to_string(), // JSRR bit
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: "00".to_string(),
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                            BinarySegment {
                                text: format_binary(fields.base_r as u16 & 0x7, 3),
                                color: theme_settings.help_operand_color,
                            },
                            BinarySegment {
                                text: "000000".to_string(),
                                color: theme_settings.help_binary_layout_fixed_bits_color,
                            },
                        ],
                        jsrr_instr,
                    );
                }
            },
            fields,
            None,
        );
    }
    fn render_trap_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "TRAP - System Call",
            theme_settings.opcode_color,
            "Performs a system call based on the trap vector.",
            |ui, fields| {
                ui_monospace_label_with_color(ui, "TRAP trapvect8", theme_settings.opcode_color);
                ui_italic_label(ui, "System call to vector specified by trapvect8");

                ui.horizontal(|ui| {
                    ui_strong_label(ui, "Trap Vector:");
                    let mut trap_hex = format!("0x{:02X}", fields.trapvector);
                    if ui
                        .add(egui::TextEdit::singleline(&mut trap_hex).id_source("trap_hex_input"))
                        .changed()
                    {
                        if let Ok(value) = u8::from_str_radix(trap_hex.trim_start_matches("0x"), 16)
                        {
                            fields.trapvector = value;
                        }
                    }
                    ui.add(
                        egui::DragValue::new(&mut fields.trapvector)
                            .range(0..=0xFF) // Updated from range
                            .speed(0.1),
                    )
                    .on_hover_text("Trap vector (0-255)");
                });

                ui.separator();
                ui_strong_label(ui, "Common TRAP vectors:");

                let trap_options = [
                    (0x20, "GETC (x20)", "Read character from keyboard -> R0"),
                    (0x21, "OUT (x21)", "Write character in R0 to console"),
                    (
                        0x22,
                        "PUTS (x22)",
                        "Output null-terminated string pointed to by R0",
                    ),
                    (0x23, "IN (x23)", "Print prompt and read character -> R0"),
                    (0x25, "HALT (x25)", "Halt execution"),
                ];

                for (vec, label, desc) in trap_options {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                fields.trapvector == vec,
                                RichText::new(label).color(theme_settings.hyperlink_color),
                            )
                            .clicked()
                        {
                            fields.trapvector = vec;
                        }
                        ui_simple_label(ui, desc);
                    });
                }

                let pseudo_code = format!("R7 = PC\nPC = MEM[x{:02X}]", fields.trapvector);
                ui_monospace_label_with_color(
                    ui,
                    &pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let trap_instr = (0b1111 << 12) | (fields.trapvector as u16);

                render_binary_representation_view(
                    ui,
                    "trap_binary",
                    "Layout: 1111 | 0000 | trapvect8",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "1111".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: "0000".to_string(),
                            color: theme_settings.help_binary_layout_fixed_bits_color,
                        },
                        BinarySegment {
                            text: format_binary(fields.trapvector as u16, 8),
                            color: theme_settings.help_operand_color,
                        },
                    ],
                    trap_instr,
                );
            },
            fields,
            None,
        );
    }
    fn render_rti_instruction(&mut self, ui: &mut Ui) {
        let fields = &mut self.instruction_fields;
        let theme_settings = CURRENT_THEME_SETTINGS.lock().unwrap();

        render_instruction_card_content(
            ui,
            "RTI - Return from Interrupt",
            theme_settings.opcode_color,
            "Returns from an interrupt service routine.",
            |ui, _fields| {
                ui_monospace_label_with_color(ui, "RTI", theme_settings.opcode_color);
                ui_italic_label(ui, "Return from interrupt - restore PC and PSR from stack");

                let pseudo_code = "if (Privilege Mode)\n    PC = MEM[R6]\n    PSR = MEM[R6+1]\n    R6 = R6 + 2\nelse\n    Privilege Mode Exception";
                ui_monospace_label_with_color(
                    ui,
                    pseudo_code,
                    theme_settings.help_pseudo_code_color,
                );

                let rti_instr = 0b1000 << 12;

                render_binary_representation_view(
                    ui,
                    "rti_binary",
                    "Layout: 1000 | 000000000000",
                    theme_settings.help_binary_layout_fixed_bits_color,
                    vec![
                        BinarySegment {
                            text: "1000".to_string(),
                            color: theme_settings.opcode_color,
                        },
                        BinarySegment {
                            text: "000000000000".to_string(),
                            color: theme_settings.help_binary_layout_fixed_bits_color,
                        },
                    ],
                    rti_instr,
                );
            },
            fields,
            None,
        );
    }
}
