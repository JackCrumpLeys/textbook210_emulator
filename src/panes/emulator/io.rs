use crate::app::EMULATOR;
use crate::panes::{Pane, PaneDisplay, PaneTree, RealPane};
use egui::{Key, OutputCommand, RichText};
use serde::{Deserialize, Serialize};

use super::EmulatorPane;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct IoPane {
    terminal_input: String,
    interactive_input: String,
}

impl PaneDisplay for IoPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut emulator = EMULATOR.lock().unwrap();

            // Display terminal output
            ui.label(RichText::new("Terminal:").strong());

            let terminal_height = 200.0;
            egui::ScrollArea::vertical()
                .max_height(terminal_height)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    // Use a clone to allow simultaneous mutable access to emulator
                    // and immutable access to its output for TextEdit.
                    let output_clone = &mut emulator.output.clone();
                    let response = ui.add(
                        egui::TextEdit::multiline(output_clone)
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                            .font(egui::TextStyle::Monospace),
                    );

                    if response.changed() && emulator.output.len() < output_clone.len() {
                        emulator.set_in_char(output_clone.chars().last().unwrap());
                    }
                });

            // Device registers information box
            ui.group(|ui| {
                ui.label(RichText::new("Device Registers").strong().heading());

                // KBSR - Keyboard Status Register
                ui.horizontal(|ui| {
                    ui.label("KBSR (xFE00):");
                    let kbsr_ready = emulator.memory[0xFE00].get() & 0x8000 != 0;
                    ui.checkbox(&mut kbsr_ready.clone(), "Ready Bit");
                    ui.label(
                        RichText::new("Indicates if keyboard has a character ready")
                            .small()
                            .italics(),
                    );
                });

                // KBDR - Keyboard Data Register
                ui.horizontal(|ui| {
                    ui.label("KBDR (xFE02):");
                    let kbdr_value = emulator.memory[0xFE02].get() & 0xFF;
                    ui.label(format!(
                        "Value: {:#04x} ('{}')",
                        kbdr_value,
                        char::from_u32(kbdr_value as u32).unwrap_or('?')
                    ));
                    ui.label(
                        RichText::new("Contains the last character typed")
                            .small()
                            .italics(),
                    );
                });
            });

            // Regular input field (kept for compatibility)
            ui.horizontal(|ui| {
                ui.label(">");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.terminal_input)
                        .desired_width(ui.available_width())
                        .hint_text("Type here and press Enter...")
                        .font(egui::TextStyle::Monospace),
                );

                // Check for input submission via Enter key or losing focus
                let input_submitted =
                    response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));

                if input_submitted && !self.terminal_input.is_empty() {
                    if let Some(c) = self.terminal_input.chars().next() {
                        // Update the emulator's last pressed key
                        emulator.set_in_char(c);
                        // Echo the character to output for visual feedback
                        emulator.output.push(c);
                        // Clear the input field
                        self.terminal_input.clear();
                    }
                }
            });

            // Control buttons
            ui.horizontal(|ui| {
                if ui.button("Clear Output").clicked() {
                    emulator.output.clear();
                }

                let output_clone = emulator.output.clone();
                if ui.button("Copy to Clipboard").clicked() {
                    ui.output_mut(|o| o.commands.push(OutputCommand::CopyText(output_clone)));
                }
            });
        });
    }

    fn title(&self) -> String {
        "Terminal".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Terminal".to_string(),
            Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Output(
                IoPane::default(),
            )))),
        )
    }
}
