use crate::app::{base_to_base, EMULATOR};
use crate::emulator::Emulator;
use crate::panes::{Pane, PaneDisplay, PaneTree};
use egui::RichText;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;

use super::editor::{CompilationArtifacts, COMPILATION_ARTIFACTS};
use super::EmulatorPane;

lazy_static! {
    pub static ref BREAKPOINTS: Mutex<HashSet<usize>> = Mutex::new(HashSet::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MachinePane {
    show_machine_code: bool,
    machine_code_base: u32,
}

impl Default for MachinePane {
    fn default() -> Self {
        Self {
            show_machine_code: false,
            machine_code_base: 16,
        }
    }
}

impl PaneDisplay for MachinePane {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let artifacts = COMPILATION_ARTIFACTS.lock().unwrap();
            let mut emulator = EMULATOR.lock().unwrap();

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

            ui.separator();

            self.render_compiled_view(ui, &artifacts, &mut emulator);
        });
    }

    fn title(&self) -> String {
        "Machine Code".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Machine Code".to_string(),
            Pane::EmulatorPanes(Box::new(EmulatorPane::Machine(MachinePane::default()))),
        )
    }
}

impl MachinePane {
    fn render_compiled_view(
        &mut self,
        ui: &mut egui::Ui,
        artifacts: &CompilationArtifacts,
        emulator: &mut Emulator,
    ) {
        if artifacts.last_compiled_source.is_empty() {
            ui.label("No program compiled yet.");
            return;
        }

        let mut longest_label = 0;
        let mut longest_body = 0;
        let mut longest_operand = 0;

        for (i, line) in artifacts.last_compiled_source.lines().enumerate() {
            if !artifacts.line_to_address.contains_key(&i) || line.is_empty() {
                continue;
            }
            let code_part = if line.contains(';') {
                line.split(';').next().unwrap().trim()
            } else {
                line.trim()
            };

            if code_part.is_empty() {
                continue;
            }

            let mut split = code_part.split_whitespace();
            let (mut len_lab, mut len_body, mut len_op) = (0, 0, 0);

            match split.clone().count() {
                0 => {} // Should not happen if code_part is not empty
                1 => len_body = split.next().unwrap().len(),
                2 => {
                    let first = split.next().unwrap();
                    if first.ends_with(':') {
                        len_lab = first.len();
                        len_body = split.next().unwrap().len();
                    } else {
                        len_body = first.len();
                        len_op = split.next().unwrap().len();
                    }
                }
                _ => {
                    let first = split.next().unwrap();
                    if first.ends_with(':') {
                        len_lab = first.len();
                        if let Some(second) = split.next() {
                            len_body = second.len();
                            len_op = split.collect::<Vec<&str>>().join(" ").len();
                        }
                    } else {
                        len_body = first.len();
                        len_op = split.collect::<Vec<&str>>().join(" ").len();
                    }
                }
            }

            longest_label = longest_label.max(len_lab);
            longest_body = longest_body.max(len_body);
            longest_operand = longest_operand.max(len_op);
        }

        let mut breakpoints = BREAKPOINTS.lock().unwrap();

        for (i, line) in artifacts.last_compiled_source.lines().enumerate() {
            let original_line = line.trim().to_string();
            let (code_part, comment_part) = if original_line.contains(';') {
                let parts: Vec<&str> = original_line.split(';').collect();
                (
                    parts[0].trim().to_ascii_uppercase(),
                    format!("; {}", parts[1..].join(";")),
                )
            } else {
                (original_line.to_ascii_uppercase(), String::new())
            };

            let mut display_text = code_part.clone();
            let is_breakpoint_line = artifacts
                .line_to_address
                .get(&i)
                .is_some_and(|addr| breakpoints.contains(addr));

            if is_breakpoint_line {
                display_text = format!("{} (breakpoint)", display_text);
            }

            let is_error_line = artifacts
                .error
                .as_ref()
                .is_some_and(|(_, line_num)| *line_num == i);
            if is_error_line {
                if let Some((error_msg, _)) = &artifacts.error {
                    display_text = format!("{} (error: {})", display_text, error_msg);
                }
            }

            let label_capitalized = display_text.to_ascii_uppercase();
            let is_directive = label_capitalized.contains(".ORIG")
                || label_capitalized.contains(".FILL")
                || label_capitalized.contains(".BLKW")
                || label_capitalized.contains(".STRINGZ")
                || label_capitalized.contains(".END");

            if let Some(address) = artifacts.line_to_address.get(&i) {
                let display_address = *address; // This is the actual memory address

                // Format the assembly code part for display
                let label_parts: Vec<&str> = code_part.split_whitespace().collect();
                let formatted_asm = match label_parts.len() {
                    0 => "".to_string(), // Empty code part
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
                    _ => {
                        if label_parts[0].ends_with(':') {
                            if label_parts.len() >= 2 {
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
                                // Only label and colon
                                format!("{:<width1$}", label_parts[0], width1 = longest_label)
                            }
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

                display_text = formatted_asm;

                if self.show_machine_code {
                    if let Some(instruction_cell) = emulator.memory.get(display_address) {
                        let instruction_val = instruction_cell.get();
                        display_text = match self.machine_code_base {
                            2 => format!("{:016b}", instruction_val),
                            16 => format!("0x{:04X}", instruction_val),
                            10 => format!("{}", instruction_val),
                            _ => base_to_base(
                                10,
                                self.machine_code_base,
                                &(instruction_val as u32).to_string(),
                                "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                            ),
                        };
                    } else {
                        display_text = "Invalid Addr".to_string();
                    }
                }

                let pc_indicator = if emulator.pc.get() as usize == display_address {
                    " (pc)"
                } else {
                    ""
                };

                display_text = format!(
                    "0x{:04X}: {}{}",
                    display_address, display_text, pc_indicator
                );

                ui.horizontal(|ui| {
                    if ui.button("🛑").clicked() {
                        if breakpoints.contains(&display_address) {
                            breakpoints.remove(&display_address);
                        } else {
                            breakpoints.insert(display_address);
                        }
                    }

                    let mut text_rich = RichText::new(&display_text).monospace();
                    let is_pc_line = emulator.pc.get() as usize == display_address;
                    let is_breakpoint = breakpoints.contains(&display_address); // Re-check after potential modification

                    text_rich = match (is_pc_line, is_error_line, is_breakpoint, is_directive) {
                        (true, _, _, _) => text_rich
                            .background_color(egui::Color32::GREEN)
                            .color(egui::Color32::BLACK),
                        (_, true, _, _) => text_rich.color(egui::Color32::YELLOW),
                        (_, _, true, _) => text_rich
                            .background_color(egui::Color32::LIGHT_RED)
                            .color(egui::Color32::BLACK),
                        (_, _, _, true) => text_rich
                            .background_color(egui::Color32::LIGHT_BLUE)
                            .color(egui::Color32::BLACK),
                        _ => text_rich,
                    };

                    ui.label(text_rich);

                    if !comment_part.is_empty() {
                        ui.label(
                            RichText::new(&comment_part)
                                .color(egui::Color32::GRAY)
                                .monospace(),
                        );
                    }

                    // Allow editing memory values for FILL or BLKW (represented as zeros)
                    if code_part.contains(".FILL") || (is_directive && code_part.contains(".BLKW"))
                    {
                        if let Some(memory_cell) = emulator.memory.get_mut(display_address) {
                            let value_u16 = memory_cell.get();
                            let mut value = value_u16 as i16;
                            if ui.add(egui::DragValue::new(&mut value)).changed() {
                                memory_cell.set(value as u16);
                            }
                        }
                    }
                });
            } else if is_error_line {
                // Display error line even if it doesn't map to an address
                ui.horizontal(|ui| {
                    ui.add_space(20.0); // Placeholder for breakpoint button width
                    ui.label(
                        RichText::new(&display_text)
                            .color(egui::Color32::YELLOW)
                            .monospace(),
                    );
                    if !comment_part.is_empty() {
                        ui.label(
                            RichText::new(&comment_part)
                                .color(egui::Color32::GRAY)
                                .monospace(),
                        );
                    }
                });
            } else if !code_part.is_empty() || !comment_part.is_empty() {
                // Display lines without addresses (like empty lines with comments, or .END)
                ui.horizontal(|ui| {
                    ui.add_space(20.0); // Placeholder width for breakpoint button
                    if !code_part.is_empty() {
                        ui.label(RichText::new(&code_part).monospace());
                    }
                    if !comment_part.is_empty() {
                        ui.label(
                            RichText::new(&comment_part)
                                .color(egui::Color32::GRAY)
                                .monospace(),
                        );
                    }
                });
            }
        }
    }
}
