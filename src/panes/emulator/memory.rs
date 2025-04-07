use crate::app::{base_to_base, EMULATOR};
use crate::emulator::{Emulator, EmulatorCell};
use crate::panes::{Pane, PaneDisplay, PaneTree};
use egui::{Align, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

use super::editor::COMPILATION_ARTIFACTS;
use super::EmulatorPane;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryPane {
    follow_pc: bool,
    jump_addr_str: String,
    #[serde(skip)] // Don't serialize the target scroll address, recalculate on load if needed
    target_scroll_addr: Option<usize>,
    display_base: u32,
}

impl Default for MemoryPane {
    fn default() -> Self {
        Self {
            follow_pc: true,
            jump_addr_str: "0000".to_string(),
            target_scroll_addr: None,
            display_base: 16,
        }
    }
}

impl PaneDisplay for MemoryPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        let mut emulator = EMULATOR.lock().unwrap();
        let artifacts = COMPILATION_ARTIFACTS.lock().unwrap(); // Lock artifacts for label lookup

        ui.horizontal(|ui| {
            // --- Controls ---
            ui.checkbox(&mut self.follow_pc, "Follow PC");

            ui.label("Jump to (Hex):");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.jump_addr_str)
                    .desired_width(50.0)
                    .font(egui::TextStyle::Monospace),
            );
            if response.lost_focus() || response.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(addr) =
                    usize::from_str_radix(self.jump_addr_str.trim_start_matches("0x"), 16)
                {
                    if addr < emulator.memory.len() {
                        self.target_scroll_addr = Some(addr);
                        self.follow_pc = false; // Jumping disables follow PC
                    } else {
                        // Handle invalid address (e.g., show error, reset input)
                        self.jump_addr_str = "FFFF".to_string();
                    }
                } else {
                    // Handle parse error
                    self.jump_addr_str = format!("{:04X}", self.target_scroll_addr.unwrap_or(0));
                    // Reset to current/last target
                }
            }

            ui.label("Base:");
            ui.radio_value(&mut self.display_base, 2, "Bin");
            ui.radio_value(&mut self.display_base, 10, "Dec");
            ui.radio_value(&mut self.display_base, 16, "Hex");
        });

        ui.separator();

        // --- Memory Table ---
        let text_height = egui::TextStyle::Monospace.resolve(ui.style()).size * 2.0; // *2 to account for buttons
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(100.0)) // Label
            .column(Column::auto().at_least(60.0)) // Address + PC indicator
            .column(Column::auto().at_least(50.0)) // Value (editable)
            .column(Column::auto().at_least(60.0)) // Value (formatted)
            .column(Column::auto().at_least(150.0)) // Instruction
            .column(Column::remainder().at_least(80.0)) // ASCII
            .min_scrolled_height(0.0);

        // --- Scrolling Logic ---
        let pc_addr = emulator.pc.get() as usize;
        let scroll_target_row = if self.follow_pc {
            Some(pc_addr)
        } else {
            self.target_scroll_addr
        };

        // Apply scrolling *before* creating the body
        if let Some(target_row) = scroll_target_row {
            // Check if the target row is within the valid range of memory indices
            if target_row < emulator.memory.len() {
                // Find the row index within the visible range (assuming full range for now)
                // The `rows_range` logic needs adjustment if you only show a subset.
                // For now, let's assume the visible index is the target address.
                let visible_row_index = target_row; // Adjust if rows_range is used differently
                table = table.scroll_to_row(visible_row_index, Some(Align::Center));
                // Clear target_scroll_addr after jumping if it wasn't set by follow_pc
                if !self.follow_pc {
                    self.target_scroll_addr = None;
                }
            }
        }

        table.body(|mut body| {
            let item_spacing = body.ui_mut().spacing().item_spacing;

            body.rows(text_height, 0xFFFF, |mut row| {
                let row_index = row.index();
                // Ensure row_index is within bounds before accessing memory
                if row_index >= emulator.memory.len() {
                    // Optionally render an error or skip the row
                    row.col(|ui| {
                        ui.label("Invalid Address");
                    });
                    row.col(|ui| {});
                    row.col(|ui| {});
                    row.col(|ui| {});
                    return;
                }
                let memory_cell = &mut emulator.memory[row_index];
                let is_pc_line = pc_addr == row_index;

                let bg = if is_pc_line {
                    Some(egui::Color32::from_rgb(50, 80, 50)) // Dark green background for PC
                } else {
                    None // Use striped background
                };

                let paint_bg = |ui: &mut egui::Ui| {
                    if let Some(color) = bg {
                        let gapless_rect = ui.max_rect().expand2(0.5 * item_spacing);
                        ui.painter().rect_filled(gapless_rect, 0.0, color);
                    }
                };

                // label
                row.col(|ui| {
                    paint_bg(ui);

                    // Try to find a label for this address
                    let label = artifacts.labels.iter().find_map(|(name, &addr)| {
                        if addr as usize == row_index {
                            Some(name.clone())
                        } else {
                            None
                        }
                    });

                    let display = match label {
                        Some(lbl) => format!("{}", lbl),
                        None => "".to_string(),
                    };

                    ui.label(RichText::new(display).monospace().strong());
                });

                // Address Column
                row.col(|ui| {
                    paint_bg(ui);

                    let addr_text = format!(
                        "0x{:04X}{}",
                        row_index,
                        if is_pc_line { " (PC)" } else { "" }
                    );
                    let rich_text = RichText::new(addr_text).monospace();
                    ui.label(rich_text);
                });

                // Value Edit Column
                row.col(|ui| {
                    paint_bg(ui);

                    let mut value_i16 = memory_cell.get() as i16; // Use i16 for DragValue editing
                                                                  // Custom formatter and parser to handle different bases
                                                                  // Corrected signature: RangeInclusive<usize>
                    let format_fn = |n: f64, _range: RangeInclusive<usize>| -> String {
                        base_to_base(
                            10,
                            self.display_base,
                            &(n as i16 as u16 as u32).to_string(),
                            "0123456789ABCDEF",
                        )
                    };
                    let parse_fn = |s: &str| -> Option<f64> {
                        match self.display_base {
                            16 => u16::from_str_radix(s.trim_start_matches("0x"), 16)
                                .ok()
                                .map(|v| v as i16 as f64),
                            10 => s.parse::<i16>().ok().map(|v| v as f64),
                            2 => u16::from_str_radix(s, 2).ok().map(|v| v as i16 as f64),
                            _ => None, // Add other bases if needed
                        }
                    };

                    if ui
                        .add(
                            egui::DragValue::new(&mut value_i16)
                                .custom_formatter(format_fn)
                                .custom_parser(parse_fn),
                        )
                        .changed()
                    {
                        memory_cell.set(value_i16 as u16);
                    }
                });

                // Formatted Value Column
                row.col(|ui| {
                    paint_bg(ui);

                    let value_u16 = memory_cell.get();
                    let formatted_val = match self.display_base {
                        2 => format!("{:016b}", value_u16),
                        10 => format!("{}", value_u16),
                        16 => format!("0x{:04X}", value_u16),
                        _ => base_to_base(
                            10,
                            self.display_base,
                            &(value_u16 as u32).to_string(),
                            "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                        ),
                    };
                    ui.label(RichText::new(formatted_val).monospace());
                });

                // Decoded Instruction Column
                row.col(|ui| {
                    paint_bg(ui);

                    let value_u16 = memory_cell.get();

                    // Try to decode as an instruction if it's a plausible address
                    let decoded_op = match crate::emulator::ops::OpCode::from_instruction(
                        EmulatorCell::new(value_u16),
                    ) {
                        Some(op) => Some(format!("{}", op)),
                        None => None,
                    };

                    if let Some(decoded_str) = decoded_op {
                        // Display decoded Op struct
                        ui.label(RichText::new(decoded_str).monospace());
                    } else {
                        ui.label(RichText::new("").monospace());
                    }
                });

                // ASCII Column
                row.col(|ui| {
                    let ascii_char = char::from_u32((memory_cell.get() & 0xFF) as u32)
                        .filter(|c| c.is_ascii_graphic() || *c == ' ') // Show printable ASCII or space
                        .map(|c| format!("'{}'", c))
                        .unwrap_or_default();

                    ui.label(RichText::new(ascii_char).monospace().weak()); // Weak color for less emphasis
                });
            });
        });
    }

    fn title(&self) -> String {
        "Memory".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Memory".to_string(),
            Pane::EmulatorPanes(Box::new(EmulatorPane::Memory(MemoryPane::default()))),
        )
    }
}
