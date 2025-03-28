use crate::app::EMULATOR;
use crate::emulator::EmulatorCell;
use crate::panes::{Pane, PaneDisplay, PaneTree};
use egui::{OutputCommand, RichText};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use super::EmulatorPane;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct IoPane {
    input_queue: String,
    shell_input: String,
}

impl Default for IoPane {
    fn default() -> Self {
        Self {
            input_queue: String::new(),
            shell_input: String::new(),
        }
    }
}

impl PaneDisplay for IoPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        let mut emulator = EMULATOR.lock().unwrap();

        ui.horizontal(|ui| {
            ui.label("Input Queue (for GETC):");
            ui.add(
                egui::TextEdit::singleline(&mut self.input_queue)
                    .hint_text("Enter characters for GETC..."),
            );
        });

        ui.separator();

        ui.label(RichText::new("Program Output:").strong());

        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                if emulator.output.is_empty() {
                    ui.label(
                        RichText::new("No output yet")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    // Use a clone to allow simultaneous mutable access to emulator
                    // and immutable access to its output for TextEdit.
                    let output_clone = &mut emulator.output.clone();
                    ui.add(
                        egui::TextEdit::multiline(output_clone)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace)
                            .interactive(false), // Output is read-only here
                    );
                }
            });

        if !emulator.output.is_empty() {
            ui.horizontal(|ui| {
                if ui.button("Clear Output").clicked() {
                    emulator.output.clear();
                }
                let output_clone = emulator.output.clone();
                if ui.button("Copy to Clipboard").clicked() {
                    ui.output_mut(|o| o.commands.push(OutputCommand::CopyText(output_clone)));
                }
            });
        }

        ui.separator();

        // Handle pending input requests
        match emulator.await_input {
            Some(is_prompted_input) => {
                if !is_prompted_input {
                    // GETC waiting
                    if !self.input_queue.is_empty() {
                        let c = self.input_queue.remove(0);
                        emulator.r[0].set(c as u16);
                        emulator.await_input = None;
                        tracing::debug!("GETC consumed character '{}' from input queue", c);
                    } else {
                        ui.label(
                            RichText::new("Waiting for GETC input...").color(egui::Color32::YELLOW),
                        );
                        ui.label("Enter characters into the 'Input Queue' above.");
                    }
                } else {
                    // IN waiting (prompted input)
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("TRAP IN Waiting for input:")
                                .strong()
                                .color(egui::Color32::YELLOW),
                        );
                        ui.horizontal(|ui| {
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.shell_input)
                                    .hint_text("Enter character..."),
                            );

                            if (response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                                || ui.button("Submit").clicked()
                            {
                                if let Some(c) = self.shell_input.chars().next() {
                                    emulator.r[0].set(c as u16);
                                    emulator.output.push(c); // Echo the character
                                    emulator.await_input = None;
                                    self.shell_input.clear();
                                    tracing::debug!("IN submitted character '{}'", c);
                                }
                            }
                        });
                        ui.label("Type a character and press Enter or click Submit.");
                    });
                }
            }
            None => {
                // No input needed
            }
        }
    }

    fn title(&self) -> String {
        "IO".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "IO".to_string(),
            Pane::EmulatorPanes(Box::new(EmulatorPane::Output(IoPane::default()))),
        )
    }
}
